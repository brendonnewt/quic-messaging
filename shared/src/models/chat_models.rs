use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatList {
    pub chats: Vec<Chat>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chat {
    pub id: i32,
    pub chat_name: String,
    pub unread_count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessages {
    pub id: i32,
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub user_id: i32,
    pub username: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Count {
    pub count: u64,
}
