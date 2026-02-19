use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Track {
    pub path: PathBuf,
    pub volume: f32,
}

impl Track {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            volume: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Song {
    pub folder: PathBuf,
    pub tracks: Vec<Track>,
    pub lrc_path: Option<PathBuf>,
}

impl Song {
    pub fn new(folder: PathBuf) -> Self {
        Self {
            folder,
            tracks: Vec::new(),
            lrc_path: None,
        }
    }
}

pub fn scan_library(_path: &str) -> anyhow::Result<Vec<Song>> {
    // TODO: Walk directory and discover songs
    Ok(Vec::new())
}
