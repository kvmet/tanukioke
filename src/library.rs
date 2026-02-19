use std::path::PathBuf;
use walkdir::WalkDir;
use anyhow::{Context, Result};

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
    pub lrx_path: Option<PathBuf>,
}

impl Song {
    pub fn new(folder: PathBuf) -> Self {
        Self {
            folder,
            tracks: Vec::new(),
            lrx_path: None,
        }
    }

    pub fn title(&self) -> String {
        self.folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }
}

pub fn scan_library(path: &str) -> Result<Vec<Song>> {
    let library_path = PathBuf::from(path);

    if !library_path.exists() {
        anyhow::bail!("Library path does not exist: {}", path);
    }

    if !library_path.is_dir() {
        anyhow::bail!("Library path is not a directory: {}", path);
    }

    let mut songs = Vec::new();
    let mut song_folders = std::collections::HashSet::new();

    // First pass: find all folders containing .lrx files
    for entry in WalkDir::new(&library_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "lrx" {
                    if let Some(parent) = entry.path().parent() {
                        song_folders.insert(parent.to_path_buf());
                    }
                }
            }
        }
    }

    // Second pass: build Song objects for each folder
    for folder in song_folders {
        let mut song = Song::new(folder.clone());

        // Find all files in this folder
        for entry in std::fs::read_dir(&folder)
            .with_context(|| format!("Failed to read directory: {:?}", folder))?
        {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext {
                    "lrx" => {
                        song.lrx_path = Some(path.clone());
                    }
                    "mp3" | "flac" | "wav" | "ogg" | "opus" => {
                        song.tracks.push(Track::new(path.clone()));
                    }
                    _ => {}
                }
            }
        }

        // Only include songs that have at least a .lrx file
        if song.lrx_path.is_some() {
            songs.push(song);
        }
    }

    // Sort by folder name for consistent ordering
    songs.sort_by(|a, b| a.folder.cmp(&b.folder));

    Ok(songs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_song_title() {
        let song = Song::new(PathBuf::from("/path/to/My Song"));
        assert_eq!(song.title(), "My Song");
    }
}
