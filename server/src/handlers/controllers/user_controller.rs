use std::sync::Arc;
use sea_orm::DatabaseConnection;
use crate::handlers::services::{auth_service, user_service};
use crate::utils::errors::server_error::ServerError;
use crate::utils::json_models::auth_models::AuthResponseModel;
use crate::utils::json_models::user_models::UserResponseModel;

pub async fn get_user_info(
    jwt: String,
    db: Arc<DatabaseConnection>,
) -> Result<UserResponseModel, ServerError> {
    // Use user service to get users basic info
    user_service::get_info(jwt, db).await
}
