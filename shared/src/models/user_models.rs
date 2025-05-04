use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.username == other.username
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserList {
    pub users: Vec<User>,
}

#[derive(Serialize)]
pub struct FriendRequestList {
    pub incoming: Vec<User>,
    pub outgoing: Vec<User>,
}