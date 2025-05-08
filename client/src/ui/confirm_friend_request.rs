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
use crate::app::{App, FormState};

pub fn render<B: Backend>(f: &mut Frame, app: &mut App) {
    let options = ["Accept", "Decline"];
    let area = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Min(0)])
        .split(f.size())[0];

    let (req_i, opt_i) = if let FormState::ConfirmFriendRequest { selected_index, selected_option } = app.state {
        (selected_index, selected_option)
    } else {
        (0, 0)
    };

    let title = "Accept or Decline Friend Request?".to_string();

    let items: Vec<ListItem> = options.iter().enumerate().map(|(i, &label)| {
        let style = if i == opt_i {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(label).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, area);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
        match key.code {
            Up | Down => {
                if let FormState::ConfirmFriendRequest { selected_option, .. } = &mut app.state {
                    *selected_option = (*selected_option + 1) % 2;
                }
            }
            Enter => {
                let (req_idx, opt) = if let FormState::ConfirmFriendRequest {
                    selected_index,
                    selected_option,
                } = &app.state {
                    (*selected_index, *selected_option)
                } else {
                    return;
                };
                let fr_req = &app.friend_requests.incoming[req_idx];
                let cmd = if opt == 0{
                    Command::AcceptFriendRequest {
                        sender_id: fr_req.id.clone(),
                    }
                }else{
                    Command::DeclineFriendRequest {
                        sender_id: fr_req.id.clone(),
                    }
                };
                let req = ClientRequest{
                    jwt: Option::from(app.jwt.clone()),
                    command: cmd,
                };
                match app.send_request(&req).await {
                    Ok(response) => {
                        if response.success{
                            app.friend_requests.incoming.remove(req_idx);
                            if let Some(message) = response.message.clone(){
                                app.message = message;
                            }
                        }
                    },
                    Err(err) => {
                        app.message = err.to_string();
                    }
                }
                app.set_friend_requests().await;
            }
            Esc => {
                app.set_friend_requests().await;
            }
            _ => {}
        }
}
