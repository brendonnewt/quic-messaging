use crate::{
    app::{App, FormState},
    event, ui,
};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

pub async fn run_app(app: &mut App, rx: spmc::Receiver<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    loop {
        match rx.try_recv() {
            Ok(_) => {
                app.refresh().await;
            }
            Err(spmc::TryRecvError::Empty) => {
                // Nothing to report, continue
            }
            Err(spmc::TryRecvError::Disconnected) => {
                // Something has gone wrong, close the program
            }
        }
        // 1) Draw the appropriate UI for the current state
        terminal.draw(|f| match &app.state {
            FormState::MainMenu => ui::main_menu::render::<CrosstermBackend<Stdout>>(f, app),
            FormState::LoginForm { .. } => ui::login::render::<CrosstermBackend<Stdout>>(f, app),
            FormState::RegisterForm { .. } => {
                ui::registration::render::<CrosstermBackend<Stdout>>(f, app)
            }
            FormState::UserMenu { .. } => ui::user_menu::render::<CrosstermBackend<Stdout>>(f, app),
            FormState::AddFriend { .. } => {
                ui::add_friends::render::<CrosstermBackend<Stdout>>(f, app)
            }
            FormState::FriendMenu { .. } => {
                ui::friends_menu::render::<CrosstermBackend<Stdout>>(f, app)
            }
            FormState::FriendRequests { .. } => {
                ui::friend_requests::render::<CrosstermBackend<Stdout>>(f, app)
            }
            FormState::ConfirmFriendRequest { .. } => {
                ui::confirm_friend_request::render::<CrosstermBackend<Stdout>>(f, app)
            }
            FormState::FriendList { .. } => {
                ui::friend_list::render::<CrosstermBackend<Stdout>>(f, app)
            }
            FormState::ConfirmUnfriend { .. } => {
                ui::confirm_unfriend::render::<CrosstermBackend<Stdout>>(f, app)
            }
            FormState::Chats { .. } => {
                ui::chats::render::<CrosstermBackend<Stdout>>(f, app)
            }
            FormState::Chat { .. } => {
                ui::chat::render::<CrosstermBackend<Stdout>>(f, app)
            }
            FormState::ChatCreation(phase) => {
                ui::create_chat::render::<CrosstermBackend<Stdout>>(f, app)
            }
            FormState::Exit => return,
        })?;

        if matches!(app.state, FormState::Exit) {
            break;
        }

        // 2) Poll for a key event
        if let Some(key) = event::poll_event()? {
            match &mut app.state {
                // Registration form input
                FormState::RegisterForm { .. } => {
                    ui::registration::handle_input(app, key).await;
                }

                // Login form input
                FormState::LoginForm { .. } => {
                    ui::login::handle_input(app, key).await;
                }

                FormState::AddFriend { .. } => {
                    ui::add_friends::handle_input(app, key).await;
                }

                FormState::FriendMenu { .. } => {
                    ui::friends_menu::handle_input(app, key).await;
                }

                FormState::FriendRequests { .. } => {
                    ui::friend_requests::handle_input(app, key).await;
                }
                FormState::ConfirmFriendRequest { .. } => {
                    ui::confirm_friend_request::handle_input(app, key).await;
                }

                FormState::FriendList { .. } => {
                    ui::friend_list::handle_input(app, key).await;
                }

                FormState::ConfirmUnfriend { .. } => {
                    ui::confirm_unfriend::handle_input(app, key).await;
                }

                // Main menu navigation
                FormState::MainMenu => {
                    ui::main_menu::handle_input(app, key).await;
                }
                
                FormState::Chats { .. } => {
                    ui::chats::handle_input(app, key).await;
                }
                
                FormState::Chat { .. } => {
                    ui::chat::handle_input(app, key).await;
                },
                
                FormState::ChatCreation(..) => {
                    ui::create_chat::handle_input(app, key).await;
                }

                // User menu navigation (post-login)
                FormState::UserMenu { .. } => {
                    ui::user_menu::handle_input(app, key).await;
                }

                // Any other state: do nothing
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
