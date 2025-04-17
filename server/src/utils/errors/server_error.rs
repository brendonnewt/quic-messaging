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
    
    #[error("Token expired")]
    TokenExpired,

    #[error("JWT creation error: {0}")]
    JWTCreationError(#[from] CreationError),
}
