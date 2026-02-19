use super::{LrxFile, Track, Part, LyricLine};
use eframe::egui::Color32;

impl LrxFile {
    /// Serialize an LRX file to a string
    pub fn to_string(&self) -> String {
        let mut output = String::new();

        // Write metadata tags
        for (key, value) in &self.metadata {
            output.push_str(&format!("[{}:{}]\n", key, value));
        }

        if !self.metadata.is_empty() {
            output.push('\n');
        }

        // Write track definitions
        for (track_id, track) in &self.tracks {
            output.push_str(&serialize_track(track_id, track));
        }

        if !self.tracks.is_empty() {
            output.push('\n');
        }

        // Write part definitions
        for (part_id, part) in &self.parts {
            output.push_str(&serialize_part(part_id, part));
        }

        if !self.parts.is_empty() {
            output.push('\n');
        }

        // Write lyric lines
        for line in &self.lines {
            output.push_str(&serialize_lyric_line(line));
        }

        output
    }
}

fn serialize_track(id: &str, track: &Track) -> String {
    let mut output = String::new();

    output.push_str(&format!("[track.{}:name={}]\n", id, track.name));
    output.push_str(&format!("[track.{}:source={}]\n", id, track.source.display()));
    output.push_str(&format!("[track.{}:volume={}]\n", id, track.volume));

    output
}

fn serialize_part(id: &str, part: &Part) -> String {
    let mut output = String::new();

    output.push_str(&format!("[part.{}:name={}]\n", id, part.name));
    output.push_str(&format!("[part.{}:color={}]\n", id, serialize_color(part.color)));

    if let Some(bg_color) = part.background_color {
        output.push_str(&format!("[part.{}:background_color={}]\n", id, serialize_color(bg_color)));
    }

    output
}

fn serialize_lyric_line(line: &LyricLine) -> String {
    let timestamp = format_timestamp(line.timestamp);

    if let Some(part_id) = &line.part_id {
        format!("[{}][{}]{}\n", timestamp, part_id, line.text)
    } else {
        format!("[{}]{}\n", timestamp, line.text)
    }
}

fn format_timestamp(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as u32;
    let secs = seconds % 60.0;

    // Format as mm:ss.xx
    format!("{:02}:{:05.2}", minutes, secs)
}

fn serialize_color(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(12.0), "00:12.00");
        assert_eq!(format_timestamp(90.5), "01:30.50");
        assert_eq!(format_timestamp(135.0), "02:15.00");
    }

    #[test]
    fn test_serialize_color() {
        let white = Color32::from_rgb(255, 255, 255);
        assert_eq!(serialize_color(white), "#FFFFFF");

        let red = Color32::from_rgb(255, 0, 0);
        assert_eq!(serialize_color(red), "#FF0000");
    }

    #[test]
    fn test_serialize_lyric_line() {
        let line = LyricLine::new(12.0, "Test lyrics".to_string());
        assert_eq!(serialize_lyric_line(&line), "[00:12.00]Test lyrics\n");

        let line_with_part = LyricLine::with_part(12.0, "Test lyrics".to_string(), "lead".to_string());
        assert_eq!(serialize_lyric_line(&line_with_part), "[00:12.00][lead]Test lyrics\n");
    }
}
