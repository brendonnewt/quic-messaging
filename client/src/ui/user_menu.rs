use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use ratatui::widgets::Paragraph;
use shared::client_response::{ClientRequest, Command};
use crate::app::{ActiveField, App, FormState};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let options = ["Chats", "Chatroom", "Add Friends", "Friend List", "Settings", "Log Out"];

    // Split the screen into menu area and message area
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Min(10),  // Menu list area
            Constraint::Length(3), // Message display
        ])
        .split(f.size());

    // Further split the top part for list items
    let selected = if let FormState::UserMenu { selected_index } = app.state {
        selected_index
    } else {
        0
    };

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

    f.render_widget(list, main_chunks[0]);

    // Render app message
    let message = Paragraph::new(app.message.clone())
        .block(Block::default().borders(Borders::ALL).title("Status"));

    f.render_widget(message, main_chunks[1]);
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

