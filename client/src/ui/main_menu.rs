use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.size());

    // Display personalized message if logged in
    let main_menu_message = if app.logged_in {
        format!("Welcome, {}", app.username) // Display the stored username
    } else {
        "Select an option".into() // Default message if not logged in
    };

    // Render the main menu greeting or instruction message
    let paragraph = Paragraph::new(main_menu_message)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .style(Style::default().fg(Color::White));
    f.render_widget(paragraph, chunks[0]);

    // Post-login options
    let post_login_options = vec![
        "Chats",
        "Chatroom",
        "Add Friends",
        "Friend List",
        "Settings",
        "Log Out",
        "Close",
    ];

    // Pre-login options
    let pre_login_options = vec!["Login", "Register", "Close"];

    // Choose the correct options based on login status
    let options = if app.logged_in {
        post_login_options
    } else {
        pre_login_options
    };

    // Create a list of items for the chosen options
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(*opt).style(style)
        })
        .collect();

    // Create the list widget
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    // Enable scrolling
    let mut list_state = app.list_state.clone();
    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Down => {
            if app.selected_index < 2 {
                app.selected_index += 1;
            }
        }
        KeyCode::Enter | KeyCode::Char('\r') => match app.selected_index {
            0 => app.set_login_form(),
            1 => app.set_register_form(),
            2 => app.set_exit(),
            _ => {}
        },
        _ => {}
    }
}
