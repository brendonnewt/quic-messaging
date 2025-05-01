use std::sync::Arc;
use sea_orm::DatabaseConnection;
use shared::models::chat_models::{ChatList, ChatMessages};
use crate::handlers::services::{chat_service};
use crate::utils::errors::server_error::ServerError;
use shared::models::server_models::ServerResponseModel;
use shared::models::user_models::{User, UserList};

pub async fn get_user_chats(
    jwt: String,
    db: Arc<DatabaseConnection>,
) -> Result<ChatList, ServerError> {
    // Get a list of users' chats in timestamp descending order
    chat_service::get_user_chats(jwt, db.clone()).await
}

pub async fn get_chat_messages(
    jwt: String,
    chat_id: i32,
    page: u64,
    page_size: u64,
    db: Arc<DatabaseConnection>,
) -> Result<ChatMessages, ServerError> {
    // Get a list of users' chats in timestamp descending order
    chat_service::get_chat_messages(jwt, chat_id, page, page_size, db.clone()).await
}
