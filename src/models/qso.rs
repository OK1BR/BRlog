use chrono::{DateTime, Utc};

use crate::app::{Band, Mode};

#[derive(Debug, Clone)]
pub struct Qso {
    /// `None` for an unsaved QSO; populated by the DB on read. Used by future edit/delete flows.
    #[allow(dead_code)]
    pub id: Option<i64>,
    pub callsign: String,
    pub qso_datetime: DateTime<Utc>,
    pub band: Band,
    pub mode: Mode,
    pub rst_sent: String,
    pub rst_rcvd: String,
    pub locator: String,
}

impl Qso {
    pub fn new_now(
        callsign: String,
        band: Band,
        mode: Mode,
        rst_sent: String,
        rst_rcvd: String,
        locator: String,
    ) -> Self {
        Self {
            id: None,
            callsign,
            qso_datetime: Utc::now(),
            band,
            mode,
            rst_sent,
            rst_rcvd,
            locator,
        }
    }
}
