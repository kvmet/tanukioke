use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SongMetadata {
    pub artist: String,
    pub album: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegistryEntry {
    artist: String,
    album: String,
    title: String,
    lrx_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct LibraryRegistry {
    songs: Vec<RegistryEntry>,
}

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
    metadata_cache: Arc<Mutex<Option<SongMetadata>>>,
}

impl Song {
    pub fn new(folder: PathBuf) -> Self {
        Self {
            folder,
            tracks: Vec::new(),
            lrx_path: None,
            metadata_cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn title(&self) -> String {
        self.folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// Get metadata (artist, album) from the LRX file, with caching
    pub fn get_metadata(&self) -> SongMetadata {
        // Check cache first
        {
            let cache = self.metadata_cache.lock().unwrap();
            if let Some(metadata) = cache.as_ref() {
                return metadata.clone();
            }
        }

        // Parse LRX file to extract metadata
        let metadata = if let Some(lrx_path) = &self.lrx_path {
            if let Ok(content) = std::fs::read_to_string(lrx_path) {
                if let Ok(lrx) = crate::lrx::LrxFile::parse(&content) {
                    SongMetadata {
                        artist: lrx.metadata.get("ar").cloned().unwrap_or_default(),
                        album: lrx.metadata.get("al").cloned().unwrap_or_default(),
                        title: lrx.metadata.get("ti").cloned().unwrap_or_default(),
                    }
                } else {
                    SongMetadata::default()
                }
            } else {
                SongMetadata::default()
            }
        } else {
            SongMetadata::default()
        };

        // Cache it
        *self.metadata_cache.lock().unwrap() = Some(metadata.clone());
        metadata
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

/// Load library from registry file if it exists, otherwise scan and create registry
pub fn load_or_scan_library(path: &str) -> Result<Vec<Song>> {
    let library_path = PathBuf::from(path);
    let registry_path = library_path.join("library.toml");

    // Try to load from registry first
    if registry_path.exists() {
        match load_registry(&registry_path) {
            Ok(songs) => {
                println!("Loaded library from registry: {} songs", songs.len());
                return Ok(songs);
            }
            Err(e) => {
                eprintln!("Failed to load registry, will rescan: {}", e);
            }
        }
    }

    // Registry doesn't exist or failed to load, scan the library
    println!("Scanning library...");
    let songs = scan_library(path)?;

    // Save registry for next time
    if let Err(e) = save_registry(&registry_path, &songs) {
        eprintln!("Warning: Failed to save library registry: {}", e);
    }

    Ok(songs)
}

/// Save library registry to file
pub fn save_registry(path: &PathBuf, songs: &[Song]) -> Result<()> {
    let entries: Vec<RegistryEntry> = songs
        .iter()
        .filter_map(|song| {
            let lrx_path = song.lrx_path.as_ref()?;
            let metadata = song.get_metadata();
            Some(RegistryEntry {
                artist: metadata.artist,
                album: metadata.album,
                title: metadata.title,
                lrx_path: lrx_path.clone(),
            })
        })
        .collect();

    let registry = LibraryRegistry { songs: entries };

    // Create content with header comment
    let mut content = String::from("# Tanukioke Library Registry\n");
    content.push_str("# This file is automatically generated. Do not edit manually.\n");
    content.push_str("# Delete this file to force a full library rescan.\n\n");
    content.push_str(&toml::to_string_pretty(&registry)
        .context("Failed to serialize library registry")?);

    std::fs::write(path, &content)
        .with_context(|| format!("Failed to write registry to {:?}", path))?;

    println!("Saved library registry: {} songs", registry.songs.len());
    Ok(())
}

/// Load library from registry file
fn load_registry(path: &PathBuf) -> Result<Vec<Song>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read registry from {:?}", path))?;

    let registry: LibraryRegistry = toml::from_str(&content)
        .context("Failed to parse library registry")?;

    let songs: Vec<Song> = registry
        .songs
        .into_iter()
        .map(|entry| {
            let folder = entry.lrx_path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default();

            let mut song = Song::new(folder);
            song.lrx_path = Some(entry.lrx_path);

            // Pre-populate metadata cache
            let metadata = SongMetadata {
                artist: entry.artist,
                album: entry.album,
                title: entry.title,
            };
            *song.metadata_cache.lock().unwrap() = Some(metadata);

            song
        })
        .collect();

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
