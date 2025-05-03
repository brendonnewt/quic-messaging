use std::sync::{Arc, Mutex};
use ratatui::widgets::ListState;
use crate::server::{login, register, ApiResponse};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActiveField {
    Username,
    Password,
    ConfirmPassword,
}

#[derive(Debug)]
pub enum FormState {
    MainMenu,
    LoginForm {
        username: String,
        password: String,
        active_field: ActiveField,
    },
    RegisterForm {
        username: String,
        password: String,
        confirm_password: String,
        active_field: ActiveField,
    },
    UserMenu {
        selected_index: usize,
    },
    Exit,
}

pub struct App {
    pub state: FormState,
    pub selected_index: usize,
    pub message: String,
    pub logged_in: bool,
    pub username: String, // Temporary username for login
    pub list_state: ListState,
    pub password: String,   // Add password field for app state
    pub confirm_password: String, // Add confirm_password field
}

impl App {
    pub fn new() -> Self {
        App {
            state: FormState::MainMenu,
            selected_index: 0,
            message: String::new(),
            logged_in: false,
            username: "".to_string(),
            list_state: ListState::default(),
            password: "".to_string(), // Initialize password
            confirm_password: "".to_string(), // Initialize confirm_password
        }
    }

    // Switch states
    pub fn set_login_form(&mut self) {
        self.state = FormState::LoginForm {
            username: String::new(),
            password: String::new(),
            active_field: ActiveField::Username,
        };
    }

    pub fn set_register_form(&mut self) {
        self.state = FormState::RegisterForm {
            username: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            active_field: ActiveField::Username,
        };
    }

    pub fn set_main_menu(&mut self) {
        self.state = FormState::MainMenu;
    }

    // Add the set_exit method
    pub fn set_exit(&mut self) {
        self.state = FormState::Exit;
    }

    // Access current active field (if applicable)
    pub fn get_active_field(&self) -> Option<ActiveField> {
        match &self.state {
            FormState::LoginForm { active_field, .. } => Some(*active_field),
            FormState::RegisterForm { active_field, .. } => Some(*active_field),
            _ => None,
        }
    }

    // Change active field
    pub fn set_active_field(&mut self, field: ActiveField) {
        match &mut self.state {
            FormState::LoginForm { active_field, .. } => *active_field = field,
            FormState::RegisterForm { active_field, .. } => *active_field = field,
            _ => {}
        }
    }

    pub fn set_user_menu(&mut self) {
        self.state = FormState::UserMenu { selected_index: 0 };
    }

    // Setters for LoginForm and RegisterForm fields
    pub fn set_username(&mut self, username: String) {
        match &mut self.state {
            FormState::LoginForm { username: u, .. } => *u = username,
            FormState::RegisterForm { username: u, .. } => *u = username,
            _ => {}
        }
    }

    pub fn set_password(&mut self, password: String) {
        match &mut self.state {
            FormState::LoginForm { password: p, .. } => *p = password,
            FormState::RegisterForm { password: p, .. } => *p = password,
            _ => {}
        }
    }

    pub fn set_confirm_password(&mut self, confirm_password: String) {
        if let FormState::RegisterForm { confirm_password: c, .. } = &mut self.state {
            *c = confirm_password;
        }
    }

    pub fn set_user_menu_selected_index(&mut self, selected_index: usize) {
        if let FormState::UserMenu { selected_index: s } = &mut self.state {
            *s = selected_index;
        }
    }

    pub fn submit_login(&mut self) {
        let username = self.username.clone();
        let password = self.password.clone();

        match login(&username, &password) {
            Ok(response) => {
                if response.success {
                    self.logged_in = true;
                    self.message = "Login successful".to_string();
                    self.set_user_menu();  // Switch to user menu after login
                } else {
                    self.message = format!("Error: {}", response.message);
                }
            }
            Err(e) => {
                self.message = format!("Error: {}", e);
            }
        }
    }

    pub fn submit_registration(&mut self) {
        let username = self.username.clone();
        let password = self.password.clone();
        let confirm_password = self.confirm_password.clone();

        if password != confirm_password {
            self.message = "Passwords do not match.".to_string();
            return;
        }

        match register(&username, &password, &confirm_password) {
            Ok(response) => {
                if response.success {
                    self.message = "Registration successful".to_string();
                    self.set_login_form();  // Switch to login form after registration
                } else {
                    self.message = format!("Error: {}", response.message);
                }
            }
            Err(e) => {
                self.message = format!("Error: {}", e);
            }
        }
    }
}

impl FormState {
    // Getter for username (for RegisterForm and LoginForm)
    pub fn get_username(&self) -> Option<String> {
        match self {
            FormState::RegisterForm { username, .. } => Some(username.clone()),
            FormState::LoginForm { username, .. } => Some(username.clone()),
            _ => None,
        }
    }

    // Getter for password (for RegisterForm and LoginForm)
    pub fn get_password(&self) -> Option<String> {
        match self {
            FormState::RegisterForm { password, .. } => Some(password.clone()),
            FormState::LoginForm { password, .. } => Some(password.clone()),
            _ => None,
        }
    }

    // Getter for confirm_password (for RegisterForm)
    pub fn get_confirm_password(&self) -> Option<String> {
        match self {
            FormState::RegisterForm { confirm_password, .. } => Some(confirm_password.clone()),
            _ => None,
        }
    }

    // Getter for active_field (for both LoginForm and RegisterForm)
    pub fn get_active_field(&self) -> Option<ActiveField> {
        match self {
            FormState::LoginForm { active_field, .. } => Some(*active_field),
            FormState::RegisterForm { active_field, .. } => Some(*active_field),
            _ => None,
        }
    }
}
