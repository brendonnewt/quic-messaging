use ratatui::{
    backend::Backend,
    layout::{Layout, Constraint, Direction},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::{App, FormState};
use crossterm::event::{KeyCode, KeyEvent};
use futures::TryFutureExt;

pub fn render<B: Backend>(f: &mut Frame, app: &mut App) {
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

    // Ensure edit_field and edit_value are initialized properly
    let fallback_field = String::from("(No field selected)");
    let field = app.edit_field.as_ref().unwrap_or(&fallback_field);
    let value: &str = if app.edit_value.is_empty() {
        "(No value set)"
    } else {
        &app.edit_value
    };




    let edit_prompt = format!("Editing: {} - Current Value: {}", field, value);
    let input_widget = Paragraph::new(edit_prompt)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Edit Field"));

    f.render_widget(input_widget, chunks[1]);

    // Footer with instructions
    let footer = Paragraph::new("Press 'Enter' to save, 'Esc' to cancel.")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Footer"));
    f.render_widget(footer, chunks[2]);
}


pub fn handle_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            // Save the new value if it exists
            if let new_value = app.edit_value.clone() {
                if let Some(field) = &app.edit_field {
                    match field.as_str() {
                        "Full Name" => {
                            if let Some(profile) = &mut app.profile {
                                profile.full_name = new_value;
                            }
                        }
                        "Username" => {
                            if let Some(profile) = &mut app.profile {
                                profile.username = new_value;
                            }
                        }
                        "Email" => {
                            if let Some(profile) = &mut app.profile {
                                profile.email = new_value;
                            }
                        }
                        "Password" => {
                            if let Some(profile) = &mut app.profile {
                                profile.password = new_value;
                            }
                        }
                        "Date of Birth" => {
                            if let Some(profile) = &mut app.profile {
                                profile.dob = new_value;
                            }
                        }
                        _ => {}
                    }
                }
                app.set_profile_view();
            }
        }
        KeyCode::Esc => {
            // Cancel editing and return to profile view
           app.set_profile_view();

        }
        _ => {}
    }
}
