use crate::entity;
use crate::handlers::repositories::chat_repository::{get_other_usernames_in_chat, get_read_entry};
use crate::handlers::repositories::{chat_repository, user_repository};
use crate::utils::errors::server_error::ServerError;
use crate::utils::jwt;
use futures::future::join_all;
use sea_orm::DatabaseConnection;
use shared::models::chat_models;
use shared::models::chat_models::{ChatList, ChatMessage, ChatMessages, Count};
use shared::models::server_models::ServerResponseModel;
use std::sync::Arc;

// Create a new chat (group or direct)
pub async fn create_chat(
    jwt: String,
    name: Option<String>,
    is_group: bool,
    member_ids: Vec<i32>,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let creator_id = claim.claims.user_id;

    let chat =
        chat_repository::create_new_chat(name, is_group, member_ids.clone(), db.clone()).await?;

    let mut members = member_ids;
    if !members.contains(&creator_id) {
        members.push(creator_id);
    }

    let chat_id = chat.id;

    for uid in members {
        chat_repository::add_chat_member(chat_id, uid, db.clone()).await?;
    }

    Ok(ServerResponseModel { success: true })
}

// Send a message to a chat
pub async fn send_message(
    jwt: String,
    chat_id: i32,
    content: String,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let sender_id = claim.claims.user_id;

    let user = user_repository::get_user_by_id(sender_id, db.clone()).await?;

    if let Some(user) = user {
        chat_repository::send_message(chat_id, sender_id, user.username, content, db.clone())
            .await?;
        return Ok(ServerResponseModel { success: true });
    }

    Err(ServerError::UserNotFound)
}

pub async fn get_user_chats(
    jwt: String,
    page: u64,
    page_size: u64,
    db: Arc<DatabaseConnection>,
) -> Result<ChatList, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    let chats = chat_repository::get_user_chats_paged(user_id, page, page_size, db.clone()).await?;

    let futures = chats.into_iter().map(|c| {
        let db = db.clone();
        async move {
            let name = if let Some(name) = &c.name {
                name.clone()
            } else {
                match get_other_usernames_in_chat(c.id, user_id, db.clone()).await {
                    Ok(usernames) => usernames.join(", "),
                    Err(_) => String::new(),
                }
            };

            let unread_count = get_unread_chat_message_count(user_id, c.id, db.clone()).await;

            chat_models::Chat {
                id: c.id,
                chat_name: name,
                unread_count: unread_count.unwrap_or_else(|_| 0),
            }
        }
    });

    let chat_results: Vec<chat_models::Chat> = join_all(futures).await;

    Ok(ChatList {
        chats: chat_results,
    })
}

// Get messages in a chat
pub async fn get_chat_messages(
    jwt: String,
    chat_id: i32,
    page: u64,
    page_size: u64,
    db: Arc<DatabaseConnection>,
) -> Result<ChatMessages, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    // Confirm user is in chat
    let is_member = chat_repository::is_user_chat_member(chat_id, user_id, db.clone()).await;

    if !is_member {
        return Err(ServerError::Forbidden);
    }

    let messages =
        chat_repository::get_paginated_messages(chat_id, page, page_size, db.clone()).await?;

    let messages: Vec<ChatMessage> = messages
        .iter()
        .map(|msg| ChatMessage {
            user_id,
            username: msg.sender_username.clone(),
            content: msg.content.clone(),
        })
        .collect();

    Ok(ChatMessages {
        id: chat_id,
        messages,
    })
}

pub async fn get_chat_page_count(
    jwt: String,
    chat_id: i32,
    page_size: u64,
    db: Arc<DatabaseConnection>,
) -> Result<Count, ServerError> {
    // Ensure token provided is valid
    jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;

    let pages = chat_repository::get_chat_page_count(chat_id, page_size, db.clone()).await?;

    Ok(Count { count: pages })
}

pub async fn get_chats_page_count(
    jwt: String,
    page_size: u64,
    db: Arc<DatabaseConnection>,
) -> Result<Count, ServerError> {
    // Ensure token provided is valid
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    let pages = chat_repository::get_chats_page_count(user_id, page_size, db.clone()).await?;

    Ok(Count { count: pages })
}

// Mark messages as read (per-user tracking)
pub async fn mark_messages_read(
    jwt: String,
    chat_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    let is_member = chat_repository::is_user_chat_member(chat_id, user_id, db.clone()).await;
    if !is_member {
        return Err(ServerError::Forbidden);
    }

    // Get all message IDs for a chat
    let message_ids: Vec<i32> = chat_repository::get_chat_message_ids(chat_id, db.clone()).await?;

    // If none, all are read
    if message_ids.is_empty() {
        return Ok(ServerResponseModel { success: true });
    }

    // Get already read message_ids for this user
    let read_ids: Vec<i32> =
        chat_repository::get_user_chat_unread_messages(user_id, message_ids.clone(), db.clone())
            .await?;

    // Compute unread message_ids
    let unread_ids: Vec<i32> = message_ids
        .into_iter()
        .filter(|id| !read_ids.contains(id))
        .collect();

    // Bulk insert the missing reads
    chat_repository::mark_messages_read(user_id, unread_ids, db.clone()).await?;

    Ok(ServerResponseModel { success: true })
}

// Get unread message count for a chat
pub async fn get_unread_chat_message_count(
    user_id: i32,
    chat_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<u64, ServerError> {
    let messages = chat_repository::get_chat_messages(chat_id, db.clone()).await?;

    let unread_count = get_unread_count(user_id, false, messages, db.clone()).await?;

    Ok(unread_count)
}

pub async fn get_unread_message_count(
    jwt: String,
    db: Arc<DatabaseConnection>,
) -> Result<Count, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    let chats = chat_repository::get_user_chats(user_id, db.clone()).await?;

    // Create a future for each chat to get its messages
    let futures = chats
        .iter()
        .map(|chat| chat_repository::get_chat_messages(chat.id, db.clone()));

    // Await all message fetches
    let results: Vec<Result<Vec<entity::messages::Model>, ServerError>> = join_all(futures).await;

    // Flatten and collect all successful message vectors
    let messages: Vec<entity::messages::Model> = results
        .into_iter()
        .collect::<Result<Vec<_>, _>>()? // Propagate error if any
        .into_iter()
        .flatten()
        .collect();

    let unread_count = get_unread_count(user_id, true, messages, db.clone()).await?;

    Ok(Count {
        count: unread_count,
    })
}

pub async fn get_unread_count(
    user_id: i32,
    multiple_chats: bool,
    messages: Vec<entity::messages::Model>,
    db: Arc<DatabaseConnection>,
) -> Result<u64, ServerError> {
    let mut unread_count = 0;
    for msg in messages {
        let read_entry = get_read_entry(user_id, msg.id, db.clone()).await?;

        if read_entry.is_some() {
            if !multiple_chats {
                break; // Stop counting once the first read message is found
            } else {
                continue;
            }
        }

        unread_count += 1;
    }
    Ok(unread_count)
}

pub async fn get_chat_user_ids(
    chat_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<Vec<i32>, ServerError> {
    chat_repository::get_chat_user_ids(chat_id, db.clone()).await
}
