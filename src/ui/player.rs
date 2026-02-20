use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn render(
    ui: &mut egui::Ui,
    audio_engine: &Arc<Mutex<crate::audio::AudioEngine>>,
    playback_state: &Arc<Mutex<crate::app::PlaybackState>>,
) {
    // Transport controls
    ui.horizontal(|ui| {
        let state = playback_state.lock().unwrap();
        let is_playing = state.is_playing;
        let is_paused = state.is_paused;
        drop(state);

        if ui.add_sized([80.0, 40.0], egui::Button::new("⏵ Play")).clicked() {
            let mut engine = audio_engine.lock().unwrap();
            engine.play();
        }

        if ui.add_sized([80.0, 40.0], egui::Button::new("⏸ Pause")).clicked() {
            let mut engine = audio_engine.lock().unwrap();
            engine.pause();
        }

        if ui.add_sized([80.0, 40.0], egui::Button::new("⏹ Stop")).clicked() {
            let mut engine = audio_engine.lock().unwrap();
            engine.stop();
        }

        ui.add_space(20.0);

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

    ui.add_space(10.0);

    // Time display and seek bar
    ui.horizontal(|ui| {
        let state = playback_state.lock().unwrap();
        let position = state.position;
        let duration = state.duration;

        ui.label(format_time(position));
        ui.add_space(5.0);

        let mut pos_f32 = position as f32;
        let max = if duration > 0.0 { duration as f32 } else { 300.0 };

        let slider = egui::Slider::new(&mut pos_f32, 0.0..=max)
            .show_value(false);

        if ui.add(slider).changed() {
            drop(state);
            let mut engine = audio_engine.lock().unwrap();
            let _ = engine.seek(std::time::Duration::from_secs_f64(pos_f32 as f64));
        } else {
            drop(state);
        }

        ui.add_space(5.0);
        let state = playback_state.lock().unwrap();
        ui.label(format_time(state.duration));
    });

    ui.add_space(20.0);
    ui.separator();

    // Per-track volume controls
    ui.heading("Track Volumes");
    ui.add_space(10.0);

    let mut engine = audio_engine.lock().unwrap();
    let tracks = engine.tracks_mut();

    if tracks.is_empty() {
        ui.label("No tracks loaded");
    } else {
        for track in tracks {
            ui.horizontal(|ui| {
                ui.label(&track.name);
                ui.add_space(10.0);

                let mut volume = track.get_volume();
                if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0)
                    .text("Vol")
                    .fixed_decimals(2))
                    .changed()
                {
                    track.set_volume(volume);
                }

                ui.label(format!("{}%", (volume * 100.0) as i32));
            });
        }
    }
}

fn format_time(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as i32;
    let secs = (seconds % 60.0).floor() as i32;
    format!("{:02}:{:02}", minutes, secs)
}
