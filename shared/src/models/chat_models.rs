use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatList {
    pub chats: Vec<Chat>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chat {
    pub id: i32,
    pub chat_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub user_id: i32,
    pub content: String,
}
