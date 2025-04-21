use std::sync::Arc;
use chrono::Utc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, QueryOrder, PaginatorTrait, NotSet};
use crate::utils::errors::server_error::ServerError;
use crate::utils::json_models::server_models::ServerResponseModel;
use crate::utils::jwt;
use crate::entity;
use sea_orm::sea_query::Expr;

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

    let new_chat = entity::chats::ActiveModel {
        name: Set(name),
        is_group: Set(is_group as i8),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let chat = new_chat.insert(&*db).await.map_err(ServerError::DatabaseError)?;

    let mut members = member_ids;
    if !members.contains(&creator_id) {
        members.push(creator_id);
    }

    for uid in members {
        let member = entity::chat_members::ActiveModel {
            chat_id: Set(chat.id),
            user_id: Set(uid),
        };
        member.insert(&*db).await.map_err(ServerError::DatabaseError)?;
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

    let new_msg = entity::messages::ActiveModel {
        chat_id: Set(chat_id),
        sender_id: Set(sender_id),
        content: Set(content),
        read: Set(false as i8),
        timestamp: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    new_msg.insert(&*db).await.map_err(ServerError::DatabaseError)?;
    Ok(ServerResponseModel { success: true })
}

// Get messages in a chat
pub async fn get_chat_messages(
    jwt: String,
    chat_id: i32,
    page: u64,
    page_size: u64,
    db: Arc<DatabaseConnection>,
) -> Result<Vec<entity::messages::Model>, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    // Confirm user is in chat
    let is_member = entity::chat_members::Entity::find()
        .filter(entity::chat_members::Column::ChatId.eq(chat_id))
        .filter(entity::chat_members::Column::UserId.eq(user_id))
        .one(&*db)
        .await?
        .is_some();

    if !is_member {
        return Err(ServerError::Forbidden);
    }

    let paginator = entity::messages::Entity::find()
        .filter(entity::messages::Column::ChatId.eq(chat_id))
        .order_by_desc(entity::messages::Column::Timestamp)
        .paginate(&*db, page_size);

    let messages = paginator.fetch_page(page).await?;

    Ok(messages)
}


// Mark messages as read (per-user tracking)
pub async fn mark_messages_read(
    jwt: String,
    chat_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    let is_member = entity::chat_members::Entity::find()
        .filter(entity::chat_members::Column::ChatId.eq(chat_id))
        .filter(entity::chat_members::Column::UserId.eq(user_id))
        .one(&*db)
        .await?
        .is_some();

    if !is_member {
        return Err(ServerError::Forbidden);
    }

    let unread_messages = entity::messages::Entity::find()
        .filter(entity::messages::Column::ChatId.eq(chat_id))
        .all(&*db)
        .await?;

    for msg in unread_messages {
        let existing = entity::message_reads::Entity::find()
            .filter(entity::message_reads::Column::MessageId.eq(msg.id))
            .filter(entity::message_reads::Column::UserId.eq(user_id))
            .one(&*db)
            .await?;

        if existing.is_none() {
            let read = entity::message_reads::ActiveModel {
                message_id: Set(msg.id),
                user_id: Set(user_id),
                read_at: Set(Utc::now().naive_utc()),
            };
            read.insert(&*db).await?;
        }
    }

    Ok(ServerResponseModel { success: true })
}

// Get unread message count for a chat
pub async fn get_unread_message_count(
    jwt: String,
    chat_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<i64, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    let total_messages = entity::messages::Entity::find()
        .filter(entity::messages::Column::ChatId.eq(chat_id))
        .all(&*db)
        .await?;

    let mut unread_count = 0;
    for msg in total_messages {
        let read_entry = entity::message_reads::Entity::find()
            .filter(entity::message_reads::Column::MessageId.eq(msg.id))
            .filter(entity::message_reads::Column::UserId.eq(user_id))
            .one(&*db)
            .await?;

        if read_entry.is_none() {
            unread_count += 1;
        }
    }

    Ok(unread_count)
}

