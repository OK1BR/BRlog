use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection};

use crate::models::qso::{parse_legacy_frequency, Qso};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS qso (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
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

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open() -> Result<Self> {
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
        Ok(Self { conn })
    }

    pub fn insert_qso(&self, qso: &Qso) -> Result<i64> {
        let datetime_iso = qso
            .qso_datetime
            .to_rfc3339_opts(SecondsFormat::Secs, true);
        self.conn
            .execute(
                "INSERT INTO qso (callsign, qso_datetime, frequency, mode, rst_sent, rst_rcvd, locator)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
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

    pub fn list_qsos(&self) -> Result<Vec<Qso>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, callsign, qso_datetime, frequency, mode, rst_sent, rst_rcvd, locator
                 FROM qso
                 ORDER BY qso_datetime DESC, id DESC",
            )
            .context("prepare list_qsos")?;

        let rows: Vec<RawRow> = stmt
            .query_map([], |row| {
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

fn db_path() -> Result<PathBuf> {
    let dir = dirs::config_dir().ok_or_else(|| anyhow!("no platform config_dir available"))?;
    Ok(dir.join("brlog").join("brlog.sqlite"))
}
