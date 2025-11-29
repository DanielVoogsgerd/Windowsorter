use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use windowsorter::{AppType, Rules};

/// Representation of a single app entry in TOML
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub name: Option<String>,
    pub classes: Vec<String>,
    pub default_workspace: u32,
    #[serde(default)]
    pub forbidden: Vec<u32>,
    #[serde(default)]
    pub mandatory_workspace: Option<u32>,
}

/// Top-level config
#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub app: Vec<AppConfig>,
}

impl ConfigFile {
    /// Load config from $XDG_CONFIG_HOME/windowsorter/config.toml, falling
    /// back to $HOME/.config/windowsorter/config.toml when XDG is unset.
    pub fn load() -> Result<Option<Self>, anyhow::Error> {
        let cfg_path = xdg_config_path();
        if let Some(p) = cfg_path {
            if p.exists() {
                let s = fs::read_to_string(&p)?;
                let cfg: ConfigFile = toml::from_str(&s)?;
                return Ok(Some(cfg));
            }
        }
        Ok(None)
    }

    /// Convert to runtime Rules
    pub fn to_rules(&self) -> Rules {
        let mut app_types = Vec::new();
        for a in &self.app {
            let classes: HashSet<String> = a.classes.iter().map(|s| s.to_lowercase()).collect();
            let forbidden: HashSet<u32> = a.forbidden.iter().copied().collect();
            let name = a
                .name
                .clone()
                .unwrap_or_else(|| classes.iter().cloned().collect::<Vec<_>>().join(","));
            let app = AppType {
                name,
                classes,
                default_workspace: a.default_workspace,
                forbidden,
                mandatory_workspace: a.mandatory_workspace,
            };
            app_types.push(app);
        }

        Rules { app_types }
    }
}

fn xdg_config_path() -> Option<PathBuf> {
    if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
        let mut p = PathBuf::from(x);
        p.push("windowsorter/config.toml");
        return Some(p);
    }

    // No further fallback: if XDG_CONFIG_HOME and HOME are unset, return None.
    None
}
