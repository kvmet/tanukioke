use serde::{Deserialize, Serialize};

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
    pub fn load() -> anyhow::Result<Self> {
        // TODO: Load from ~/.config/tanukioke/config.toml
        Ok(Self::default())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        // TODO: Save to ~/.config/tanukioke/config.toml
        Ok(())
    }
}
