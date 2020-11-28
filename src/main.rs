use std::io;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

mod santorini;
mod ui;

use ui::{new_app, UpdateError};

fn main() -> Result<(), UpdateError> {
    let stdout = MouseTerminal::from(io::stdout().into_raw_mode()?);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = new_app();

    terminal.clear()?;
    loop {
        app = app.update(&mut terminal)?;
    }
}
