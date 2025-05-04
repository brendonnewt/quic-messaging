use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use crate::app::{App, FormState};

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let options = ["Chats", "Chatroom", "Add Friends", "Friends", "Settings", "Log Out"];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints(vec![Constraint::Length(3); 6].into_iter().chain([Constraint::Min(0)]).collect::<Vec<_>>())
        .split(f.size());

    let selected = if let FormState::UserMenu { selected_index } = app.state {
        selected_index
    } else { 0 };

    let items: Vec<ListItem> = options.iter().enumerate().map(|(i, &opt)| {
        let style = if i == selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(opt).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Logged In"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    let area = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([Constraint::Min(0)])
        .split(f.size())[0];

    f.render_widget(list, area);
}
