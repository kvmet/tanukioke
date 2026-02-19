use eframe::egui;
use crate::audio::{AudioEngine, PlaybackState};

pub fn render(ui: &mut egui::Ui, _engine: &mut AudioEngine) {
    ui.heading("Player");

    // TODO: Transport controls (play/pause/stop)
    // TODO: Seek bar
    // TODO: Time display (current / total)
    // TODO: Per-track volume sliders

    ui.horizontal(|ui| {
        if ui.button("Play").clicked() {
            // TODO: Start playback
        }
        if ui.button("Pause").clicked() {
            // TODO: Pause playback
        }
        if ui.button("Stop").clicked() {
            // TODO: Stop playback
        }
    });

    ui.separator();
    ui.label("Volume controls will go here");
}
