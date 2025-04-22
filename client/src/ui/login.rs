use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::text::Text;
use crate::app::{App, ActiveField, FormState};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.size());

    // Retrieve the active field using the get_active_field method
    let active_field = app.get_active_field();

    // Styling to highlight the active field
    let username_style = if active_field == Some(ActiveField::Username) {
        Style::default()
            .bg(ratatui::style::Color::Yellow)
            .fg(ratatui::style::Color::Black)  // Ensuring text contrast
    } else {
        Style::default()
    };
    let password_style = if active_field == Some(ActiveField::Password) {
        Style::default()
            .bg(ratatui::style::Color::Yellow)
            .fg(ratatui::style::Color::Black)
    } else {
        Style::default()
    };

    // Render the actual text for username, and asterisks for password fields
    let username = Paragraph::new(Text::from(match &app.state {
        FormState::RegisterForm { username, .. } => username.clone(),
        _ => String::new(),
    }))
        .block(Block::default().borders(Borders::ALL).title("Username"))
        .style(username_style);

    let password = Paragraph::new(Text::from("*".repeat(match &app.state {
        FormState::RegisterForm { password, .. } => password.len(),
        _ => 0,
    })))
        .block(Block::default().borders(Borders::ALL).title("Password"))
        .style(password_style);


    let message = Paragraph::new(app.message.clone())
        .style(Style::default());

    // Render the widgets in the correct chunks
    f.render_widget(username, chunks[0]);
    f.render_widget(password, chunks[1]);
    f.render_widget(message, chunks[2]);
}
pub fn handle_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Down => {
            if let Some(active_field) = app.get_active_field() {
                let next_field = match active_field {
                    ActiveField::Username => ActiveField::Password,
                    ActiveField::Password => ActiveField::ConfirmPassword, // Moving to ConfirmPassword
                    ActiveField::ConfirmPassword => ActiveField::Username, // Looping back to Username
                };
                app.set_active_field(next_field);
            }
        }
        KeyCode::Up => {
            if let Some(active_field) = app.get_active_field() {
                let prev_field = match active_field {
                    ActiveField::Username => ActiveField::ConfirmPassword, // Moving to ConfirmPassword
                    ActiveField::Password => ActiveField::Username,
                    ActiveField::ConfirmPassword => ActiveField::Password, // Looping back to Password
                };
                app.set_active_field(prev_field);
            }
        }
        KeyCode::Tab => {
            if let Some(active_field) = app.get_active_field() {
                let next_field = match active_field {
                    ActiveField::Username => ActiveField::Password,
                    ActiveField::Password => ActiveField::ConfirmPassword, // Moving to ConfirmPassword
                    ActiveField::ConfirmPassword => ActiveField::Username, // Looping back to Username
                };
                app.set_active_field(next_field);
            }
        }
        KeyCode::Backspace => {
            match &mut app.state {
                FormState::LoginForm { username, password, active_field } => {
                    let field = match active_field {
                        ActiveField::Username => username,
                        ActiveField::Password => password,
                        ActiveField::ConfirmPassword => todo!(), // Handle ConfirmPassword here if needed
                    };
                    field.pop(); // Remove last character
                },
                FormState::RegisterForm { username, password, confirm_password, active_field } => {
                    let field = match active_field {
                        ActiveField::Username => username,
                        ActiveField::Password => password,
                        ActiveField::ConfirmPassword => confirm_password,
                    };
                    field.pop(); // Remove last character
                },
                _ => {},
            }
        }
        KeyCode::Char(c) => {
            match &mut app.state {
                FormState::LoginForm { username, password, active_field } => {
                    let field = match active_field {
                        ActiveField::Username => username,
                        ActiveField::Password => password,
                        ActiveField::ConfirmPassword => todo!(), // Handle ConfirmPassword here if needed
                    };
                    field.push(c); // Add character to the field
                },
                FormState::RegisterForm { username, password, confirm_password, active_field } => {
                    let field = match active_field {
                        ActiveField::Username => username,
                        ActiveField::Password => password,
                        ActiveField::ConfirmPassword => confirm_password,
                    };
                    field.push(c); // Add character to the field
                },
                _ => {},
            }
        }
        KeyCode::Enter => {
            match &app.state {
                FormState::LoginForm { username, password, .. } => {
                    // Validate username and password
                    if username == "user" && password == "password" { // Example validation
                        app.message = "Login successful!".to_string();
                    } else {
                        app.message = "Invalid credentials. Try again.".to_string();
                    }
                },
                FormState::RegisterForm { password, confirm_password, .. } => {
                    if password == confirm_password {
                        app.message = "Account created successfully!".to_string();
                    } else {
                        app.message = "Passwords do not match. Try again.".to_string();
                    }
                },
                _ => {},
            }
        }
        KeyCode::Esc => {
            app.set_main_menu();  // Navigate back to the main menu
            app.message = "Returning to main menu...".to_string();
        }
        _ => {}
    }
}
