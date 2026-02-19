mod app;
mod audio;
mod config;
mod library;
mod lrx;
mod queue;
mod ui;

use eframe::egui;

fn main() -> anyhow::Result<()> {
    // Check for test mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "test" {
        return test_library();
    }

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

fn test_library() -> anyhow::Result<()> {
    use std::fs;
    use lrx::LrxFile;

    println!("=== Tanukioke Library Test ===\n");

    // Try to load config
    println!("Loading config...");
    let config = config::Config::load()?;
    println!("Config loaded: {:?}\n", config);

    // Get library path
    let library_path = config.library_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No library path configured"))?;

    println!("Scanning library at: {}\n", library_path);
    let songs = library::scan_library(library_path)?;
    println!("Found {} song(s)\n", songs.len());

    // Display each song
    for (i, song) in songs.iter().enumerate() {
        println!("Song {}: {}", i + 1, song.title());
        println!("  Folder: {:?}", song.folder);
        println!("  LRX file: {:?}", song.lrx_path);
        println!("  Tracks: {} audio file(s)", song.tracks.len());
        for track in &song.tracks {
            println!("    - {:?}", track.path);
        }

        // Try to parse the LRX file
        if let Some(lrx_path) = &song.lrx_path {
            println!("  Parsing LRX file...");
            match fs::read_to_string(lrx_path) {
                Ok(content) => {
                    match LrxFile::parse(&content) {
                        Ok(lrx) => {
                            println!("    ✓ Parsed successfully");

                            // Metadata
                            println!("    Metadata: {} tag(s)", lrx.metadata.len());
                            for (key, value) in &lrx.metadata {
                                println!("      - {}: {}", key, value);
                            }

                            // Tracks
                            println!("    Tracks: {} track(s)", lrx.tracks.len());
                            for (id, track) in &lrx.tracks {
                                println!("      - [{}] {} (source: {}, volume: {})",
                                    id, track.name, track.source.display(), track.volume);
                            }

                            // Parts
                            println!("    Parts: {} part(s)", lrx.parts.len());
                            for (id, part) in &lrx.parts {
                                println!("      - [{}] {} (fg: #{:02X}{:02X}{:02X}{})",
                                    id, part.name,
                                    part.color.r(), part.color.g(), part.color.b(),
                                    part.background_color.map(|c| format!(", bg: #{:02X}{:02X}{:02X}", c.r(), c.g(), c.b()))
                                        .unwrap_or_default());
                            }

                            // Lyrics
                            println!("    Lyrics: {} line(s)", lrx.lines.len());
                            let preview_count = 5.min(lrx.lines.len());
                            for i in 0..preview_count {
                                let line = &lrx.lines[i];
                                let part_str = line.part_id.as_ref()
                                    .map(|p| format!("[{}] ", p))
                                    .unwrap_or_default();
                                println!("      [{:05.2}] {}{}", line.timestamp, part_str, line.text);
                            }
                            if lrx.lines.len() > preview_count {
                                println!("      ... and {} more line(s)", lrx.lines.len() - preview_count);
                            }
                        }
                        Err(e) => {
                            println!("    ✗ Parse error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("    ✗ Read error: {}", e);
                }
            }
        }
        println!();
    }

    Ok(())
}
