/* Holds the App state and state logic */

pub enum AppState {
    MainMenu,
    LoginForm,
    RegisterForm,
    Exit,
}

pub struct App {
    pub state: AppState,
    pub selected_index: usize,
    // Add input buffers later for username/password
}

impl App {
    pub fn new() -> Self {
        App {
            state: AppState::MainMenu,
            selected_index: 0,
        }
    }
}
