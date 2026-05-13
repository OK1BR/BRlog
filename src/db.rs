use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection};

use crate::models::log::{Log, LogKind};
use crate::models::qso::{parse_legacy_frequency, Qso};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL DEFAULT 'general',
    is_active   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_log_active ON log(is_active) WHERE is_active = 1;

CREATE TABLE IF NOT EXISTS qso (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    log_id        INTEGER NOT NULL DEFAULT 0,
    callsign      TEXT NOT NULL,
    qso_datetime  TEXT NOT NULL,
    frequency     INTEGER NOT NULL DEFAULT 0,
    mode          TEXT NOT NULL,
    rst_sent      TEXT NOT NULL DEFAULT '',
    rst_rcvd      TEXT NOT NULL DEFAULT '',
    locator       TEXT NOT NULL DEFAULT '',
    created_at    TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_qso_datetime ON qso(qso_datetime DESC);
";
// Note: the `idx_qso_log_id` index is created inside `migrate_introduce_logs`
// after the `log_id` column is guaranteed to exist. Putting it in SCHEMA would
// fail on a legacy single-log database whose `qso` table predates the column —
// `CREATE TABLE IF NOT EXISTS qso` is a no-op there, so the index would refer
// to a column the migration hasn't added yet.

pub struct Db {
    conn: Connection,
}

impl Db {
    /// Open the SQLite database, applying schema and one-shot migrations.
    /// `default_log_name` is used only when an empty `log` table needs an initial
    /// row (fresh install or first run after upgrading from single-log).
    pub fn open(default_log_name: &str) -> Result<Self> {
        let path = db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create db dir {}", parent.display()))?;
        }
        let conn = Connection::open(&path)
            .with_context(|| format!("open sqlite at {}", path.display()))?;
        conn.execute_batch(SCHEMA).context("init schema")?;
        migrate_band_to_frequency(&conn).context("migrate band → frequency")?;
        migrate_frequency_text_to_hz(&conn).context("migrate frequency text → Hz")?;
        migrate_introduce_logs(&conn, default_log_name).context("migrate introduce logs")?;
        Ok(Self { conn })
    }

    // ── Logs ───────────────────────────────────────────────────────────────

    pub fn list_logs(&self) -> Result<Vec<Log>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, kind, is_active FROM log ORDER BY created_at ASC, id ASC",
            )
            .context("prepare list_logs")?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Log {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: LogKind::from_db_str(&row.get::<_, String>(2)?),
                    is_active: row.get::<_, i64>(3)? != 0,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn create_log(&self, name: &str, kind: LogKind) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO log (name, kind, is_active) VALUES (?1, ?2, 0)",
                params![name, kind.as_db_str()],
            )
            .context("insert log")?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn rename_log(&self, id: i64, name: &str) -> Result<()> {
        self.conn
            .execute("UPDATE log SET name = ?1 WHERE id = ?2", params![name, id])
            .with_context(|| format!("rename log #{id}"))?;
        Ok(())
    }

    pub fn set_log_kind(&self, id: i64, kind: LogKind) -> Result<()> {
        self.conn
            .execute(
                "UPDATE log SET kind = ?1 WHERE id = ?2",
                params![kind.as_db_str(), id],
            )
            .with_context(|| format!("set kind of log #{id}"))?;
        Ok(())
    }

    /// Delete a log. Refuses if the log holds any QSOs or if it is the last
    /// remaining log — the UI translates these into user-facing messages.
    pub fn delete_log(&self, id: i64) -> Result<()> {
        let total_logs: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM log", [], |r| r.get(0))?;
        if total_logs <= 1 {
            return Err(anyhow!("cannot delete the last remaining log"));
        }
        let qso_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM qso WHERE log_id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        if qso_count > 0 {
            return Err(anyhow!(
                "log #{id} still holds {qso_count} QSO(s); empty it before deleting"
            ));
        }
        let was_active: i64 = self
            .conn
            .query_row(
                "SELECT is_active FROM log WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        self.conn
            .execute("DELETE FROM log WHERE id = ?1", params![id])
            .with_context(|| format!("delete log #{id}"))?;
        if was_active != 0 {
            // Promote the oldest remaining log to active so there is always
            // exactly one active row.
            let next: Option<i64> = self
                .conn
                .query_row(
                    "SELECT id FROM log ORDER BY created_at ASC, id ASC LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .ok();
            if let Some(next_id) = next {
                self.set_active_log(next_id)?;
            }
        }
        Ok(())
    }

    /// Make `id` the active log. The unique partial index on `is_active`
    /// guarantees only one row carries the flag.
    pub fn set_active_log(&self, id: i64) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("UPDATE log SET is_active = 0 WHERE is_active = 1", [])?;
        tx.execute(
            "UPDATE log SET is_active = 1 WHERE id = ?1",
            params![id],
        )?;
        tx.commit().context("commit set_active_log")?;
        Ok(())
    }

    // ── QSOs ───────────────────────────────────────────────────────────────

    pub fn insert_qso(&self, log_id: i64, qso: &Qso) -> Result<i64> {
        let datetime_iso = qso
            .qso_datetime
            .to_rfc3339_opts(SecondsFormat::Secs, true);
        self.conn
            .execute(
                "INSERT INTO qso (log_id, callsign, qso_datetime, frequency, mode, rst_sent, rst_rcvd, locator)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    log_id,
                    qso.callsign,
                    datetime_iso,
                    qso.frequency as i64,
                    qso.mode,
                    qso.rst_sent,
                    qso.rst_rcvd,
                    qso.locator,
                ],
            )
            .context("insert qso")?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn delete_qso(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM qso WHERE id = ?1", params![id])
            .with_context(|| format!("delete qso #{id}"))?;
        Ok(())
    }

    pub fn list_qsos(&self, log_id: i64) -> Result<Vec<Qso>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, callsign, qso_datetime, frequency, mode, rst_sent, rst_rcvd, locator
                 FROM qso
                 WHERE log_id = ?1
                 ORDER BY qso_datetime DESC, id DESC",
            )
            .context("prepare list_qsos")?;

        let rows: Vec<RawRow> = stmt
            .query_map(params![log_id], |row| {
                Ok(RawRow {
                    id: row.get(0)?,
                    callsign: row.get(1)?,
                    datetime: row.get(2)?,
                    frequency: row.get::<_, i64>(3)?,
                    mode: row.get(4)?,
                    rst_sent: row.get(5)?,
                    rst_rcvd: row.get(6)?,
                    locator: row.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut qsos = Vec::with_capacity(rows.len());
        for r in rows {
            let dt = DateTime::parse_from_rfc3339(&r.datetime)
                .with_context(|| format!("parse datetime '{}' (qso #{})", r.datetime, r.id))?
                .with_timezone(&Utc);
            qsos.push(Qso {
                id: Some(r.id),
                callsign: r.callsign,
                qso_datetime: dt,
                frequency: r.frequency as u64,
                mode: r.mode,
                rst_sent: r.rst_sent,
                rst_rcvd: r.rst_rcvd,
                locator: r.locator,
            });
        }
        Ok(qsos)
    }

    pub fn count_qsos(&self, log_id: i64) -> Result<i64> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) FROM qso WHERE log_id = ?1",
            params![log_id],
            |r| r.get(0),
        )?)
    }
}

