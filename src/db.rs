use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection};

use crate::models::qso::Qso;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS qso (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    callsign      TEXT NOT NULL,
    qso_datetime  TEXT NOT NULL,
    frequency     TEXT NOT NULL DEFAULT '',
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
                    qso.frequency,
                    qso.mode,
                    qso.rst_sent,
                    qso.rst_rcvd,
                    qso.locator,
                ],
            )
            .context("insert qso")?;
        Ok(self.conn.last_insert_rowid())
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
                    frequency: row.get(3)?,
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
                frequency: r.frequency,
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
