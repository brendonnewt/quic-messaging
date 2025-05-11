use crate::app::{ActiveField, App, FormState};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::text::Text;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use shared::client_response::{ClientRequest, Command};
use shared::models::auth_models::AuthResponseModel;

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
            .fg(ratatui::style::Color::Black) // Ensuring text contrast
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
        FormState::RegisterForm {
            confirm_password, ..
        } => confirm_password.len(),
        _ => 0,
    })))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Confirm Password"),
    )
    .style(confirm_style);

    let message = Paragraph::new(app.message.clone()).style(Style::default());

    // Render the widgets in the correct chunks
    f.render_widget(username, chunks[0]);
    f.render_widget(password, chunks[1]);
    f.render_widget(confirm, chunks[2]);
    f.render_widget(message, chunks[3]);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    let (username, password, confirm_password, active_field) = match &mut app.state {
        FormState::RegisterForm {
            username,
            password,
            confirm_password,
            active_field,
        } => (username, password, confirm_password, active_field),
        _ => return,
    };
    match key.code {
        KeyCode::Down => {
            let next_field = match active_field {
                ActiveField::Username => ActiveField::Password,
                ActiveField::Password => ActiveField::ConfirmPassword,
                ActiveField::ConfirmPassword => ActiveField::Username,
            };
            app.set_active_field(next_field);
        }
        KeyCode::Up => {
            let prev_field = match active_field {
                ActiveField::Username => ActiveField::ConfirmPassword,
                ActiveField::Password => ActiveField::Username,
                ActiveField::ConfirmPassword => ActiveField::Password,
            };
            app.set_active_field(prev_field);
        }
        KeyCode::Tab => {
            let next_field = match active_field {
                ActiveField::Username => ActiveField::Password,
                ActiveField::Password => ActiveField::ConfirmPassword,
                ActiveField::ConfirmPassword => ActiveField::Username,
            };
            app.set_active_field(next_field);
        }
        KeyCode::Backspace => {
            let field = match active_field {
                ActiveField::Username => username,
                ActiveField::Password => password,
                ActiveField::ConfirmPassword => confirm_password,
            };
            field.pop(); // Remove last character
        }
        KeyCode::Char(c) => {
            let field = match active_field {
                ActiveField::Username => username,
                ActiveField::Password => password,
                ActiveField::ConfirmPassword => confirm_password,
            };
            field.push(c); // Add character to the field
        }
        KeyCode::Enter => {
            if password != confirm_password {
                app.message = "Passwords do not match!".to_string();
                return;
            }
            if username.trim().is_empty() || password.trim().is_empty() {
                app.message = "Username and password cannot be empty!".to_string();
                return;
            }
            let req = ClientRequest {
                jwt: None,
                command: Command::Register {
                    username: username.clone().trim().to_string(),
                    password: password.clone().trim().to_string(),
                },
            };
            let username = username.clone().trim().to_string();
            match app.send_request(&req).await {
                Ok(response) => {
                    if response.success {
                        if let Some(data) = response.data {
                            match serde_json::from_value::<AuthResponseModel>(data) {
                                Ok(auth_response) => {
                                    if let Some(jwt) = response.jwt.clone() {
                                        app.jwt = jwt;
                                        app.user_id = auth_response.user_id;
                                        app.username = username.clone();
                                        app.message = format!("Welcome {}!", username);
                                        app.set_user_menu().await;
                                    }
                                }
                                Err(e) => {
                                    app.message = format!("Parse error: {}", e);
                                }
                            }
                        }
                    } else if let Some(message) = response.message.clone() {
                        app.message = message;
                    }
                }
                Err(err) => {
                    app.message = err.to_string();
                }
            }
        }
        KeyCode::Esc => {
            app.message.clear();
            app.set_main_menu(); // Navigate back to the main menu
        }
        _ => {}
    }
}
