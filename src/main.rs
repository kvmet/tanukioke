mod app;
mod audio;
mod config;
mod library;
mod lrc;
mod queue;
mod ui;

use eframe::egui;

fn main() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Tanukioke"),
        ..Default::default()
    };

    eframe::run_native(
        "Tanukioke",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new()))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))
}
