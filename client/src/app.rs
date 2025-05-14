use crate::ui::create_chat::ChatCreationPhase;
use quinn::Connection;
use ratatui::widgets::ListState;
use shared::client_response::Command::{CreateChat, GetFriends};
use shared::client_response::{ClientRequest, Command};
use shared::models::chat_models::{Chat, ChatList, ChatMessage, ChatMessages, Count};
use shared::models::user_models::FriendRequestList;
use shared::models::user_models::{User, UserList};
use shared::server_response::ServerResponse;
use std::sync::Arc;
use tracing::error;

const PAGE_SIZE: u64 = 10;

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
    AddFriend {
        id: String,
        active_field: ActiveField,
    },
    FriendMenu {
        selected_index: usize,
    },
    FriendRequests {
        selected_index: usize,
    },
    ConfirmFriendRequest {
        selected_index: usize,
        selected_option: usize, // Usize where 0 = accept and 1 = decline
    },
    FriendList {
        selected_index: usize,
    },
    ConfirmUnfriend {
        selected_index: usize,
        selected_option: usize,
    },
    Chats {
        selected_index: usize,
        page: u64,
        page_count: u64,
    },
    Chat {
        chat_name: String,
        chat_id: i32,
        page: u64,
        page_count: u64,
        page_size: u64,
        input_buffer: String,
        messages: Vec<ChatMessage>,
    },
    ProfileView {
        new_password: String,
        active_field: ActiveField,
    },
    ChatCreation(ChatCreationPhase),
    Close,
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
    pub unread_count: u64,
    pub list_state: ListState,
    pub friend_requests: FriendRequestList,
    pub friend_request_num: usize,
    pub friend_list: UserList,
    pub friend_list_num: usize,
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
            unread_count: 0,
            list_state: ListState::default(),
            friend_requests: FriendRequestList {
                incoming: vec![],
                outgoing: vec![],
            },
            friend_request_num: 0,
            friend_list: UserList { users: vec![] },
            friend_list_num: 0,
            chats: Vec::new(),
        }
    }

    pub async fn refresh(&mut self) {
        match &self.state {
            FormState::Chat {
                chat_id,
                chat_name,
                page,
                input_buffer,
                ..
            } => {
                let input_buffer = Some(input_buffer.clone());
                self.enter_chat_view(*chat_id, chat_name.clone(), *page, PAGE_SIZE, input_buffer)
                    .await;
            }
            FormState::Chats { page, .. } => {
                self.enter_chats_view(*page, PAGE_SIZE).await;
            }
            FormState::UserMenu { .. } => {
                self.set_user_menu().await;
            }
            FormState::FriendList { .. } => {
                self.set_friend_list().await;
            }
            FormState::FriendRequests { .. } => {
                self.set_friend_requests().await;
            }
            _ => { // These states don't need refreshing as they have no values that could be stale
            }
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

    pub async fn create_chat(&mut self, users: Vec<User>, name: Option<String>) {
        let member_ids: Vec<i32> = users.iter().map(|u| u.id).collect();
        let is_group = member_ids.len() > 2;
        let request = ClientRequest {
            jwt: Some(self.jwt.clone()),
            command: CreateChat {
                name,
                is_group,
                member_ids,
            },
        };
        match self.send_request(&request).await {
            Ok(response) => {
                if !response.success {
                    if let Some(message) = response.message {
                        self.message = message;
                    } else {
                        self.message = "Chat couldn't be created!".to_string();
                    }
                } else {
                    self.message = "Chat created successfully!".to_string();
                }
            }
            Err(err) => {
                self.message = format!("Error: {}", err);
            }
        }
    }

    pub async fn get_friends(&mut self) -> Vec<User> {
        let request = ClientRequest {
            jwt: Some(self.jwt.clone()),
            command: GetFriends,
        };
        match self.send_request(&request).await {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        match serde_json::from_value::<UserList>(data) {
                            Ok(friends) => {
                                return friends.users;
                            }
                            Err(e) => {
                                self.message = format!("Parse error: {}", e);
                            }
                        }
                    } else {
                        self.message = "No friends data returned".into();
                    }
                } else {
                    self.message = response.message.unwrap_or("Failed to get friends".into());
                }
            }
            Err(err) => {
                self.message = format!("Error: {}", err);
            }
        }
        Vec::new()
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

    pub fn set_add_friend(&mut self) {
        self.state = FormState::AddFriend {
            id: String::new(),
            active_field: ActiveField::Username,
        };
    }

    pub fn set_friend_menu(&mut self) {
        self.state = FormState::FriendMenu { selected_index: 0 }
    }

    pub async fn set_friend_requests(&mut self) {
        let req = ClientRequest {
            jwt: Option::from(self.jwt.clone()),
            command: Command::GetFriendRequests {},
        };
        match self.send_request(&req).await {
            Ok(resp) => {
                if resp.success {
                    if let Some(data) = resp.data {
                        match serde_json::from_value::<FriendRequestList>(data) {
                            Ok(requests) => {
                                self.friend_request_num = requests.incoming.len();
                                self.friend_requests = requests;
                                self.state = FormState::FriendRequests { selected_index: 0 };
                            }
                            Err(err) => {
                                self.message = format!("Parse error: {}", err);
                            }
                        }
                    }
                } else if let Some(message) = resp.message.clone() {
                    self.message = message;
                }
            }
            Err(e) => {
                self.message = e.to_string();
            }
        }
    }

    pub fn set_confirm_friend_request(&mut self, req_index: usize) {
        self.state = FormState::ConfirmFriendRequest {
            selected_index: req_index,
            selected_option: 0,
        };
    }

    pub fn set_confirm_unfriend(&mut self, req_index: usize) {
        self.state = FormState::ConfirmUnfriend {
            selected_index: req_index,
            selected_option: 0,
        }
    }

    pub async fn set_friend_list(&mut self) {
        let req = ClientRequest {
            jwt: Option::from(self.jwt.clone()),
            command: Command::GetFriends {},
        };
        match self.send_request(&req).await {
            Ok(resp) => {
                if resp.success {
                    if let Some(data) = resp.data {
                        match serde_json::from_value::<UserList>(data) {
                            Ok(friends) => {
                                self.friend_list_num = friends.users.len();
                                self.friend_list = friends;
                                self.state = FormState::FriendList { selected_index: 0 }
                            }
                            Err(e) => {
                                self.message = format!("Parse error: {}", e);
                            }
                        }
                    }
                } else if let Some(message) = resp.message.clone() {
                    self.message = message;
                }
            }
            Err(e) => {
                self.message = e.to_string();
            }
        }
    }

    // Add the set_exit method
    pub fn set_exit(&mut self) {
        self.state = FormState::Exit;
    }

    // Access the current active field (if applicable)
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

    pub async fn set_user_menu(&mut self) {
        let request = ClientRequest {
            jwt: Some(self.jwt.clone()),
            command: Command::GetUnreadMessageCount,
        };

        match self.send_request(&request).await {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        match serde_json::from_value::<Count>(data) {
                            Ok(count) => {
                                self.unread_count = count.count;
                                self.state = FormState::UserMenu { selected_index: 0 };
                            }
                            Err(e) => {
                                self.message = format!("Parse error: {}", e);
                            }
                        }
                    } else {
                        self.message = "No unread message count returned".into();
                    }
                } else {
                    self.message = response
                        .message
                        .unwrap_or("Failed to get unread count".into());
                }
            }
            Err(err) => {
                self.message = format!("Error: {}", err);
            }
        }
        self.state = FormState::UserMenu { selected_index: 0 };
    }

    pub fn set_friend_request_num(&mut self, friend_request_num: usize) {
        self.friend_request_num = friend_request_num;
    }

    pub fn set_friend_list_num(&mut self, friend_list_num: usize) {
        self.friend_list_num = friend_list_num;
    }

    pub fn set_user_menu_selected_index(&mut self, selected_index: usize) {
        if let FormState::UserMenu { selected_index: s } = &mut self.state {
            *s = selected_index;
        }
    }

    pub fn set_profile_view(&mut self) {
        self.state = FormState::ProfileView {
            new_password: String::new(),

            active_field: ActiveField::Password,
        }
    }

    pub async fn logout(&mut self) -> () {
        let req = ClientRequest {
            jwt: Option::from(self.jwt.clone()),
            command: Command::Logout {
                username: self.username.clone(),
            },
        };
        match self.send_request(&req).await {
            Ok(response) => {
                if response.success {
                    self.set_main_menu();
                }
            }
            Err(e) => {
                error!("Error sending logout request: {:?}", e);
            }
        }
        self.jwt = "".to_string();
        self.username = "".to_string();
        self.user_id = -1;
    }

    pub async fn enter_chats_view(&mut self, page: u64, page_size: u64) {
        let page_count = self.get_chats_page_count(page_size).await;

        if let Some(page_count) = page_count {
            self.get_chat_list(page, page_size, page_count).await;
        }
    }

    pub async fn enter_chat_view(
        &mut self,
        chat_id: i32,
        chat_name: String,
        mut page: u64,
        page_size: u64,
        input_buffer: Option<String>,
    ) {
        // Get the number of pages in the chat
        let page_count = self.get_chat_page_count(chat_id, page_size).await;

        if let Some(page_count) = page_count {
            if page > page_count {
               page = page_count; 
            }
            self.get_chat_messages(
                chat_id,
                chat_name,
                page_count,
                page,
                page_size,
                input_buffer,
            )
            .await;
        }
    }

    pub async fn get_chat_messages(
        &mut self,
        chat_id: i32,
        chat_name: String,
        page_count: u64,
        page: u64,
        page_size: u64,
        input_buffer: Option<String>,
    ) {
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
                                    page_count,
                                    page_size,
                                    messages: messages.messages,
                                    page,
                                    input_buffer: input_buffer.unwrap_or("".to_string()),
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

        // Mark the messages in the chat as read if they were retrieved
        let request = ClientRequest {
            jwt: Some(self.jwt.clone()),
            command: Command::MarkMessagesRead { chat_id },
        };

        match self.send_request(&request).await {
            Ok(response) => {
                if !response.success {
                    if let Some(message) = response.message {
                        self.message = message;
                    } else {
                        self.message = "Couldn't mark messages as read!".into();
                    }
                }
            }
            Err(err) => {
                self.message = format!("Error: {}", err);
            }
        }
    }

    pub async fn get_chat_list(&mut self, page: u64, page_size: u64, page_count: u64) {
        let request = ClientRequest {
            jwt: Some(self.jwt.clone()),
            command: Command::GetChats { page, page_size },
        };
        match self.send_request(&request).await {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        match serde_json::from_value::<ChatList>(data) {
                            Ok(chats) => {
                                self.chats = chats.chats;
                                self.state = FormState::Chats {
                                    page_count,
                                    page,
                                    selected_index: self.selected_index,
                                };
                                self.message = "".into();
                            }
                            Err(e) => {
                                self.message = format!("Parse error: {}", e);
                            }
                        }
                    } else {
                        self.message = "No chats data returned".into();
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

    pub async fn get_chat_page_count(&mut self, chat_id: i32, page_size: u64) -> Option<u64> {
        let request = ClientRequest {
            jwt: Some(self.jwt.clone()),
            command: Command::GetChatPages { chat_id, page_size },
        };
        match self.send_request(&request).await {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        match serde_json::from_value::<Count>(data) {
                            Ok(count) => return Some(count.count),
                            Err(e) => {
                                self.message = format!("Parse error: {}", e);
                            }
                        }
                    } else {
                        self.message = "No page count returned".into();
                    }
                } else {
                    self.message = response
                        .message
                        .unwrap_or("Failed to get chat pages".into());
                }
            }
            Err(err) => {
                self.message = format!("Error: {}", err);
            }
        }
        None
    }

    pub async fn get_chats_page_count(&mut self, page_size: u64) -> Option<u64> {
        let request = ClientRequest {
            jwt: Some(self.jwt.clone()),
            command: Command::GetChatsPages { page_size },
        };
        match self.send_request(&request).await {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        match serde_json::from_value::<Count>(data) {
                            Ok(count) => return Some(count.count),
                            Err(e) => {
                                self.message = format!("Parse error: {}", e);
                            }
                        }
                    } else {
                        self.message = "No page count returned".into();
                    }
                } else {
                    self.message = response
                        .message
                        .unwrap_or("Failed to get chats pages".into());
                }
            }
            Err(err) => {
                self.message = format!("Error: {}", err);
            }
        }
        None
    }
}
