use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, NotSet, Condition};
use sea_orm::sea_query::Expr;
use crate::{entity, utils};
use utils::errors::server_error::ServerError;
use crate::entity::sea_orm_active_enums::Status;
use crate::entity::users::Model;
use crate::utils::json_models::auth_models::AuthResponseModel;

pub async fn register_user(username: String, hashed: String, db: Arc<DatabaseConnection>) -> Result<entity::users::ActiveModel, ServerError> {
    // Create a new user
    let new_user = entity::users::ActiveModel {
        id: NotSet,
        username: Set(username),
        password_hash: Set(hashed.clone()),
    };

    // Save the user to DB
    new_user.save(&*db).await.map_err(|err| ServerError::DatabaseError(err))
}

pub async fn get_user_by_username(username: String, db: Arc<DatabaseConnection>) -> Result<Option<Model>, ServerError> {
    entity::users::Entity::find().filter(entity::users::Column::Username.eq(username.clone())).one(&*db).await.map_err(|err| ServerError::DatabaseError(err))
}

pub async fn get_user_by_id(id: i32, db: Arc<DatabaseConnection>) -> Result<Option<Model>, ServerError> {
    entity::users::Entity::find().filter(entity::users::Column::Id.eq(id)).one(&*db).await.map_err(|err| ServerError::DatabaseError(err))
}

pub async fn send_friend_request(sender_id: i32, receiver_id: i32, db: Arc<DatabaseConnection>) -> Result<(), ServerError> {
    let new_request = entity::friend_requests::ActiveModel {
        sender_id: Set(sender_id),
        receiver_id: Set(receiver_id),
        status: Set(Status::Pending),
        ..Default::default()
    };

    new_request.insert(&*db).await.map_err(ServerError::DatabaseError)?;
    Ok(())
}

pub async fn update_friend_request_status(sender_id: i32, receiver_id: i32, status: Status, db: Arc<DatabaseConnection>) -> Result<(), ServerError> {
    if status == Status::Rejected {
        entity::friend_requests::Entity::delete_many()
            .filter(entity::friend_requests::Column::SenderId.eq(sender_id))
            .filter(entity::friend_requests::Column::ReceiverId.eq(receiver_id))
            .exec(&*db)
            .await?;
    } else {
        entity::friend_requests::Entity::update_many()
            .col_expr(entity::friend_requests::Column::Status, Expr::value(status))
            .filter(entity::friend_requests::Column::SenderId.eq(sender_id))
            .filter(entity::friend_requests::Column::ReceiverId.eq(receiver_id))
            .exec(&*db)
            .await?;
    }
    
    Ok(())
}

pub async fn create_friendship(u1: i32, u2: i32, db: Arc<DatabaseConnection>) -> Result<(), ServerError> {
    let friendship = entity::friends::ActiveModel {
        user_id: Set(u1),
        friend_id: Set(u2),
    };
    friendship.insert(&*db).await?;
    Ok(())
}

pub async fn get_user_blocked(sender_id: i32, receiver_id: i32, db: Arc<DatabaseConnection>) -> Result<Option<entity::blocked_users::Model>, ServerError> {
    entity::blocked_users::Entity::find()
        .filter(
            Condition::any()
                .add(entity::blocked_users::Column::UserId.eq(sender_id).and(entity::blocked_users::Column::BlockedId.eq(receiver_id)))
                .add(entity::blocked_users::Column::UserId.eq(receiver_id).and(entity::blocked_users::Column::BlockedId.eq(sender_id)))
        )
        .one(&*db)
        .await.map_err(|err| ServerError::DatabaseError(err))
}