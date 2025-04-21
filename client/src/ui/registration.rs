use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render<B: Backend>(f: &mut Frame, _app: &App) {
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

    let username = Paragraph::new("Username: ________")
        .block(Block::default().borders(Borders::ALL).title("Username"));
    let password = Paragraph::new("Password: ________")
        .block(Block::default().borders(Borders::ALL).title("Password"));
    let confirm = Paragraph::new("Re-enter Password: ________")
        .block(Block::default().borders(Borders::ALL).title("Confirm Password"));
    let message = Paragraph::new("Press [Enter] to submit or [Esc] to go back")
        .style(Style::default());

    f.render_widget(username, chunks[0]);
    f.render_widget(password, chunks[1]);
    f.render_widget(confirm, chunks[2]);
    f.render_widget(message, chunks[3]);
}
