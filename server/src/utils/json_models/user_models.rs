use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponseModel {
    pub id: i32,
    pub username: String,
}