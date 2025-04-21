use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration};

struct App<'a> {
    items: Vec<&'a str>,
    state: ListState,
}

impl<'a> App<'a> {
    fn new(items: Vec<&'a str>) -> App<'a> {
        let mut state = ListState::default();
        state.select(Some(0));
        App { items, state }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub(crate) fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new(vec!["Login", "Register", "Close"]);
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let items: Vec<ListItem> = app
                .items
                .iter()
                .map(|i| ListItem::new(*i))
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Menu"))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, size, &mut app.state);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => app.previous(),
                    KeyCode::Down => app.next(),
                    KeyCode::Enter => {
                        if let Some(selected) = app.state.selected() {
                            match app.items[selected] {
                                "Login" => {
                                    // Handle login
                                }
                                "Register" => {
                                    // Handle register
                                }
                                "Close" => return Ok(()),
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Esc => return Ok(()),
                    _ => {}
                }
            }
        }
    }
}
