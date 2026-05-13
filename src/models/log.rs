use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogKind {
    #[default]
    General,
    Contest,
    Dxpedition,
    Sota,
}

impl LogKind {
    pub const ALL: &'static [LogKind] = &[
        LogKind::General,
        LogKind::Contest,
        LogKind::Dxpedition,
        LogKind::Sota,
    ];

    pub fn as_db_str(self) -> &'static str {
        match self {
            LogKind::General => "general",
            LogKind::Contest => "contest",
            LogKind::Dxpedition => "dxpedition",
            LogKind::Sota => "sota",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "contest" => LogKind::Contest,
            "dxpedition" => LogKind::Dxpedition,
            "sota" => LogKind::Sota,
            _ => LogKind::General,
        }
    }
}

impl fmt::Display for LogKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = match self {
            LogKind::General => "log-kind-general",
            LogKind::Contest => "log-kind-contest",
            LogKind::Dxpedition => "log-kind-dxpedition",
            LogKind::Sota => "log-kind-sota",
        };
        f.write_str(&crate::i18n::tr(key))
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub id: i64,
    pub name: String,
    pub kind: LogKind,
    pub is_active: bool,
}

impl PartialEq for Log {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Log {}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}
