use eframe::egui;
use std::path::PathBuf;
use crate::library::Song;

#[derive(Debug, Clone)]
pub enum LibraryAction {
    Load(PathBuf),
    Enqueue(PathBuf),
    Rescan,
}

pub fn render(ui: &mut egui::Ui, songs: &[Song], is_playing: bool, search_query: &mut String, show_rescan_confirm: &mut bool) -> Option<LibraryAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        ui.heading("Library");

        ui.add_space(10.0);

        // Rescan button
        if ui.button("🔄 Rescan").clicked() {
            *show_rescan_confirm = true;
        }

        ui.add_space(10.0);

        // Search box
        ui.add(
            egui::TextEdit::singleline(search_query)
                .hint_text("Search...")
                .desired_width(200.0)
        );

        // Clear search button - always visible, greyed when empty
        let clear_button = ui.add_enabled(!search_query.is_empty(), egui::Button::new("✖"));
        if clear_button.clicked() {
            search_query.clear();
        }
    });

    // Rescan confirmation popup
    let mut confirmed_rescan = false;
    if *show_rescan_confirm {
        egui::Window::new("Confirm Rescan")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("Are you sure you want to rescan the library?");
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        confirmed_rescan = true;
                        *show_rescan_confirm = false;
                    }
                    if ui.button("Cancel").clicked() {
                        *show_rescan_confirm = false;
                    }
                });
            });
    }

    if confirmed_rescan {
        return Some(LibraryAction::Rescan);
    }

    if songs.is_empty() {
        ui.label("No songs loaded yet");
        return None;
    }

    ui.separator();

    // Filter songs based on search query
    let filtered_songs: Vec<&Song> = if search_query.is_empty() {
        songs.iter().collect()
    } else {
        let query_lower = search_query.to_lowercase();
        songs.iter().filter(|song| {
            let metadata = song.get_metadata();
            metadata.artist.to_lowercase().contains(&query_lower)
                || metadata.album.to_lowercase().contains(&query_lower)
                || metadata.title.to_lowercase().contains(&query_lower)
                || song.title().to_lowercase().contains(&query_lower)
        }).collect()
    };

    egui::ScrollArea::vertical().show(ui, |ui| {
        use egui_extras::{TableBuilder, Column};

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(150.0)) // Artist
            .column(Column::auto().at_least(150.0)) // Album
            .column(Column::auto().at_least(200.0)) // Track
            .column(Column::exact(120.0))           // Actions
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Artist");
                });
                header.col(|ui| {
                    ui.strong("Album");
                });
                header.col(|ui| {
                    ui.strong("Track");
                });
                header.col(|ui| {
                    ui.strong("Actions");
                });
            })
            .body(|mut body| {
                for song in filtered_songs {
                    let metadata = song.get_metadata();
                    let lrx_path = song.lrx_path.clone();

                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label(if metadata.artist.is_empty() {
                                "Unknown"
                            } else {
                                &metadata.artist
                            });
                        });
                        row.col(|ui| {
                            ui.label(if metadata.album.is_empty() {
                                "Unknown"
                            } else {
                                &metadata.album
                            });
                        });
                        row.col(|ui| {
                            ui.label(if metadata.title.is_empty() {
                                song.title()
                            } else {
                                metadata.title
                            });
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if let Some(path) = &lrx_path {
                                    let load_button = egui::Button::new("Load");
                                    let load_response = if is_playing {
                                        ui.add_enabled(false, load_button)
                                    } else {
                                        ui.add(load_button.fill(egui::Color32::from_rgb(60, 120, 60)))
                                    };

                                    if load_response.clicked() {
                                        action = Some(LibraryAction::Load(path.clone()));
                                    }

                                    if ui.button("Enqueue").clicked() {
                                        action = Some(LibraryAction::Enqueue(path.clone()));
                                    }
                                }
                            });
                        });
                    });
                }
            });
    });

    action
}
