use eframe::egui;
use std::path::PathBuf;
use crate::library::Song;

#[derive(Debug, Clone)]
pub enum LibraryAction {
    Load(PathBuf),
    Enqueue(PathBuf),
}

pub fn render(ui: &mut egui::Ui, songs: &[Song], is_playing: bool) -> Option<LibraryAction> {
    let mut action = None;

    ui.heading("Library");

    if songs.is_empty() {
        ui.label("No songs loaded yet");
        return None;
    }

    ui.separator();

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
                for song in songs {
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
