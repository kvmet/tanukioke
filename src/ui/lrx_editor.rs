use eframe::egui;
use std::path::PathBuf;

pub struct EditorState {
    pub file_path: Option<PathBuf>,
    pub original_content: String,
    pub current_content: String,
    pub show_close_confirm: bool,
    pub show_save_confirm: bool,
    pub show_help: bool,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            file_path: None,
            original_content: String::new(),
            current_content: String::new(),
            show_close_confirm: false,
            show_save_confirm: false,
            show_help: false,
        }
    }

    pub fn load(&mut self, path: PathBuf, content: String) {
        self.file_path = Some(path);
        self.original_content = content.clone();
        self.current_content = content;
        self.show_close_confirm = false;
        self.show_save_confirm = false;
        self.show_help = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.original_content != self.current_content
    }

    pub fn clear(&mut self) {
        self.file_path = None;
        self.original_content.clear();
        self.current_content.clear();
        self.show_close_confirm = false;
        self.show_save_confirm = false;
        self.show_help = false;
    }
}

pub enum EditorAction {
    Save(PathBuf, String),
    Close,
}

pub fn render(ui: &mut egui::Ui, state: &mut EditorState) -> Option<EditorAction> {
    let mut action = None;

    // Top bar with buttons
    ui.horizontal(|ui| {
        if let Some(path) = &state.file_path {
            ui.label(format!("Editing: {}", path.display()));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Close button
            if ui.button("Close").clicked() {
                if state.is_dirty() {
                    state.show_close_confirm = true;
                } else {
                    action = Some(EditorAction::Close);
                }
            }

            // Save button - enabled only if dirty
            if ui.add_enabled(state.is_dirty(), egui::Button::new("💾 Save")).clicked() {
                state.show_save_confirm = true;
            }

            if state.is_dirty() {
                ui.label("*");
            }

            ui.separator();

            // Help button
            if ui.button("❓ Help").clicked() {
                state.show_help = true;
            }
        });
    });

    ui.separator();

    // Main text editor
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut state.current_content)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace)
            );
        });

    // Close confirmation dialog
    if state.show_close_confirm {
        egui::Window::new("Unsaved Changes")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("You have unsaved changes. Are you sure you want to close?");
                ui.horizontal(|ui| {
                    if ui.button("Close Without Saving").clicked() {
                        action = Some(EditorAction::Close);
                        state.show_close_confirm = false;
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_close_confirm = false;
                    }
                });
            });
    }

    // Save confirmation dialog
    if state.show_save_confirm {
        egui::Window::new("Confirm Save")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("Save changes to file?");
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        if let Some(path) = &state.file_path {
                            action = Some(EditorAction::Save(path.clone(), state.current_content.clone()));
                        }
                        state.show_save_confirm = false;
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_save_confirm = false;
                    }
                });
            });
    }

    // Help dialog
    if state.show_help {
        crate::ui::lrx_help::render(ui.ctx(), &mut state.show_help);
    }

    action
}
