# LRX Format Specification

## Overview

LRX (Lyrics eXtended) is an extended lyrics format based on the LRC (Lyric) standard. It maintains backward compatibility with basic LRC while adding support for multi-track audio, multi-part vocals, and enhanced styling.

**File Extension:** `.lrx`

## Format Structure

An LRX file consists of three sections:

1. **Metadata Tags** - Song information
2. **Track Definitions** - Audio file references
3. **Part Definitions** - Vocal part styling
4. **Timed Lyrics** - Timestamped lyric lines

## Metadata Tags

Standard LRC metadata tags using square bracket notation:

```
[tag:value]
```

### Supported Tags

| Tag | Description | Example |
|-----|-------------|---------|
| `ar` | Artist name | `[ar:Artist Name]` |
| `ti` | Title | `[ti:Song Title]` |
| `al` | Album | `[al:Album Name]` |
| `length` | Duration (mm:ss or mm:ss.xx) | `[length:03:45]` |
| `key` | Musical key | `[key:C]` or `[key:G#]` |
| `by` | LRX file creator | `[by:Your Name]` |
| `offset` | Global timing offset in milliseconds | `[offset:+100]` or `[offset:-50]` |
| `au` | Song author/composer | `[au:Composer Name]` |
| `lr` | Lyricist | `[lr:Lyricist Name]` |

## Track Definitions

Tracks define audio files to be played in sync. Uses dot notation:

```
[track.{id}:{property}={value}]
```

### Track Properties

| Property | Type | Description | Default |
|----------|------|-------------|---------|
| `name` | string | Display name for the track | Required |
| `source` | string | Audio file path (relative to LRX file) | Required |
| `volume` | float | Default volume (0.0 to 1.0) | `1.0` |

### Example

```
[track.instrumental:name=Instrumental]
[track.instrumental:source=instrumental.mp3]
[track.instrumental:volume=0.8]

[track.vocals:name=Lead Vocals]
[track.vocals:source=vocals.mp3]
[track.vocals:volume=1.0]

[track.harmony:name=Backing Vocals]
[track.harmony:source=harmony.flac]
[track.harmony:volume=0.9]
```

## Part Definitions

Parts define vocal roles with custom styling. Uses dot notation:

```
[part.{id}:{property}={value}]
```

### Part Properties

| Property | Type | Description | Default |
|----------|------|-------------|---------|
| `name` | string | Display name for the part | Required |
| `fg_color` | hex color | Foreground/text color | `#FFFFFF` |
| `bg_color` | hex color | Background color (optional) | None |

### Example

```
[part.lead:name=Lead Vocal]
[part.lead:fg_color=#FF6B9D]
[part.lead:bg_color=#000000]

[part.harmony:name=Harmony]
[part.harmony:fg_color=#6B9DFF]

[part.alto:name=Alto]
[part.alto:fg_color=#9DFF6B]

[part.tenor:name=Tenor]
[part.tenor:fg_color=#FFD700]
```

## Timed Lyrics

Lyrics follow the standard LRC timestamp format with optional part tags:

```
[mm:ss.xx][part]Lyric text
```

- **Timestamp:** `[mm:ss.xx]` where:
  - `mm` = minutes (2 digits)
  - `ss` = seconds (2 digits)
  - `xx` = centiseconds/hundredths (2 digits, optional)
- **Part Tag:** `[part_id]` references a defined part (optional)
- **Lyric Text:** The actual lyrics to display

### Rules

- Lines without a part tag use default styling (white text, no background)
- Multiple timestamps can reference the same lyric line
- Empty lines are ignored
- Lines starting with `#` are comments

### Example

```
[00:12.00][lead]Lorem ipsum dolor sit amet
[00:18.50][lead]Consectetur adipiscing elit
[00:24.00]Both parts singing (no tag = default)
[00:30.00][harmony]Harmony part
[00:30.00][lead]Lead singing at same time
```

## Complete Example

```
[ar:Lorem Artist]
[ti:Ipsum Song]
[al:Dolor Album]
[length:03:45]
[key:C]
[by:Tanukioke User]
[offset:0]

[track.instrumental:name=Instrumental]
[track.instrumental:source=instrumental.mp3]
[track.instrumental:volume=1.0]

[track.vocals:name=Lead Vocals]
[track.vocals:source=vocals.mp3]
[track.vocals:volume=1.0]

[part.lead:name=Lead]
[part.lead:fg_color=#FF6B9D]

[part.harmony:name=Harmony]
[part.harmony:fg_color=#6B9DFF]

[00:12.00][lead]Lorem ipsum dolor sit amet consectetur
[00:18.50][lead]Adipiscing elit sed do eiusmod tempor
[00:24.00][lead]Incididunt ut labore et dolore magna
[00:30.00][harmony]Aliqua ut enim ad minim veniam
```

## Compatibility

### LRC Compatibility

LRX files can be parsed by standard LRC players, though:
- Extended tags (track.*, part.*) will be ignored
- Part tags in lyrics will appear as extra text brackets
- Only basic timestamps and lyrics will be displayed

### File Organization

Recommended folder structure:

```
song_folder/
  ├── song_name.lrx
  ├── instrumental.mp3
  ├── vocals.mp3
  └── harmony.mp3
```

Track source paths are relative to the LRX file location.

## Color Format

Colors use standard hex notation: `#RRGGBB`

Examples:
- `#FFFFFF` - White
- `#000000` - Black
- `#FF0000` - Red
- `#00FF00` - Green
- `#0000FF` - Blue
- `#FF6B9D` - Pink
- `#6B9DFF` - Light Blue

## Notes

- All tags are case-sensitive
- Track and part IDs must be unique within their category
- Track/part IDs should use alphanumeric characters and underscores only
- Times are in mm:ss.xx format (minutes:seconds.centiseconds)
- Global offset affects all lyric timestamps uniformly