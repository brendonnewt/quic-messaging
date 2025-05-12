use crate::handlers::repositories::user_repository;
use crate::handlers::services::user_service::get_info;
use crate::utils;
use sea_orm::DatabaseConnection;
use shared::models::auth_models::AuthResponseModel;
use shared::models::server_models::ServerResponseModel;
use std::sync::Arc;
use utils::errors::server_error::ServerError;
use utils::jwt;

pub async fn register(
    username: String,
    password: String,
    db: Arc<DatabaseConnection>,
) -> Result<AuthResponseModel, ServerError> {
    // Check if the username is already in use
    let existing_user = user_repository::get_user_by_username(username.clone(), db.clone()).await?;
    if let Some(_) = existing_user {
        return Err(ServerError::UserAlreadyExists);
    }

    let hashed = utils::security::hash_password(password.as_str())?;

    // Register the user in the database
    let mut user = user_repository::register_user(username, hashed, db.clone()).await?;

    // Generate a response
    if let Some(user_id) = user.id.take() {
        // Create a token
        let token =
            jwt::encode_jwt(user_id).map_err(|err| ServerError::JWTCreationError(err.into()))?;
        return Ok(AuthResponseModel {
            success: true,
            token,
            user_id,
        });
    }

    // If we make it here, there's a problem generating the token
    Err(ServerError::JWTCreationError(
        jwt::CreationError::Unexpected,
    ))
}

pub async fn login(
    username: String,
    password: String,
    db: Arc<DatabaseConnection>,
) -> Result<AuthResponseModel, ServerError> {
    // Find the user
    let user = user_repository::get_user_by_username(username.clone(), db.clone()).await?;

    // If a user is found, verify the password
    if let Some(user) = user {
        return if utils::security::verify_password(password.as_str(), user.password_hash.as_str())?
        {
            let token = jwt::encode_jwt(user.id)
                .map_err(|err| ServerError::JWTCreationError(err.into()))?;
            Ok(AuthResponseModel {
                success: true,
                token,
                user_id: user.id,
            })
        } else {
            Err(ServerError::UserNotFound)
        };
    }

    Err(ServerError::UserNotFound)
}

pub async fn update_password(
    jwt: String,
    new_password: String,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    let trimmed = new_password.trim();
    if trimmed.is_empty() {
        return Err(ServerError::RequestInvalid(
            "Password cannot be empty.".into(),
        ));
    }

    // Hash the new password
    let hashed = utils::security::hash_password(&new_password)?;

    // Ensure the user exists
    let user = get_info(jwt, db.clone()).await?;

    // Call the repository to update the password in the database
    let username = user.username; // Assuming you have `username` in the user object
    user_repository::update_password(username, hashed, db.clone()).await?;

    // Return a success response
    return Ok(ServerResponseModel { success: true });
}
