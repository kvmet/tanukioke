use super::{LrxFile, Track, Part, LyricLine};
use std::path::PathBuf;
use eframe::egui::Color32;
use anyhow::{anyhow, Context, Result};

impl LrxFile {
    /// Parse an LRX file from a string
    pub fn parse(content: &str) -> Result<Self> {
        let mut lrx = LrxFile::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // All tags are in square brackets
            if !line.starts_with('[') {
                continue;
            }

            parse_line(&mut lrx, line)
                .with_context(|| format!("Error parsing line {}: {}", line_num + 1, line))?;
        }

        lrx.finalize();

        Ok(lrx)
    }
}

fn parse_line(lrx: &mut LrxFile, line: &str) -> Result<()> {
    // Extract all bracketed segments
    let segments = extract_brackets(line)?;

    if segments.is_empty() {
        return Ok(());
    }

    // Check if first segment is a timestamp
    if let Some(timestamp) = parse_timestamp(&segments[0]) {
        // This is a lyric line
        parse_lyric_line(lrx, timestamp, &segments, line)?;
    } else if segments[0].contains(':') {
        // This is a metadata/track/part definition
        parse_tag(lrx, &segments[0])?;
    }

    Ok(())
}

fn extract_brackets(line: &str) -> Result<Vec<String>> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_bracket = false;

    for ch in line.chars() {
        match ch {
            '[' => {
                if in_bracket {
                    return Err(anyhow!("Nested brackets not allowed"));
                }
                in_bracket = true;
                current.clear();
            }
            ']' => {
                if !in_bracket {
                    return Err(anyhow!("Unmatched closing bracket"));
                }
                in_bracket = false;
                segments.push(current.clone());
                current.clear();
            }
            _ => {
                if in_bracket {
                    current.push(ch);
                }
            }
        }
    }

    if in_bracket {
        return Err(anyhow!("Unclosed bracket"));
    }

    Ok(segments)
}

fn parse_timestamp(s: &str) -> Option<f64> {
    // Format: mm:ss.xx or mm:ss
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let minutes: f64 = parts[0].parse().ok()?;

    // Handle seconds with optional centiseconds
    let seconds: f64 = parts[1].parse().ok()?;

    Some(minutes * 60.0 + seconds)
}

fn parse_lyric_line(lrx: &mut LrxFile, timestamp: f64, segments: &[String], line: &str) -> Result<()> {
    // After timestamp, there might be a [part] tag, then the text
    let part_id = if segments.len() > 1 && !segments[1].contains(':') && !segments[1].contains('.') {
        Some(segments[1].clone())
    } else {
        None
    };

    // Extract text after all brackets
    let text = extract_text_after_brackets(line)?;

    let lyric_line = if let Some(part_id) = part_id {
        LyricLine::with_part(timestamp, text, part_id)
    } else {
        LyricLine::new(timestamp, text)
    };

    lrx.lines.push(lyric_line);
    Ok(())
}

fn extract_text_after_brackets(line: &str) -> Result<String> {
    let mut last_bracket = 0;
    let mut depth = 0;

    for (i, ch) in line.chars().enumerate() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    last_bracket = i + 1;
                }
            }
            _ => {}
        }
    }

    Ok(line[last_bracket..].trim().to_string())
}

fn parse_tag(lrx: &mut LrxFile, tag: &str) -> Result<()> {
    // Split on first colon
    let parts: Vec<&str> = tag.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid tag format: {}", tag));
    }

    let key = parts[0];
    let value = parts[1];

    // Check if it's a dot notation (track.id:prop or part.id:prop)
    if key.contains('.') {
        parse_dot_notation(lrx, key, value)?;
    } else if key == "color" {
        // Global foreground color
        lrx.color = Some(parse_color(value)?);
        lrx.metadata.insert(key.to_string(), value.to_string());
    } else if key == "background_color" {
        // Global background color
        lrx.background_color = Some(parse_color(value)?);
        lrx.metadata.insert(key.to_string(), value.to_string());
    } else {
        // Simple metadata tag
        lrx.metadata.insert(key.to_string(), value.to_string());
    }

    Ok(())
}

fn parse_dot_notation(lrx: &mut LrxFile, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.splitn(2, '.').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid dot notation: {}", key));
    }

    let category = parts[0]; // "track" or "part"
    let id = parts[1]; // just the id

    // Value should be "property=value"
    let prop_value: Vec<&str> = value.splitn(2, '=').collect();
    if prop_value.len() != 2 {
        return Err(anyhow!("Invalid dot notation value format: {}", value));
    }

    let property = prop_value[0];
    let actual_value = prop_value[1];

    match category {
        "track" => parse_track_property(lrx, id, property, actual_value)?,
        "part" => parse_part_property(lrx, id, property, actual_value)?,
        _ => return Err(anyhow!("Unknown category: {}", category)),
    }

    Ok(())
}

fn parse_track_property(lrx: &mut LrxFile, id: &str, property: &str, value: &str) -> Result<()> {
    let track = lrx.tracks.entry(id.to_string()).or_insert_with(|| Track {
        id: id.to_string(),
        name: String::new(),
        source: PathBuf::new(),
        volume: 1.0,
    });

    match property {
        "name" => track.name = value.to_string(),
        "source" => track.source = PathBuf::from(value),
        "volume" => track.volume = value.parse()
            .with_context(|| format!("Invalid volume value: {}", value))?,
        _ => return Err(anyhow!("Unknown track property: {}", property)),
    }

    Ok(())
}

fn parse_part_property(lrx: &mut LrxFile, id: &str, property: &str, value: &str) -> Result<()> {
    let part = lrx.parts.entry(id.to_string()).or_insert_with(|| Part {
        id: id.to_string(),
        name: String::new(),
        color: Color32::WHITE,
    });

    match property {
        "name" => part.name = value.to_string(),
        "color" => part.color = parse_color(value)?,
        _ => return Err(anyhow!("Unknown part property: {}", property)),
    }

    Ok(())
}

fn parse_color(s: &str) -> Result<Color32> {
    // Expected format: #RRGGBB
    if !s.starts_with('#') || s.len() != 7 {
        return Err(anyhow!("Invalid color format: {}. Expected #RRGGBB", s));
    }

    let r = u8::from_str_radix(&s[1..3], 16)
        .with_context(|| format!("Invalid red component in color: {}", s))?;
    let g = u8::from_str_radix(&s[3..5], 16)
        .with_context(|| format!("Invalid green component in color: {}", s))?;
    let b = u8::from_str_radix(&s[5..7], 16)
        .with_context(|| format!("Invalid blue component in color: {}", s))?;

    Ok(Color32::from_rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(parse_timestamp("00:12.00"), Some(12.0));
        assert_eq!(parse_timestamp("01:30.50"), Some(90.5));
        assert_eq!(parse_timestamp("02:15"), Some(135.0));
        assert_eq!(parse_timestamp("invalid"), None);
    }

    #[test]
    fn test_parse_color() {
        let white = parse_color("#FFFFFF").unwrap();
        assert_eq!(white, Color32::from_rgb(255, 255, 255));

        let red = parse_color("#FF0000").unwrap();
        assert_eq!(red, Color32::from_rgb(255, 0, 0));

        assert!(parse_color("FFFFFF").is_err());
        assert!(parse_color("#FFF").is_err());
    }

    #[test]
    fn test_extract_brackets() {
        let result = extract_brackets("[00:12.00][lead]Text here").unwrap();
        assert_eq!(result, vec!["00:12.00", "lead"]);

        let result = extract_brackets("[ar:Artist]").unwrap();
        assert_eq!(result, vec!["ar:Artist"]);
    }
}
