# Transport Controls and Lyrics Synchronization

## Overview

Transport controls have been implemented in the main app with real-time synchronization to a separate lyrics window. Both windows share playback state via `Arc<Mutex<PlaybackState>>`.

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

**Lyrics Window Button:**
- Opens lyrics window in separate thread/process
- Shares playback state reference

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
3. Lyrics window reads state every frame in `update()`
4. `find_current_line_index()` determines which lyric to highlight
5. Both windows request repaint continuously for smooth updates

## Testing

1. Run `cargo run`
2. Open main window
3. Set duration in Debug Controls (e.g., "Set 3:30 duration")
4. Click "🎤 Open Lyrics (Example File)" to load `Destroy Me.lrx`
5. Scrub the seek slider in main window
6. Watch lyrics window highlight change in real-time

## What's Next

### Required for Full Functionality:
- **Audio Engine Integration** - Connect `rodio` playback to update `position` automatically
- **Real Lyrics Loading** - Load via library browser instead of hardcoded path
- **Automatic Duration Detection** - Extract from audio file metadata
- **Play/Pause/Stop Functionality** - Wire up to audio engine

### Future Enhancements:
- Smooth scrolling animation in lyrics window
- Show previous/next lines with reduced opacity
- Word-by-word highlighting (if LRX format supports it)
- Background color support for lyrics
- Per-track volume controls in transport

## File Structure

```
src/
├── app.rs                    # Main app with transport controls + PlaybackState
├── ui/
│   └── lyrics_window.rs      # Separate lyrics window with sync
└── lrx/
    └── mod.rs                # LRX types (used for lyrics display)
```

## Notes

- egui doesn't natively support multiple windows well, so lyrics window spawns in separate thread
- `Arc<Mutex<>>` chosen for simplicity; could use channels for more complex scenarios
- Position is the single source of truth - all UI derives from it
- No actual audio playback yet - transport is UI-only for now