use crate::app::{App, FormState};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;
use shared::client_response::ClientRequest;
use shared::client_response::Command::SendMessage;
use shared::models::chat_models::{ChatMessage, ChatMessages};

pub fn render<B: Backend>(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Min(10),   // Messages list
            Constraint::Length(3), // New message input
            Constraint::Length(3), // Message area
        ])
        .split(f.size());

    if let FormState::Chat {
        chat_name,
        chat_id,
        page,
        messages,
        input_buffer,
    } = &mut app.state {
        use ratatui::text::{Line, Span};

        let lines: Vec<Line> = messages
            .iter()
            .rev()
            .map(|msg| {
                Line::from(Span::raw(format!("{}: {}", msg.username, msg.content)))
            })
            .collect();

        let chat_paragraph = Paragraph::new(lines)
            .block(Block::default().title(chat_name.as_str()).borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(chat_paragraph, chunks[0]);

        let scroll_offset = input_buffer.len().saturating_sub(chunks[1].width as usize - 4); // account for borders

        let new_chat = Paragraph::new(Text::from(input_buffer.clone()))
            .block(Block::default().title("New Message").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .scroll((0, scroll_offset as u16));

        // Calculate the maximum width for text inside the block (excluding borders)
        let inner_width = chunks[1].width.saturating_sub(2);

        // Determine how far the cursor should go (clamp it inside the box)
        let cursor_offset = input_buffer.width().min(inner_width as usize - 1);

        // Set cursor just before the right border
        let cursor_x = chunks[1].x + 1 + cursor_offset as u16;
        let cursor_y = chunks[1].y + 1;
        f.set_cursor(cursor_x, cursor_y);

        f.render_widget(new_chat, chunks[1]);

        let combined_message = if app.message.is_empty() {
            "Press [Esc] to return to chat list".to_string()
        } else {
            format!("{} | Press [Esc] to return to chat list", app.message)
        };

        let message = Paragraph::new(Text::from(combined_message)).style(Style::default());
        f.render_widget(message, chunks[2]);
    } else {
        let fallback = Paragraph::new("Invalid state or failed to load chat view")
            .block(Block::default().title("Error").borders(Borders::ALL))
            .style(Style::default().fg(Color::Red));
        f.render_widget(fallback, chunks[0]);
    }
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    let (input_buffer, chat_id) = match &mut app.state {
        FormState::Chat {
            input_buffer,
            chat_id,
            ..
        } => (input_buffer, chat_id),
        _ => return,
    };

    match key.code {
        KeyCode::Char(c) => input_buffer.push(c),
        KeyCode::Backspace => {
            input_buffer.pop();
        }
        KeyCode::Enter => {
            let request = ClientRequest {
                jwt: Some(app.jwt.clone()),
                command: SendMessage {
                    chat_id: *chat_id,
                    content: input_buffer.clone(),
                },
            };
            let message_text = input_buffer.clone();
            let user_id = app.user_id;
            let response = match app.send_request(&request).await {
                Ok(response) => response,
                Err(err) => {
                    app.message = format!("Error: {}", err);
                    return;
                }
            };
            if response.success {
                let (input_buffer, messages) = match &mut app.state {
                    FormState::Chat {
                        input_buffer,
                        messages,
                        ..
                    } => (input_buffer, messages),
                    _ => return,
                };
                messages.insert(0, ChatMessage {
                    user_id,
                    username: app.username.clone(),
                    content: message_text,
                });
                input_buffer.clear();
                app.message.clear();
            } else {
                app.message = response.message.unwrap_or("Failed to send message".into());
            }
        }

        KeyCode::Esc => {
            app.state = FormState::Chats { selected_index: 0 };
        }
        _ => {}
    }
}
