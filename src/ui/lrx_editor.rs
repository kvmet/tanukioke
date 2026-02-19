use eframe::egui;

pub fn render(ui: &mut egui::Ui) {
    ui.heading("LRX Editor");
    ui.separator();

    ui.label("Editor functionality coming soon...");

    ui.add_space(10.0);

    // Placeholder for future editor features
    ui.collapsing("Future Features", |ui| {
        ui.label("• Edit LRX metadata (artist, title, album)");
        ui.label("• Edit track assignments");
        ui.label("• Edit lyrics and timing");
        ui.label("• Add/remove parts");
        ui.label("• Color customization");
        ui.label("• Save changes");
    });
}
