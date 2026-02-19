use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use std::time::{Duration, Instant};

pub struct TrackSink {
    pub id: String,
    pub name: String,
    pub sink: Sink,
    pub duration: Duration,
}

impl TrackSink {
    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }
}

pub struct AudioEngine {
    stream_handle: OutputStream,
    tracks: Vec<TrackSink>,
    playback_start: Option<Instant>,
    paused_at: Option<Duration>,
    base_dir: Option<PathBuf>,
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
            .context("Failed to create audio output stream")?;

        Ok(Self {
            stream_handle,
            tracks: Vec::new(),
            playback_start: None,
            paused_at: None,
            base_dir: None,
        })
    }

    pub fn set_base_dir(&mut self, dir: PathBuf) {
        self.base_dir = Some(dir);
    }

    pub fn load_tracks(&mut self, track_infos: Vec<(String, String, PathBuf, f32)>) -> Result<()> {
        // Clear existing tracks
        self.tracks.clear();
        self.playback_start = None;
        self.paused_at = None;

        let mut max_duration = Duration::ZERO;

        for (id, name, source, volume) in track_infos {
            let path = if source.is_relative() {
                if let Some(ref base) = self.base_dir {
                    base.join(&source)
                } else {
                    source
                }
            } else {
                source
            };

            let file = File::open(&path)
                .with_context(|| format!("Failed to open audio file: {}", path.display()))?;
            let buf_reader = BufReader::new(file);
            let source = Decoder::new(buf_reader)
                .with_context(|| format!("Failed to decode audio file: {}", path.display()))?;

            let duration = source.total_duration()
                .unwrap_or(Duration::ZERO);

            if duration > max_duration {
                max_duration = duration;
            }

            let sink = Sink::connect_new(&self.stream_handle.mixer());
            sink.set_volume(volume);
            sink.append(source);
            sink.pause(); // Start paused

            self.tracks.push(TrackSink {
                id,
                name,
                sink,
                duration,
            });
        }

        Ok(())
    }

    pub fn play(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        if let Some(paused_at) = self.paused_at.take() {
            // Resume from pause
            self.playback_start = Some(Instant::now() - paused_at);
        } else {
            // Start from beginning
            self.playback_start = Some(Instant::now());
        }

        for track in &self.tracks {
            track.sink.play();
        }
    }

    pub fn pause(&mut self) {
        if let Some(start) = self.playback_start {
            self.paused_at = Some(Instant::now() - start);
        }

        for track in &self.tracks {
            track.sink.pause();
        }
    }

    pub fn stop(&mut self) {
        self.playback_start = None;
        self.paused_at = None;

        for track in &self.tracks {
            track.sink.stop();
        }
    }

    pub fn seek(&mut self, position: Duration) -> Result<()> {
        // Rodio doesn't support seeking on Sink directly
        // We need to reload tracks and skip to position
        // For now, just update the internal position tracking
        let was_playing = self.is_playing();

        if was_playing {
            self.playback_start = Some(Instant::now() - position);
        } else {
            self.paused_at = Some(position);
        }

        Ok(())
    }

    pub fn position(&self) -> Duration {
        if let Some(paused_at) = self.paused_at {
            return paused_at;
        }

        if let Some(start) = self.playback_start {
            return Instant::now() - start;
        }

        Duration::ZERO
    }

    pub fn duration(&self) -> Duration {
        self.tracks.iter()
            .map(|t| t.duration)
            .max()
            .unwrap_or(Duration::ZERO)
    }

    pub fn is_playing(&self) -> bool {
        self.playback_start.is_some() && self.paused_at.is_none()
    }

    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some()
    }

    pub fn tracks(&self) -> &[TrackSink] {
        &self.tracks
    }

    pub fn tracks_mut(&mut self) -> &mut [TrackSink] {
        &mut self.tracks
    }

    pub fn get_track_mut(&mut self, id: &str) -> Option<&mut TrackSink> {
        self.tracks.iter_mut().find(|t| t.id == id)
    }
}

impl AudioEngine {
    /// Update the given playback state with current engine state
    pub fn update_playback_state(&self, state: &mut crate::app::PlaybackState) {
        state.position = self.position().as_secs_f64();
        state.duration = self.duration().as_secs_f64();
        state.is_playing = self.is_playing();
        state.is_paused = self.is_paused();
    }
}
