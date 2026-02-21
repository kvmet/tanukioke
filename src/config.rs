use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub library_path: Option<String>,

    // Lyrics display settings
    #[serde(default = "default_opacity_current")]
    pub lyrics_opacity_current: f32,

    #[serde(default = "default_opacity_upcoming")]
    pub lyrics_opacity_upcoming: f32,

    #[serde(default = "default_opacity_past")]
    pub lyrics_opacity_past: f32,

    #[serde(default = "default_font_size")]
    pub lyrics_font_size: f32,

    #[serde(default = "default_line_spacing")]
    pub lyrics_line_spacing: f32,

    #[serde(default = "default_font_weight")]
    pub lyrics_font_weight: f32,

    #[serde(default = "default_fg_color")]
    pub lyrics_default_fg_color: String,

    #[serde(default = "default_bg_color")]
    pub lyrics_default_bg_color: Option<String>,

    #[serde(default = "default_snappiness")]
    pub lyrics_snappiness: f32,

    #[serde(default = "default_timing_offset")]
    pub lyrics_timing_offset: f64,
}

fn default_opacity_current() -> f32 { 1.0 }
fn default_opacity_upcoming() -> f32 { 0.7 }
fn default_opacity_past() -> f32 { 0.4 }
fn default_font_size() -> f32 { 36.0 }
fn default_line_spacing() -> f32 { 16.0 }
fn default_font_weight() -> f32 { 400.0 }
fn default_fg_color() -> String { "#FFFFFF".to_string() }
fn default_bg_color() -> Option<String> { None }
fn default_snappiness() -> f32 { 15.0 }
fn default_timing_offset() -> f64 { 0.0 }

impl Default for Config {
    fn default() -> Self {
        Self {
            library_path: None,
            lyrics_opacity_current: default_opacity_current(),
            lyrics_opacity_upcoming: default_opacity_upcoming(),
            lyrics_opacity_past: default_opacity_past(),
            lyrics_font_size: default_font_size(),
            lyrics_line_spacing: default_line_spacing(),
            lyrics_font_weight: default_font_weight(),
            lyrics_default_fg_color: default_fg_color(),
            lyrics_default_bg_color: default_bg_color(),
            lyrics_snappiness: default_snappiness(),
            lyrics_timing_offset: default_timing_offset(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = PathBuf::from("config.toml");
        println!("Looking for config at: {:?}", config_path);

        if config_path.exists() {
            println!("Loading config from: {:?}", config_path);
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from {:?}", config_path))?;

            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config from {:?}", config_path))?;

            return Ok(config);
        }

        // No config file found, use defaults
        println!("No config file found, using defaults");
        Ok(Self::default())
    }

    pub fn save(&self) -> Result<()> {
        let config_path = PathBuf::from("config.toml");

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;

        Ok(())
    }
}
