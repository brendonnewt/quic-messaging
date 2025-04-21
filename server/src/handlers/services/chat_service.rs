use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, QueryOrder};
use crate::utils::errors::server_error::ServerError;
use crate::utils::json_models::server_models::ServerResponseModel;
use crate::utils::jwt;
use crate::entity;

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
        name: Set(Option::from(name.unwrap_or_default())),
        is_group: Set(is_group as i8),
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
        ..Default::default()
    };

    new_msg.insert(&*db).await.map_err(ServerError::DatabaseError)?;
    Ok(ServerResponseModel { success: true })
}

// Get messages in a chat
pub async fn get_chat_messages(
    jwt: String,
    chat_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<Vec<entity::messages::Model>, ServerError> {
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

    let messages = entity::messages::Entity::find()
        .filter(entity::messages::Column::ChatId.eq(chat_id))
        .order_by_asc(entity::messages::Column::Timestamp)
        .all(&*db)
        .await?;

    Ok(messages)
}

// Mark messages as read in a chat
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

    use sea_orm::sea_query::Expr;
    entity::messages::Entity::update_many()
        .col_expr(entity::messages::Column::Read, Expr::value(true))
        .filter(entity::messages::Column::ChatId.eq(chat_id))
        .exec(&*db)
        .await?;

    Ok(ServerResponseModel { success: true })
}
