use crate::{
    app::{App, FormState},
    event, ui,
};
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::sync::Arc;

pub async fn run_app(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    loop {
        // Draw UI
        terminal.draw(|f| {
            match &app.state {
                FormState::MainMenu => ui::main_menu::render::<CrosstermBackend<Stdout>>(f, app),
                FormState::LoginForm { .. } => ui::login::render::<CrosstermBackend<Stdout>>(f, app),
                FormState::RegisterForm { .. } => ui::registration::render::<CrosstermBackend<Stdout>>(f, app),
                FormState::UserMenu { .. } => ui::user_menu::render::<CrosstermBackend<Stdout>>(f, app),
                FormState::Exit => return,
                _ => {}
            }
        })?;

        // Exit condition
        if matches!(app.state, FormState::Exit) {
            break;
        }

        // Handle input
        if let Some(key) = event::poll_event()? {
            match &mut app.state {
                FormState::RegisterForm { .. } => {
                    ui::registration::handle_input(app, key).await;
                }
                FormState::LoginForm { .. } => {
                    ui::login::handle_input(app, key).await;
                }
                FormState::MainMenu => match key.code {
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
                        0 => app.set_login_form(),
                        1 => app.set_register_form(),
                        2 => app.set_exit(),
                        _ => {}
                    },
                    _ => {}
                },
                FormState::UserMenu { .. } => {
                    ui::user_menu::handle_input(app, key);
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
