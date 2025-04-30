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

pub async fn run_app(
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    loop {
        // 1) Draw the appropriate UI for the current state
        terminal.draw(|f| {
            match &app.state {
                FormState::MainMenu => ui::main_menu::render::<CrosstermBackend<Stdout>>(f, app),
                FormState::LoginForm { .. } => {
                    ui::login::render::<CrosstermBackend<Stdout>>(f, app)
                }
                FormState::RegisterForm { .. } => {
                    ui::registration::render::<CrosstermBackend<Stdout>>(f, app)
                }
                FormState::UserMenu { .. } => {
                    ui::user_menu::render::<CrosstermBackend<Stdout>>(f, app)
                }
                FormState::AddFriend { .. } => {
                    ui::add_friends::render::<CrosstermBackend<Stdout>>(f, app)
                }
                FormState::Exit => return, // stops drawing, we’ll break below
            }
        })?;

        if matches!(app.state, FormState::Exit) {
            break;
        }

        // 2) Poll for a key event
        if let Some(key) = event::poll_event()? {
            match &mut app.state {
                // Registration form input
                FormState::RegisterForm { .. } => {
                    ui::registration::handle_input(app, key).await;
                }

                // Login form input
                FormState::LoginForm { .. } => {
                    ui::login::handle_input(app, key).await;
                }

                FormState::AddFriend { .. } => {
                    ui::add_friends::handle_input(app, key).await;
                }

                // Main menu navigation
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

                // User menu navigation (post-login)
                FormState::UserMenu { selected_index } => match key.code {
                    KeyCode::Up => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if *selected_index + 1 < 6 {
                            *selected_index += 1;
                        }
                    }
                    KeyCode::Enter | KeyCode::Char('\r') => {
                        match *selected_index {
                            0 => { /* Chats */ }
                            1 => { /* Chatroom */ }
                            2 => { app.set_add_friend() }
                            3 => { /* Friend List */ }
                            4 => { /* Settings */ }
                            5 => app.set_main_menu(), // Log Out -> back to main menu
                            _ => {}
                        }
                    }
                    KeyCode::Esc => {
                        app.set_main_menu();
                    }
                    _ => {}
                },

                // Any other state: do nothing
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
