use eframe::egui;
use crate::lrx::LrxFile;
use crate::app::PlaybackState;
use crate::config::Config;
use std::sync::{Arc, Mutex};

pub struct LyricsWindow {
    playback_state: Arc<Mutex<PlaybackState>>,
    lyrics: Option<LrxFile>,
    config: Config,
    // Store measured heights for each lyric line
    line_heights: Vec<f32>,
}

impl LyricsWindow {
    pub fn new(playback_state: Arc<Mutex<PlaybackState>>, lyrics: Option<LrxFile>, config: Config) -> Self {
        Self {
            playback_state,
            lyrics,
            config,
            line_heights: Vec::new(),
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, window_height: f32) -> bool {
        let state = self.playback_state.lock().unwrap();
        let current_position = state.position;
        let duration = state.duration;
        drop(state);

        // Calculate scroll offset based on stable height measurements
        let scroll_y = self.calculate_scroll_offset(current_position, window_height);

        // Clear line heights for this frame's measurements
        self.line_heights.clear();

        // Get global background color
        let bg_color = if let Some(lyrics) = &self.lyrics {
            lyrics.background_color
                .or_else(|| {
                    self.config.lyrics_default_bg_color.as_ref()
                        .and_then(|s| Self::parse_hex_color(s))
                })
        } else {
            None
        };

        let mut central_panel = egui::CentralPanel::default();
        if let Some(bg_color) = bg_color {
            central_panel = central_panel.frame(
                egui::Frame::default()
                    .fill(bg_color)
                    .inner_margin(egui::Margin::same(0))
            );
        }

        central_panel.show(ctx, |ui| {
            let scroll_area = egui::ScrollArea::vertical()
                .id_salt("lyrics_scroll_area")
                .auto_shrink([false; 2])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .scroll_offset([0.0, scroll_y].into());

            scroll_area.show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    // Set item spacing to our configured value
                    ui.spacing_mut().item_spacing.y = self.config.lyrics_line_spacing;

                    // Add top padding to allow centering
                    ui.add_space(window_height / 2.0);

                    if let Some(lyrics) = &self.lyrics {
                        let current_line_idx = self.find_current_line_index(current_position);
                        let font_size = self.config.lyrics_font_size;

                        // Render all lines and measure their heights
                        for (i, line) in lyrics.lines.iter().enumerate() {
                            let is_current = Some(i) == current_line_idx;
                            let is_past = current_line_idx.map(|c| i < c).unwrap_or(false);

                            // Measure height before rendering
                            let before_y = ui.cursor().top();

                            // Color fallback hierarchy: part > lrx global > config default
                            let fg_color = if let Some(part_id) = &line.part_id {
                                if let Some(part) = lyrics.get_part(part_id) {
                                    part.color
                                } else {
                                    // Part doesn't exist, fall back to global/config
                                    self.get_default_color(lyrics)
                                }
                            } else {
                                // No part specified, use global/config defaults
                                self.get_default_color(lyrics)
                            };

                            let opacity = if is_current {
                                self.config.lyrics_opacity_current
                            } else if is_past {
                                self.config.lyrics_opacity_past
                            } else {
                                self.config.lyrics_opacity_upcoming
                            };

                            let mut text = egui::RichText::new(&line.text)
                                .size(font_size)
                                .color(fg_color.linear_multiply(opacity));

                            // Apply global font weight uniformly to all lines
                            if self.config.lyrics_font_weight >= 600.0 {
                                text = text.strong();
                            }

                            ui.label(text);

                            // Measure height after rendering
                            let after_y = ui.cursor().top();
                            let line_height = after_y - before_y;

                            // Store measured height (stable, independent of scroll position)
                            self.line_heights.push(line_height);
                        }
                    } else {
                        ui.heading("Tanukioke!!");
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

    /// Calculate scroll position to center the appropriate line based on time
    fn calculate_scroll_offset(&self, current_position: f64, window_height: f32) -> f32 {
        let lyrics = match &self.lyrics {
            Some(l) => l,
            None => return 0.0,
        };

        if lyrics.lines.is_empty() || self.line_heights.is_empty() {
            return 0.0;
        }

        // Calculate stable content-space positions from measured heights
        let line_spacing = self.config.lyrics_line_spacing;
        let mut line_centers = Vec::new();
        let mut cumulative_y = window_height / 2.0; // Top padding

        for &height in self.line_heights.iter() {
            let line_center = cumulative_y + height / 2.0;
            line_centers.push(line_center);

            // Height already includes egui's spacing
            cumulative_y += height;
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
                let current_center = line_centers[current];
                let next_center = line_centers[next];

                let current_time = lyrics.lines[current].timestamp;
                let next_time = lyrics.lines[next].timestamp;
                let time_range = next_time - current_time;

                if time_range > 0.0 {
                    let progress = ((current_position - current_time) / time_range) as f32;
                    let progress = progress.clamp(0.0, 1.0);

                    // Apply exponential ease-in for a "snap into place" effect
                    // This makes the transition start slow and accelerate as it approaches the target
                    // Snappiness of 0 = linear, higher values = more exponential
                    let snappiness = self.playback_state.lock().unwrap().lyrics_snappiness;
                    let eased_progress = if snappiness == 0.0 {
                        progress
                    } else {
                        2.0_f32.powf(snappiness * (progress - 1.0))
                    };

                    // Interpolate between the two centers in content space
                    let target_center = current_center + (next_center - current_center) * eased_progress;

                    // Scroll so that target_center is at viewport center
                    target_center - viewport_center
                } else {
                    current_center - viewport_center
                }
            }
            (Some(current), None) => {
                // At or past last lyric
                line_centers[current] - viewport_center
            }
            (None, Some(next)) => {
                // Before first lyric - stay at top
                0.0
            }
            (None, None) => 0.0,
        }
    }

    /// Get default foreground color with fallback: lrx global > config default
    fn get_default_color(&self, lyrics: &LrxFile) -> egui::Color32 {
        lyrics.color
            .or_else(|| Self::parse_hex_color(&self.config.lyrics_default_fg_color))
            .unwrap_or(egui::Color32::WHITE)
    }

    /// Parse a hex color string like "#RRGGBB"
    fn parse_hex_color(s: &str) -> Option<egui::Color32> {
        if !s.starts_with('#') || s.len() != 7 {
            return None;
        }

        let r = u8::from_str_radix(&s[1..3], 16).ok()?;
        let g = u8::from_str_radix(&s[3..5], 16).ok()?;
        let b = u8::from_str_radix(&s[5..7], 16).ok()?;

        Some(egui::Color32::from_rgb(r, g, b))
    }
}
