use std::sync::Arc;
use sea_orm::DatabaseConnection;
use crate::handlers::services::{auth_service, user_service};
use crate::utils::errors::server_error::ServerError;
use crate::utils::json_models::auth_models::AuthResponseModel;
use crate::utils::json_models::server_models::ServerResponseModel;
use crate::utils::json_models::user_models::{User, UserList};

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
    Ok(ServerResponseModel{success: true})
}

pub async fn remove_friend(jwt: String, friend_id: i32, db: Arc<DatabaseConnection>) -> Result<ServerResponseModel, ServerError> {
    Ok(ServerResponseModel{success: true})
}

pub async fn get_friends(jwt: String, db: Arc<DatabaseConnection>) -> Result<UserList, ServerError> {
    // Returns UserList of friends, which is a JSON vector of User JSON models
    user_service::get_friends(jwt, db).await
}
