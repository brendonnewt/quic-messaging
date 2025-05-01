use crate::{entity, utils};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;
use shared::models::user_models::User;
use utils::errors::server_error::ServerError;

pub async fn create_new_chat(
    name: Option<String>,
    is_group: bool,
    db: Arc<DatabaseConnection>,
) -> Result<entity::chats::Model, ServerError> {
    let new_chat = entity::chats::ActiveModel {
        name: Set(name),
        is_group: Set(is_group as i8),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    Ok(new_chat
        .insert(&*db)
        .await
        .map_err(ServerError::DatabaseError)?)
}

pub async fn add_chat_member(
    chat_id: i32,
    user_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<(), ServerError> {
    let member = entity::chat_members::ActiveModel {
        chat_id: Set(chat_id),
        user_id: Set(user_id),
    };
    member
        .insert(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;
    Ok(())
}

pub async fn is_user_chat_member(chat_id: i32, user_id: i32, db: Arc<DatabaseConnection>) -> bool {
    if let Ok(result) = entity::chat_members::Entity::find()
        .filter(entity::chat_members::Column::ChatId.eq(chat_id))
        .filter(entity::chat_members::Column::UserId.eq(user_id))
        .one(&*db)
        .await
    {
        result.is_some()
    } else {
        false
    }
}

use std::collections::HashMap;

pub async fn get_user_chats(
    user_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<Vec<entity::chats::Model>, ServerError> {
    // Get all chat IDs the user is in
    let chat_ids: Vec<i32> = entity::chat_members::Entity::find()
        .filter(entity::chat_members::Column::UserId.eq(user_id))
        .select_only()
        .column(entity::chat_members::Column::ChatId)
        .into_tuple()
        .all(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    // Get chats
    let chats: Vec<entity::chats::Model> = entity::chats::Entity::find()
        .filter(entity::chats::Column::Id.is_in(chat_ids.clone()))
        .all(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    // Get latest message per chat
    let last_messages: Vec<entity::messages::Model> = entity::messages::Entity::find()
        .filter(entity::messages::Column::ChatId.is_in(chat_ids))
        .select_only()
        .columns([entity::messages::Column::ChatId, entity::messages::Column::Timestamp])
        .order_by_desc(entity::messages::Column::Timestamp)
        .all(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    // Map of chat_id -> last timestamp
    let mut latest_by_chat: HashMap<i32, chrono::NaiveDateTime> = HashMap::new();
    for msg in last_messages {
        latest_by_chat.entry(msg.chat_id).or_insert(msg.timestamp);
    }

    // Sort chats by last message timestamp descending
    let mut chats = chats;
    chats.sort_by_key(|chat| {
        // Default to the chat's created_at if no message
        std::cmp::Reverse(latest_by_chat.get(&chat.id).cloned().unwrap_or(chat.created_at))
    });

    Ok(chats)
}


pub async fn get_other_usernames_in_chat(
    chat_id: i32,
    current_user_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<Vec<String>, ServerError> {
    // Get all user IDs in the chat except the current user
    let user_ids: Vec<i32> = entity::chat_members::Entity::find()
        .filter(entity::chat_members::Column::ChatId.eq(chat_id))
        .filter(entity::chat_members::Column::UserId.ne(current_user_id))
        .select_only()
        .column(entity::chat_members::Column::UserId)
        .into_tuple()
        .all(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    // Fetch the usernames for those user IDs
    let users = entity::users::Entity::find()
        .filter(entity::users::Column::Id.is_in(user_ids))
        .select_only()
        .column(entity::users::Column::Username)
        .into_tuple()
        .all(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    Ok(users)
}



pub async fn get_paginated_messages(
    chat_id: i32,
    page: u64,
    page_size: u64,
    db: Arc<DatabaseConnection>,
) -> Result<Vec<entity::messages::Model>, ServerError> {
    let paginator = entity::messages::Entity::find()
        .filter(entity::messages::Column::ChatId.eq(chat_id))
        .order_by_desc(entity::messages::Column::Timestamp)
        .paginate(&*db, page_size);

    let messages: Vec<entity::messages::Model> = paginator.fetch_page(page).await?;
    Ok(messages)
}

pub async fn send_message(
    chat_id: i32,
    sender_id: i32,
    content: String,
    db: Arc<DatabaseConnection>,
) -> Result<(), ServerError> {
    let new_msg = entity::messages::ActiveModel {
        chat_id: Set(chat_id),
        sender_id: Set(sender_id),
        content: Set(content),
        read: Set(false as i8),
        timestamp: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    new_msg
        .insert(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;
    Ok(())
}

pub async fn get_chat_message_ids(
    chat_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<Vec<i32>, ServerError> {
    Ok(entity::messages::Entity::find()
        .filter(entity::messages::Column::ChatId.eq(chat_id))
        .select_only()
        .column(entity::messages::Column::Id)
        .into_tuple::<i32>()
        .all(&*db)
        .await?)
}

pub async fn get_user_chat_unread_messages(
    user_id: i32,
    message_ids: Vec<i32>,
    db: Arc<DatabaseConnection>,
) -> Result<Vec<i32>, ServerError> {
    Ok(entity::message_reads::Entity::find()
        .filter(entity::message_reads::Column::UserId.eq(user_id)) // Get messages user has read
        .filter(entity::message_reads::Column::MessageId.is_in(message_ids.clone())) // Narrow to messages in the chat
        .select_only()
        .column(entity::message_reads::Column::MessageId) // Get the IDs
        .into_tuple()
        .all(&*db)
        .await?)
}

pub async fn mark_messages_read(
    user_id: i32,
    unread_ids: Vec<i32>,
    db: Arc<DatabaseConnection>,
) -> Result<(), ServerError> {
    let now = Utc::now().naive_utc();
    let new_reads: Vec<_> = unread_ids
        .into_iter()
        .map(|msg_id| entity::message_reads::ActiveModel {
            message_id: Set(msg_id),
            user_id: Set(user_id),
            read_at: Set(now),
        })
        .collect();

    if !new_reads.is_empty() {
        entity::message_reads::Entity::insert_many(new_reads)
            .exec(&*db)
            .await?;
    }

    Ok(())
}

pub async fn get_chat_messages(
    chat_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<Vec<entity::messages::Model>, ServerError> {
    Ok(entity::messages::Entity::find()
        .filter(entity::messages::Column::ChatId.eq(chat_id))
        .order_by_desc(entity::messages::Column::Timestamp)
        .all(&*db)
        .await?)
}

pub async fn get_read_entry(
    user_id: i32,
    msg_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<Option<entity::message_reads::Model>, ServerError> {
    entity::message_reads::Entity::find()
        .filter(entity::message_reads::Column::MessageId.eq(msg_id))
        .filter(entity::message_reads::Column::UserId.eq(user_id))
        .one(&*db)
        .await
        .map_err(|err| ServerError::DatabaseError(err))
}
