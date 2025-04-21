use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::text::Text;
use crate::app::{App, ActiveField};

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
        Style::default()
            .bg(ratatui::style::Color::Yellow)
            .fg(ratatui::style::Color::Black)  // Ensuring text contrast
    } else {
        Style::default()
    };
    let password_style = if app.active_field == ActiveField::Password {
        Style::default()
            .bg(ratatui::style::Color::Yellow)
            .fg(ratatui::style::Color::Black)
    } else {
        Style::default()
    };
    let confirm_style = if app.active_field == ActiveField::ConfirmPassword {
        Style::default()
            .bg(ratatui::style::Color::Yellow)
            .fg(ratatui::style::Color::Black)
    } else {
        Style::default()
    };

    // Render the actual text for username, and asterisks for password fields
    let username = Paragraph::new(Text::from(app.username.clone()))
        .block(Block::default().borders(Borders::ALL).title("Username"))
        .style(username_style);

    let password = Paragraph::new(Text::from("*".repeat(app.password.len())))
        .block(Block::default().borders(Borders::ALL).title("Password"))
        .style(password_style);

    let confirm = Paragraph::new(Text::from("*".repeat(app.confirm_password.len())))
        .block(Block::default().borders(Borders::ALL).title("Confirm Password"))
        .style(confirm_style);

    let message = Paragraph::new(app.message.clone())
        .style(Style::default());

    // Render the widgets in the correct chunks
    f.render_widget(username, chunks[0]);
    f.render_widget(password, chunks[1]);
    f.render_widget(confirm, chunks[2]);
    f.render_widget(message, chunks[3]);
}

pub fn handle_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Down => {
            app.active_field = match app.active_field {
                ActiveField::Username => ActiveField::Password,
                ActiveField::Password => ActiveField::ConfirmPassword,
                ActiveField::ConfirmPassword => ActiveField::Username,
            };
        }
        KeyCode::Up => {
            app.active_field = match app.active_field {
                ActiveField::Username => ActiveField::ConfirmPassword,
                ActiveField::Password => ActiveField::Username,
                ActiveField::ConfirmPassword => ActiveField::Password,
            };
        }
        KeyCode::Tab => {
            app.active_field = match app.active_field {
                ActiveField::Username => ActiveField::Password,
                ActiveField::Password => ActiveField::ConfirmPassword,
                ActiveField::ConfirmPassword => ActiveField::Username,
            };
        }
        KeyCode::Backspace => {
            let field = match app.active_field {
                ActiveField::Username => &mut app.username,
                ActiveField::Password => &mut app.password,
                ActiveField::ConfirmPassword => &mut app.confirm_password,
            };
            field.pop(); // Remove last character
        }
        KeyCode::Char(c) => {
            let field = match app.active_field {
                ActiveField::Username => &mut app.username,
                ActiveField::Password => &mut app.password,
                ActiveField::ConfirmPassword => &mut app.confirm_password,
            };
            field.push(c); // Add character to the field
        }
        KeyCode::Enter => {
            // Check if password and confirm password match
            if app.password == app.confirm_password {
                app.message = "Account created successfully!".to_string();
            } else {
                app.message = "Passwords do not match. Try again.".to_string();
            }
        }
        KeyCode::Esc => {
            // Navigate back to the main menu
            app.message = "Returning to main menu...".to_string();
            // Reset any fields if needed
            app.username.clear();
            app.password.clear();
            app.confirm_password.clear();
        }
        _ => {}
    }
}
