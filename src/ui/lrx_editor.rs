use eframe::egui;
use std::path::PathBuf;

pub struct EditorState {
    pub file_path: Option<PathBuf>,
    pub original_content: String,
    pub current_content: String,
    pub show_close_confirm: bool,
    pub show_save_confirm: bool,
    pub show_help: bool,
    pub text_edit_id: egui::Id,
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
            text_edit_id: egui::Id::new("lrx_editor_text"),
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

    pub fn insert_timestamp(&mut self, ui: &mut egui::Ui, timestamp_seconds: f64) {
        let timestamp = format_timestamp(timestamp_seconds);

        // Get cursor position from text edit state
        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), self.text_edit_id) {
            let cursor_pos = match state.cursor.char_range() {
                Some(range) => range.primary.index,
                None => return, // No cursor position available
            };

            // Find the current line
            let mut line_start = 0;
            let mut line_end = self.current_content.len();

            // Find line start
            for (i, c) in self.current_content.char_indices().rev() {
                if i >= cursor_pos {
                    continue;
                }
                if c == '\n' {
                    line_start = i + 1;
                    break;
                }
            }

            // Find line end
            for (i, c) in self.current_content.char_indices() {
                if i <= cursor_pos {
                    continue;
                }
                if c == '\n' {
                    line_end = i;
                    break;
                }
            }

            let line = &self.current_content[line_start..line_end];

            // Check if line already starts with a timestamp [mm:ss.xx]
            let has_timestamp = line.len() >= 11
                && line.starts_with('[')
                && line.chars().nth(10) == Some(']')
                && line.chars().nth(2) == Some(':')
                && line.chars().nth(5) == Some('.');

            let new_line = if has_timestamp {
                // Replace existing timestamp - find the closing bracket and keep everything after it
                if let Some(bracket_pos) = line.find(']') {
                    format!("{}{}", timestamp, &line[bracket_pos + 1..])
                } else {
                    format!("{}{}", timestamp, line)
                }
            } else {
                // Insert new timestamp at the beginning
                format!("{}{}", timestamp, line)
            };

            // Replace the line
            self.current_content.replace_range(line_start..line_end, &new_line);

            // Move cursor to the start of the next line
            // Check if there's a newline after this line
            let after_line_pos = line_start + new_line.len();
            if after_line_pos < self.current_content.len() {
                // Skip the newline character to go to the start of next line
                let new_cursor_pos = after_line_pos + 1;
                let ccursor = egui::text::CCursor::new(new_cursor_pos);
                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
            } else {
                // We're at the end of the file, stay at the end of the current line
                let ccursor = egui::text::CCursor::new(after_line_pos);
                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
            }

            // Store the updated state
            state.store(ui.ctx(), self.text_edit_id);
        }
    }
}

fn format_timestamp(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    let centiseconds = ((seconds % 1.0) * 100.0).floor() as u32;
    format!("[{:02}:{:02}.{:02}]", minutes, secs, centiseconds)
}

pub enum EditorAction {
    Save(PathBuf, String),
    Close,
}

pub fn render(ui: &mut egui::Ui, state: &mut EditorState, playback_position: Option<f64>) -> Option<EditorAction> {
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
    let available_height = ui.available_height() - 40.0; // Reserve space for timestamp button
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(available_height)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut state.current_content)
                    .id(state.text_edit_id)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace)
            );
        });

    ui.separator();

    // Timestamp insertion button
    let button_enabled = playback_position.is_some();
    let button_text = if let Some(pos) = playback_position {
        format!("⏱ Insert Timestamp at {}", format_timestamp(pos))
    } else {
        "⏱ Insert Timestamp (No playback)".to_string()
    };

    if ui.add_sized(
        [ui.available_width(), 30.0],
        egui::Button::new(button_text).fill(if button_enabled {
            egui::Color32::from_rgb(60, 100, 140)
        } else {
            egui::Color32::from_gray(60)
        })
    ).clicked() && button_enabled {
        if let Some(pos) = playback_position {
            state.insert_timestamp(ui, pos);
        }
    }

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
