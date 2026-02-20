use eframe::egui;
use std::path::PathBuf;
use regex::Regex;

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
            let cursor_char_pos = match state.cursor.char_range() {
                Some(range) => range.primary.index,
                None => return, // No cursor position available
            };

            // Convert character position to byte position
            let cursor_pos = self.current_content
                .char_indices()
                .nth(cursor_char_pos)
                .map(|(byte_pos, _)| byte_pos)
                .unwrap_or(self.current_content.len());

            // Find the current line
            // Find line start - scan backwards from cursor to find previous newline
            let line_start = if cursor_pos > 0 {
                self.current_content[..cursor_pos]
                    .rfind('\n')
                    .map(|pos| pos + 1)
                    .unwrap_or(0)
            } else {
                0
            };

            // Find line end - scan forwards from line_start to find next newline
            let line_end = self.current_content[line_start..]
                .find('\n')
                .map(|pos| line_start + pos)
                .unwrap_or(self.current_content.len());

            let line = &self.current_content[line_start..line_end];

            // Check if line already starts with a timestamp [mm:ss.xx] or [mm:ss]
            let timestamp_regex = Regex::new(r"^\[\d{2}:\d{2}(?:\.\d{2})?\]").unwrap();

            let new_line = if let Some(mat) = timestamp_regex.find(line) {
                // Replace existing timestamp
                format!("{}{}", timestamp, &line[mat.end()..])
            } else {
                // Insert new timestamp at the beginning
                format!("{}{}", timestamp, line)
            };

            // Replace the line
            self.current_content.replace_range(line_start..line_end, &new_line);

            // Move cursor to the start of the next line
            // Check if there's a newline after this line
            let after_line_byte_pos = line_start + new_line.len();
            if after_line_byte_pos < self.current_content.len() {
                // Skip the newline character to go to the start of next line
                let new_cursor_byte_pos = after_line_byte_pos + 1;
                // Convert byte position back to character position
                let new_cursor_char_pos = self.current_content[..new_cursor_byte_pos].chars().count();
                let ccursor = egui::text::CCursor::new(new_cursor_char_pos);
                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
            } else {
                // We're at the end of the file, stay at the end of the current line
                let cursor_char_pos = self.current_content[..after_line_byte_pos].chars().count();
                let ccursor = egui::text::CCursor::new(cursor_char_pos);
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
    let available_height = ui.available_height() - 70.0; // Reserve space for current lyric display and timestamp button
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

    // Show current lyric preview (without timestamp)
    if let Some(text_state) = egui::TextEdit::load_state(ui.ctx(), state.text_edit_id) {
        if let Some(cursor_range) = text_state.cursor.char_range() {
            let cursor_char_pos = cursor_range.primary.index;

            // Convert character position to byte position
            let cursor_pos = state.current_content
                .char_indices()
                .nth(cursor_char_pos)
                .map(|(byte_pos, _)| byte_pos)
                .unwrap_or(state.current_content.len());

            // Find the current line
            let line_start = if cursor_pos > 0 {
                state.current_content[..cursor_pos]
                    .rfind('\n')
                    .map(|pos| pos + 1)
                    .unwrap_or(0)
            } else {
                0
            };

            let line_end = state.current_content[line_start..]
                .find('\n')
                .map(|pos| line_start + pos)
                .unwrap_or(state.current_content.len());

            let current_line = &state.current_content[line_start..line_end];

            // Strip timestamp if present
            let timestamp_regex = Regex::new(r"^\[\d{2}:\d{2}(?:\.\d{2})?\]").unwrap();
            let lyric_text = if let Some(mat) = timestamp_regex.find(current_line) {
                &current_line[mat.end()..]
            } else {
                current_line
            };

            ui.horizontal(|ui| {
                ui.label("Current Lyric:");
                let text = if lyric_text.is_empty() {
                    "(empty line)".to_string()
                } else if lyric_text.len() > 80 {
                    format!("{}...", &lyric_text[..80])
                } else {
                    lyric_text.to_string()
                };

                ui.label(
                    egui::RichText::new(text)
                        .monospace()
                        .background_color(egui::Color32::from_rgb(50, 50, 50))
                        .color(egui::Color32::from_rgb(220, 220, 220))
                );
            });
        }
    }

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
