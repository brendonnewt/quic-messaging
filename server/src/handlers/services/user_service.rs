use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait, NotSet};
use crate::{entity, utils};
use utils::errors::server_error::ServerError;
use crate::utils::json_models::user_models::UserResponseModel;

pub async fn get_info(
    jwt: String,
    db: Arc<DatabaseConnection>,
) -> Result<UserResponseModel, ServerError> {
    // Decode JWT
    let claim = match utils::jwt::decode_jwt(&jwt) {
        Ok(claim) => claim,
        Err(e) => return Err(ServerError::InvalidToken(e.to_string())),
    };
    // Get the user by their id from the jwt
    let user = utils::db::getters::get_user_by_id(claim.claims.user_id, db).await?;
    
    // If a user is found, return their info
    match user {
        Some(user) => {
            Ok(UserResponseModel {
                id: user.id,
                username: user.username,
            })
        },
        None => Err(ServerError::UserNotFound),
    }
}

