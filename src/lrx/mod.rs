use std::collections::HashMap;
use std::path::PathBuf;
use eframe::egui::Color32;

pub mod parse;
pub mod serialize;

#[derive(Debug, Clone)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub source: PathBuf,
    pub volume: f32,
}

impl Track {
    pub fn new(id: String, name: String, source: PathBuf) -> Self {
        Self {
            id,
            name,
            source,
            volume: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub fg_color: Color32,
    pub bg_color: Option<Color32>,
}

impl Part {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            fg_color: Color32::WHITE,
            bg_color: None,
        }
    }

    pub fn with_colors(id: String, name: String, fg_color: Color32, bg_color: Option<Color32>) -> Self {
        Self {
            id,
            name,
            fg_color,
            bg_color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LyricLine {
    pub timestamp: f64, // seconds
    pub text: String,
    pub part_id: Option<String>, // References a Part by id
}

impl LyricLine {
    pub fn new(timestamp: f64, text: String) -> Self {
        Self {
            timestamp,
            text,
            part_id: None,
        }
    }

    pub fn with_part(timestamp: f64, text: String, part_id: String) -> Self {
        Self {
            timestamp,
            text,
            part_id: Some(part_id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LrxFile {
    pub metadata: HashMap<String, String>,
    pub tracks: HashMap<String, Track>,
    pub parts: HashMap<String, Part>,
    pub lines: Vec<LyricLine>,
}

impl LrxFile {
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            tracks: HashMap::new(),
            parts: HashMap::new(),
            lines: Vec::new(),
        }
    }

    pub fn get_part(&self, part_id: &str) -> Option<&Part> {
        self.parts.get(part_id)
    }

    pub fn get_track(&self, track_id: &str) -> Option<&Track> {
        self.tracks.get(track_id)
    }
}
