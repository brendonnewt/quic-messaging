use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::text::Text;
use crate::app::{App, ActiveField, AppState};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.size());

    // Styling to highlight the active field
    let username_style = if app.active_field == ActiveField::Username {
        Style::default().bg(ratatui::style::Color::Yellow) // Highlight with yellow if active
    } else {
        Style::default()
    };
    let password_style = if app.active_field == ActiveField::Password {
        Style::default().bg(ratatui::style::Color::Yellow) // Highlight with yellow if active
    } else {
        Style::default()
    };
    let confirm_style = if app.active_field == ActiveField::ConfirmPassword {
        Style::default().bg(ratatui::style::Color::Yellow) // Highlight with yellow if active
    } else {
        Style::default()
    };

    // Render the actual text for username, and asterisks for password fields
    let username = Paragraph::new(Text::from(app.username.clone()))
        .block(Block::default().borders(Borders::ALL).title("Username"))
        .style(username_style);

    // For password and confirm, show * to hide the input
    let password = Paragraph::new(Text::from("*".repeat(app.password.len())))
        .block(Block::default().borders(Borders::ALL).title("Password"))
        .style(password_style);

    let confirm = Paragraph::new(Text::from("*".repeat(app.confirm_password.len())))
        .block(Block::default().borders(Borders::ALL).title("Confirm Password"))
        .style(confirm_style);

    // Default message and dynamic feedback message
    let message = if !app.message.is_empty() {
        // If there's a dynamic message, show it
        Paragraph::new(Text::from(app.message.as_str())) // Convert String to &str
            .style(Style::default())
    } else {
        // Otherwise, show the default instruction
        Paragraph::new("Press [Enter] to submit or [Esc] to go back")
            .style(Style::default())
    };

    // Render the widgets
    f.render_widget(username, chunks[0]);
    f.render_widget(password, chunks[1]);
    f.render_widget(confirm, chunks[2]);
    f.render_widget(message, chunks[3]);
}
pub fn handle_input(app: &mut App, key: KeyEvent) {
    match key.code {
        // Move to the next field when pressing the down arrow
        KeyCode::Down => {
            app.active_field = match app.active_field {
                ActiveField::Username => ActiveField::Password,
                ActiveField::Password => ActiveField::ConfirmPassword,
                ActiveField::ConfirmPassword => ActiveField::Username,
            };
        }
        // Move to the previous field when pressing the up arrow
        KeyCode::Up => {
            app.active_field = match app.active_field {
                ActiveField::Username => ActiveField::ConfirmPassword,
                ActiveField::Password => ActiveField::Username,
                ActiveField::ConfirmPassword => ActiveField::Password,
            };
        }
        // Tab to cycle through fields
        KeyCode::Tab => {
            app.active_field = match app.active_field {
                ActiveField::Username => ActiveField::Password,
                ActiveField::Password => ActiveField::ConfirmPassword,
                ActiveField::ConfirmPassword => ActiveField::Username,
            };
        }
        KeyCode::Backspace => {
            // Handle backspace to delete characters from the current field
            let field = match app.active_field {
                ActiveField::Username => &mut app.username,
                ActiveField::Password => &mut app.password,
                ActiveField::ConfirmPassword => &mut app.confirm_password,
            };
            field.pop(); // Remove last character
        }
        KeyCode::Char(c) => {
            // Insert character into the active field
            let field = match app.active_field {
                ActiveField::Username => &mut app.username,
                ActiveField::Password => &mut app.password,
                ActiveField::ConfirmPassword => &mut app.confirm_password,
            };
            field.push(c); // Add character to the field
        }
        KeyCode::Enter => {
            // Check if any field is empty
            if app.username.is_empty() || app.password.is_empty() || app.confirm_password.is_empty() {
                app.message = String::from("All fields are required! Please fill in every field.");
            } else if app.password != app.confirm_password {
                // Check if password and confirm password match
                app.message = String::from("Passwords do not match! Please try again.");
            } else {
                // Success, account created
                app.message = String::from("Account successfully created!");
                // Optionally, reset fields or change state as needed
                app.username.clear();
                app.password.clear();
                app.confirm_password.clear();
            }
        }
        KeyCode::Esc => {
            // Go back to the main menu
            app.state = AppState::MainMenu;
            app.message.clear();
        }
        _ => {}
    }
}
