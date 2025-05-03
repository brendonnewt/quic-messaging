use crossterm::event::{KeyCode, KeyEvent};
use reqwest::{self, Client, Error}; // To access reqwest::Error and reqwest::ErrorKind
use serde::Serialize;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::text::Text;
use crate::app::{App, ActiveField, FormState};

#[derive(Serialize)]
struct RegisterData {
    username: String,
    password: String,
}

async fn register_user(username: &str, password: &str) -> Result<String, Error> {
    let url = "http://127.0.0.1:8080/api/register";

    let client = Client::new();
    let register_data = RegisterData {
        username: username.to_string(),
        password: password.to_string(),
    };

    // Send the registration data to the server
    let response = client
        .post(url)
        .json(&register_data) // Serialize the data into JSON format
        .send()
        .await?;

    // This will automatically handle status codes and return an error if it's not a success
    response.error_for_status()?;  // This will error if the status is not 2xx

    Ok("Registration successful.".to_string())
}
pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.size());

    // Retrieve the active field using the get_active_field method
    let active_field = app.get_active_field();

    // Styling to highlight the active field
    let username_style = if active_field == Some(ActiveField::Username) {
        Style::default()
            .bg(ratatui::style::Color::Yellow)
            .fg(ratatui::style::Color::Black)  // Ensuring text contrast
    } else {
        Style::default()
    };
    let password_style = if active_field == Some(ActiveField::Password) {
        Style::default()
            .bg(ratatui::style::Color::Yellow)
            .fg(ratatui::style::Color::Black)
    } else {
        Style::default()
    };
    let confirm_style = if active_field == Some(ActiveField::ConfirmPassword) {
        Style::default()
            .bg(ratatui::style::Color::Yellow)
            .fg(ratatui::style::Color::Black)
    } else {
        Style::default()
    };

    // Render the actual text for username, and asterisks for password fields
    let username = Paragraph::new(Text::from(match &app.state {
        FormState::RegisterForm { username, .. } => username.clone(),
        _ => String::new(),
    }))
        .block(Block::default().borders(Borders::ALL).title("Username"))
        .style(username_style);

    let password = Paragraph::new(Text::from("*".repeat(match &app.state {
        FormState::RegisterForm { password, .. } => password.len(),
        _ => 0,
    })))
        .block(Block::default().borders(Borders::ALL).title("Password"))
        .style(password_style);

    let confirm = Paragraph::new(Text::from("*".repeat(match &app.state {
        FormState::RegisterForm { confirm_password, .. } => confirm_password.len(),
        _ => 0,
    })))
        .block(Block::default().borders(Borders::ALL).title("Confirm Password"))
        .style(confirm_style);

    let message = Paragraph::new(app.message.clone())
        .style(Style::default());

    // Render the widgets in the correct chunks
    f.render_widget(username, chunks[0]);
    f.render_widget(password, chunks[1]);
    f.render_widget(confirm, chunks[2]);
    f.render_widget(message, chunks[3]);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Down => {
            if let Some(active_field) = app.get_active_field() {
                let next_field = match active_field {
                    ActiveField::Username => ActiveField::Password,
                    ActiveField::Password => ActiveField::ConfirmPassword,
                    ActiveField::ConfirmPassword => ActiveField::Username,
                };
                app.set_active_field(next_field);
            }
        }
        KeyCode::Up => {
            if let Some(active_field) = app.get_active_field() {
                let prev_field = match active_field {
                    ActiveField::Username => ActiveField::ConfirmPassword,
                    ActiveField::Password => ActiveField::Username,
                    ActiveField::ConfirmPassword => ActiveField::Password,
                };
                app.set_active_field(prev_field);
            }
        }
        KeyCode::Tab => {
            if let Some(active_field) = app.get_active_field() {
                let next_field = match active_field {
                    ActiveField::Username => ActiveField::Password,
                    ActiveField::Password => ActiveField::ConfirmPassword,
                    ActiveField::ConfirmPassword => ActiveField::Username,
                };
                app.set_active_field(next_field);
            }
        }
        KeyCode::Backspace => {
            if let FormState::RegisterForm { username, password, confirm_password, active_field } = &mut app.state {
                let field = match active_field {
                    ActiveField::Username => username,
                    ActiveField::Password => password,
                    ActiveField::ConfirmPassword => confirm_password,
                };
                field.pop(); // Remove last character
            }
        }
        KeyCode::Char(c) => {
            if let FormState::RegisterForm { username, password, confirm_password, active_field } = &mut app.state {
                let field = match active_field {
                    ActiveField::Username => username,
                    ActiveField::Password => password,
                    ActiveField::ConfirmPassword => confirm_password,
                };
                field.push(c); // Add character to the field
            }
        }
        KeyCode::Enter => {
            if let FormState::RegisterForm { username, password, confirm_password, .. } = &app.state {
                if password == confirm_password {
                    app.message = "Sending registration data to the server...".to_string();
                    // Call the function to send data to the server
                    match register_user(username, password).await {  // Add `.await` here
                        Ok(message) => {
                            app.message = message;
                            app.set_login_form();  // Transition to login form or another appropriate state
                        }
                        Err(e) => {
                            app.message = format!("Error registering: {}", e);
                        }
                    }
                } else {
                    app.message = "Passwords do not match. Try again.".to_string();
                }
            }
        }
        KeyCode::Esc => {
            app.set_main_menu();  // Navigate back to the main menu
            app.message = "Returning to main menu...".to_string();
        }
        _ => {}
    }
}
