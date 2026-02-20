use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn render(
    ui: &mut egui::Ui,
    audio_engine: &Arc<Mutex<crate::audio::AudioEngine>>,
    playback_state: &Arc<Mutex<crate::app::PlaybackState>>,
    config: &mut crate::config::Config,
) {
    // Top section: Track info + transport (left) and volumes (right)
    ui.horizontal(|ui| {
        // Left side: Track info and transport controls
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            ui.vertical(|ui| {
                // Lyrics snappiness control
                ui.horizontal(|ui| {
                    ui.label("Snappiness:");

                    let mut state = playback_state.lock().unwrap();

                    // Map internal value to 0-100 slider display range
                    let mut slider_value = if state.lyrics_snappiness >= 10000.0 {
                        100.0
                    } else {
                        state.lyrics_snappiness
                    };

                    if ui.add(egui::Slider::new(&mut slider_value, 0.0..=100.0)
                        .text("📊")
                        .fixed_decimals(0))
                        .changed()
                    {
                        // Map slider value: 100 = instant (10000), 0-99 = use as-is
                        state.lyrics_snappiness = if slider_value >= 100.0 {
                            10000.0
                        } else {
                            slider_value
                        };

                        // Sync to config for persistence
                        config.lyrics_snappiness = state.lyrics_snappiness;
                        let _ = config.save();
                    }
                    drop(state);
                });

                ui.add_space(5.0);

                // Track details (placeholder)
                ui.heading("Track Title");
                ui.label("Artist Name");
                ui.label("Album Name");

                ui.add_space(5.0);

                // Transport controls
                ui.horizontal(|ui| {
                    let state = playback_state.lock().unwrap();
                    let is_playing = state.is_playing;
                    let is_paused = state.is_paused;
                    drop(state);

                    if ui.add_sized([60.0, 35.0], egui::Button::new("⏵")).clicked() {
                        let mut engine = audio_engine.lock().unwrap();
                        engine.play();
                    }

                    if ui.add_sized([60.0, 35.0], egui::Button::new("⏸")).clicked() {
                        let mut engine = audio_engine.lock().unwrap();
                        engine.pause();
                    }

                    if ui.add_sized([60.0, 35.0], egui::Button::new("⏹")).clicked() {
                        let mut engine = audio_engine.lock().unwrap();
                        engine.stop();
                    }

                    ui.add_space(10.0);

                    // Status indicator
                    let status = if is_playing && !is_paused {
                        "⏵ Playing"
                    } else if is_paused {
                        "⏸ Paused"
                    } else {
                        "⏹ Stopped"
                    };
                    ui.label(status);
                });
            });
        });

        // Right side: Volume controls (right-aligned, fixed width, scrollable)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(300.0, 100.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::ScrollArea::vertical()
                        .show(ui, |ui| {
                            let mut engine = audio_engine.lock().unwrap();
                            let tracks = engine.tracks_mut();

                            if tracks.is_empty() {
                                ui.label("No tracks loaded");
                            } else {
                                for track in tracks {
                                    ui.horizontal(|ui| {
                                        ui.label(&track.name);

                                        let mut volume = track.get_volume();
                                        if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0)
                                            .text("🔊")
                                            .fixed_decimals(2))
                                            .changed()
                                        {
                                            track.set_volume(volume);
                                        }

                                        ui.label(format!("{}%", (volume * 100.0) as i32));
                                    });
                                }
                            }
                        });
                }
            );
        });
    });

    ui.separator();

    // Bottom section: Full-width seek bar spanning the entire app width
    ui.horizontal(|ui| {
        let state = playback_state.lock().unwrap();
        let position = state.position;
        let duration = state.duration;

        ui.label(format_time(position));

        let mut pos_f32 = position as f32;
        let max = if duration > 0.0 { duration as f32 } else { 300.0 };

        // Try to make slider fill available space
        ui.style_mut().spacing.slider_width = ui.available_width() - 55.0;

        let slider = egui::Slider::new(&mut pos_f32, 0.0..=max)
            .show_value(false);

        if ui.add(slider).changed() {
            drop(state);
            let mut engine = audio_engine.lock().unwrap();
            let _ = engine.seek(std::time::Duration::from_secs_f64(pos_f32 as f64));
        } else {
            drop(state);
        }

        // Push the end timestamp to the right edge
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let state = playback_state.lock().unwrap();
            ui.label(format_time(state.duration));
        });
    });
}

fn format_time(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as i32;
    let secs = (seconds % 60.0).floor() as i32;
    format!("{:02}:{:02}", minutes, secs)
}
