use crate::{entity, utils};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;
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
