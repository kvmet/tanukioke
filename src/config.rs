use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub library_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            library_path: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config from {:?}", config_path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config from {:?}", config_path))?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;

        Ok(())
    }

    fn config_file_path() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        Ok(config_dir.join("config.toml"))
    }

    fn config_dir() -> Result<PathBuf> {
        // Try to get config directory in a cross-platform way
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return Ok(PathBuf::from(home).join(".config").join("tanukioke"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Try XDG_CONFIG_HOME first
            if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
                return Ok(PathBuf::from(xdg_config).join("tanukioke"));
            }
            // Fall back to ~/.config
            if let Ok(home) = std::env::var("HOME") {
                return Ok(PathBuf::from(home).join(".config").join("tanukioke"));
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return Ok(PathBuf::from(appdata).join("tanukioke"));
            }
        }

        // Final fallback
        anyhow::bail!("Could not determine config directory")
    }
}
