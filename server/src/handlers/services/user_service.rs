use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, Condition};
use sea_orm::sea_query::Expr;
use crate::{entity, utils};
use utils::errors::server_error::ServerError;
use crate::entity::sea_orm_active_enums::Status;
use crate::utils::json_models::server_models::ServerResponseModel;
use crate::utils::json_models::user_models::{FriendRequestList, User, UserList};
use crate::utils::jwt;

pub async fn get_info(
    jwt: String,
    db: Arc<DatabaseConnection>,
) -> Result<User, ServerError> {
    // Decode JWT
    let claim = match jwt::decode_jwt(&jwt) {
        Ok(claim) => claim,
        Err(e) => return Err(ServerError::InvalidToken(e.to_string())),
    };
    // Get the user by their id from the jwt
    let user = utils::db::getters::get_user_by_id(claim.claims.user_id, db).await?;

    // If a user is found, return their info
    match user {
        Some(user) => {
            Ok(User {
                id: user.id,
                username: user.username,
            })
        },
        None => Err(ServerError::UserNotFound),
    }
}

pub async fn send_friend_request(
    jwt: String,
    receiver_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let sender_id = claim.claims.user_id;

    // Check if either user has blocked the other
    let blocked = entity::blocked_users::Entity::find()
        .filter(
            Condition::any()
                .add(entity::blocked_users::Column::UserId.eq(sender_id).and(entity::blocked_users::Column::BlockedId.eq(receiver_id)))
                .add(entity::blocked_users::Column::UserId.eq(receiver_id).and(entity::blocked_users::Column::BlockedId.eq(sender_id)))
        )
        .one(&*db)
        .await?;

    if blocked.is_some() {
        return Err(ServerError::ActionBlocked);
    }

    let new_request = entity::friend_requests::ActiveModel {
        sender_id: Set(sender_id),
        receiver_id: Set(receiver_id),
        status: Set(Status::Pending),
        ..Default::default()
    };

    new_request.insert(&*db).await.map_err(ServerError::DatabaseError)?;
    Ok(ServerResponseModel { success: true })
}

pub async fn accept_friend_request(
    jwt: String,
    sender_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let receiver_id = claim.claims.user_id;

    // Update request to accept
    entity::friend_requests::Entity::update_many()
        .col_expr(entity::friend_requests::Column::Status, Expr::value(Status::Accepted))
        .filter(entity::friend_requests::Column::SenderId.eq(sender_id))
        .filter(entity::friend_requests::Column::ReceiverId.eq(receiver_id))
        .exec(&*db)
        .await?;

    // Create mutual friendships
    for (u1, u2) in [(sender_id, receiver_id), (receiver_id, sender_id)] {
        let friendship = entity::friends::ActiveModel {
            user_id: Set(u1),
            friend_id: Set(u2),
        };
        friendship.insert(&*db).await?;
    }

    Ok(ServerResponseModel { success: true })
}

pub async fn decline_friend_request(
    jwt: String,
    sender_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let receiver_id = claim.claims.user_id;

    entity::friend_requests::Entity::delete_many()
        .filter(entity::friend_requests::Column::SenderId.eq(sender_id))
        .filter(entity::friend_requests::Column::ReceiverId.eq(receiver_id))
        .exec(&*db)
        .await?;

    Ok(ServerResponseModel { success: true })
}

pub async fn get_friend_requests(
    jwt: String,
    db: Arc<DatabaseConnection>,
) -> Result<FriendRequestList, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    // Incoming: others sent to user
    let incoming_requests = entity::friend_requests::Entity::find()
        .filter(entity::friend_requests::Column::ReceiverId.eq(user_id))
        .filter(entity::friend_requests::Column::Status.eq(Status::Pending))
        .all(&*db)
        .await?;

    // Outgoing: User sent to others
    let outgoing_requests = entity::friend_requests::Entity::find()
        .filter(entity::friend_requests::Column::SenderId.eq(user_id))
        .filter(entity::friend_requests::Column::Status.eq(Status::Pending))
        .all(&*db)
        .await?;

    // Collect ids
    let incoming_ids: Vec<i32> = incoming_requests.iter().map(|r| r.sender_id).collect();
    let outgoing_ids: Vec<i32> = outgoing_requests.iter().map(|r| r.receiver_id).collect();

    // Get incoming user info
    let incoming_users = entity::users::Entity::find()
        .filter(entity::users::Column::Id.is_in(incoming_ids))
        .all(&*db)
        .await?;
    
    // Get outgoing user info
    let outgoing_users = entity::users::Entity::find()
        .filter(entity::users::Column::Id.is_in(outgoing_ids))
        .all(&*db)
        .await?;
    
    // Create incoming JSON vector
    let incoming = incoming_users
        .into_iter()
        .map(|u| User {
            id: u.id,
            username: u.username,
        })
        .collect();
    
    // Create outgoing JSON vector
    let outgoing = outgoing_users
        .into_iter()
        .map(|u| User {
            id: u.id,
            username: u.username,
        })
        .collect();

    Ok(FriendRequestList { incoming, outgoing })
}

pub async fn remove_friend(
    jwt: String,
    friend_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    entity::friends::Entity::delete_many()
        .filter(
            Condition::any()
                .add(entity::friends::Column::UserId.eq(user_id).and(entity::friends::Column::FriendId.eq(friend_id)))
                .add(entity::friends::Column::UserId.eq(friend_id).and(entity::friends::Column::FriendId.eq(user_id)))
        )
        .exec(&*db)
        .await?;

    Ok(ServerResponseModel { success: true })
}

pub async fn block_user(
    jwt: String,
    blocked_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    let new_block = entity::blocked_users::ActiveModel {
        user_id: Set(user_id),
        blocked_id: Set(blocked_id),
    };

    new_block.insert(&*db).await.map_err(ServerError::DatabaseError)?;

    // Remove any existing friendship (both directions)
    entity::friends::Entity::delete_many()
        .filter(
            Condition::any()
                .add(entity::friends::Column::UserId.eq(user_id).and(entity::friends::Column::FriendId.eq(blocked_id)))
                .add(entity::friends::Column::UserId.eq(blocked_id).and(entity::friends::Column::FriendId.eq(user_id)))
        )
        .exec(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    entity::friend_requests::Entity::delete_many()
        .filter(
            Condition::any()
                .add(entity::friend_requests::Column::SenderId.eq(user_id).and(entity::friend_requests::Column::ReceiverId.eq(blocked_id)))
                .add(entity::friend_requests::Column::SenderId.eq(blocked_id).and(entity::friend_requests::Column::ReceiverId.eq(user_id)))
        )
        .exec(&*db)
        .await
        .map_err(ServerError::DatabaseError)?;

    Ok(ServerResponseModel { success: true })
}

pub async fn get_friends(jwt: String, db: Arc<DatabaseConnection>) -> Result<UserList, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user =  utils::db::getters::get_user_by_id(claim.claims.user_id, db.clone()).await?;
    if let Some(user) = user {
        // Get user friends and collect them into a vector
        let friends = entity::friends::Entity::find().filter(entity::friends::Column::UserId.eq(user.id)).all(&*db).await?;
        let friend_ids: Vec<i32> = friends.into_iter().map(|f| f.friend_id).collect();

        // Get all friends user info and collect them into a JSON response
        let users = entity::users::Entity::find().filter(entity::users::Column::Id.is_in(friend_ids)).all(&*db).await?;
        let friends = users.into_iter().map(|u| User {
            id: u.id,
            username: u.username,
        }).collect();

        Ok(UserList { users: friends })
    } else {
        Err(ServerError::UserNotFound)
    }
}

