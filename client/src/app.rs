/* Holds the App state and state logic */

#[derive(Clone, Copy, PartialEq)]
pub enum AppState {
    MainMenu,
    LoginForm,
    RegisterForm,
    Exit,
}

pub struct App {
    pub state: AppState,
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub selected_index: usize,
}

impl App {
    pub fn new() -> Self {
        App {
            state: AppState::MainMenu,
            username: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            selected_index: 0,
        }
    }
}
