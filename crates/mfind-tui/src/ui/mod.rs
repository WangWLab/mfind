//! TUI UI components

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

/// Render the main UI
pub fn render_app(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("mfind - Fast File Search for macOS")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Status bar
    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[1]);

    // Input
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Search Pattern"));
    f.render_widget(input, chunks[2]);

    // Results
    render_results(f, app, chunks[3]);
}

/// Render search results list
pub fn render_results(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let content = if i == app.selected {
                Line::from(vec![
                    Span::styled("> ", Style::default().fg(Color::Green)),
                    Span::raw(r.clone()),
                ])
            } else {
                Line::from(vec![
                    Span::raw("  "),
                    Span::raw(r.clone()),
                ])
            };
            ListItem::new(content)
        })
        .collect();

    let results_count = app.results.len();
    let results_title = format!("Results ({})", results_count);

    let results = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(results_title))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::White),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(results, area, &mut app.list_state);
}

/// Help text widget
pub fn render_help(f: &mut Frame, area: ratatui::layout::Rect) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Select  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));

    f.render_widget(help, area);
}
