use crate::handlers::services::chat_service;
use crate::utils::errors::server_error::ServerError;
use sea_orm::DatabaseConnection;
use shared::models::chat_models::{ChatList, ChatMessages, Count};
use shared::models::server_models::ServerResponseModel;
use std::sync::Arc;

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

pub async fn send_message(
    jwt: String,
    chat_id: i32,
    content: String,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    chat_service::send_message(jwt, chat_id, content, db.clone()).await
}

pub async fn get_chat_page_count(
    jwt: String,
    chat_id: i32,
    page_size: u64,
    db: Arc<DatabaseConnection>,
) -> Result<Count, ServerError> {
    chat_service::get_chat_page_count(jwt, chat_id, page_size, db.clone()).await
}

pub async fn create_chat(
    jwt: String,
    name: Option<String>,
    is_group: bool,
    member_ids: Vec<i32>,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    chat_service::create_chat(jwt, name, is_group, member_ids, db.clone()).await
}

pub async fn get_unread_message_count(
    jwt: String,
    db: Arc<DatabaseConnection>,
) -> Result<Count, ServerError> {
    chat_service::get_unread_message_count(jwt, db.clone()).await
}

pub async fn mark_messages_read(
    jwt: String,
    chat_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    chat_service::mark_messages_read(jwt, chat_id, db.clone()).await
}

pub async fn get_chat_user_ids(
    chat_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<Vec<i32>, ServerError> {
    chat_service::get_chat_user_ids(chat_id, db.clone()).await
}
