use eframe::egui;
use crate::queue::Queue;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum QueueAction {
    Load(PathBuf),
    Edit(usize),
    Delete(usize),
    MoveUp(usize),
    MoveDown(usize),
    OpenUrl(String),
    CopyUrl(String),
    AddManual,
}

// Dialog state structs
#[derive(Default)]
pub struct AddManualDialog {
    pub name: String,
    pub song: String,
    pub url: String,
}

pub struct AddFromLibraryDialog {
    pub name: String,
    pub song_title: String,
    pub path: PathBuf,
}

pub struct EditEntryDialog {
    pub id: usize,
    pub name: String,
    pub song: String,
    pub url: String,
    pub is_library_entry: bool,
}

pub fn render(ui: &mut egui::Ui, queue: &Queue, is_playing: bool) -> Option<QueueAction> {
    let mut action = None;

    // Header section
    ui.horizontal(|ui| {
        ui.heading("Queue");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("➕ Add").clicked() {
                action = Some(QueueAction::AddManual);
            }
        });
    });

    ui.separator();

    // Queue list
    if queue.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label("Queue is empty");
            ui.label("Use the '+' button to add entries manually");
            ui.label("or 'Enqueue' from the library");
        });
    } else {
        egui::ScrollArea::vertical()
            .id_salt("queue_scroll_area")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let num_entries = queue.entries.len();
                let current_index = queue.current_index;

                for (index, entry) in queue.entries.iter().enumerate() {
                    let is_current = current_index == Some(index);

                    // Highlight current entry
                    let frame = if is_current {
                        egui::Frame::default()
                            .fill(ui.style().visuals.selection.bg_fill)
                            .inner_margin(egui::Margin::same(8))
                    } else {
                        egui::Frame::default()
                            .inner_margin(egui::Margin::same(8))
                    };

                    frame.show(ui, |ui| {
                        ui.vertical(|ui| {
                            // Entry info section
                            ui.spacing_mut().item_spacing.y = 2.0;

                            // Singer name (bold)
                            ui.label(
                                egui::RichText::new(&entry.singer_name)
                                    .strong()
                                    .size(14.0)
                            );

                            // Song title
                            ui.label(&entry.song_title);

                            // URL indicator
                            if let Some(url) = &entry.url {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    if ui.small_button("🔗 Open URL").clicked() {
                                        action = Some(QueueAction::OpenUrl(url.clone()));
                                    }
                                    if ui.small_button("📋 Copy URL").clicked() {
                                        action = Some(QueueAction::CopyUrl(url.clone()));
                                    }
                                });
                            }

                            // Buttons below
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Delete button
                                    if ui.button("✖ Delete").clicked() {
                                        action = Some(QueueAction::Delete(entry.id));
                                    }

                                    // Move down button (disabled for last item)
                                    ui.add_enabled_ui(index < num_entries - 1, |ui| {
                                        if ui.button("v").on_hover_text("Move Down").clicked() {
                                            action = Some(QueueAction::MoveDown(entry.id));
                                        }
                                    });

                                    // Move up button (disabled for first item)
                                    ui.add_enabled_ui(index > 0, |ui| {
                                        if ui.button("^").on_hover_text("Move Up").clicked() {
                                            action = Some(QueueAction::MoveUp(entry.id));
                                        }
                                    });

                                    // Edit button
                                    if ui.button("✏ Edit").clicked() {
                                        action = Some(QueueAction::Edit(entry.id));
                                    }

                                    // Load button (only if there's an LRX path)
                                    if let Some(lrx_path) = &entry.lrx_path {
                                        let load_button = egui::Button::new("▶ Load");
                                        let load_response = if is_playing {
                                            ui.add_enabled(false, load_button)
                                        } else {
                                            ui.add(load_button)
                                        };

                                        if load_response.clicked() {
                                            action = Some(QueueAction::Load(lrx_path.clone()));
                                        }
                                    }
                                });
                            });
                        });
                    });

                    ui.separator();
                }
            });
    }

    action
}

/// Render the add manual entry dialog
pub fn render_add_manual_dialog(
    ctx: &egui::Context,
    dialog: &mut AddManualDialog,
    queue: &mut Queue,
) -> bool {
    let mut should_close = false;
    let mut should_add = false;

    egui::Window::new("Add Queue Entry")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Singer Name:");
            ui.text_edit_singleline(&mut dialog.name);

            ui.label("Song Title:");
            ui.text_edit_singleline(&mut dialog.song);

            ui.label("URL (optional):");
            ui.text_edit_singleline(&mut dialog.url);

            ui.horizontal(|ui| {
                let can_add = !dialog.name.is_empty() && !dialog.song.is_empty();
                ui.add_enabled_ui(can_add, |ui| {
                    if ui.button("Add").clicked() {
                        should_add = true;
                        should_close = true;
                    }
                });
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_add && !dialog.name.is_empty() && !dialog.song.is_empty() {
        let url_opt = if dialog.url.is_empty() {
            None
        } else {
            Some(dialog.url.clone())
        };
        queue.add(
            dialog.name.clone(),
            dialog.song.clone(),
            None,
            url_opt,
        );
        *dialog = AddManualDialog::default();
    }

    should_close
}

/// Render the add from library dialog
pub fn render_add_from_library_dialog(
    ctx: &egui::Context,
    dialog: &mut AddFromLibraryDialog,
    queue: &mut Queue,
) -> bool {
    let mut should_close = false;
    let mut should_add = false;

    egui::Window::new("Add to Queue")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Singer Name:");
            ui.text_edit_singleline(&mut dialog.name);

            ui.label("Song:");
            ui.label(&dialog.song_title);

            ui.horizontal(|ui| {
                let can_add = !dialog.name.is_empty();
                ui.add_enabled_ui(can_add, |ui| {
                    if ui.button("Add").clicked() {
                        should_add = true;
                        should_close = true;
                    }
                });
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_add && !dialog.name.is_empty() {
        queue.add(
            dialog.name.clone(),
            dialog.song_title.clone(),
            Some(dialog.path.clone()),
            None,
        );
    }

    should_close
}

/// Render the edit entry dialog
pub fn render_edit_entry_dialog(
    ctx: &egui::Context,
    dialog: &mut EditEntryDialog,
    queue: &mut Queue,
) -> bool {
    let mut should_close = false;
    let mut should_save = false;

    egui::Window::new("Edit Queue Entry")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Singer Name:");
            ui.text_edit_singleline(&mut dialog.name);

            if dialog.is_library_entry {
                // Library entries: only name is editable
                ui.label("Song:");
                ui.label(&dialog.song);
            } else {
                // Manual entries: all fields editable
                ui.label("Song Title:");
                ui.text_edit_singleline(&mut dialog.song);

                ui.label("URL (optional):");
                ui.text_edit_singleline(&mut dialog.url);
            }

            ui.horizontal(|ui| {
                let can_save = !dialog.name.is_empty() && !dialog.song.is_empty();
                ui.add_enabled_ui(can_save, |ui| {
                    if ui.button("Save").clicked() {
                        should_save = true;
                        should_close = true;
                    }
                });
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_save {
        if let Some(entry) = queue.get_mut(dialog.id) {
            entry.singer_name = dialog.name.clone();
            // Only update song and URL for manual entries
            if !dialog.is_library_entry {
                entry.song_title = dialog.song.clone();
                entry.url = if dialog.url.is_empty() {
                    None
                } else {
                    Some(dialog.url.clone())
                };
            }
        }
    }

    should_close
}
