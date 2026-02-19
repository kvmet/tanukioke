use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct QueueEntry {
    pub id: usize,
    pub singer_name: String,
    pub song_title: String,
    pub lrx_path: Option<PathBuf>,
    pub url: Option<String>,
}

impl QueueEntry {
    pub fn new(
        id: usize,
        singer_name: String,
        song_title: String,
        lrx_path: Option<PathBuf>,
        url: Option<String>,
    ) -> Self {
        Self {
            id,
            singer_name,
            song_title,
            lrx_path,
            url,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Queue {
    pub entries: Vec<QueueEntry>,
    pub current_index: Option<usize>,
    next_id: usize,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: None,
            next_id: 0,
        }
    }

    /// Add a new entry to the queue and return its ID
    pub fn add(
        &mut self,
        singer_name: String,
        song_title: String,
        lrx_path: Option<PathBuf>,
        url: Option<String>,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let entry = QueueEntry::new(id, singer_name, song_title, lrx_path, url);
        self.entries.push(entry);

        id
    }

    /// Remove an entry by its ID
    pub fn remove(&mut self, id: usize) -> Option<QueueEntry> {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            // Adjust current_index if needed
            if let Some(current) = self.current_index {
                if pos < current {
                    self.current_index = Some(current - 1);
                } else if pos == current {
                    self.current_index = None;
                }
            }
            Some(self.entries.remove(pos))
        } else {
            None
        }
    }

    /// Move an entry up in the queue (towards the front)
    pub fn move_up(&mut self, id: usize) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            if pos > 0 {
                self.entries.swap(pos, pos - 1);

                // Adjust current_index if needed
                if let Some(current) = self.current_index {
                    if current == pos {
                        self.current_index = Some(pos - 1);
                    } else if current == pos - 1 {
                        self.current_index = Some(pos);
                    }
                }

                return true;
            }
        }
        false
    }

    /// Move an entry down in the queue (towards the back)
    pub fn move_down(&mut self, id: usize) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            if pos < self.entries.len() - 1 {
                self.entries.swap(pos, pos + 1);

                // Adjust current_index if needed
                if let Some(current) = self.current_index {
                    if current == pos {
                        self.current_index = Some(pos + 1);
                    } else if current == pos + 1 {
                        self.current_index = Some(pos);
                    }
                }

                return true;
            }
        }
        false
    }

    /// Get an entry by its ID
    pub fn get(&self, id: usize) -> Option<&QueueEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get a mutable reference to an entry by its ID
    pub fn get_mut(&mut self, id: usize) -> Option<&mut QueueEntry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Clear all entries from the queue
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_index = None;
    }

    /// Get the current entry
    pub fn current(&self) -> Option<&QueueEntry> {
        self.current_index.and_then(|idx| self.entries.get(idx))
    }

    /// Move to the next entry in the queue
    pub fn next(&mut self) -> Option<&QueueEntry> {
        if let Some(idx) = self.current_index {
            if idx + 1 < self.entries.len() {
                self.current_index = Some(idx + 1);
            } else {
                // No more entries
                return None;
            }
        } else if !self.entries.is_empty() {
            self.current_index = Some(0);
        }
        self.current()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of entries in the queue
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl Default for Queue {
    fn default() -> Self {
        Self::new()
    }
}
