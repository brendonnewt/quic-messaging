use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng, PasswordHash};
use crate::utils::errors::server_error::ServerError;

pub fn hash_password(password: &str) -> Result<String, ServerError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt).map_err(|e| ServerError::PasswordInvalid(e.to_string()))?
        .to_string();

    Ok(hash)
}

pub fn verify_password(
    password: &str,
    hashed: &str,
) -> Result<bool, ServerError> {
    let parsed_hash = PasswordHash::new(hashed).map_err(|e| ServerError::PasswordInvalid(e.to_string()))?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}
