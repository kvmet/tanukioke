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
    show_lyrics_window: bool,
    lyrics: Option<crate::lrx::LrxFile>,
}

impl App {
    pub fn new() -> Self {
        Self {
            playback_state: Arc::new(Mutex::new(PlaybackState::new())),
            show_lyrics_window: false,
            lyrics: None,
        }
    }

    /// Format time in MM:SS format
    fn format_time(seconds: f64) -> String {
        let minutes = (seconds / 60.0).floor() as i32;
        let secs = (seconds % 60.0).floor() as i32;
        format!("{:02}:{:02}", minutes, secs)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show lyrics window as a separate viewport if requested
        if self.show_lyrics_window {
            let state = self.playback_state.lock().unwrap();
            let current_position = state.position;
            let duration = state.duration;
            drop(state);

            let mut close_window = false;

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("lyrics_window"),
                egui::ViewportBuilder::default()
                    .with_title("Lyrics")
                    .with_inner_size([800.0, 600.0]),
                |ctx, _class| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.style_mut().visuals.panel_fill = egui::Color32::from_rgb(20, 20, 30);

                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);

                            if let Some(lyrics) = &self.lyrics {
                                let current_line = self.find_current_line_index(current_position);

                                for (i, line) in lyrics.lines.iter().enumerate() {
                                    let is_current = Some(i) == current_line;

                                    let (fg_color, _bg_color) = if let Some(part_id) = &line.part_id {
                                        if let Some(part) = lyrics.get_part(part_id) {
                                            (part.fg_color, part.bg_color)
                                        } else {
                                            (egui::Color32::WHITE, None)
                                        }
                                    } else {
                                        (egui::Color32::WHITE, None)
                                    };

                                    let text = if is_current {
                                        egui::RichText::new(&line.text)
                                            .size(48.0)
                                            .color(fg_color)
                                            .strong()
                                    } else {
                                        egui::RichText::new(&line.text)
                                            .size(32.0)
                                            .color(fg_color.linear_multiply(0.5))
                                    };

                                    ui.label(text);
                                }
                            } else {
                                ui.heading("No lyrics loaded");
                            }
                        });
                    });

                    egui::TopBottomPanel::bottom("progress").show(ctx, |ui| {
                        ui.add_space(10.0);

                        let progress = if duration > 0.0 {
                            (current_position / duration) as f32
                        } else {
                            0.0
                        };

                        ui.add(
                            egui::ProgressBar::new(progress)
                                .show_percentage()
                                .animate(true)
                        );

                        ui.horizontal(|ui| {
                            let minutes = (current_position / 60.0).floor() as i32;
                            let secs = (current_position % 60.0).floor() as i32;
                            let total_minutes = (duration / 60.0).floor() as i32;
                            let total_secs = (duration % 60.0).floor() as i32;

                            ui.label(format!("{:02}:{:02} / {:02}:{:02}", minutes, secs, total_minutes, total_secs));
                        });

                        ui.add_space(10.0);
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        close_window = true;
                    }
                },
            );

            if close_window {
                self.show_lyrics_window = false;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tanukioke");
            ui.label("Karaoke player");

            ui.separator();
            ui.add_space(20.0);

            // Transport controls
            ui.vertical_centered(|ui| {
                let mut state = self.playback_state.lock().unwrap();

                // Play/Pause/Stop buttons
                ui.horizontal(|ui| {
                    ui.add_space(20.0);

                    if ui.add_sized([80.0, 40.0], egui::Button::new("⏵ Play")).clicked() {
                        state.is_playing = true;
                        state.is_paused = false;
                        println!("Play clicked");
                    }

                    if ui.add_sized([80.0, 40.0], egui::Button::new("⏸ Pause")).clicked() {
                        state.is_paused = !state.is_paused;
                        println!("Pause clicked (paused: {})", state.is_paused);
                    }

                    if ui.add_sized([80.0, 40.0], egui::Button::new("⏹ Stop")).clicked() {
                        state.is_playing = false;
                        state.is_paused = false;
                        state.position = 0.0;
                        println!("Stop clicked");
                    }
                });

                ui.add_space(20.0);

                // Time display and seek slider
                ui.horizontal(|ui| {
                    ui.add_space(20.0);

                    // Current time
                    ui.label(Self::format_time(state.position));

                    ui.add_space(10.0);

                    // Seek slider
                    let mut position = state.position as f32;
                    let max_duration = if state.duration > 0.0 { state.duration } else { 300.0 }; // Default 5 min max

                    let slider = egui::Slider::new(&mut position, 0.0..=max_duration as f32)
                        .show_value(false)
                        .fixed_decimals(1);

                    if ui.add(slider).changed() {
                        state.position = position as f64;
                        println!("Seek to: {:.2}s", state.position);
                    }

                    ui.add_space(10.0);

                    // Total time
                    ui.label(Self::format_time(state.duration));

                    ui.add_space(20.0);
                });

                ui.add_space(20.0);

                // Playback status
                let status = if state.is_playing && !state.is_paused {
                    "⏵ Playing"
                } else if state.is_paused {
                    "⏸ Paused"
                } else {
                    "⏹ Stopped"
                };
                ui.label(status);

                ui.add_space(20.0);

                // Button to toggle lyrics window
                if ui.button("🎤 Open Lyrics Window").clicked() {
                    self.show_lyrics_window = true;
                }
            });

            ui.separator();
            ui.add_space(20.0);

            // Debug controls for testing
            ui.collapsing("Debug Controls", |ui| {
                let mut state = self.playback_state.lock().unwrap();

                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    ui.add(egui::DragValue::new(&mut state.duration).speed(1.0).suffix(" s"));
                });

                ui.horizontal(|ui| {
                    if ui.button("Set 3:30 duration").clicked() {
                        state.duration = 210.0; // 3 minutes 30 seconds
                    }
                    if ui.button("Reset").clicked() {
                        state.position = 0.0;
                        state.duration = 0.0;
                        state.is_playing = false;
                        state.is_paused = false;
                    }
                });

                ui.separator();
                ui.label("Test with example file:");

                drop(state); // Release lock

                if ui.button("🎤 Open Lyrics (Random Library File)").clicked() {
                    // Load a random LRX file from the library for testing
                    match crate::config::Config::load() {
                        Ok(config) => {
                            if let Some(library_path) = config.library_path {
                                match crate::library::scan_library(&library_path) {
                                    Ok(songs) => {
                                        // Filter songs that have LRX files
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
                                                match std::fs::read_to_string(lrx_path) {
                                                    Ok(content) => {
                                                        match crate::lrx::LrxFile::parse(&content) {
                                                            Ok(lrx) => {
                                                                println!("Loaded: {}", songs_with_lrx[index].title());

                                                                // Set duration based on last lyric timestamp
                                                                let mut state = self.playback_state.lock().unwrap();
                                                                if state.duration == 0.0 && !lrx.lines.is_empty() {
                                                                    let last_timestamp = lrx.lines.last()
                                                                        .map(|line| line.timestamp)
                                                                        .unwrap_or(0.0);
                                                                    state.duration = last_timestamp + 10.0;
                                                                }
                                                                drop(state);

                                                                self.lyrics = Some(lrx);
                                                                self.show_lyrics_window = true;
                                                            }
                                                            Err(e) => eprintln!("Failed to parse LRX file: {}", e),
                                                        }
                                                    }
                                                    Err(e) => eprintln!("Failed to read LRX file: {}", e),
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
                        Err(e) => eprintln!("Failed to load config: {}", e),
                    }
                }
            });
        });

        // Request repaint for smooth UI updates
        ctx.request_repaint();
    }
}

impl App {
    fn find_current_line_index(&self, current_position: f64) -> Option<usize> {
        if let Some(lyrics) = &self.lyrics {
            for (i, line) in lyrics.lines.iter().enumerate().rev() {
                if line.timestamp <= current_position {
                    return Some(i);
                }
            }
        }
        None
    }
}
