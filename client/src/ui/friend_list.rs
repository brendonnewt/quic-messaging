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
        .split(f.area());

    let fr_list = &app.friend_list;

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

    let list = {
        if app.friend_list_num == 0 {
            List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("No friends")
                        .title("Much sad :("),
                )
                .highlight_style(Style::default().bg(Color::DarkGray))
        } else {
            List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Friend List")
                        .title("Select Friend to Unfriend Them"),
                )
                .highlight_style(Style::default().bg(Color::DarkGray))
        }
    };

    let area = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Min(0)])
        .split(f.area())[0];

    f.render_widget(list, area);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
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
                        if *selected_index + 1 < app.friend_list_num {
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

        Esc => {
            app.message.clear();
            app.set_friend_menu()
        }

        _ => {}
    }
}
