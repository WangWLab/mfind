//! TUI application

use anyhow::Result;

/// Application state
pub struct App {
    /// Current input
    pub input: String,
    /// Search results
    pub results: Vec<String>,
    /// Selected result index
    pub selected: usize,
    /// Whether the app should quit
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            results: Vec::new(),
            selected: 0,
            should_quit: false,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // TODO: Implement TUI main loop
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
