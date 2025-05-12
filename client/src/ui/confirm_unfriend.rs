use crate::app::{App, FormState};
use crossterm::event::KeyCode::{Down, Enter, Esc, Up};
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use shared::client_response::{ClientRequest, Command};

pub fn render<B: Backend>(f: &mut Frame, app: &mut App) {
    let options = ["Unfriend", "Go Back"];
    let area = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Min(0)])
        .split(f.area())[0];

    let opt_i = if let FormState::ConfirmUnfriend {
        selected_option, ..
    } = app.state
    {
        selected_option
    } else {
        0
    };

    let title = "Are you sure?".to_string();

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, &label)| {
            let style = if i == opt_i {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(label).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, area);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    match key.code {
        Up | Down => {
            if let FormState::ConfirmUnfriend {
                selected_option, ..
            } = &mut app.state
            {
                *selected_option = (*selected_option + 1) % 2;
            }
        }
        Enter => {
            let (req_idx, opt) = if let FormState::ConfirmUnfriend {
                selected_index,
                selected_option,
            } = &app.state
            {
                (*selected_index, *selected_option)
            } else {
                return;
            };
            let fr_req = &app.friend_list.users[req_idx];
            if opt == 0 {
                let cmd = {
                    Command::RemoveFriend {
                        friend_id: fr_req.id,
                    }
                };
                let req = ClientRequest {
                    jwt: Option::from(app.jwt.clone()),
                    command: cmd,
                };
                match app.send_request(&req).await {
                    Ok(response) => {
                        if response.success {
                            app.friend_list.users.remove(req_idx);
                        }
                    }
                    Err(err) => {
                        app.message = err.to_string();
                    }
                }
            }
            app.set_friend_list().await;
        }
        Esc => {
            app.message.clear();
            app.set_friend_list().await;
        }
        _ => {}
    }
}
