use eframe::egui;
use crate::lrx::LrxFile;

pub struct LyricsWindow {
    pub lyrics: Option<LrxFile>,
    pub current_position: f64, // seconds
    pub duration: f64,         // total duration in seconds
}

impl LyricsWindow {
    pub fn new() -> Self {
        Self {
            lyrics: None,
            current_position: 0.0,
            duration: 0.0,
        }
    }

    pub fn load_lyrics(&mut self, lyrics: LrxFile) {
        self.lyrics = Some(lyrics);
    }

    pub fn update_position(&mut self, position: f64) {
        self.current_position = position;
    }

    pub fn set_duration(&mut self, duration: f64) {
        self.duration = duration;
    }

    fn find_current_line_index(&self) -> Option<usize> {
        // TODO: Binary search for performance
        if let Some(lyrics) = &self.lyrics {
            for (i, line) in lyrics.lines.iter().enumerate().rev() {
                if line.timestamp <= self.current_position {
                    return Some(i);
                }
            }
        }
        None
    }
}

impl eframe::App for LyricsWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // TODO: Set background color (maybe from metadata?)
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);

                if let Some(lyrics) = &self.lyrics {
                    let current_line = self.find_current_line_index();

                    // TODO: Show previous/next lines with reduced opacity
                    // TODO: Display lines with their fg/bg colors
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

            let progress = if self.duration > 0.0 {
                (self.current_position / self.duration) as f32
            } else {
                0.0
            };

            ui.add(
                egui::ProgressBar::new(progress)
                    .show_percentage()
                    .animate(true)
            );

            ui.add_space(10.0);
        });

        // Request repaint to keep animation smooth
        ctx.request_repaint();
    }
}

pub fn spawn_lyrics_window() -> anyhow::Result<()> {
    // TODO: Spawn secondary window
    // TODO: Pass shared state for lyrics and position
    // TODO: Handle window close events
    Ok(())
}
