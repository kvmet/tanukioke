use eframe::egui;

pub fn render(ctx: &egui::Context, show: &mut bool) {
    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("lrx_help_window"),
        egui::ViewportBuilder::default()
            .with_title("LRX Format Help")
            .with_inner_size([700.0, 600.0]),
        |ctx, _class| {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("LRX Format Overview");
                    ui.label("LRX (Lyrics eXtended) is an extended lyrics format based on the LRC standard.");
                    ui.add_space(10.0);

                    ui.separator();

                    ui.heading("File Structure");
                    ui.label("An LRX file consists of four sections:");
                    ui.label("  1. Metadata Tags - Song information");
                    ui.label("  2. Track Definitions - Audio file references");
                    ui.label("  3. Part Definitions - Vocal part styling");
                    ui.label("  4. Timed Lyrics - Timestamped lyric lines");
                    ui.add_space(10.0);

                    ui.separator();

                    ui.heading("Metadata Tags");
                    ui.label("Standard LRC metadata using square bracket notation: [tag:value]");
                    ui.add_space(5.0);

                    egui::Grid::new("metadata_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Tag");
                            ui.label("Description");
                            ui.label("Example");
                            ui.end_row();

                            ui.label("ar");
                            ui.label("Artist name");
                            ui.label("[ar:Artist Name]");
                            ui.end_row();

                            ui.label("ti");
                            ui.label("Title");
                            ui.label("[ti:Song Title]");
                            ui.end_row();

                            ui.label("al");
                            ui.label("Album");
                            ui.label("[al:Album Name]");
                            ui.end_row();

                            ui.label("length");
                            ui.label("Duration");
                            ui.label("[length:03:45]");
                            ui.end_row();

                            ui.label("key");
                            ui.label("Musical key");
                            ui.label("[key:C] or [key:G#]");
                            ui.end_row();

                            ui.label("offset");
                            ui.label("Global timing offset (ms)");
                            ui.label("[offset:+100]");
                            ui.end_row();

                            ui.label("color");
                            ui.label("Global foreground color");
                            ui.label("[color:#FFFFFF]");
                            ui.end_row();

                            ui.label("background_color");
                            ui.label("Background color");
                            ui.label("[background_color:#000000]");
                            ui.end_row();
                        });
                    ui.add_space(10.0);

                    ui.separator();

                    ui.heading("Track Definitions");
                    ui.label("Tracks define audio files using dot notation: [track.{id}:{property}={value}]");
                    ui.add_space(5.0);
                    ui.label("Example:");
                    ui.code("[track.instrumental:name=Instrumental]\n[track.instrumental:source=instrumental.mp3]\n[track.instrumental:volume=0.8]");
                    ui.add_space(5.0);
                    ui.label("Properties: name (required), source (required), volume (0.0-1.0, default 1.0)");
                    ui.add_space(10.0);

                    ui.separator();

                    ui.heading("Part Definitions");
                    ui.label("Parts define vocal roles with styling: [part.{id}:{property}={value}]");
                    ui.add_space(5.0);
                    ui.label("Example:");
                    ui.code("[part.lead:name=Lead Vocal]\n[part.lead:color=#FF6B9D]\n\n[part.harmony:name=Harmony]\n[part.harmony:color=#6B9DFF]");
                    ui.add_space(5.0);
                    ui.label("Properties: name (required), color (hex format, default #FFFFFF)");
                    ui.add_space(10.0);

                    ui.separator();

                    ui.heading("Timed Lyrics");
                    ui.label("Format: [mm:ss.xx][part]Lyric text");
                    ui.add_space(5.0);
                    ui.label("• Timestamp: [mm:ss.xx] where mm=minutes, ss=seconds, xx=centiseconds");
                    ui.label("• Part Tag: [part_id] references a defined part (optional)");
                    ui.label("• Lines without a part tag use global or default colors");
                    ui.label("• Multiple timestamps can reference the same lyric line");
                    ui.label("• Lines starting with # are comments");
                    ui.add_space(5.0);
                    ui.label("Example:");
                    ui.code("[00:12.00][lead]Lorem ipsum dolor sit amet\n[00:18.50][lead]Consectetur adipiscing elit\n[00:24.00]Both parts singing\n[00:30.00][harmony]Harmony part");
                    ui.add_space(10.0);

                    ui.separator();

                    ui.heading("Complete Example");
                    ui.code(
                        "[ar:Lorem Artist]\n\
                        [ti:Ipsum Song]\n\
                        [color:#FFFFFF]\n\
                        \n\
                        [track.instrumental:name=Instrumental]\n\
                        [track.instrumental:source=instrumental.mp3]\n\
                        \n\
                        [part.lead:name=Lead]\n\
                        [part.lead:color=#FF6B9D]\n\
                        \n\
                        [00:12.00][lead]Lorem ipsum dolor sit amet"
                    );
                });
            });

            if ctx.input(|i| i.viewport().close_requested()) {
                *show = false;
            }
        },
    );
}
