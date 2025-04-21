use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::text::Text; // Add the import for Text
use crate::app::{AppState, App};

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

    let username = Paragraph::new(Text::from(app.username.clone()))  // Convert String to Text
        .block(Block::default().borders(Borders::ALL).title("Username"));
    let password = Paragraph::new(Text::from("*".repeat(app.password.len())))  // Hide password with asterisks
        .block(Block::default().borders(Borders::ALL).title("Password"));
    let confirm = Paragraph::new(Text::from("*".repeat(app.confirm_password.len())))  // Hide confirm password with asterisks
        .block(Block::default().borders(Borders::ALL).title("Confirm Password"));
    let message = Paragraph::new("Press [Enter] to submit or [Esc] to go back")
        .style(Style::default());

    f.render_widget(username, chunks[0]);
    f.render_widget(password, chunks[1]);
    f.render_widget(confirm, chunks[2]);
    f.render_widget(message, chunks[3]);
}
pub fn handle_input(key: crossterm::event::KeyEvent, app: &mut App) {
    match key.code {
        crossterm::event::KeyCode::Esc => {
            app.state = AppState::MainMenu; // Escape returns to the MainMenu
        }
        crossterm::event::KeyCode::Enter => {
            // Handle the submission of the form
            if app.password == app.confirm_password {
                println!("Account registered with username: {}", app.username);
                app.state = AppState::MainMenu; // After registration, return to the main menu
            } else {
                // Passwords don't match
                println!("Passwords do not match. Please try again.");
            }
        }
        crossterm::event::KeyCode::Backspace => {
            // Handle Backspace for both fields (Username and Password)
            if app.username.len() > 0 {
                app.username.pop();
            }
        }
        crossterm::event::KeyCode::Char(c) => {
            // Handle typing for username and password
            if app.username.len() < 20 { // Limit username length (optional)
                app.username.push(c);
            }
        }
        _ => {}
    }
}
