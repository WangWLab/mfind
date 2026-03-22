//! Event batch processing

use std::time::{Duration, Instant};

use crate::fs::FSEvent;

/// Batch of filesystem events
pub struct EventBatch {
    pub events: Vec<FSEvent>,
    pub created_at: Instant,
}

impl EventBatch {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            created_at: Instant::now(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            created_at: Instant::now(),
        }
    }

    pub fn add(&mut self, event: FSEvent) {
        self.events.push(event);
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.created_at = Instant::now();
    }
}

impl Default for EventBatch {
    fn default() -> Self {
        Self::new()
    }
}
