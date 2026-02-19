use eframe::egui;
use crate::lrc::Lyrics;

pub fn render(ui: &mut egui::Ui, _lyrics: Option<&Lyrics>, _current_position: f64) {
    ui.heading("Lyrics");

    // TODO: Display lyrics lines with timestamps
    // TODO: Highlight current line based on playback position
    // TODO: Add edit mode for modifying lyrics
    // TODO: Add buttons for adding/removing lines
    // TODO: Show metadata (artist, title, key, etc.)

    ui.label("No lyrics loaded");
}
