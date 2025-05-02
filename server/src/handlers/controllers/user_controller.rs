use std::sync::Arc;
use sea_orm::DatabaseConnection;
use crate::handlers::services::{user_service};
use crate::utils::errors::server_error::ServerError;
use shared::models::server_models::ServerResponseModel;
use shared::models::user_models::{FriendRequestList, User, UserList};

pub async fn get_user_info(
    jwt: String,
    db: Arc<DatabaseConnection>,
) -> Result<User, ServerError> {
    // Use user service to get users basic info
    user_service::get_info(jwt, db).await
}

/// Send a friend request
pub async fn add_friend(
    jwt: String,
    friend_id: i32,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    // TODO: SEND FRIEND REQUEST
    user_service::send_friend_request(jwt, friend_id, db).await
    //Ok(ServerResponseModel{success: true})
}

pub async fn remove_friend(jwt: String, friend_id: i32, db: Arc<DatabaseConnection>) -> Result<ServerResponseModel, ServerError> {
    Ok(ServerResponseModel{success: true})
}

pub async fn get_friends(jwt: String, db: Arc<DatabaseConnection>) -> Result<UserList, ServerError> {
    // Returns UserList of friends, which is a JSON vector of User JSON models
    user_service::get_friends(jwt, db).await
}

pub async fn get_friend_requests(jwt: String, db: Arc<DatabaseConnection>) -> Result<FriendRequestList, ServerError> {
    user_service::get_friend_requests(jwt, db).await
}
