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
    pub source: PathBuf,
    pub volume: f32,
}

impl TrackSink {
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.sink.set_volume(volume);
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }
}

pub struct AudioEngine {
    stream_handle: OutputStream,
    tracks: Vec<TrackSink>,
    playback_start: Option<Instant>,
    paused_at: Option<Duration>,
    base_dir: Option<PathBuf>,
    seek_position: Option<Duration>,
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
            seek_position: None,
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
        self.seek_position = None;

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
                source: path,
                volume,
            });
        }

        Ok(())
    }

    pub fn play(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        // If we have a seek position, reload tracks at that position
        if let Some(seek_pos) = self.seek_position.take() {
            if self.reload_at_position(seek_pos).is_err() {
                eprintln!("Failed to seek to position");
                return;
            }
            self.playback_start = Some(Instant::now() - seek_pos);
            self.paused_at = None;
        } else if let Some(paused_at) = self.paused_at.take() {
            // Resume from pause
            self.playback_start = Some(Instant::now() - paused_at);
        } else {
            // Start from beginning
            self.playback_start = Some(Instant::now());
        }

        // Start all tracks simultaneously
        for track in &self.tracks {
            track.sink.play();
        }
    }

    fn reload_at_position(&mut self, position: Duration) -> Result<()> {
        // Stop and clear all sinks
        for track in &self.tracks {
            track.sink.stop();
        }

        // Reload all tracks at the seek position
        for track in &mut self.tracks {
            let file = File::open(&track.source)
                .with_context(|| format!("Failed to open audio file: {}", track.source.display()))?;
            let buf_reader = BufReader::new(file);
            let source = Decoder::new(buf_reader)
                .with_context(|| format!("Failed to decode audio file: {}", track.source.display()))?;

            // Skip to position
            let source = source.skip_duration(position);

            // Create new sink
            let new_sink = Sink::connect_new(&self.stream_handle.mixer());
            new_sink.set_volume(track.volume);
            new_sink.append(source);
            new_sink.pause(); // Will be unpaused by play()

            track.sink = new_sink;
        }

        Ok(())
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
        // Always pause on seek
        self.pause();

        // Store the seek position for next play
        self.seek_position = Some(position);
        self.paused_at = Some(position);
        self.playback_start = None;

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
