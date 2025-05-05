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
use crate::ui::create_chat::ChatCreationPhase;

const PAGE_SIZE: u64 = 10;

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

    let selected = if let FormState::Chats { selected_index } = app.state {
        selected_index
    } else {
        0
    };

    let items: Vec<ListItem> = app.chats.iter().enumerate().map(|(i, chat)| {
        let style = if i == selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(chat.chat_name.clone()).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title("Chats").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

    f.render_widget(list, chunks[0]);

    let add_chat = Paragraph::new(Text::from("[Tab] Add New Chat"))
        .block(Block::default().title("New Chat").borders(Borders::ALL));
    f.render_widget(add_chat, chunks[1]);

    let message = Paragraph::new(Text::from(app.message.clone())).style(Style::default());
    f.render_widget(message, chunks[2]);
}


pub async fn handle_input(app: &mut App, key: KeyEvent) {
    if let FormState::Chats { selected_index } = &mut app.state {
        match key.code {
            KeyCode::Enter | KeyCode::Char('\r') =>  {
                if let Some(chat) = app.chats.get(*selected_index) {
                    app.enter_chat_view(chat.id, chat.chat_name.clone(), 0, PAGE_SIZE).await;
                }
            },
            KeyCode::Tab => {
                let friends = app.get_friends().await;
                app.state = FormState::ChatCreation(
                    ChatCreationPhase::FriendSelection {
                        selected: 0,
                        chosen: Vec::new(),
                        friends,
                    }
                );
            }
            KeyCode::Up => {
                if *selected_index > 0 {
                    *selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if *selected_index < app.chats.len() - 1 {
                    *selected_index += 1;
                }
            }
            KeyCode::Esc => {
                app.message.clear();
                app.state = FormState::UserMenu { selected_index: 0 };
            }
            _ => {}
        }
    }
}
