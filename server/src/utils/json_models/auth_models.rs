use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthModel {
    pub username: String,
    pub encrypted_password: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponseModel {
    pub success: bool,
    pub token: String,
}
