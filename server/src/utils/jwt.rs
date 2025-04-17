use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use jsonwebtoken::errors::Error as JWTLibError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use super::constants;

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub user_id: i32,
}

#[derive(Debug, Error)]
pub enum CreationError {
    #[error("Failed to encode JWT: {0}")]
    Encode(#[from] JWTLibError),

    #[error("Unexpected error during JWT creation")]
    Unexpected,
}

/// Encodes user info into a JWT string
pub fn encode_jwt(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expire = Duration::hours(24);

    let claim = Claims {
        exp: (now + expire).timestamp() as usize,
        iat: now.timestamp() as usize,
        user_id,
    };

    let secret = constants::SECRET.clone();

    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

/// Decodes a JWT and returns the embedded claims
pub fn decode_jwt(jwt: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let secret = constants::SECRET.clone();

    decode(
        jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
}
