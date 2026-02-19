use crate::library::Song;

#[derive(Debug, Clone)]
pub struct QueueEntry {
    pub song: Song,
    pub singer_name: String,
}

impl QueueEntry {
    pub fn new(song: Song, singer_name: String) -> Self {
        Self { song, singer_name }
    }
}

#[derive(Debug, Clone)]
pub struct Queue {
    pub entries: Vec<QueueEntry>,
    pub current_index: Option<usize>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: None,
        }
    }

    pub fn add(&mut self, entry: QueueEntry) {
        // TODO: Add entry to queue
        self.entries.push(entry);
    }

    pub fn remove(&mut self, index: usize) -> Option<QueueEntry> {
        // TODO: Remove entry at index
        if index < self.entries.len() {
            Some(self.entries.remove(index))
        } else {
            None
        }
    }

    pub fn current(&self) -> Option<&QueueEntry> {
        // TODO: Get current queue entry
        self.current_index
            .and_then(|idx| self.entries.get(idx))
    }

    pub fn next(&mut self) -> Option<&QueueEntry> {
        // TODO: Move to next entry in queue
        if let Some(idx) = self.current_index {
            self.current_index = Some(idx + 1);
        } else if !self.entries.is_empty() {
            self.current_index = Some(0);
        }
        self.current()
    }

    pub fn clear(&mut self) {
        // TODO: Clear all entries
        self.entries.clear();
        self.current_index = None;
    }
}
