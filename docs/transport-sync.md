# Transport Controls and Lyrics Synchronization

## Overview

Transport controls have been implemented in the main app with real-time synchronization to a separate lyrics window. Both windows share playback state via `Arc<Mutex<PlaybackState>>`.

The lyrics window features smooth auto-scrolling to keep the current line centered, with configurable opacity levels for current, upcoming, and past lyrics.

## Architecture

### Shared State

`PlaybackState` struct in `src/app.rs`:
- `position: f64` - Current playback position in seconds
- `duration: f64` - Total track duration in seconds  
- `is_playing: bool` - Playing state
- `is_paused: bool` - Paused state

Wrapped in `Arc<Mutex<>>` for thread-safe sharing across windows.

### Main App (`src/app.rs`)

**Transport Controls:**
- Play/Pause/Stop buttons (UI only - no audio engine yet)
- Seek slider - functional, updates `position` in real-time
- Time display (MM:SS format)
- Debug controls for testing duration/position

**Lyrics Window:**
- Opens as separate viewport using egui's viewport system
- Shares playback state reference
- Loads config on startup for display preferences
</text>

<old_text line=29>
### Lyrics Window (`src/ui/lyrics_window.rs`)

**Features:**
- Reads `PlaybackState` every frame
- Highlights current lyric line based on `position`
- Displays colored lyrics per vocal part
- Progress bar at bottom
- Time display synced with main app

**Spawning:**
- `spawn_lyrics_window(state)` - Empty lyrics window
- `spawn_lyrics_window_with_file(state, path)` - Load LRX file for testing

### Lyrics Window (`src/ui/lyrics_window.rs`)

**Features:**
- Reads `PlaybackState` every frame
- Highlights current lyric line based on `position`
- Displays colored lyrics per vocal part
- Progress bar at bottom
- Time display synced with main app

**Spawning:**
- `spawn_lyrics_window(state)` - Empty lyrics window
- `spawn_lyrics_window_with_file(state, path)` - Load LRX file for testing

## How Sync Works

1. Main app updates `position` via seek slider or play controls
2. `PlaybackState` is locked and modified
3. Lyrics window reads state every frame in viewport update
4. `find_current_line_index()` determines which lyric to highlight
5. Only context lines around current line are rendered
6. Current line position is tracked and scroll target is updated
7. Scroll position smoothly interpolates (lerp) towards target
8. Both windows request repaint continuously for smooth updates

## Testing

1. Run `cargo run`
2. Open main window
3. Set duration in Debug Controls (e.g., "Set 3:30 duration")
4. Click "🎤 Open Lyrics (Random Library File)" to load a random song
5. Scrub the seek slider in main window
6. Watch lyrics scroll smoothly and highlight change in real-time
7. Adjust config settings in `~/.config/tanukioke/config.toml` to customize display

## What's Next

### Required for Full Functionality:
- **Audio Engine Integration** - Connect `rodio` playback to update `position` automatically
- **Real Lyrics Loading** - Load via library browser (not just random test)
- **Automatic Duration Detection** - Extract from audio file metadata
- **Play/Pause/Stop Functionality** - Wire up to audio engine

### Future Enhancements:
- Word-by-word highlighting (if LRX format supports it)
- Background color support for lyrics
- Per-track volume controls in transport
- Adjustable scroll speed/smoothness
- Lookahead emphasis for upcoming lines

## File Structure

```
src/
├── app.rs                    # Main app with transport + lyrics viewport + smooth scrolling
├── config.rs                 # Config with lyrics display settings
├── ui/
│   └── lyrics_window.rs      # Legacy (now integrated into app.rs viewport)
└── lrx/
    └── mod.rs                # LRX types (used for lyrics display)
```

## Notes

- Lyrics window uses egui's viewport system for proper multi-window support
- `Arc<Mutex<>>` chosen for simplicity; could use channels for more complex scenarios
- Position is the single source of truth - all UI derives from it
- No actual audio playback yet - transport is UI-only for now
- Smooth scrolling uses linear interpolation (lerp) for natural motion
- Only visible context lines are rendered for performance