use crate::{app::{App, AppState}, ui, event};
use crossterm::{
    event::{Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use std::time::Duration;
use std::io::Stdout;
use crate::ui::registration;
use crate::ui::registration::handle_input;

pub fn run_app(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Set up the terminal with the correct backend
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Enter alternate screen for better terminal UI
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    loop {
        terminal.draw(|f| {
            match app.state {
                AppState::MainMenu => ui::main_menu::render::<CrosstermBackend<Stdout>>(f, app),
                AppState::LoginForm => ui::main_menu::render::<CrosstermBackend<Stdout>>(f, app),
                AppState::RegisterForm => ui::registration::render::<CrosstermBackend<Stdout>>(f, app),
                AppState::Exit => return, // Exit if we reach this state
            }
        })?;

        // Break loop on Exit
        if matches!(app.state, AppState::Exit) {
            break;
        }

        // Handle key events
        if let Some(key) = event::poll_event()? {
            match app.state {
                AppState::RegisterForm => {
                    handle_input(app, key);
                    terminal.draw(|f| registration::render::<CrosstermBackend<Stdout>>(f, &app))?;
                }
                AppState::MainMenu => match key.code {
                    KeyCode::Up => {
                        if app.selected_index > 0 {
                            app.selected_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if app.selected_index < 2 {
                            app.selected_index += 1;
                        }
                    }
                    KeyCode::Enter | KeyCode::Char('\r') => match app.selected_index {
                        0 => app.state = AppState::LoginForm,
                        1 => app.state = AppState::RegisterForm,
                        2 => app.state = AppState::Exit,
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }
    }

    // Exit from raw mode and restore the screen
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
