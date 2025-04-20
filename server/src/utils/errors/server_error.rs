use sea_orm::DbErr;
use thiserror::Error;
use crate::utils::jwt;
use crate::utils::jwt::CreationError;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbErr),

    #[error("User already exists")]
    UserAlreadyExists,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Invalid Token: {0}")]
    InvalidToken(String),

    #[error("JWT creation error: {0}")]
    JWTCreationError(#[from] CreationError),

    #[error("Invalid Password: {0}")]
    PasswordInvalid(String),
}
