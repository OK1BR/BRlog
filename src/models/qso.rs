use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Qso {
    /// `None` for an unsaved QSO; populated by the DB on read. Used by future edit/delete flows.
    #[allow(dead_code)]
    pub id: Option<i64>,
    pub callsign: String,
    pub qso_datetime: DateTime<Utc>,
    /// Frequency in kHz as a free-form string. Currently hardcoded in the entry form;
    /// later phases will sync this from the transceiver via TCI.
    pub frequency: String,
    /// Operating mode (SSB/CW/FT8/…). Free-form string; later phases sync it from the
    /// transceiver via TCI.
    pub mode: String,
    pub rst_sent: String,
    pub rst_rcvd: String,
    pub locator: String,
}

impl Qso {
    pub fn new_now(
        callsign: String,
        frequency: String,
        mode: String,
        rst_sent: String,
        rst_rcvd: String,
        locator: String,
    ) -> Self {
        Self {
            id: None,
            callsign,
            qso_datetime: Utc::now(),
            frequency,
            mode,
            rst_sent,
            rst_rcvd,
            locator,
        }
    }
}
