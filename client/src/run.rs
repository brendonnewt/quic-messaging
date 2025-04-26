use crate::{app::{App, FormState}, ui, event};
use crossterm::{
    event::{Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::sync::Arc;
use tracing::info;
use shared::client_response::{ClientRequest, Command, ServerResponse};
use crate::app::ActiveField;
use crate::ui::{registration, login, user_menu};

pub async fn run_app(app: &mut App, conn: Arc<quinn::Connection>) -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    loop {
        // 1) Draw the appropriate UI for the current state
        terminal
            .draw(|f| {
            match &app.state {
                FormState::MainMenu => ui::main_menu::render::<CrosstermBackend<Stdout>>(f, app),
                FormState::LoginForm { .. } => login::render::<CrosstermBackend<Stdout>>(f, app),
                FormState::RegisterForm { .. } => registration::render::<CrosstermBackend<Stdout>>(f, app),
                FormState::UserMenu { .. } => user_menu::render::<CrosstermBackend<Stdout>>(f, app),
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
                FormState::RegisterForm { username, password, confirm_password, .. } => {
                    match key.code{
                        KeyCode::Enter => {
                            if password != confirm_password {
                                app.message = "Passwords do not match!".to_string();
                                continue;
                            }
                            let req = ClientRequest {
                                jwt: None,
                                command: Command::Register {
                                    username: username.clone(),
                                    password: password.clone(),
                                },
                            };
                            let bytes = serde_json::to_vec(&req)?;
                            // --- logging added here ---
                            tracing::info!("[Client] Opening QUIC stream for Register");
                            let (mut send, mut recv) = conn.open_bi().await?;
                            tracing::info!("[Client] Sending {} bytes", bytes.len());
                            send.write_all(&bytes).await?;
                            tracing::info!("[Client] Finishing send half");
                            send.finish().await?;
                            tracing::info!("[Client] Waiting for response…");
                            // --- end logging ---

                            let resp_bytes = recv.read_to_end(usize::MAX).await?;
                            info!("[Client] Received {} bytes", resp_bytes.len());
                            let response: ServerResponse = serde_json::from_slice(&resp_bytes)?;

                            if response.success {
                                app.message = "Registered! Please log in.".into();
                                app.set_main_menu();
                            }else{
                                app.message = response.message.unwrap_or("Registration failed".into());
                            }
                        }
                        _ => ui::registration::handle_input(app, key),
                    }
                }

                // Login form input
                FormState::LoginForm { .. } => {
                    login::handle_input(app, key);
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
                            0 => {/* Chats */},
                            1 => {/* Chatroom */},
                            2 => {/* Add Friends */},
                            3 => {/* Friend List */},
                            4 => {/* Settings */},
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
