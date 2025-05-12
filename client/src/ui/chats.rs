use crate::app::{App, FormState};
use crate::ui::create_chat::ChatCreationPhase;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

const PAGE_SIZE: u64 = 10;

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Min(5),    // Chat list
            Constraint::Length(1), // Page info
            Constraint::Length(3), // Add Chat
            Constraint::Length(3), // Message
        ])
        .split(f.area());

    if let FormState::Chats {
        selected_index,
        page,
        page_count,
    } = &app.state
    {
        let items: Vec<ListItem> = app
            .chats
            .iter()
            .enumerate()
            .map(|(i, chat)| {
                let style = if i == *selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let display_name = if chat.unread_count > 0 {
                    format!("{} ({})", chat.chat_name, chat.unread_count)
                } else {
                    chat.chat_name.clone()
                };

                ListItem::new(display_name).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Chats").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

        f.render_widget(list, chunks[0]);

        let page_number = {
            if *page_count == 0 {
                0
            } else {
                *page + 1
            }
        };

        let page_info = Paragraph::new(format!("Page {} of {}", page_number, *page_count)).style(
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        );
        f.render_widget(page_info, chunks[1]);

        let add_chat = Paragraph::new(Text::from("[Tab] Add New Chat"))
            .block(Block::default().title("New Chat").borders(Borders::ALL));
        f.render_widget(add_chat, chunks[2]);

        let message = Paragraph::new(Text::from(app.message.clone())).style(Style::default());
        f.render_widget(message, chunks[3]);
    }
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    if let FormState::Chats { selected_index, .. } = &mut app.state {
        match key.code {
            KeyCode::Enter | KeyCode::Char('\r') => {
                if let Some(chat) = app.chats.get(*selected_index) {
                    app.enter_chat_view(chat.id, chat.chat_name.clone(), 0, PAGE_SIZE, None)
                        .await;
                }
            }
            KeyCode::Tab => {
                let friends = app.get_friends().await;
                app.state = FormState::ChatCreation(ChatCreationPhase::FriendSelection {
                    selected: 0,
                    chosen: Vec::new(),
                    friends,
                });
            }
            KeyCode::Right => {
                let (page, page_count) = match &mut app.state {
                    FormState::Chats {
                        page, page_count, ..
                    } => (page, page_count),
                    _ => return,
                };
                if *page_count == 0 || *page >= *page_count - 1 {
                    app.message = "No more chats to view!".to_string();
                    return;
                }
                let next_page = *page + 1;

                app.enter_chats_view(next_page, PAGE_SIZE).await;
            }
            KeyCode::Left => {
                let page = match &mut app.state {
                    FormState::Chats { page, .. } => page,
                    _ => return,
                };
                if *page <= 0 {
                    app.message = "Already at most recent chats!".to_string();
                    return;
                }
                let next_page = *page - 1;

                app.enter_chats_view(next_page, PAGE_SIZE).await;
            }
            KeyCode::Up => {
                if *selected_index > 0 {
                    *selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if app.chats.len() <= 0 {
                    return;
                }
                
                if app.chats.len() == 1 {
                    *selected_index = 0;
                    return;
                }

                if *selected_index < app.chats.len() - 2 {
                    *selected_index += 1;
                }
            }
            KeyCode::Esc => {
                app.message.clear();
                app.set_user_menu().await;
            }
            _ => {}
        }
    }
}