/// One-shot migration for early-development databases that still carry the legacy
/// `band` column. Adds `frequency` if missing, drops `band` if present. Both steps
/// are idempotent so it is safe to run on every open until the last legacy DB is gone.
fn migrate_band_to_frequency(conn: &Connection) -> Result<()> {
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(qso)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    if !columns.iter().any(|c| c == "frequency") {
        conn.execute(
            "ALTER TABLE qso ADD COLUMN frequency TEXT NOT NULL DEFAULT ''",
            [],
        )
        .context("add frequency column")?;
    }
    if columns.iter().any(|c| c == "band") {
        conn.execute("ALTER TABLE qso DROP COLUMN band", [])
            .context("drop band column")?;
    }
    Ok(())
}

struct RawRow {
    id: i64,
    callsign: String,
    datetime: String,
    frequency: i64,
    mode: String,
    rst_sent: String,
    rst_rcvd: String,
    locator: String,
}

/// One-shot migration converting a legacy text-typed `frequency` column into
/// integer Hz. SQLite assigns affinity per column declaration: a column declared
/// `TEXT` coerces every integer we insert back to a string, so `row.get::<_, i64>`
/// fails on read. Fix by rebuilding the table with `INTEGER` affinity and
/// translating the cell values on the way:
/// - `"14.200.00"` (pre-Hz UI format) → parsed via [`parse_legacy_frequency`]
/// - `"14200000"` (already-Hz, but stored as text by the old column) → direct parse
fn migrate_frequency_text_to_hz(conn: &Connection) -> Result<()> {
    let column_type: Option<String> = conn
        .prepare("SELECT type FROM pragma_table_info('qso') WHERE name = 'frequency'")?
        .query_row([], |row| row.get(0))
        .ok();

    let is_text = column_type
        .as_deref()
        .map(|t| t.eq_ignore_ascii_case("TEXT"))
        .unwrap_or(false);
    if !is_text {
        return Ok(());
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute_batch("ALTER TABLE qso RENAME TO qso_legacy;")
        .context("rename legacy qso table")?;
    // Re-create the qso table from the canonical schema. This intentionally
    // omits the legacy log_id-less DDL because by the time this migration is
    // first relevant the schema already carries log_id (default 0 for legacy
    // rows; the introduce-logs migration backfills them shortly after).
    tx.execute_batch(SCHEMA).context("recreate qso table")?;

    let rows: Vec<LegacyRow> = tx
        .prepare(
            "SELECT id, callsign, qso_datetime, frequency, mode, rst_sent, rst_rcvd, locator
             FROM qso_legacy",
        )?
        .query_map([], |row| {
            Ok(LegacyRow {
                id: row.get(0)?,
                callsign: row.get(1)?,
                datetime: row.get(2)?,
                frequency: row.get(3)?,
                mode: row.get(4)?,
                rst_sent: row.get(5)?,
                rst_rcvd: row.get(6)?,
                locator: row.get(7)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    for r in rows {
        let trimmed = r.frequency.trim();
        let hz: u64 = if trimmed.is_empty() {
            0
        } else if trimmed.contains('.') {
            parse_legacy_frequency(trimmed).unwrap_or(0)
        } else {
            trimmed.parse().unwrap_or(0)
        };
        tx.execute(
            "INSERT INTO qso (id, callsign, qso_datetime, frequency, mode, rst_sent, rst_rcvd, locator)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                r.id,
                r.callsign,
                r.datetime,
                hz as i64,
                r.mode,
                r.rst_sent,
                r.rst_rcvd,
                r.locator,
            ],
        )
        .with_context(|| format!("copy qso #{}", r.id))?;
    }

    tx.execute_batch("DROP TABLE qso_legacy;")
        .context("drop legacy qso table")?;
    tx.commit().context("commit frequency migration")?;
    Ok(())
}

struct LegacyRow {
    id: i64,
    callsign: String,
    datetime: String,
    frequency: String,
    mode: String,
    rst_sent: String,
    rst_rcvd: String,
    locator: String,
}

/// Bring single-log databases up to the multi-log schema: ensure the `log` table
/// is populated with at least one (active) row, add the `log_id` column to `qso`
/// if it is missing, and backfill any unassigned QSOs into the active log.
/// Idempotent — safe to run on every open.
fn migrate_introduce_logs(conn: &Connection, default_log_name: &str) -> Result<()> {
    let log_count: i64 = conn.query_row("SELECT COUNT(*) FROM log", [], |r| r.get(0))?;
    let default_id = if log_count == 0 {
        conn.execute(
            "INSERT INTO log (name, kind, is_active) VALUES (?1, 'general', 1)",
            params![default_log_name],
        )
        .context("insert default log")?;
        conn.last_insert_rowid()
    } else {
        let active: Option<i64> = conn
            .query_row("SELECT id FROM log WHERE is_active = 1 LIMIT 1", [], |r| {
                r.get(0)
            })
            .ok();
        match active {
            Some(id) => id,
            None => {
                let first: i64 = conn.query_row(
                    "SELECT id FROM log ORDER BY created_at ASC, id ASC LIMIT 1",
                    [],
                    |r| r.get(0),
                )?;
                conn.execute(
                    "UPDATE log SET is_active = 1 WHERE id = ?1",
                    params![first],
                )?;
                first
            }
        }
    };

    let qso_cols: Vec<String> = conn
        .prepare("PRAGMA table_info(qso)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if !qso_cols.iter().any(|c| c == "log_id") {
        conn.execute(
            "ALTER TABLE qso ADD COLUMN log_id INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .context("add log_id column")?;
    }
    // Create the log_id index here (not in SCHEMA) so a legacy database whose
    // ALTER TABLE just happened still gets indexed, while a fresh database
    // built from SCHEMA picks it up on first open.
    conn.execute("CREATE INDEX IF NOT EXISTS idx_qso_log_id ON qso(log_id)", [])
        .context("create log_id index")?;

    conn.execute(
        "UPDATE qso SET log_id = ?1 WHERE log_id = 0",
        params![default_id],
    )
    .context("backfill qso.log_id")?;
    Ok(())
}

fn db_path() -> Result<PathBuf> {
    let dir = dirs::config_dir().ok_or_else(|| anyhow!("no platform config_dir available"))?;
    Ok(dir.join("brlog").join("brlog.sqlite"))
}
