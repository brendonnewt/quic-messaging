/* Holds the App state and state logic */

#[derive(Clone, Copy, PartialEq)]
pub enum AppState {
    MainMenu,
    LoginForm,
    RegisterForm,
    Exit,
}
#[derive(PartialEq)]
pub enum ActiveField {
    Username,
    Password,
    ConfirmPassword,
}

pub struct App {
    pub state: AppState,
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub selected_index: usize,
    pub active_field: ActiveField,
    pub message: String,
}

impl App {
    pub fn new() -> Self {
        App {
            state: AppState::MainMenu,
            username: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            selected_index: 0,
            active_field: ActiveField::Username,
            message: String::new(),
        }
    }
}
