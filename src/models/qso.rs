use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Qso {
    /// `None` for an unsaved QSO; populated by the DB on read. Used by future edit/delete flows.
    #[allow(dead_code)]
    pub id: Option<i64>,
    pub callsign: String,
    pub qso_datetime: DateTime<Utc>,
    /// Frequency in Hz. Later phases will sync this from the transceiver via TCI.
    pub frequency: u64,
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
        frequency: u64,
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

/// Format a Hz value as `MHz.kHz.Hz`. Example: `14_200_000` → `"14.200.000"`.
pub fn format_frequency_hz(hz: u64) -> String {
    let mhz = hz / 1_000_000;
    let khz = (hz % 1_000_000) / 1_000;
    let hz_part = hz % 1_000;
    format!("{mhz}.{khz:03}.{hz_part:03}")
}

/// Parse the legacy `MHz.kHz.HH` string format into Hz. Used by the one-shot DB
/// migration that converts text-typed `frequency` values to integer Hz.
pub fn parse_legacy_frequency(s: &str) -> Option<u64> {
    let mut parts = s.split('.');
    let mhz: u64 = parts.next()?.parse().ok()?;
    let khz: u64 = parts.next()?.parse().ok()?;
    let hh: u64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(mhz * 1_000_000 + khz * 1_000 + hh * 10)
}
