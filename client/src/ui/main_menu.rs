use ratatui::{
    backend::Backend,
    layout::{Layout, Constraint, Direction},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;

pub fn render<B: Backend>(f: &mut Frame, app: &App) {
    let options = vec!["Login", "Register", "Close"];
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let style = if i == app.selected_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(*opt).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, f.size());
}