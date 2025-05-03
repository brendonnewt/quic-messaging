use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::{App, ActiveField, FormState};
use reqwest::Client; // HTTP client
use tokio::runtime::Runtime;

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    if let FormState::LoginForm { username, password, active_field } = &app.state {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(4)
            .constraints([
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(3), // Message
                Constraint::Min(0),
            ])
            .split(f.area());

        let user_style = if *active_field == ActiveField::Username {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };
        let pass_style = if *active_field == ActiveField::Password {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        // Username box
        let user_para = Paragraph::new(Text::from(username.clone()))
            .block(Block::default().borders(Borders::ALL).title("Username"))
            .style(user_style);
        f.render_widget(user_para, chunks[0]);

        // Password box (masked)
        let pass_para = Paragraph::new(Text::from("*".repeat(password.len())))
            .block(Block::default().borders(Borders::ALL).title("Password"))
            .style(pass_style);
        f.render_widget(pass_para, chunks[1]);

        // Message box
        let msg_para = Paragraph::new(app.message.clone())
            .block(Block::default().title("")) // no borders for message
            .style(Style::default());
        f.render_widget(msg_para, chunks[2]);
    }
}

pub fn handle_input(app: &mut App, key: KeyEvent) {
    use KeyCode::*;

    if let FormState::LoginForm { username, password, active_field } = &mut app.state {
        match key.code {
            Down | Tab => {
                *active_field = if *active_field == ActiveField::Username {
                    ActiveField::Password
                } else {
                    ActiveField::Username
                };
            }
            Up => {
                *active_field = if *active_field == ActiveField::Password {
                    ActiveField::Username
                } else {
                    ActiveField::Password
                };
            }
            Backspace => {
                if *active_field == ActiveField::Password {
                    password.pop();
                } else {
                    username.pop();
                }
            }
            Char(c) => {
                if *active_field == ActiveField::Password {
                    password.push(c);
                } else {
                    username.push(c);
                }
            }
            Enter => {
                if username.is_empty() || password.is_empty() {
                    app.message = "Please fill both fields.".into();
                } else {
                    // Asynchronous login request to the server
                    let login_result = Runtime::new().unwrap().block_on(async {
                        authenticate_user(username.clone(), password.clone()).await
                    });

                    match login_result {
                        Ok(success_message) => {
                            app.message = success_message;
                            app.username = username.clone();
                            app.logged_in = true;
                            app.set_main_menu();
                        }
                        Err(error_message) => {
                            app.message = error_message;
                        }
                    }
                }
            }
            Esc => {
                app.set_main_menu();
                app.message = "Returning to main menu...".into();
            }
            _ => {}
        }
    }
}

async fn authenticate_user(username: String, password: String) -> Result<String, String> {
    let url = "http://127.0.0.1:8080/api/login"; // Replace with your server URL
    let client = Client::new();

    // Prepare the login payload
    let credentials = serde_json::json!({
        "username": username,
        "password": password,
    });

    // Send the login request
    match client.post(url).json(&credentials).send().await {
        Ok(response) => {
            if response.status().is_success() {
                Ok("Login successful!".to_string())
            } else {
                // Parse error message from the server if available
                let error_text = response.text().await.unwrap_or("Invalid credentials.".to_string());
                Err(error_text)
            }
        }
        Err(e) => {
            Err(format!("Failed to connect to server: {}", e))
        }
    }
}