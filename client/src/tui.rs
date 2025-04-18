// tui.rs

use ratatui::{Terminal, backend::CrosstermBackend, widgets::{Block, Borders}, layout::{Layout, Constraint, Direction}};
use crossterm::event::{self, KeyCode};

pub async fn run_ui() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the terminal
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    loop {
        // Listen for events (e.g., key presses)
        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(key_event) = event::read()? {
                if key_event.code == KeyCode::Esc {
                    break; // Exit on Esc key
                }
            }
        }

        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().borders(Borders::ALL).title("TUI Example");
            f.render_widget(block, size);
        })?;
    }

    terminal.clear()?;
    Ok(())
}
