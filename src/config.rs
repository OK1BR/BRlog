use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::theme::AppTheme;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default)]
    pub operator: OperatorConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct OperatorConfig {
    pub callsign: String,
    pub name: String,
    pub qth: String,
    pub locator: String,
    pub license_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub theme: AppTheme,
    pub window_border: bool,
    pub language: Language,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: AppTheme::default(),
            window_border: true,
            language: Language::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Follow the OS locale at startup.
    #[default]
    Auto,
    Czech,
    English,
}

impl Language {
    pub const ALL: &'static [Language] = &[Language::Auto, Language::Czech, Language::English];
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = match self {
            Language::Auto => "language-auto",
            Language::Czech => "language-czech",
            Language::English => "language-english",
        };
        f.write_str(&crate::i18n::tr(key))
    }
}

impl AppConfig {
    /// Load from disk; on any error log to stderr and return Default.
    /// Migrates legacy flat-format configs to the new sectioned format.
    pub fn load() -> Self {
        match try_load() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("[config] load failed, using default: {e:#}");
                Self::default()
            }
        }
    }

    /// Atomic save: write to `<path>.tmp`, then rename over the target.
    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create config dir {}", parent.display()))?;
        }

        let serialized = toml::to_string_pretty(self).context("serialize config to TOML")?;

        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, serialized).with_context(|| format!("write {}", tmp.display()))?;
        fs::rename(&tmp, &path)
            .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
        Ok(())
    }
}

fn try_load() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;

    // Try the current sectioned format first.
    if let Ok(cfg) = toml::from_str::<AppConfig>(&raw) {
        return Ok(cfg);
    }

    // Fall back to legacy flat OperatorConfig (pre-sectioning) and migrate.
    if let Ok(operator) = toml::from_str::<OperatorConfig>(&raw) {
        eprintln!("[config] migrating legacy flat config to sectioned format");
        let cfg = AppConfig {
            operator,
            appearance: AppearanceConfig::default(),
        };
        if let Err(e) = cfg.save() {
            eprintln!("[config] migration save failed: {e:#}");
        }
        return Ok(cfg);
    }

    Err(anyhow!("could not parse {} in any known format", path.display()))
}

fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir().ok_or_else(|| anyhow!("no platform config_dir available"))?;
    Ok(dir.join("brlog").join("config.toml"))
}
