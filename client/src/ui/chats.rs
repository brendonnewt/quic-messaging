use crate::app::{App, FormState};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Min(5),   // Chat list
            Constraint::Length(3), // Add Chat
            Constraint::Length(3), // Message
        ])
        .split(f.size());

    let chats: Vec<String> = Vec::new(); //app.get_chats(); // Your getter function
    let selected = if let FormState::Chats { selected_index } = app.state {
        selected_index
    } else { 0 };

    let items: Vec<ListItem> = chats.iter().enumerate().map(|(i, chat)| {
        let style = if i == selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(chat.clone()).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title("Chats").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

    f.render_widget(list, chunks[0]);

    let add_chat = Paragraph::new(Text::from("[Enter] Add New Chat"))
        .block(Block::default().title("New Chat").borders(Borders::ALL));
    f.render_widget(add_chat, chunks[1]);

    let message = Paragraph::new(Text::from(app.message.clone())).style(Style::default());
    f.render_widget(message, chunks[2]);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    if let FormState::Chats { selected_index } = app.state {
        match key.code {
            KeyCode::Enter | KeyCode::Char('\r') => match selected_index {
                0 => app.state = FormState::Chats { selected_index: 0 }, // index 0 = Chats
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
            KeyCode::Esc => {
                app.state = FormState::UserMenu { selected_index: 0 };
            }
            _ => {}
        }
    }
}
