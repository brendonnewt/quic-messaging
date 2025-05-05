use ratatui::{
    backend::Backend,
    layout::{Layout, Constraint, Direction},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, FormState};
use ratatui::widgets::ListState;
use crossterm::event::{self, KeyCode, KeyEvent};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    if app.is_quitting {
        return;  // Early return, effectively quitting the app
    }

    let size = f.area(); // Use .area() instead of .size()
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
            match app.selected_index {
                0..=4 => {
                    app.state = FormState::EditingField { field_index: app.selected_index };
                }
                5 => {
                    app.message = "Date Added cannot be edited.".into();
                }
                _ => {}
            }
        }
        KeyCode::Esc => {
            app.message = "Exiting profile settings...".into();
            app.state = FormState::UserMenu {
                selected_index: 0, // Pass a value for selected_index
            }; // Switch to the user menu instead of quitting
        }
        KeyCode::Char('q') => {
            app.state = FormState::UserMenu {
                selected_index: 0, // Pass a value for selected_index
            };
            app.message = "Returning to User Menu...".into();
        }
        _ => {}
    }
}
