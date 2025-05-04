use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use shared::client_response::{ClientRequest, Command};
use crate::app::{App, FormState};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let options = ["Friend Requests", "Current Friends", "Remove Friends"];
    Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints(vec![Constraint::Length(3); 6].into_iter().chain([Constraint::Min(0)]).collect::<Vec<_>>())
        .split(f.size());

    let selected = if let FormState::FriendMenu { selected_index } = app.state {
        selected_index
    } else { 0 };

    let items: Vec<ListItem> = options.iter().enumerate().map(|(i, &opt)| {
        let style = if i == selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(opt).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Friends"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    let area = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Min(0)])
        .split(f.size())[0];

    f.render_widget(list, area);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    use KeyCode::*;

    if let FormState::FriendMenu { selected_index } = &mut app.state {
        match key.code {
            Up => {
                if *selected_index > 0 {
                    *selected_index -= 1;
                }
            }
            Down => {
                if *selected_index < 2 {
                    *selected_index += 1;
                }
            }
            Enter | Char('\r') => {
                match *selected_index {
                    0 => {
                        let req = ClientRequest {
                            jwt: Option::from(app.jwt.clone()),
                            command: Command::GetFriendRequests {}
                        };
                        match app.send_request(&req).await {
                            Ok(resp) => {
                                if resp.success {
                                    if let Some(data) = resp.data.clone() {
                                        app.friend_requests = serde_json::from_value(resp.data.into())
                                    }
                                }else if let Some(message) = resp.message.clone() {
                                    app.message = message;
                                }
                            },
                            Err(e) => {
                                app.message = e.to_string();
                            }
                        }
                        app.set_friend_requests();
                    }
                    1 => { /* View Friends */ }
                    2 => { /* Remove Friends */ }
                    _ => {}
                }
            }
            Esc => {
                app.set_user_menu();
            }
            _ => {}
        }

    }
}