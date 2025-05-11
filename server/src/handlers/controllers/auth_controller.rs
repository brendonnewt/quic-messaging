use crate::handlers::services::auth_service;
use crate::utils::errors::server_error::ServerError;
use sea_orm::DatabaseConnection;
use shared::models::auth_models::AuthResponseModel;
use shared::models::server_models::ServerResponseModel;
use std::sync::Arc;

pub async fn register(
    username: String,
    hashed_password: String,
    db: Arc<DatabaseConnection>,
) -> Result<AuthResponseModel, ServerError> {
    // Call register service
    auth_service::register(username, hashed_password, db).await
}

pub async fn login(
    username: String,
    password: String,
    db: Arc<DatabaseConnection>,
) -> Result<AuthResponseModel, ServerError> {
    // Call login service
    auth_service::login(username, password, db).await
}

pub async fn update_password(
    jwt: String,
    new_password: String,
    db: Arc<DatabaseConnection>,
) -> Result<ServerResponseModel, ServerError> {
    auth_service::update_password(jwt, new_password, db).await
}
