use eframe::egui;
use crate::lrx::LrxFile;
use crate::app::PlaybackState;
use crate::config::Config;
use std::sync::{Arc, Mutex};

pub struct LyricsWindow {
    playback_state: Arc<Mutex<PlaybackState>>,
    lyrics: Option<LrxFile>,
    config: Config,
    // Store measured center Y positions for each lyric line from previous frame
    line_positions: Vec<(usize, f32)>,
}

impl LyricsWindow {
    pub fn new(playback_state: Arc<Mutex<PlaybackState>>, lyrics: Option<LrxFile>, config: Config) -> Self {
        Self {
            playback_state,
            lyrics,
            config,
            line_positions: Vec::new(),
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, window_height: f32) -> bool {
        let state = self.playback_state.lock().unwrap();
        let current_position = state.position;
        let duration = state.duration;
        drop(state);

        // Calculate scroll from previous frame's measurements
        let scroll_y = self.calculate_scroll_from_measurements(current_position, window_height);

        // Clear line positions for this frame's measurements
        self.line_positions.clear();

        egui::CentralPanel::default().show(ctx, |ui| {
            let scroll_area = egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .scroll_offset([0.0, scroll_y].into());

            scroll_area.show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    // Add top padding to allow centering
                    ui.add_space(window_height / 2.0);

                    if let Some(lyrics) = &self.lyrics {
                        let current_line_idx = self.find_current_line_index(current_position);
                        let font_size = self.config.lyrics_font_size;

                        // Render all lines and measure their positions
                        for (i, line) in lyrics.lines.iter().enumerate() {
                            let is_current = Some(i) == current_line_idx;
                            let is_past = current_line_idx.map(|c| i < c).unwrap_or(false);

                            // Measure position before rendering
                            let before_y = ui.cursor().top();

                            let (fg_color, _bg_color) = if let Some(part_id) = &line.part_id {
                                if let Some(part) = lyrics.get_part(part_id) {
                                    (part.fg_color, part.bg_color)
                                } else {
                                    (egui::Color32::WHITE, None)
                                }
                            } else {
                                (egui::Color32::WHITE, None)
                            };

                            let opacity = if is_current {
                                self.config.lyrics_opacity_current
                            } else if is_past {
                                self.config.lyrics_opacity_past
                            } else {
                                self.config.lyrics_opacity_upcoming
                            };

                            let text = egui::RichText::new(&line.text)
                                .size(font_size)
                                .color(fg_color.linear_multiply(opacity));

                            let text = if is_current {
                                text.strong()
                            } else {
                                text
                            };

                            ui.label(text);

                            // Measure position after rendering - get center of the line
                            let after_y = ui.cursor().top();
                            let line_center_y = (before_y + after_y) / 2.0;

                            // Store measured center position for this line
                            self.line_positions.push((i, line_center_y));
                        }
                    } else {
                        ui.heading("No lyrics loaded");
                    }

                    // Add bottom padding
                    ui.add_space(window_height / 2.0);
                });
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

        // Request repaint for smooth updates
        ctx.request_repaint();

        // Return whether window should close
        ctx.input(|i| i.viewport().close_requested())
    }

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

    /// Calculate scroll position based on measured line center positions
    /// Lerps between centers so each line arrives at viewport center at its timestamp
    fn calculate_scroll_from_measurements(&self, current_position: f64, window_height: f32) -> f32 {
        let lyrics = match &self.lyrics {
            Some(l) => l,
            None => return 0.0,
        };

        if lyrics.lines.is_empty() || self.line_positions.is_empty() {
            return 0.0;
        }

        // Find which two lyrics we're between
        let mut current_idx = None;
        let mut next_idx = None;

        for (i, line) in lyrics.lines.iter().enumerate() {
            if line.timestamp <= current_position {
                current_idx = Some(i);
            } else {
                next_idx = Some(i);
                break;
            }
        }

        // Find measured center position for a line
        let find_center_y = |idx: usize| -> Option<f32> {
            self.line_positions.iter()
                .find(|(i, _)| *i == idx)
                .map(|(_, y)| *y)
        };

        let center_y = window_height / 2.0;

        match (current_idx, next_idx) {
            (Some(current), Some(next)) => {
                // Lerp between measured center positions
                if let (Some(current_center), Some(next_center)) = (find_center_y(current), find_center_y(next)) {
                    let current_time = lyrics.lines[current].timestamp;
                    let next_time = lyrics.lines[next].timestamp;
                    let time_range = next_time - current_time;

                    if time_range > 0.0 {
                        let progress = ((current_position - current_time) / time_range) as f32;
                        let progress = progress.clamp(0.0, 1.0);

                        // Interpolate between the two center positions
                        let target_center = current_center + (next_center - current_center) * progress;

                        // Scroll so that target_center is at viewport center
                        target_center - center_y
                    } else {
                        current_center - center_y
                    }
                } else {
                    0.0
                }
            }
            (Some(current), None) => {
                // At or past last lyric
                if let Some(current_center) = find_center_y(current) {
                    current_center - center_y
                } else {
                    0.0
                }
            }
            (None, Some(next)) => {
                // Before first lyric
                if let Some(next_center) = find_center_y(next) {
                    next_center - center_y
                } else {
                    0.0
                }
            }
            (None, None) => 0.0,
        }
    }
}
