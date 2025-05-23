use crate::app::{App, FormState};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Position;
use ratatui::text::{Line, Span};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use shared::client_response::Command::{GetChatMessages, SendMessage};
use shared::client_response::{ClientRequest, Command};
use shared::models::chat_models::{ChatMessages, Count};
use std::cmp::PartialEq;
use unicode_width::UnicodeWidthStr;

const CHATS_PAGE_SIZE: u64 = 10;

enum PageSize {
    Small = 5,
    Medium = 10,
    Large = 20,
    XLarge = 40,
}

impl PageSize {
    fn get_size(&self) -> u64 {
        match self {
            PageSize::Small => 5,
            PageSize::Medium => 10,
            PageSize::Large => 20,
            PageSize::XLarge => 40,
        }
    }

    fn decrease(&mut self) -> Self {
        match self {
            PageSize::Small => PageSize::Small,
            PageSize::Medium => PageSize::Small,
            PageSize::Large => PageSize::Medium,
            PageSize::XLarge => PageSize::Large,
        }
    }

    fn increase(&mut self) -> Self {
        match self {
            PageSize::Small => PageSize::Medium,
            PageSize::Medium => PageSize::Large,
            PageSize::Large => PageSize::XLarge,
            PageSize::XLarge => PageSize::XLarge,
        }
    }

    fn from_u64(size: u64) -> Self {
        match size {
            5 => PageSize::Small,
            10 => PageSize::Medium,
            20 => PageSize::Large,
            40 => PageSize::XLarge,
            _ => PageSize::Medium,
        }
    }
}

pub fn render<B: Backend>(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Min(9),    // Messages list
            Constraint::Length(1), // Page info
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // New message input
            Constraint::Length(3), // Message area
        ])
        .split(f.area());

    if let FormState::Chat {
        chat_name,
        page,
        page_count,
        messages,
        input_buffer,
        ..
    } = &mut app.state
    {
        let lines: Vec<Line> = messages
            .iter()
            .map(|msg| {
                let name_span = if msg.username == app.username {
                    Span::styled(
                        format!("{}: ", msg.username),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::raw(format!("{}: ", msg.username))
                };
                let content_span = Span::raw(&msg.content);
                Line::from(vec![name_span, content_span])
            })
            .collect();

        let chat_paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(chat_name.as_str())
                    .borders(Borders::ALL),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(chat_paragraph, chunks[0]);

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

        let visible_width = chunks[3].width.saturating_sub(4) as usize;
        let scroll_offset = if visible_width == 0 {
            0
        } else {
            input_buffer.len().saturating_sub(visible_width)
        };

        let new_chat = Paragraph::new(Text::from(input_buffer.clone()))
            .block(Block::default().title("New Message").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .scroll((0, scroll_offset as u16));

        let inner_width = chunks[3].width.saturating_sub(2);
        let cursor_offset = input_buffer
            .width()
            .min(inner_width.saturating_sub(1) as usize);

        // Set cursor just before the right border
        let cursor_x = chunks[3].x + 1 + cursor_offset as u16;
        let cursor_y = chunks[3].y + 1;
        f.set_cursor_position(Position::from((cursor_x, cursor_y)));

        f.render_widget(new_chat, chunks[3]);

        let combined_message = if app.message.is_empty() {
            "Press [Esc] to return to chat list".to_string()
        } else {
            format!("{} | Press [Esc] to return to chat list", app.message)
        };

        let message = Paragraph::new(Text::from(combined_message)).style(Style::default());
        f.render_widget(message, chunks[4]);
    } else {
        let fallback = Paragraph::new("Invalid state or failed to load chat view")
            .block(Block::default().title("Error").borders(Borders::ALL))
            .style(Style::default().fg(Color::Red));
        f.render_widget(fallback, chunks[0]);
    }
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => handle_char(app, c).await,
        KeyCode::Backspace => handle_backspace(app).await,
        KeyCode::Enter => handle_enter(app).await,
        KeyCode::Up => handle_up(app).await,
        KeyCode::Down => handle_down(app).await,
        KeyCode::Left => handle_left(app).await,
        KeyCode::Right => handle_right(app).await,
        KeyCode::Esc => {
            app.message.clear();
            app.enter_chats_view(0, CHATS_PAGE_SIZE).await;
        }
        _ => {}
    }
}

pub async fn handle_char(app: &mut App, c: char) {
    let input_buffer = match &mut app.state {
        FormState::Chat { input_buffer, .. } => input_buffer,
        _ => return,
    };
    input_buffer.push(c);
}

pub async fn handle_backspace(app: &mut App) {
    let input_buffer = match &mut app.state {
        FormState::Chat { input_buffer, .. } => input_buffer,
        _ => return,
    };
    input_buffer.pop();
}

