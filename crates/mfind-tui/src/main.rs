//! mfind-tui: Terminal user interface for mfind

mod app;
mod ui;

use app::App;

fn main() -> anyhow::Result<()> {
    let mut app = App::new();
    app.run()
}
