use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct OperatorConfig {
    pub callsign: String,
    pub name: String,
    pub qth: String,
    pub locator: String,
    pub license_class: String,
}

impl OperatorConfig {
    /// Load from disk; on any error log to stderr and return Default.
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

fn try_load() -> Result<OperatorConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(OperatorConfig::default());
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let cfg: OperatorConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(cfg)
}

pub fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir().ok_or_else(|| anyhow!("no platform config_dir available"))?;
    Ok(dir.join("brlog").join("config.toml"))
}
