// This module defines a Tab-triggered chat creation UI flow for a terminal chat app.
// It supports selecting friends and optionally naming group chats.

use crate::app::{App, FormState};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use shared::models::user_models::User;

// Additional FormState variant to support the chat creation flow
#[derive(Debug, Clone)]
pub enum ChatCreationPhase {
    FriendSelection { selected: usize, chosen: Vec<User>, friends: Vec<User> },
    NamingGroup { name_input: String, chosen: Vec<User> },
}

// Render function for the chat creation UI
pub fn render<B: Backend>(f: &mut Frame, app: &mut App) {
    let size = f.size();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(size);

    let phase = if let FormState::ChatCreation(phase) = &mut app.state {
        phase
    } else {
        return
    };

    match phase {
        ChatCreationPhase::FriendSelection { selected, chosen, friends } => {
            let items: Vec<ListItem> = friends
                .iter()
                .map(|friend| {
                    let style = if chosen.contains(friend) {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(friend.username.clone()).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Select Friends (Space to toggle, Enter to continue)").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

            f.render_stateful_widget(list, layout[0], &mut app.list_state.clone());

            let summary = Paragraph::new(Text::from(format!(
                "Selected: {}",
                chosen.iter().map(|friend| friend.username.clone()).collect::<Vec<_>>().join(", ")
            )))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(summary, layout[1]);
        }
        ChatCreationPhase::NamingGroup { name_input, chosen } => {
            let input = Paragraph::new(name_input.clone())
                .block(Block::default().title("Group Name").borders(Borders::ALL))
                .style(Style::default().fg(Color::White));
            f.render_widget(input, layout[0]);

            let info = Paragraph::new(Text::from(format!(
                "Creating chat with: {}",
                chosen.iter().map(|friend| friend.username.clone()).collect::<Vec<_>>().join(", ")
            )))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(info, layout[1]);
        }
    }

    let hint = Paragraph::new("Press [Esc] to cancel").style(Style::default().fg(Color::DarkGray));
    f.render_widget(hint, layout[2]);
}

// Handle input during chat creation
pub async fn handle_input(app: &mut App, key: KeyEvent) {
    let phase = if let FormState::ChatCreation(phase) = &mut app.state {
        phase
    } else {
        return
    };
    match phase {
        ChatCreationPhase::FriendSelection { selected, chosen, friends } => match key.code {
            KeyCode::Down => {
                if *selected + 1 < friends.len() {
                    *selected += 1;
                }
            }
            KeyCode::Up => {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
            KeyCode::Char(' ') => {
                let friend = &friends[*selected];
                if chosen.contains(friend) {
                    chosen.retain(|f| f != friend);
                } else {
                    chosen.push(friend.clone());
                }
            }
            KeyCode::Enter => {
                if chosen.len() > 1 {
                    *phase = ChatCreationPhase::NamingGroup {
                        name_input: String::new(),
                        chosen: chosen.clone(),
                    };
                } else if chosen.len() == 1 {
                    // Single chat: directly create chat
                    //app.create_chat(chosen.clone(), None).await;
                    app.enter_chats_view().await;
                }
            }
            KeyCode::Esc => app.enter_chats_view().await,
            _ => {}
        },
        ChatCreationPhase::NamingGroup { name_input, chosen } => match key.code {
            KeyCode::Char(c) => name_input.push(c),
            KeyCode::Backspace => {
                name_input.pop();
            }
            KeyCode::Enter => {
                let name = name_input.trim().to_string();
                if !name.is_empty() {
                    //app.create_chat(chosen.clone(), Some(name)).await;
                    app.enter_chats_view().await;
                } else {
                    app.message = "Group name cannot be empty.".into();
                }
            }
            KeyCode::Esc => app.enter_chats_view().await,
            _ => {}
        },
    }
}
