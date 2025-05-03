use ratatui::{
    backend::Backend,
    layout::{Layout, Constraint, Direction},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::App;
use ratatui::widgets::ListState;
use crossterm::event::{self, KeyCode, KeyEvent};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let size = f.size(); // Get the area of the frame
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]
                .as_ref(),
        )
        .split(size);

    // Profile header with "Profile Settings"
    let profile_header = Paragraph::new("Profile Settings")
        .style(Style::default().fg(Color::LightGreen))
        .block(Block::default().borders(Borders::ALL).title("Profile"));
    f.render_widget(profile_header, chunks[0]);

    // List of editable fields (Full Name, Username, Email, Password, Date of Birth, Date Added)
    let fields = vec![
        if let Some(profile) = &app.profile {
            ListItem::new(format!("Full Name: {} ✏️", profile.full_name))
        } else {
            ListItem::new("Full Name: (Not Available) ✏️")
        },
        if let Some(profile) = &app.profile {
            ListItem::new(format!("Username: {} ✏️", profile.username))
        } else {
            ListItem::new("Username: (Not Available) ✏️")
        },
        if let Some(profile) = &app.profile {
            ListItem::new(format!("Email: {} ✏️", profile.email))
        } else {
            ListItem::new("Email: (Not Available) ✏️")
        },
        if let Some(profile) = &app.profile {
            ListItem::new(format!("Password: {} ✏️", profile.password))
        } else {
            ListItem::new("Password: (Not Available) ✏️")
        },
        if let Some(profile) = &app.profile {
            ListItem::new(format!("Date of Birth: {} ✏️", profile.dob))
        } else {
            ListItem::new("Date of Birth: (Not Available) ✏️")
        },
        if let Some(profile) = &app.profile {
            ListItem::new(format!("Date Added: {}", profile.date_added))
        } else {
            ListItem::new("Date Added: (Not Available)")
        },
    ];

    let profile_list = List::new(fields)
        .block(Block::default().borders(Borders::ALL).title("Editable Fields"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
        .highlight_symbol("-> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));  // Highlight the current field
    f.render_stateful_widget(profile_list, chunks[1], &mut list_state);

    // Footer section (instructions or quit option)
    let footer = Paragraph::new("Press 'Enter' to edit, 'q' to quit.")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Footer"));
    f.render_widget(footer, chunks[2]);
}

pub fn handle_input(app: &mut App, key: KeyEvent) {
    // Handle key events for navigation
    match key.code {
        KeyCode::Up => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Down => {
            if app.selected_index < 5 { // 6 fields (0-5)
                app.selected_index += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(profile) = &mut app.profile {
                match app.selected_index {
                    0 => {
                        // Handle editing "Full Name"
                        app.message = "Editing Full Name...".into();
                        // Implement actual input handling here
                    }
                    1 => {
                        // Handle editing "Username"
                        app.message = "Editing Username...".into();
                        // Implement actual input handling here
                    }
                    2 => {
                        // Handle editing "Email"
                        app.message = "Editing Email...".into();
                        // Implement actual input handling here
                    }
                    3 => {
                        // Handle editing "Password"
                        app.message = "Editing Password...".into();
                        // Implement actual input handling here
                    }
                    4 => {
                        // Handle editing "Date of Birth"
                        app.message = "Editing Date of Birth...".into();
                        // Implement actual input handling here
                    }
                    5 => {
                        // Date Added is non-editable, do nothing if selected
                        app.message = "Date Added cannot be edited.".into();
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Esc => {
            app.message = "Exiting profile settings...".into();
            // Handle exit logic (e.g., go back to the previous menu)
            // You may want to implement a specific action for Esc (e.g., navigating back).
        }
        _ => {}
    }
}
