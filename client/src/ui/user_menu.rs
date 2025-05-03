use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use crate::app::{App, FormState};
use crossterm::event::{KeyCode, KeyEvent};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let options = ["Chats", "Chatroom", "Add Friends", "Friend List", "Profile", "Log Out"];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints(vec![Constraint::Length(3); 7].into_iter().chain([Constraint::Min(0)]).collect::<Vec<_>>())
        .split(f.size());

    let selected = if let FormState::UserMenu { selected_index } = app.state {
        selected_index.min(options.len() - 1) // Ensure the index is within bounds
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

pub fn handle_input(app: &mut App, input: KeyEvent) {
    match input.code {
        KeyCode::Up => {
            // Navigate up
            if let FormState::UserMenu { selected_index } = &mut app.state {
                if *selected_index > 0 {
                    *selected_index -= 1;
                }
            }
        }
        KeyCode::Down => {
            // Navigate down
            if let FormState::UserMenu { selected_index } = &mut app.state {
                if *selected_index < 6 { // 7 options, 0 to 6
                    *selected_index += 1;
                }
            }
        }
        KeyCode::Enter => {
            // Handle selection (Enter key press)
            if let FormState::UserMenu { selected_index } = &mut app.state {
                match *selected_index {
                    0 => println!("Navigating to Chats..."),
                    1 => println!("Navigating to Chatroom..."),
                    2 => println!("Navigating to Add Friends..."),
                    3 => println!("Navigating to Friend List..."),
                    4 => {
                        if let Some(profile) = &app.profile {
                            app.set_profile_view(profile.clone());
                        } else {
                            println!("No profile loaded.");
                        }
                    }
                    5 => {
                        app.state = FormState::MainMenu;
                    }
                    _ => println!("Invalid selection"),
                }
            }
        }
        KeyCode::Esc => {
            // Escape logic (Exit or go back to the previous menu)
            println!("Exiting the menu...");
            // You can add more logic for handling the back action if needed.
        }
        _ => {}
    }
}
