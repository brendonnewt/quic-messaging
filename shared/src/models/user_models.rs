use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserList {
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendRequestList {
    pub incoming: Vec<User>,
    pub outgoing: Vec<User>,
}