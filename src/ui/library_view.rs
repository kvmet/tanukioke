use eframe::egui;
use crate::library::Song;

pub fn render(ui: &mut egui::Ui, _songs: &[Song]) {
    ui.heading("Library");

    // TODO: Display list of songs
    // TODO: Add search/filter functionality
    // TODO: Handle song selection

    ui.label("No songs loaded yet");
}
