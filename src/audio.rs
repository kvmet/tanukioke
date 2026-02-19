use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

pub struct Transport {
    pub state: PlaybackState,
    pub position: f64, // seconds
}

impl Transport {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
            position: 0.0,
        }
    }

    pub fn play(&mut self) {
        // TODO: Start/resume playback
        self.state = PlaybackState::Playing;
    }

    pub fn pause(&mut self) {
        // TODO: Pause playback
        self.state = PlaybackState::Paused;
    }

    pub fn stop(&mut self) {
        // TODO: Stop playback
        self.state = PlaybackState::Stopped;
        self.position = 0.0;
    }

    pub fn seek(&mut self, position: f64) {
        // TODO: Seek to position
        self.position = position;
    }
}

pub struct AudioEngine {
    pub transport: Transport,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            transport: Transport::new(),
        }
    }

    pub fn load_tracks(&mut self, _tracks: &[&Path]) -> anyhow::Result<()> {
        // TODO: Load audio files with rodio
        Ok(())
    }
}
