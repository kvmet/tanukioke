use eframe::egui;
use crate::lrx::LrxFile;
use crate::app::PlaybackState;
use std::sync::{Arc, Mutex};

pub struct LyricsWindow {
    pub lyrics: Option<LrxFile>,
    pub playback_state: Arc<Mutex<PlaybackState>>,
}

impl LyricsWindow {
    pub fn new(playback_state: Arc<Mutex<PlaybackState>>) -> Self {
        Self {
            lyrics: None,
            playback_state,
        }
    }

    pub fn load_lyrics(&mut self, lyrics: LrxFile) {
        self.lyrics = Some(lyrics);
    }

    fn find_current_line_index(&self, current_position: f64) -> Option<usize> {
        // Binary search would be better for performance, but linear search works for now
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

impl eframe::App for LyricsWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let state = self.playback_state.lock().unwrap();
        let current_position = state.position;
        let duration = state.duration;
        drop(state); // Release lock early

        egui::CentralPanel::default().show(ctx, |ui| {
            // Set dark background for better visibility
            ui.style_mut().visuals.panel_fill = egui::Color32::from_rgb(20, 20, 30);

            ui.vertical_centered(|ui| {
                ui.add_space(50.0);

                if let Some(lyrics) = &self.lyrics {
                    let current_line = self.find_current_line_index(current_position);

                    // TODO: Show previous/next lines with reduced opacity
                    // TODO: Add smooth scrolling animation
                    // TODO: Handle word-by-word highlighting if supported

                    for (i, line) in lyrics.lines.iter().enumerate() {
                        let is_current = Some(i) == current_line;

                        // Get part colors if part_id is set
                        let (fg_color, bg_color) = if let Some(part_id) = &line.part_id {
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

                        // TODO: Apply background color if set
                        // Background color support would require custom rendering
                        // For now, just display the text
                        let _ = bg_color; // Suppress unused warning
                        ui.label(text);
                    }
                } else {
                    ui.heading("No lyrics loaded");
                }
            });
        });

        // Progress bar at bottom
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

            // Time display
            ui.horizontal(|ui| {
                let minutes = (current_position / 60.0).floor() as i32;
                let secs = (current_position % 60.0).floor() as i32;
                let total_minutes = (duration / 60.0).floor() as i32;
                let total_secs = (duration % 60.0).floor() as i32;

                ui.label(format!("{:02}:{:02} / {:02}:{:02}", minutes, secs, total_minutes, total_secs));
            });

            ui.add_space(10.0);
        });

        // Request repaint to keep animation smooth
        ctx.request_repaint();
    }
}

pub fn spawn_lyrics_window(playback_state: Arc<Mutex<PlaybackState>>) -> anyhow::Result<()> {
    spawn_lyrics_window_with_file(playback_state, None)
}

/// Spawn a lyrics window, optionally loading an LRX file from the given path
pub fn spawn_lyrics_window_with_file(
    playback_state: Arc<Mutex<PlaybackState>>,
    lrx_path: Option<std::path::PathBuf>,
) -> anyhow::Result<()> {
    use std::thread;

    thread::spawn(move || {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([800.0, 600.0])
                .with_title("Lyrics"),
            ..Default::default()
        };

        let _ = eframe::run_native(
            "Lyrics",
            options,
            Box::new(move |_cc| {
                let mut window = LyricsWindow::new(playback_state.clone());

                // Load lyrics from file if path provided
                if let Some(path) = lrx_path {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            match crate::lrx::LrxFile::parse(&content) {
                                Ok(lrx) => {
                                    window.load_lyrics(lrx);

                                    // Set a default duration for testing if not already set
                                    let mut state = window.playback_state.lock().unwrap();
                                    if state.duration == 0.0 && !window.lyrics.as_ref().unwrap().lines.is_empty() {
                                        // Set duration to last lyric timestamp + 10 seconds
                                        let last_timestamp = window.lyrics.as_ref()
                                            .and_then(|l| l.lines.last())
                                            .map(|line| line.timestamp)
                                            .unwrap_or(0.0);
                                        state.duration = last_timestamp + 10.0;
                                    }
                                }
                                Err(e) => eprintln!("Failed to parse LRX file: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Failed to read LRX file: {}", e),
                    }
                }

                Ok(Box::new(window))
            }),
        );
    });

    Ok(())
}
