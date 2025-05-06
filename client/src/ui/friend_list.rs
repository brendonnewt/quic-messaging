use crate::app::{ActiveField, App, FormState};
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::event::KeyCode::{Down, Enter, Esc, Up};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use shared::client_response::{ClientRequest, Command};

pub fn render<B: Backend>(f: &mut Frame, app: &mut App) {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints(
            vec![Constraint::Length(3); 6]
                .into_iter()
                .chain([Constraint::Min(0)])
                .collect::<Vec<_>>(),
        )
        .split(f.size());

    let fr_list = match &app.friend_list {
        Ok(list) => list,
        Err(_) => {
            // Render an empty list
            let empty = List::new(vec![ListItem::new("No Friends")]).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("No Friends, Much Sad"),
            );
            return f.render_widget(empty, f.size());
        }
    };

    let selected = if let FormState::FriendList { selected_index } = app.state {
        selected_index
    } else {
        0
    };

    let items: Vec<ListItem> = fr_list
        .users
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let display = opt.username.clone();
            let style = if i == selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(display).style(style)
        })
        .collect();

    app.set_friend_list_num(items.len());

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Friend List").title("Select Friend to Unfriend Them"))
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

    match key.code {
        Up | Down => {
            if let FormState::FriendList { selected_index } = &mut app.state {
                match key.code {
                    Up => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                        }
                    }
                    Down => {
                        if *selected_index < app.friend_list_num {
                            *selected_index += 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        Enter => {
            if app.friend_list_num <= 0 {
                return;
            }
            let idx = if let FormState::FriendList { selected_index } = &app.state {
                *selected_index
            } else {
                return;
            };
            app.set_confirm_unfriend(idx);
        }

        Esc => app.set_friend_menu(),

        _ => {}
    }
}