use std::sync::Arc;
use quinn::{Connection, RecvStream, SendStream};
use ratatui::widgets::ListState;
use shared::client_response::{ClientRequest, Command};
use shared::models::chat_models::{Chat, ChatList, ChatMessage, ChatMessages};
use shared::server_response::ServerResponse;

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
    Chats {
        selected_index: usize,
    },
    Chat {
        chat_name: String,
        chat_id: i32,
        page: usize,
        input_buffer: String,
        messages: Vec<ChatMessage>,
    },
    Exit,
}

pub struct App {
    pub state: FormState,
    pub conn: Arc<Connection>,
    pub selected_index: usize,
    pub message: String,
    pub logged_in: bool,
    pub username: String,
    pub jwt: String,
    pub user_id: i32,
    pub list_state: ListState,
    pub chats: Vec<Chat>,
}

impl App {
    pub fn new(conn: Arc<Connection>) -> Self {
        App {
            state: FormState::MainMenu,
            conn,
            selected_index: 0,
            message: String::new(),
            logged_in: false,
            username: "".to_string(),
            jwt: "".to_string(),
            user_id: -1,
            list_state: ListState::default(),
            chats: Vec::new(),
        }
    }

    pub async fn send_request(
        &mut self,
        request: &ClientRequest,
    ) -> Result<ServerResponse, Box<dyn std::error::Error>> {
        let bytes = serde_json::to_vec(request)?;
        let len = (bytes.len() as u32).to_be_bytes();

        let (mut send, mut recv) = self.conn.open_bi().await?;
        
        send.write_all(&len).await?;
        send.write_all(&bytes).await?;
        send.finish().await?;

        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await?;
        let resp_len = u32::from_be_bytes(len_buf) as usize;

        let mut resp_buf = vec![0u8; resp_len];
        recv.read_exact(&mut resp_buf).await?;
        let response = serde_json::from_slice(&resp_buf)?;

        Ok(response)
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

    pub async fn enter_chats_view(&mut self) {
        self.message = "Loading chats...".to_string();
        let request = ClientRequest {
            jwt: Some(self.jwt.clone()),
            command: Command::GetChats,
        };

        match self.send_request(&request).await {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        match serde_json::from_value::<ChatList>(data) {
                            Ok(chats) => {
                                self.chats = chats.chats;
                                self.state = FormState::Chats { selected_index: 0 };
                                self.message = "".into();
                            }
                            Err(e) => {
                                self.message = format!("Parse error: {}", e);
                            }
                        }
                    } else {
                        self.message = "No chat data returned".into();
                    }
                } else {
                    self.message = response.message.unwrap_or("Failed to get chats".into());
                }
            }
            Err(err) => {
                self.message = format!("Error: {}", err);
            }
        }
    }

    pub async fn enter_chat_view(&mut self, chat_id: i32, chat_name: String, page: u64, page_size: u64) {
        self.message = "Loading chat...".to_string();
        let request = ClientRequest {
            jwt: Some(self.jwt.clone()),
            command: Command::GetChatMessages {
                chat_id,
                page,
                page_size,
            },
        };

        match self.send_request(&request).await {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        match serde_json::from_value::<ChatMessages>(data) {
                            Ok(messages) => {
                                self.state = FormState::Chat {
                                    chat_name,
                                    chat_id,
                                    messages: messages.messages,
                                    page: 0,
                                    input_buffer: "".to_string(),
                                };
                                self.message = "".into();
                            }
                            Err(e) => {
                                self.message = format!("Parse error: {}", e);
                            }
                        }
                    } else {
                        self.message = "No chat data returned".into();
                    }
                } else {
                    self.message = response.message.unwrap_or("Failed to get chats".into());
                }
            }
            Err(err) => {
                self.message = format!("Error: {}", err);
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