pub async fn handle_enter(app: &mut App) {
    let (input_buffer, chat_id) = match &mut app.state {
        FormState::Chat {
            input_buffer,
            chat_id,
            ..
        } => (input_buffer, chat_id),
        _ => return,
    };
    if input_buffer.trim().is_empty() {
        return;
    }
    let request = ClientRequest {
        jwt: Some(app.jwt.clone()),
        command: SendMessage {
            chat_id: *chat_id,
            content: input_buffer.clone(),
        },
    };
    let response = match app.send_request(&request).await {
        Ok(response) => response,
        Err(err) => {
            app.message = format!("Error: {}", err);
            return;
        }
    };
    if response.success {
        let (input_buffer, chat_id, page_size) = match &mut app.state {
            FormState::Chat {
                input_buffer,
                chat_id,
                page_size,
                ..
            } => (input_buffer, chat_id, page_size),
            _ => return,
        };
        input_buffer.clear();
        app.message.clear();
        let chat_id = *chat_id;
        let page_size = *page_size;
        get_messages(app, chat_id, 0, page_size).await;
    } else {
        app.message = response.message.unwrap_or("Failed to send message".into());
    }
}

pub async fn handle_up(app: &mut App) {
    let (page, page_count, chat_id, page_size) = match &mut app.state {
        FormState::Chat {
            page,
            page_count,
            chat_id,
            page_size,
            ..
        } => (page, page_count, chat_id, page_size),
        _ => return,
    };
    if *page_count == 0 || *page >= *page_count - 1 {
        app.message = "No more messages in chat!".to_string();
        return;
    }
    let chat_id = *chat_id;
    let next_page = *page + 1;
    let page_size = *page_size;

    get_messages(app, chat_id, next_page, page_size).await;
}

pub async fn handle_down(app: &mut App) {
    let (page, chat_id, page_size) = match &mut app.state {
        FormState::Chat {
            page,
            chat_id,
            page_size,
            ..
        } => (page, chat_id, page_size),
        _ => return,
    };
    if *page <= 0 {
        app.message = "Already at most recent messages!".to_string();
        return;
    }
    let chat_id = *chat_id;
    let next_page = *page - 1;
    let page_size = *page_size;

    get_messages(app, chat_id, next_page, page_size).await;
}

pub async fn handle_left(app: &mut App) {
    let (page, page_count, chat_id, page_size) = match &mut app.state {
        FormState::Chat {
            page,
            page_count,
            chat_id,
            page_size,
            ..
        } => (page, page_count, chat_id, page_size),
        _ => return,
    };

    if *page_size == PageSize::Small.get_size() {
        return;
    }

    *page_size = PageSize::from_u64(*page_size).decrease().get_size();

    let chat_id = *chat_id;
    let next_page = *page;
    let page_size = *page_size;

    get_messages(app, chat_id, next_page, page_size).await;
}

pub async fn handle_right(app: &mut App) {
    let (page, chat_id, page_size) = match &mut app.state {
        FormState::Chat {
            page,
            chat_id,
            page_size,
            ..
        } => (page, chat_id, page_size),
        _ => return,
    };

    if *page_size == PageSize::XLarge.get_size() {
        return;
    }

    *page_size = PageSize::from_u64(*page_size).increase().get_size();

    let chat_id = *chat_id;
    let next_page = *page;
    let page_size = *page_size;

    get_messages(app, chat_id, next_page, page_size).await;
}

pub async fn get_messages(app: &mut App, chat_id: i32, mut new_page: u64, page_size: u64) {
    let request = ClientRequest {
        jwt: Some(app.jwt.clone()),
        command: Command::GetChatPages { chat_id, page_size },
    };

    match app.send_request(&request).await {
        Ok(response) => {
            if response.success {
                if let Some(data) = response.data {
                    match serde_json::from_value::<Count>(data) {
                        Ok(count) => {
                            let page_count = match &mut app.state {
                                FormState::Chat { page_count, .. } => page_count,
                                _ => return,
                            };
                            *page_count = count.count;
                            if new_page >= count.count && new_page != 0 {
                                new_page = count.count - 1;
                            }
                        }
                        Err(e) => {
                            app.message = format!("Parse error: {}", e);
                        }
                    }
                } else {
                    app.message = "No page count returned".into();
                }
            } else {
                app.message = response.message.unwrap_or("Failed to get chat".into());
            }
        }
        Err(err) => {
            app.message = format!("Error: {}", err);
        }
    }
    
    let request = ClientRequest {
        jwt: Some(app.jwt.clone()),
        command: GetChatMessages {
            chat_id,
            page: new_page,
            page_size,
        },
    };
    let response = match app.send_request(&request).await {
        Ok(response) => response,
        Err(err) => {
            app.message = format!("Error: {}", err);
            return;
        }
    };
    if response.success {
        let (messages, page) = match &mut app.state {
            FormState::Chat { messages, page, .. } => (messages, page),
            _ => return,
        };
        if let Some(data) = response.data {
            match serde_json::from_value::<ChatMessages>(data) {
                Ok(new_messages) => {
                    *messages = new_messages.messages;
                    *page = new_page;
                }
                Err(e) => {
                    app.message = format!("Parse error: {}", e);
                }
            }
        } else {
            app.message = "No chat data returned".into();
        }
    } else {
        app.message = response.message.unwrap_or("Failed to send message".into());
    }
}
