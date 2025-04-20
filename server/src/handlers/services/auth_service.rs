use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, NotSet};
use crate::{entity, utils};
use utils::errors::server_error::ServerError;
use crate::utils::json_models::auth_models::AuthResponseModel;
use crate::utils::jwt;

// TODO: Password hashing/security stuff
pub async fn register(
    username: String,
    password: String,
    db: Arc<DatabaseConnection>,
) -> Result<AuthResponseModel, ServerError> {
    // Check if the username is already in use
    let existing_user = entity::users::Entity::find().filter(entity::users::Column::Username.eq(username.clone())).one(&*db).await.map_err(|err| ServerError::DatabaseError(err))?;
    if let Some(_) = existing_user {
        return Err(ServerError::UserAlreadyExists);
    }

    let hashed = utils::security::hash_password(password.as_str())?;

    // Create a new user
    let new_user = entity::users::ActiveModel {
        id: NotSet,
        username: Set(username),
        password_hash: Set(hashed.clone()),
    };

    // Save the user to DB
    let mut user = new_user.save(&*db).await.map_err(|err| ServerError::DatabaseError(err))?;

    // Generate a response
    if let Some(user_id) = user.id.take() {
        // Create a token
        let token = jwt::encode_jwt(user_id).map_err(|err| ServerError::JWTCreationError(err.into()))?;
        return Ok(AuthResponseModel { success: true, token })
    }

    // If we make it here, there's a problem generating the token
    Err(ServerError::JWTCreationError(jwt::CreationError::Unexpected))
}

pub async fn login(username: String, hashed_password: String, db: Arc<DatabaseConnection>) -> Result<AuthResponseModel, ServerError> {
    // Find the user
    let user = entity::users::Entity::find().filter(entity::users::Column::Username.eq(username)).one(&*db).await.map_err(|err| ServerError::DatabaseError(err))?;

    // If a user is found, verify the password
    if let Some(user) = user {
        return if utils::security::verify_password(user.password_hash.as_str(), hashed_password.as_str())? {
            let token = jwt::encode_jwt(user.id).map_err(|err| ServerError::JWTCreationError(err.into()))?;
            Ok(AuthResponseModel { success: true, token })
        } else {
            Err(ServerError::UserNotFound)
        }
    }

    Err(ServerError::UserNotFound)
}
