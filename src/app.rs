use eframe::egui;
use std::sync::{Arc, Mutex};


/// Shared playback state that can be accessed from multiple windows
#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub position: f64,      // Current position in seconds
    pub duration: f64,      // Total duration in seconds
    pub is_playing: bool,   // Whether audio is playing
    pub is_paused: bool,    // Whether audio is paused
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            position: 0.0,
            duration: 0.0,
            is_playing: false,
            is_paused: false,
        }
    }
}

impl PlaybackState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct App {
    pub playback_state: Arc<Mutex<PlaybackState>>,
    pub audio_engine: Arc<Mutex<crate::audio::AudioEngine>>,
    show_lyrics_window: bool,
    lyrics_window: Option<crate::ui::lyrics_window::LyricsWindow>,
    config: crate::config::Config,
    library_songs: Vec<crate::library::Song>,
}

impl App {
    pub fn new() -> Self {
        let config = crate::config::Config::load().unwrap_or_default();

        let audio_engine = crate::audio::AudioEngine::new()
            .expect("Failed to initialize audio engine");

        let playback_state = Arc::new(Mutex::new(PlaybackState::new()));
        let audio_engine = Arc::new(Mutex::new(audio_engine));

        // Scan library on startup
        let library_songs = if let Some(library_path) = &config.library_path {
            match crate::library::scan_library(library_path) {
                Ok(songs) => {
                    println!("Scanned library: found {} songs", songs.len());
                    songs
                }
                Err(e) => {
                    eprintln!("Failed to scan library: {}", e);
                    Vec::new()
                }
            }
        } else {
            println!("No library path configured");
            Vec::new()
        };

        Self {
            playback_state,
            audio_engine,
            show_lyrics_window: false,
            lyrics_window: None,
            config,
            library_songs,
        }
    }

    /// Load a song from an LRX file and its associated audio tracks
    pub fn load_song(&mut self, lrx_path: std::path::PathBuf) -> anyhow::Result<()> {
        use anyhow::Context;

        // Read and parse LRX file
        let content = std::fs::read_to_string(&lrx_path)
            .with_context(|| format!("Failed to read LRX file: {}", lrx_path.display()))?;

        let lrx = crate::lrx::LrxFile::parse(&content)
            .with_context(|| format!("Failed to parse LRX file: {}", lrx_path.display()))?;

        // Get the directory containing the LRX file (for resolving relative audio paths)
        let song_dir = lrx_path.parent()
            .ok_or_else(|| anyhow::anyhow!("LRX file has no parent directory"))?
            .to_path_buf();

        // Prepare track info for audio engine
        let track_infos: Vec<(String, String, std::path::PathBuf, f32)> = lrx.tracks
            .values()
            .map(|track| {
                (
                    track.id.clone(),
                    track.name.clone(),
                    track.source.clone(),
                    track.volume,
                )
            })
            .collect();

        // Load tracks into audio engine
        let mut engine = self.audio_engine.lock().unwrap();
        engine.set_base_dir(song_dir.clone());
        engine.load_tracks(track_infos)
            .context("Failed to load audio tracks")?;

        // Update playback state duration
        let duration = engine.duration();
        drop(engine);

        let mut state = self.playback_state.lock().unwrap();
        state.duration = duration.as_secs_f64();
        state.position = 0.0;
        drop(state);

        // Update lyrics window if it exists
        self.lyrics_window = Some(
            crate::ui::lyrics_window::LyricsWindow::new(
                self.playback_state.clone(),
                Some(lrx),
                self.config.clone()
            )
        );

        println!("Loaded song from: {}", lrx_path.display());
        Ok(())
    }

}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update playback state from audio engine
        {
            let mut engine = self.audio_engine.lock().unwrap();
            let mut state = self.playback_state.lock().unwrap();
            engine.update_playback_state(&mut state);
        }

        // Show lyrics window as a separate viewport if requested
        if self.show_lyrics_window {
            if let Some(mut lyrics_window) = self.lyrics_window.take() {
                let mut should_close = false;

                ctx.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("lyrics_window"),
                    egui::ViewportBuilder::default()
                        .with_title("Lyrics")
                        .with_inner_size([800.0, 600.0]),
                    |ctx, _class| {
                        let window_height = ctx.screen_rect().height();

                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.style_mut().visuals.panel_fill = egui::Color32::from_rgb(20, 20, 30);
                        });

                        should_close = lyrics_window.render(ctx, window_height);
                    },
                );

                if should_close {
                    self.show_lyrics_window = false;
                    // lyrics_window is dropped
                } else {
                    self.lyrics_window = Some(lyrics_window);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tanukioke");
            ui.label("Karaoke player");

            ui.separator();
            ui.add_space(20.0);

            // Player controls
            crate::ui::player::render(ui, &self.audio_engine, &self.playback_state);

            ui.add_space(20.0);

            // Button to toggle lyrics window
            if ui.button("🎤 Open Lyrics Window").clicked() {
                self.show_lyrics_window = true;
            }

            ui.separator();
            ui.add_space(20.0);

            // Library view
            ui.collapsing("Library", |ui| {
                let is_playing = {
                    let state = self.playback_state.lock().unwrap();
                    state.is_playing
                };

                if let Some(action) = crate::ui::library_view::render(ui, &self.library_songs, is_playing) {
                    match action {
                        crate::ui::library_view::LibraryAction::Load(path) => {
                            match self.load_song(path) {
                                Ok(_) => {
                                    self.show_lyrics_window = true;
                                }
                                Err(e) => {
                                    eprintln!("Failed to load song: {}", e);
                                }
                            }
                        }
                        crate::ui::library_view::LibraryAction::Enqueue(path) => {
                            // TODO: Implement queue functionality
                            println!("Enqueue not yet implemented: {:?}", path);
                        }
                    }
                }
            });

            ui.separator();
            ui.add_space(20.0);

            // Debug controls for testing
            ui.collapsing("Debug Controls", |ui| {
                ui.label("Load a song from the library:");

                if ui.button("📁 Load Random Song").clicked() {
                    if let Some(library_path) = &self.config.library_path {
                        match crate::library::scan_library(library_path) {
                            Ok(songs) => {
                                let songs_with_lrx: Vec<_> = songs.iter()
                                    .filter(|s| s.lrx_path.is_some())
                                    .collect();

                                if songs_with_lrx.is_empty() {
                                    eprintln!("No songs with LRX files found in library");
                                } else {
                                    // Pick a random song
                                    use std::collections::hash_map::RandomState;
                                    use std::hash::{BuildHasher, Hash, Hasher};
                                    let s = RandomState::new();
                                    let mut hasher = s.build_hasher();
                                    std::time::SystemTime::now().hash(&mut hasher);
                                    let index = (hasher.finish() as usize) % songs_with_lrx.len();

                                    if let Some(lrx_path) = &songs_with_lrx[index].lrx_path {
                                        match self.load_song(lrx_path.clone()) {
                                            Ok(_) => {
                                                println!("Loaded: {}", songs_with_lrx[index].title());
                                                self.show_lyrics_window = true;
                                            }
                                            Err(e) => eprintln!("Failed to load song: {}", e),
                                        }
                                    }
                                }
                            }
                            Err(e) => eprintln!("Failed to scan library: {}", e),
                        }
                    } else {
                        eprintln!("No library path configured");
                    }
                }
            });
        });

        // Request repaint for smooth UI updates
        ctx.request_repaint();
    }
}

impl App {
}
