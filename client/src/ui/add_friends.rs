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
    //eprintln!("RENDER called; state={:?}", app.state);

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

        // Id box
        let id_para = Paragraph::new(Text::from(id.clone()))
            .block(Block::default().borders(Borders::ALL).title("Friend ID"))
            .style(id_style);
        f.render_widget(id_para, chunks[0]);

        // Message box
        let msg_para = Paragraph::new(app.message.clone())
            .block(Block::default().title("")) // no borders for message
            .style(Style::default());
        f.render_widget(msg_para, chunks[2]);
    }
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    use KeyCode::*;

    // Pattern match early, then borrow rest of app freely
    let (id, active_field) = match &mut app.state {
        FormState::AddFriend { id, active_field } => (id, active_field),
        _ => return,
    };

    match key.code {
        Backspace => {
            id.pop();
        }
        Char(c) => {
            if c.is_ascii_digit() {
                id.push(c);
            }
        }
        Enter => {
            let req = ClientRequest {
                jwt: None,
                command: Command::SendFriendRequest {
                    receiver_id: id.clone().parse().unwrap(),
                },
            };
            match app.send_request(&req).await {
                Ok(response) => {
                    if response.success {
                        if let Some(jwt) = response.jwt.clone() {
                            app.jwt = jwt;
                            app.state = FormState::UserMenu { selected_index: 0 };
                        }
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
            app.set_user_menu();
            app.message = "Returning to user menu...".into();
        }
        _ => {}
    }
}
