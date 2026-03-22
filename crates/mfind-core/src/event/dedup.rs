//! Event deduplication

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::fs::FSEvent;

/// Deduplicate filesystem events
pub struct EventDeduplicator {
    seen: HashMap<String, Instant>,
    window: Duration,
}

impl EventDeduplicator {
    pub fn new(window_ms: u64) -> Self {
        Self {
            seen: HashMap::new(),
            window: Duration::from_millis(window_ms),
        }
    }

    /// Check if event should be deduplicated
    pub fn should_dedup(&mut self, event: &FSEvent) -> bool {
        let now = Instant::now();
        let path = event.path.to_string_lossy().to_string();

        // Clean old entries
        self.seen.retain(|_, time| now.duration_since(*time) < self.window);

        // Check if seen within window
        if let Some(last_seen) = self.seen.get(&path) {
            if now.duration_since(*last_seen) < self.window {
                return true;
            }
        }

        // Mark as seen
        self.seen.insert(path, now);
        false
    }

    /// Deduplicate a batch of events
    pub fn dedup_batch(&mut self, events: Vec<FSEvent>) -> Vec<FSEvent> {
        events
            .into_iter()
            .filter(|e| !self.should_dedup(e))
            .collect()
    }
}

impl Default for EventDeduplicator {
    fn default() -> Self {
        Self::new(50)
    }
}
