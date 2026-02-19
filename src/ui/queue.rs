use eframe::egui;
use crate::queue::Queue;

pub fn render(ui: &mut egui::Ui, _queue: &mut Queue) {
    ui.heading("Queue");

    // TODO: Display current song playing
    // TODO: Display list of upcoming queue entries
    // TODO: Show singer name for each entry
    // TODO: Add button to remove entries
    // TODO: Add button to move to next song
    // TODO: Add input field to add new song with singer name
    // TODO: Maybe drag-and-drop reordering

    ui.label("Queue is empty");
}
