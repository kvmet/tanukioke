Tanukioke is a live karaoke playback application that supports:

- Multiple, synchronized audio tracks with independent level control (instrumental, lead vocal, backup vocal, etc.)
- Queueing (Both in-library and URLs/arbitrary text to allow for people to add YouTube links)
- Library management (Very basic library handling and searching. Metadata files required.)
- `lrx` files which are an extended version of `lrc` or "lyric" files with additional metadata embedded (Custom for this software)
- An `lrx` file live-editor that supports lyric-by-lyric timestamp insertion synced with the playback transport
- Syncronized "scrubbing" through lyrics and audio (audio stops on scrub and resumes on playback. Fastest with `flac` and `wav` files.)
- Lyric display in a separate window for display on a 'performance' screen while the library and queue can be managed on another display.
- Dynamically rendered display styles (mostly just color selection for now) per track as well as per vocal part.
- Smooth-scrolling lyric display with lyric-by-lyric sync to playback transport.
