#[derive(PartialEq, Clone, Copy)]
pub enum ActiveField {
    Username,
    Password,
    ConfirmPassword,
}

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
    Exit,
}

pub struct App {
    pub state: FormState,
    pub selected_index: usize,
    pub message: String,
}

impl App {
    pub fn new() -> Self {
        App {
            state: FormState::MainMenu,
            selected_index: 0,
            message: String::new(),
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
