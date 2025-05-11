use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use KeyCode::*;
use shared::client_response::{ClientRequest, Command};
use crate::app::{App, ActiveField, FormState};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {

    if let FormState::AddFriend { id, active_field } = &app.state {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(4)
            .constraints([
                Constraint::Length(3), // ID
                Constraint::Length(3), // Message
                Constraint::Min(0),
            ])
            .split(f.area());

        let id_style = if *active_field == ActiveField::Username {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        // ID box
        let id_para = Paragraph::new(Text::from(id.clone()))
            .block(Block::default().borders(Borders::ALL).title("Friend Username"))
            .style(id_style);
        f.render_widget(id_para, chunks[0]);

        // Message box
        let msg_para = Paragraph::new(app.message.clone())
            .block(Block::default().title("")) // no borders for a message
            .style(Style::default());
        f.render_widget(msg_para, chunks[2]);
    }
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    // Pattern match early, then borrow rest of app freely
    let id= match &mut app.state {
        FormState::AddFriend { id, .. } => id,
        _ => return,
    };

    let app_jwt = app.jwt.clone();

    match key.code {
        Backspace => {
            id.pop();
        }
        Char(c) => {
            id.push(c)
        }
        Enter => {
            if id.clone().trim().is_empty() {
                return;
            }
            let req = ClientRequest {
                jwt: Option::from(app_jwt),
                command: Command::SendFriendRequest {
                    receiver_username: id.clone(),
                },
            };
            match app.send_request(&req).await {
                Ok(response) => {
                    if response.success {
                        app.set_user_menu().await;
                        app.message = "Friend request sent!".to_string()
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
            app.message = "".to_string();
        }
        _ => {}
    }
}
