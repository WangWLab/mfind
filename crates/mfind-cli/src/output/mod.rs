//! Output formatting module

use clap::ValueEnum;
use console::style;
use serde::Serialize;

/// Output format options
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    /// Plain list output
    #[default]
    List,

    /// Table format with details
    Table,

    /// JSON format
    Json,

    /// Null-separated output (for xargs -0)
    Null,
}

/// Output writer
pub struct OutputWriter {
    format: OutputFormat,
}

impl OutputWriter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Print search results
    pub fn print_results<T: Serialize + std::fmt::Debug>(&self, results: &[T]) {
        match self.format {
            OutputFormat::List => self.print_list(results),
            OutputFormat::Table => self.print_table(results),
            OutputFormat::Json => self.print_json(results),
            OutputFormat::Null => self.print_null(results),
        }
    }

    fn print_list<T: Serialize + std::fmt::Debug>(&self, results: &[T]) {
        for result in results {
            // For now, just debug print
            println!("{:?}", result);
        }
    }

    fn print_table<T: Serialize + std::fmt::Debug>(&self, results: &[T]) {
        for result in results {
            println!("{:?}", result);
        }
    }

    fn print_json<T: Serialize>(&self, results: &[T]) {
        match serde_json::to_string_pretty(results) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    }

    fn print_null<T: Serialize + std::fmt::Debug>(&self, results: &[T]) {
        for result in results {
            print!("{:?}\0", result);
        }
    }

    /// Print a single line with optional color
    pub fn print_line(&self, text: &str, color: Option<&str>) {
        match color {
            Some("red") => println!("{}", style(text).red()),
            Some("green") => println!("{}", style(text).green()),
            Some("yellow") => println!("{}", style(text).yellow()),
            Some("blue") => println!("{}", style(text).blue()),
            Some("magenta") => println!("{}", style(text).magenta()),
            Some("cyan") => println!("{}", style(text).cyan()),
            _ => println!("{}", text),
        }
    }

    /// Print status message
    pub fn print_status(&self, status: &str, message: &str) {
        let icon = match status {
            "info" => "ℹ",
            "success" => "✓",
            "warning" => "⚠",
            "error" => "✗",
            _ => "•",
        };

        let color = match status {
            "info" => Some("blue"),
            "success" => Some("green"),
            "warning" => Some("yellow"),
            "error" => Some("red"),
            _ => None,
        };

        eprint!("{} ", style(icon).fg(color.map(|c| match c {
            "red" => console::Color::Red,
            "green" => console::Color::Green,
            "yellow" => console::Color::Yellow,
            "blue" => console::Color::Blue,
            _ => console::Color::White,
        }).unwrap_or(console::Color::White)));

        eprintln!("{}", message);
    }
}
