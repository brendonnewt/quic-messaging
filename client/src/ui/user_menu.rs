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
use crate::app::{App, FormState};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let options = ["Chats", "Add Friends", "Friend List", "Settings", "Log Out"];

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
        .block(Block::default().borders(Borders::ALL).title("User Menu"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, main_chunks[0]);

    let combined_message = if app.message.is_empty() {
        format!("You have {} unread messages", app.unread_count)
    } else {
        format!("{} | You have {} unread messages", app.message, app.unread_count)
    };

    // Render app message
    let message = Paragraph::new(combined_message)
        .block(Block::default().borders(Borders::ALL).title("Status"));

    f.render_widget(message, main_chunks[1]);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    if let FormState::UserMenu { selected_index } = app.state {
        match key.code {
            KeyCode::Enter | KeyCode::Char('\r') => match selected_index {
                0 => {
                    app.message.clear();
                    app.enter_chats_view().await
                }, // index 0 = Chats
                1 => {
                    app.message.clear();
                    app.set_add_friend()
                },
                2 => {
                    app.message.clear();
                    app.set_friend_menu();
                },
                // 4 => app.state = FormState::Settings,
                4 => {
                    app.message.clear();
                    app.logout().await
                }, // Log Out
                _ => {}
            },
            KeyCode::Up => {
                if selected_index > 0 {
                    app.set_user_menu_selected_index(selected_index - 1);
                }
            }
            KeyCode::Down => {
                if selected_index < 4 {
                    app.set_user_menu_selected_index(selected_index + 1);
                }
            }
            _ => {}
        }
    }
}

