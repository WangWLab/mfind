//! TUI application

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

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
    /// Search pattern
    pub pattern: String,
    /// Current status message
    pub status: String,
    /// List state for scroll position
    pub list_state: ListState,
}

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            input: String::new(),
            results: Vec::new(),
            selected: 0,
            should_quit: false,
            pattern: String::new(),
            status: String::from("Ready - Type to search"),
            list_state,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run main loop
        let res = self.run_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            eprintln!("Error: {:?}", err);
        }

        Ok(())
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            self.should_quit = true
                        }
                        KeyCode::Up => self.select_previous(),
                        KeyCode::Down => self.select_next(),
                        KeyCode::Enter => self.on_enter(),
                        KeyCode::Backspace => {
                            self.input.pop();
                            self.update_status();
                        }
                        KeyCode::Esc => self.input.clear(),
                        KeyCode::Char(c) => {
                            self.input.push(c);
                            self.update_status();
                        }
                        _ => {}
                    }
                }
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    fn select_next(&mut self) {
        if !self.results.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i >= self.results.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
            self.selected = i;
        }
    }

    fn select_previous(&mut self) {
        if !self.results.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.results.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
            self.selected = i;
        }
    }

    fn on_enter(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.results.len() {
                self.status = format!("Selected: {}", self.results[selected]);
            }
        }
    }

    fn update_status(&mut self) {
        self.status = format!("Searching: {}", self.input);
        // Simple prefix search simulation
        self.results = self
            .input
            .chars()
            .take(100)
            .map(|c| format!("result_{}.txt", c))
            .collect();
    }

    fn ui(&mut self, f: &mut Frame) {
        let area = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        // Title
        let title = Paragraph::new("mfind - Fast File Search")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Input
        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Search"));
        f.render_widget(input, chunks[1]);

        // Results
        let items: Vec<ListItem> = self
            .results
            .iter()
            .map(|r| ListItem::new(Line::from(r.clone())))
            .collect();
        let results = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Results"))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::White),
            );
        f.render_stateful_widget(results, chunks[2], &mut self.list_state);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
