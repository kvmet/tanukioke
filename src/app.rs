use eframe::egui;

pub struct App {
    // Placeholder for future state
}

impl App {
    pub fn new() -> Self {
        Self {}
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tanukioke");
            ui.label("Karaoke player - coming soon!");
        });
    }
}
