use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use shared::client_response::{ClientRequest, Command};
use crate::app::{ActiveField, App, FormState};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let options = ["Chats", "Chatroom", "Add Friends", "Friend List", "Settings", "Log Out"];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints(vec![Constraint::Length(3); 6].into_iter().chain([Constraint::Min(0)]).collect::<Vec<_>>())
        .split(f.size());

    let selected = if let FormState::UserMenu { selected_index } = app.state {
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
        .block(Block::default().borders(Borders::ALL).title("Your Menu"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    let area = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Min(0)])
        .split(f.size())[0];

    f.render_widget(list, area);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    if let FormState::UserMenu { selected_index } = app.state {
        match key.code {
            KeyCode::Enter | KeyCode::Char('\r') => match selected_index {
                0 => app.enter_chats_view().await, // index 0 = Chats
                // 1 => app.state = FormState::Chatroom,
                // 2 => app.state = FormState::AddFriends,
                // 3 => app.state = FormState::FriendList,
                // 4 => app.state = FormState::Settings,
                5 => app.set_main_menu(), // Log Out
                _ => {}
            },
            KeyCode::Up => {
                if selected_index > 0 {
                    app.set_user_menu_selected_index(selected_index - 1);
                }
            }
            KeyCode::Down => {
                if selected_index < 5 {
                    app.set_user_menu_selected_index(selected_index + 1);
                }
            }
            _ => {}
        }
    }
}

