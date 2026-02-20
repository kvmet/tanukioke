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
    library_search_query: String,
    show_rescan_confirm: bool,
    queue: crate::queue::Queue,
    show_add_manual: bool,
    add_manual_dialog: Option<crate::ui::queue::AddManualDialog>,
    show_add_from_library: bool,
    add_from_library_dialog: Option<crate::ui::queue::AddFromLibraryDialog>,
    show_edit_queue: bool,
    edit_entry_dialog: Option<crate::ui::queue::EditEntryDialog>,
    show_editor_window: bool,
    editor_state: crate::ui::lrx_editor::EditorState,
}

impl App {
    pub fn new() -> Self {
        let config = crate::config::Config::load().unwrap_or_default();

        let audio_engine = crate::audio::AudioEngine::new()
            .expect("Failed to initialize audio engine");

        let playback_state = Arc::new(Mutex::new(PlaybackState::new()));
        let audio_engine = Arc::new(Mutex::new(audio_engine));

        // Load library from registry or scan on startup
        let library_songs = if let Some(library_path) = &config.library_path {
            match crate::library::load_or_scan_library(library_path) {
                Ok(songs) => {
                    songs
                }
                Err(e) => {
                    eprintln!("Failed to load library: {}", e);
                    Vec::new()
                }
            }
        } else {
            println!("No library path configured");
            Vec::new()
        };

        let lyrics_window = Some(crate::ui::lyrics_window::LyricsWindow::new(
            playback_state.clone(),
            None,
            config.clone(),
        ));

        Self {
            playback_state,
            audio_engine,
            show_lyrics_window: true,
            lyrics_window,
            config,
            library_songs,
            library_search_query: String::new(),
            show_rescan_confirm: false,
            queue: crate::queue::Queue::new(),
            show_add_manual: false,
            add_manual_dialog: None,
            show_add_from_library: false,
            add_from_library_dialog: None,
            show_edit_queue: false,
            edit_entry_dialog: None,
            show_editor_window: false,
            editor_state: crate::ui::lrx_editor::EditorState::new(),
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
                ctx.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("lyrics_window"),
                    egui::ViewportBuilder::default()
                        .with_title("Lyrics")
                        .with_inner_size([800.0, 600.0])
                        .with_close_button(false),
                    |ctx, _class| {
                        let window_height = ctx.screen_rect().height();

                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.style_mut().visuals.panel_fill = egui::Color32::from_rgb(20, 20, 30);
                        });

                        lyrics_window.render(ctx, window_height);
                    },
                );

                self.lyrics_window = Some(lyrics_window);
            }
        }

        // Show editor window as a separate viewport if requested
        if self.show_editor_window {
            let mut should_close = false;

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("editor_window"),
                egui::ViewportBuilder::default()
                    .with_title("LRX Editor")
                    .with_inner_size([800.0, 600.0]),
                |ctx, _class| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let playback_position = {
                            let state = self.playback_state.lock().unwrap();
                            if state.is_playing || state.is_paused {
                                Some(state.position)
                            } else {
                                None
                            }
                        };

                        if let Some(action) = crate::ui::lrx_editor::render(ui, &mut self.editor_state, playback_position) {
                            match action {
                                crate::ui::lrx_editor::EditorAction::Save(path, content) => {
                                    match std::fs::write(&path, content) {
                                        Ok(_) => {
                                            // Reload to update original_content
                                            if let Ok(new_content) = std::fs::read_to_string(&path) {
                                                self.editor_state.load(path, new_content);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to save file: {}", e);
                                        }
                                    }
                                }
                                crate::ui::lrx_editor::EditorAction::Close => {
                                    should_close = true;
                                }
                            }
                        }
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        if self.editor_state.is_dirty() {
                            self.editor_state.show_close_confirm = true;
                        } else {
                            should_close = true;
                        }
                    }
                },
            );

            if should_close {
                self.show_editor_window = false;
                self.editor_state.clear();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Top section - Player controls
            egui::TopBottomPanel::top("player_panel").show_inside(ui, |ui| {
                // Player controls
                crate::ui::player::render(ui, &self.audio_engine, &self.playback_state);
            });

            // Bottom section - Library (2/3) and Queue (1/3)
            egui::CentralPanel::default().show_inside(ui, |ui| {
                use egui_extras::{StripBuilder, Size};

                StripBuilder::new(ui)
                    .size(Size::relative(0.66).at_least(300.0)) // Library - 2/3 width
                    .size(Size::exact(1.0)) // Separator
                    .size(Size::remainder().at_least(200.0)) // Queue - remaining width
                    .horizontal(|mut strip| {
                        strip.cell(|ui| {
                            let is_playing = {
                                let state = self.playback_state.lock().unwrap();
                                state.is_playing
                            };

                            if let Some(action) = crate::ui::library_view::render(
                                ui,
                                &self.library_songs,
                                is_playing,
                                &mut self.library_search_query,
                                &mut self.show_rescan_confirm
                            ) {
                                match action {
                                    crate::ui::library_view::LibraryAction::Load(path) => {
                                        match self.load_song(path) {
                                            Ok(_) => {
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to load song: {}", e);
                                            }
                                        }
                                    }
                                    crate::ui::library_view::LibraryAction::Enqueue(path) => {
                                        // Get song title from metadata
                                        let song_title = if let Some(song) = self.library_songs.iter().find(|s| s.lrx_path.as_ref() == Some(&path)) {
                                            let metadata = song.get_metadata();
                                            if !metadata.title.is_empty() {
                                                metadata.title.clone()
                                            } else {
                                                path.file_stem()
                                                    .and_then(|s| s.to_str())
                                                    .unwrap_or("Unknown")
                                                    .to_string()
                                            }
                                        } else {
                                            path.file_stem()
                                                .and_then(|s| s.to_str())
                                                .unwrap_or("Unknown")
                                                .to_string()
                                        };

                                        self.add_from_library_dialog = Some(crate::ui::queue::AddFromLibraryDialog {
                                            name: String::new(),
                                            song_title,
                                            path,
                                        });
                                        self.show_add_from_library = true;
                                    }
                                    crate::ui::library_view::LibraryAction::Rescan => {
                                        if let Some(library_path) = &self.config.library_path {
                                            match crate::library::scan_library(library_path) {
                                                Ok(songs) => {
                                                    println!("Rescanned library: found {} songs", songs.len());

                                                    // Save registry
                                                    let library_path_buf = std::path::PathBuf::from(library_path);
                                                    let registry_path = library_path_buf.join("library.toml");
                                                    if let Err(e) = crate::library::save_registry(&registry_path, &songs) {
                                                        eprintln!("Warning: Failed to save library registry: {}", e);
                                                    }

                                                    self.library_songs = songs;
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to rescan library: {}", e);
                                                }
                                            }
                                        }
                                    }
                                    crate::ui::library_view::LibraryAction::Edit(path) => {
                                        match std::fs::read_to_string(&path) {
                                            Ok(content) => {
                                                self.editor_state.load(path, content);
                                                self.show_editor_window = true;
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to open file: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        });

                        strip.cell(|ui| {
                            ui.separator();
                        });

                        strip.cell(|ui| {
                            // Queue view
                            let is_playing = {
                                let state = self.playback_state.lock().unwrap();
                                state.is_playing
                            };

                            if let Some(action) = crate::ui::queue::render(ui, &self.queue, is_playing) {
                                match action {
                                    crate::ui::queue::QueueAction::Load(path) => {
                                        match self.load_song(path) {
                                            Ok(_) => {
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to load song: {}", e);
                                            }
                                        }
                                    }
                                    crate::ui::queue::QueueAction::Edit(id) => {
                                        if let Some(entry) = self.queue.get(id) {
                                            self.edit_entry_dialog = Some(crate::ui::queue::EditEntryDialog {
                                                id: entry.id,
                                                name: entry.singer_name.clone(),
                                                song: entry.song_title.clone(),
                                                url: entry.url.clone().unwrap_or_default(),
                                                is_library_entry: entry.lrx_path.is_some(),
                                            });
                                            self.show_edit_queue = true;
                                        }
                                    }
                                    crate::ui::queue::QueueAction::Delete(id) => {
                                        self.queue.remove(id);
                                    }
                                    crate::ui::queue::QueueAction::MoveUp(id) => {
                                        self.queue.move_up(id);
                                    }
                                    crate::ui::queue::QueueAction::MoveDown(id) => {
                                        self.queue.move_down(id);
                                    }
                                    crate::ui::queue::QueueAction::OpenUrl(url) => {
                                        // TODO: Add 'open' crate to Cargo.toml to enable URL opening
                                        println!("Open URL: {}", url);
                                    }
                                    crate::ui::queue::QueueAction::CopyUrl(url) => {
                                        ctx.copy_text(url);
                                    }
                                    crate::ui::queue::QueueAction::AddManual => {
                                        self.add_manual_dialog = Some(crate::ui::queue::AddManualDialog::default());
                                        self.show_add_manual = true;
                                    }

                                }
                            }
                        });
                    });
            });
        });

        // Handle dialogs
        if self.show_add_manual {
            if let Some(dialog) = &mut self.add_manual_dialog {
                if crate::ui::queue::render_add_manual_dialog(ctx, dialog, &mut self.queue) {
                    self.show_add_manual = false;
                    self.add_manual_dialog = None;
                }
            }
        }

        if self.show_add_from_library {
            if let Some(dialog) = &mut self.add_from_library_dialog {
                if crate::ui::queue::render_add_from_library_dialog(ctx, dialog, &mut self.queue) {
                    self.show_add_from_library = false;
                    self.add_from_library_dialog = None;
                }
            }
        }

        if self.show_edit_queue {
            if let Some(dialog) = &mut self.edit_entry_dialog {
                if crate::ui::queue::render_edit_entry_dialog(ctx, dialog, &mut self.queue) {
                    self.show_edit_queue = false;
                    self.edit_entry_dialog = None;
                }
            }
        }

        // Request repaint for smooth UI updates
        ctx.request_repaint();
    }
}

impl App {
}
