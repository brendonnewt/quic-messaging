use std::sync::Arc;
use sea_orm::DatabaseConnection;
use crate::{entity, utils};
use utils::errors::server_error::ServerError;
use crate::entity::sea_orm_active_enums::Status;
use shared::models::server_models::ServerResponseModel;
use shared::models::user_models::{FriendRequestList, User, UserList};
use crate::utils::jwt;
use crate::handlers::repositories::user_repository;

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
    let user = user_repository::get_user_by_id(claim.claims.user_id, db).await?;

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
    let blocked = user_repository::get_user_blocked(sender_id, receiver_id, db.clone()).await?;

    if blocked.is_some() {
        return Err(ServerError::ActionBlocked);
    }

    // Send friend request through the database
    user_repository::send_friend_request(sender_id, receiver_id, db.clone()).await?;

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
    user_repository::update_friend_request_status(sender_id, receiver_id, Status::Accepted, db.clone()).await?;

    // Create mutual friendships
    for (u1, u2) in [(sender_id, receiver_id), (receiver_id, sender_id)] {
        user_repository::create_friendship(u1, u2, db.clone()).await?;
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

    // Mark request as rejected and delete it
    user_repository::update_friend_request_status(sender_id, receiver_id, Status::Rejected, db.clone()).await?;

    Ok(ServerResponseModel { success: true })
}

pub async fn cancel_friend_request(jwt: String, receiver_id: i32, db: Arc<DatabaseConnection>) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let sender_id = claim.claims.user_id;

    // Check if either user has blocked the other
    let blocked = user_repository::get_user_blocked(sender_id, receiver_id, db.clone()).await?;

    if blocked.is_some() {
        return Err(ServerError::ActionBlocked);
    }

    // Delete friend request through the database
    user_repository::update_friend_request_status(sender_id, receiver_id, Status::Rejected, db.clone()).await?;

    Ok(ServerResponseModel { success: true })

}

pub async fn get_friend_requests(
    jwt: String,
    db: Arc<DatabaseConnection>,
) -> Result<FriendRequestList, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    // Incoming: others sent to user
    let incoming_requests = user_repository::get_user_friend_requests(user_id, Some(entity::friend_requests::Column::ReceiverId), db.clone()).await?;

    // Outgoing: User sent to others
    let outgoing_requests = user_repository::get_user_friend_requests(user_id, Some(entity::friend_requests::Column::SenderId), db.clone()).await?;

    // Collect ids
    let incoming_ids: Vec<i32> = incoming_requests.iter().map(|r| r.sender_id).collect();
    let outgoing_ids: Vec<i32> = outgoing_requests.iter().map(|r| r.receiver_id).collect();

    // Get incoming user info
    let incoming_users = user_repository::get_users_from_list(incoming_ids, db.clone()).await?;
    
    // Get outgoing user info
    let outgoing_users = user_repository::get_users_from_list(outgoing_ids, db.clone()).await?;
    
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

    // Delete the friendship from the database
    user_repository::delete_friendship(user_id, friend_id, db.clone()).await?;

    Ok(ServerResponseModel { success: true })
}

pub async fn block_user(
    jwt: String,
    blocked_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user_id = claim.claims.user_id;

    // Block the user in the database
    user_repository::block_user(user_id, blocked_id, db.clone()).await?;

    // Remove any existing friendship (bidirectional)
    user_repository::delete_friendship(user_id, blocked_id, db.clone()).await?;

    // Delete any active friend requests (bidirectional)
    user_repository::delete_friend_requests(user_id, blocked_id, db.clone()).await?;

    Ok(ServerResponseModel { success: true })
}

pub async fn get_friends(jwt: String, db: Arc<DatabaseConnection>) -> Result<UserList, ServerError> {
    let claim = jwt::decode_jwt(&jwt).map_err(|e| ServerError::InvalidToken(e.to_string()))?;
    let user =  user_repository::get_user_by_id(claim.claims.user_id, db.clone()).await?;
    if let Some(user) = user {
        // Get user friends and collect them into a vector
        let friends = user_repository::get_user_friends(user.id, db.clone()).await?;
        let friend_ids: Vec<i32> = friends.into_iter().map(|f| f.friend_id).collect();

        // Get all friends user info and collect them into a JSON response
        let users = user_repository::get_users_from_list(friend_ids, db.clone()).await?;
        let friends = users.into_iter().map(|u| User {
            id: u.id,
            username: u.username,
        }).collect();

        Ok(UserList { users: friends })
    } else {
        Err(ServerError::UserNotFound)
    }
}

