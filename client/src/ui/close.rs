use crate::app::{App, FormState};
use crossterm::event::KeyCode::Enter;
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render<B: Backend>(f: &mut Frame, app: &mut App) {
    let options = ["Close Program"];
    let area = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Min(0)])
        .split(f.area())[0];

    let opt_i = if let FormState::ConfirmFriendRequest {
        selected_option, ..
    } = app.state
    {
        selected_option
    } else {
        0
    };

    let title = "Program Disconnected - Close and Restart".to_string();

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, &label)| {
            let style = if i == opt_i {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(label).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, area);
}

pub async fn handle_input(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        Enter => {
            app.set_exit();
            return true;
        }
        _ => {}
    }
    false
}
