use std::sync::Arc;
use sea_orm::DatabaseConnection;
use crate::handlers::services::auth_service;
use crate::utils::errors::server_error::ServerError;
use crate::utils::json_models::auth_models::AuthResponseModel;

async fn register(
    username: String,
    hashed_password: String,
    db: Arc<DatabaseConnection>,
) -> Result<AuthResponseModel, ServerError> {
    // Call register service
    auth_service::register(username, hashed_password, db).await
}
