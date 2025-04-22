use crate::{app::{App, FormState}, ui, event};
use crossterm::{
    event::{Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use std::time::Duration;
use std::io::Stdout;
use crate::app::ActiveField;
use crate::ui::registration;
use crate::ui::login; // <-- Import the login module
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
            match &app.state {
                FormState::MainMenu => ui::main_menu::render::<CrosstermBackend<Stdout>>(f, app),
                FormState::LoginForm { .. } => ui::login::render::<CrosstermBackend<Stdout>>(f, app), // <-- Handle Login UI
                FormState::RegisterForm { .. } => ui::registration::render::<CrosstermBackend<Stdout>>(f, app),
                FormState::Exit => return, // Exit if we reach this state
            }
        })?;

        // Break loop on Exit
        if matches!(app.state, FormState::Exit) {
            break;
        }

        // Handle key events
        if let Some(key) = event::poll_event()? {
            match &mut app.state {
                FormState::RegisterForm { .. } => {
                    handle_input(app, key); // Handle input for registration form
                    terminal.draw(|f| registration::render::<CrosstermBackend<Stdout>>(f, &app))?;
                }
                FormState::LoginForm { username, password, active_field } => {
                    match key.code {
                        KeyCode::Enter | KeyCode::Char('\r') => {
                            // Handle login logic here (e.g., validate credentials)
                            if *username == "admin" && *password == "password" {
                                // If login is successful, go to main menu or next state
                                app.state = FormState::MainMenu;
                            } else {
                                // If login fails, show error message
                                app.message = String::from("Invalid username or password.");
                            }
                        }
                        KeyCode::Esc => {
                            // If Esc is pressed, go back to the main menu
                            app.state = FormState::MainMenu;
                        }
                        KeyCode::Backspace => {
                            // Handle backspace (remove last character) for username/password input
                            if *active_field == ActiveField::Username && !username.is_empty() {
                                username.pop();
                            } else if *active_field == ActiveField::Password && !password.is_empty() {
                                password.pop();
                            }
                        }
                        KeyCode::Tab => {
                            // Switch active field between username and password
                            if *active_field == ActiveField::Username {
                                app.set_active_field(ActiveField::Password);
                            } else {
                                app.set_active_field(ActiveField::Username);
                            }
                        }
                        _ => {}
                    }
                    terminal.draw(|f| login::render::<CrosstermBackend<Stdout>>(f, app))?;
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
                        0 => app.state = FormState::LoginForm {
                            username: String::new(),
                            password: String::new(),
                            active_field: ActiveField::Username,
                        },
                        1 => app.state = FormState::RegisterForm {
                            username: String::new(),
                            password: String::new(),
                            confirm_password: String::new(),
                            active_field: ActiveField::Username,
                        },
                        2 => app.state = FormState::Exit,
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
