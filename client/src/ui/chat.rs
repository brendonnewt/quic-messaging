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

    // Pattern match early, then borrow rest of app freely
    let (chat_name, page, messages, input_buffer) = match &mut app.state {
        FormState::Chat { chat_name, page, messages, input_buffer } => (chat_name, page, messages, input_buffer),
        _ => return,
    };

    // Build chat messages list with username prepended
    let items: Vec<ListItem> = messages.iter().enumerate().map(|(i, msg)| {
        let styled = Text::from(format!("{}: {}", msg.username.clone(), msg.content));
        ListItem::new(styled)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(chat_name.as_str()).borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

    f.render_widget(list, chunks[0]);

    // New chat input field (active)
    let new_chat = Paragraph::new(Text::from(input_buffer.clone()))
        .block(Block::default().title("New Message").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black));

    f.render_widget(new_chat, chunks[1]);

    let message = Paragraph::new(Text::from(app.message.clone())).style(Style::default());
    f.render_widget(message, chunks[2]);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    if let FormState::Chat { input_buffer, .. } = &mut app.state {
        match key.code {
            KeyCode::Char(c) => input_buffer.push(c),
            KeyCode::Backspace => {
                input_buffer.pop();
            }
            KeyCode::Enter => {
                // Logic to send message goes here
                // For example: call app.send_message(input_buffer.clone()).await;
                // Then clear buffer:
                input_buffer.clear();
            }
            KeyCode::Esc => {
                app.state = FormState::UserMenu { selected_index: 0 };
            }
            _ => {}
        }
    }
}