use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LyricLine {
    pub timestamp: f64, // seconds
    pub text: String,
}

impl LyricLine {
    pub fn new(timestamp: f64, text: String) -> Self {
        Self { timestamp, text }
    }
}

#[derive(Debug, Clone)]
pub struct Lyrics {
    pub metadata: HashMap<String, String>,
    pub lines: Vec<LyricLine>,
}

impl Lyrics {
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            lines: Vec::new(),
        }
    }

    pub fn parse(_content: &str) -> anyhow::Result<Self> {
        // TODO: Parse LRC file format
        Ok(Self::new())
    }

    pub fn to_string(&self) -> String {
        // TODO: Serialize to LRC format
        String::new()
    }
}
