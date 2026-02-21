use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn render(
    ui: &mut egui::Ui,
    config: &mut crate::config::Config,
    playback_state: &Arc<Mutex<crate::app::PlaybackState>>,
) -> bool {
    let mut config_changed = false;

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Settings");
        ui.add_space(10.0);

        // Lyrics Behavior Section
        ui.group(|ui| {
            ui.label(egui::RichText::new("Lyrics Behavior").strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Snappiness:");

                let mut state = playback_state.lock().unwrap();

                // Map internal value to 0-100 slider display range
                let mut slider_value = if state.lyrics_snappiness >= 10000.0 {
                    100.0
                } else {
                    state.lyrics_snappiness
                };

                if ui.add(egui::Slider::new(&mut slider_value, 0.0..=100.0)
                    .text("📊")
                    .fixed_decimals(0))
                    .on_hover_text("How quickly lyrics snap to the current line (0=smooth, 100=instant)")
                    .changed()
                {
                    // Map slider value: 100 = instant (10000), 0-99 = use as-is
                    state.lyrics_snappiness = if slider_value >= 100.0 {
                        10000.0
                    } else {
                        slider_value
                    };

                    config.lyrics_snappiness = state.lyrics_snappiness;
                    config_changed = true;
                }
                drop(state);
            });
        });

        ui.add_space(10.0);

        // Lyrics Display Section
        ui.group(|ui| {
            ui.label(egui::RichText::new("Lyrics Display").strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Current Line Opacity:");
                if ui.add(egui::Slider::new(&mut config.lyrics_opacity_current, 0.0..=1.0)
                    .fixed_decimals(2))
                    .changed()
                {
                    config_changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Upcoming Lines Opacity:");
                if ui.add(egui::Slider::new(&mut config.lyrics_opacity_upcoming, 0.0..=1.0)
                    .fixed_decimals(2))
                    .changed()
                {
                    config_changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Past Lines Opacity:");
                if ui.add(egui::Slider::new(&mut config.lyrics_opacity_past, 0.0..=1.0)
                    .fixed_decimals(2))
                    .changed()
                {
                    config_changed = true;
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Default Foreground Color:")
                    .on_hover_text("Used when LRX file doesn't specify a color");
                let mut color = parse_color(&config.lyrics_default_fg_color);
                if ui.color_edit_button_srgba(&mut color).changed() {
                    config.lyrics_default_fg_color = format_color(color);
                    config_changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Default Background Color:")
                    .on_hover_text("Used when no LRX file is loaded, or when the LRX doesn't specify a background");

                // Ensure we always have a background color set
                if config.lyrics_default_bg_color.is_none() {
                    config.lyrics_default_bg_color = Some("#14141E".to_string());
                }

                if let Some(bg_color_str) = &mut config.lyrics_default_bg_color {
                    let mut color = parse_color(bg_color_str);
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        *bg_color_str = format_color(color);
                        config_changed = true;
                    }
                }
            });
        });

        ui.add_space(10.0);

        // Lyrics Font Section
        ui.group(|ui| {
            ui.label(egui::RichText::new("Lyrics Font").strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Font Size:");
                if ui.add(egui::Slider::new(&mut config.lyrics_font_size, 12.0..=120.0)
                    .fixed_decimals(0))
                    .changed()
                {
                    config_changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Line Spacing:");
                if ui.add(egui::Slider::new(&mut config.lyrics_line_spacing, 0.0..=100.0)
                    .fixed_decimals(0))
                    .changed()
                {
                    config_changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Bold Text:");
                let mut is_bold = config.lyrics_font_weight >= 600.0;
                if ui.checkbox(&mut is_bold, "").changed() {
                    config.lyrics_font_weight = if is_bold { 700.0 } else { 400.0 };
                    config_changed = true;
                }
            });
        });

        ui.add_space(10.0);

        // Library Section
        ui.group(|ui| {
            ui.label(egui::RichText::new("Library").strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Library Path:");
                let path_text = config.library_path.as_deref().unwrap_or("Not set");
                ui.label(path_text);
            });
            ui.label("(Library path is configured in config.toml)");
        });
    });

    config_changed
}

fn parse_color(hex: &str) -> egui::Color32 {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return egui::Color32::from_rgb(r, g, b);
        }
    }
    egui::Color32::WHITE
}

fn format_color(color: egui::Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}
