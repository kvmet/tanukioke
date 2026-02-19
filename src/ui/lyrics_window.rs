use eframe::egui;
use crate::lrx::LrxFile;
use crate::app::PlaybackState;
use crate::config::Config;
use std::sync::{Arc, Mutex};

pub struct LyricsWindow {
    playback_state: Arc<Mutex<PlaybackState>>,
    lyrics: Option<LrxFile>,
    config: Config,
    // Store center Y position in content space for each lyric line
    line_centers: Vec<f32>,
}

impl LyricsWindow {
    pub fn new(playback_state: Arc<Mutex<PlaybackState>>, lyrics: Option<LrxFile>, config: Config) -> Self {
        Self {
            playback_state,
            lyrics,
            config,
            line_centers: Vec::new(),
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, window_height: f32) -> bool {
        let state = self.playback_state.lock().unwrap();
        let current_position = state.position;
        let duration = state.duration;
        drop(state);

        // First pass: measure line heights to build stable content-space positions
        self.measure_line_positions(ctx, window_height);

        // Calculate scroll offset based on stable measurements
        let scroll_y = self.calculate_scroll_offset(current_position, window_height);

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

                        // Render all lines
                        for (i, line) in lyrics.lines.iter().enumerate() {
                            let is_current = Some(i) == current_line_idx;
                            let is_past = current_line_idx.map(|c| i < c).unwrap_or(false);

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

    /// Measure line heights in content space (independent of scrolling)
    fn measure_line_positions(&mut self, ctx: &egui::Context, window_height: f32) {
        self.line_centers.clear();

        let lyrics = match &self.lyrics {
            Some(l) => l,
            None => return,
        };

        if lyrics.lines.is_empty() {
            return;
        }

        // Use egui's font system to measure heights without rendering
        let mut cumulative_y = window_height / 2.0; // Start after top padding

        for line in &lyrics.lines {
            let font_size = self.config.lyrics_font_size;

            // Use egui's font system to measure text height
            let font_id = egui::FontId::proportional(font_size);
            let galley = ctx.fonts(|fonts| {
                fonts.layout_no_wrap(line.text.clone(), font_id, egui::Color32::WHITE)
            });

            let line_height = galley.rect.height();

            // Center of this line in content space
            let line_center = cumulative_y + line_height / 2.0;
            self.line_centers.push(line_center);

            // Move to next line position
            cumulative_y += line_height;
        }
    }

    /// Calculate scroll position to center the appropriate line based on time
    fn calculate_scroll_offset(&self, current_position: f64, window_height: f32) -> f32 {
        let lyrics = match &self.lyrics {
            Some(l) => l,
            None => return 0.0,
        };

        if lyrics.lines.is_empty() || self.line_centers.is_empty() {
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

        let viewport_center = window_height / 2.0;

        match (current_idx, next_idx) {
            (Some(current), Some(next)) => {
                // Lerp between line centers based on time
                let current_center = self.line_centers[current];
                let next_center = self.line_centers[next];

                let current_time = lyrics.lines[current].timestamp;
                let next_time = lyrics.lines[next].timestamp;
                let time_range = next_time - current_time;

                if time_range > 0.0 {
                    let progress = ((current_position - current_time) / time_range) as f32;
                    let progress = progress.clamp(0.0, 1.0);

                    // Interpolate between the two centers in content space
                    let target_center = current_center + (next_center - current_center) * progress;

                    // Scroll so that target_center is at viewport center
                    target_center - viewport_center
                } else {
                    current_center - viewport_center
                }
            }
            (Some(current), None) => {
                // At or past last lyric
                self.line_centers[current] - viewport_center
            }
            (None, Some(next)) => {
                // Before first lyric - stay at top
                0.0
            }
            (None, None) => 0.0,
        }
    }
}
