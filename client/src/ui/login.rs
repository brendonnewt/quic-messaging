use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::{App, ActiveField, FormState};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    // Debug: show when render runs and current state
    //eprintln!("RENDER called; state={:?}", app.state);

    if let FormState::LoginForm { username, password, active_field } = &app.state {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(4)
            .constraints([
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(3), // Message
                Constraint::Min(0),
            ])
            .split(f.area());

        let user_style = if *active_field == ActiveField::Username {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };
        let pass_style = if *active_field == ActiveField::Password {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default()
        };

        // Username box
        let user_para = Paragraph::new(Text::from(username.clone()))
            .block(Block::default().borders(Borders::ALL).title("Username"))
            .style(user_style);
        f.render_widget(user_para, chunks[0]);

        // Password box (masked)
        let pass_para = Paragraph::new(Text::from("*".repeat(password.len())))
            .block(Block::default().borders(Borders::ALL).title("Password"))
            .style(pass_style);
        f.render_widget(pass_para, chunks[1]);

        // Message box
        let msg_para = Paragraph::new(app.message.clone())
            .block(Block::default().title("")) // no borders for message
            .style(Style::default());
        f.render_widget(msg_para, chunks[2]);
    }
}

pub fn handle_input(app: &mut App, key: KeyEvent) {
    use KeyCode::*;

    if let FormState::LoginForm { username, password, active_field } = &mut app.state {
        match key.code {
            Down | Tab => {
                *active_field = if *active_field == ActiveField::Username {
                    ActiveField::Password
                } else {
                    ActiveField::Username
                };
            }
            Up => {
                *active_field = if *active_field == ActiveField::Password {
                    ActiveField::Username
                } else {
                    ActiveField::Password
                };
            }
            Backspace => {
                if *active_field == ActiveField::Password {
                    password.pop();
                } else {
                    username.pop();
                }
            }
            Char(c) => {
                if *active_field == ActiveField::Password {
                    password.push(c);
                } else {
                    username.push(c);
                }
            }
            Enter => {
                use std::time::Duration;
                use std::thread::sleep;

                if username.is_empty() || password.is_empty() {
                    app.message = "Please fill both fields.".into();
                } else if username == "goober" && password == "password" {
                    app.message = "Login successful!".into();
                    app.username = username.clone(); // Store the temporary
                    app.logged_in = true;

                    app.set_main_menu();
                } else {
                    app.message = "Invalid credentials.".into();
                }
            }
            Esc => {
                app.set_main_menu();
                app.message = "Returning to main menu...".into();
            }
            _ => {}
        }
    }
}
