use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientRequest {
    pub jwt: Option<String>,
    pub command: Command,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Command {
    Login { username: String, password: String },
    Register { username: String, password: String },
    GetInfo {},
    SendFriendRequest { receiver_id: i32 },
    AcceptFriendRequest { sender_id: i32 },
    DeclineFriendRequest { sender_id: i32 },
    CancelFriendRequest { receiver_id: i32 },
    GetFriendRequests {},
    RemoveFriend { friend_id: i32 },
    BlockUser { blocked_id: i32 },
    GetFriends,
    CreateChat { name: Option<String>, is_group: bool, member_ids: Vec<i32> },
    SendMessage { chat_id: i32, content: String },
    GetChatMessages { chat_id: i32 , page: u64, page_size: u64 },
    MarkMessagesRead { chat_id: i32 },
    GetUnreadMessageCount { chat_id: i32 },

}
