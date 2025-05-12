use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use shared::client_response::{ClientRequest, Command};
use crate::app::{App, ActiveField, FormState};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    // Debug: show when render runs and current state


    if let FormState::ProfileView { new_password, active_field } = &app.state {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(4)
            .constraints([
                Constraint::Length(3), // Password
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(f.area());

        let pass_style = if *active_field == ActiveField::Password {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        // Debug: show the current password length being displayed (masked)


        // Password box (masked)
        let pass_para = Paragraph::new(Text::from("*".repeat(new_password.len())))
            .block(Block::default().borders(Borders::ALL).title("Password"))
            .style(pass_style);
        f.render_widget(pass_para, chunks[1]);
    }
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    use KeyCode::*;

    // Only match if we're in the ProfileView
    let (new_password, active_field) = match &mut app.state {
        FormState::ProfileView { new_password, active_field } => (new_password, active_field),
        _ => return,
    };

    match key.code {
        Backspace => {
            if *active_field == ActiveField::Password {

                new_password.pop();

            }
        }
        Char(c) => {
            if *active_field == ActiveField::Password {

                new_password.push(c);

            }
        }
        Enter => {
            let req = ClientRequest {
                jwt: Some(app.jwt.clone()),
                command: Command::UpdateProfile {
                    new_password: new_password.clone(),
                },
            };

            match app.send_request(&req).await {
                Ok(response) => {
                    if response.success {
                        app.message = "Password updated successfully.".into();
                        app.set_user_menu().await; // <-- return to user menu
                    } else if let Some(message) = response.message.clone() {
                        app.message = message;
                    }
                },
                Err(err) => {
                    app.message = err.to_string();
                }
            }
        }

        Esc => {

            app.set_user_menu().await;
            app.message = "Returning to main menu...".into();
        }
        _ => {
            // No print for unhandled keys like arrow keys, tabs, etc.
        }
    }
}
