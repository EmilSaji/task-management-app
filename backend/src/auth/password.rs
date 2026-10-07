use std::sync::OnceLock;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::error::{AppError, AppResult};

/// Hash a password with Argon2id (default params) and a random salt.
pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("password hashing failed: {e}")))
}

/// Verify a password against a PHC-formatted Argon2 hash.
pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("stored password hash is invalid: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// A fixed hash used to spend the same verification time when the email does
/// not exist, so login response times don't reveal which emails are registered.
pub fn dummy_hash() -> &'static str {
    static DUMMY: OnceLock<String> = OnceLock::new();
    DUMMY.get_or_init(|| hash_password("not-a-real-password").expect("hashing a constant works"))
}

/// Argon2 is deliberately CPU-heavy; run it off the async executor threads.
pub async fn verify_password_blocking(password: String, hash: String) -> AppResult<bool> {
    tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .map_err(|e| AppError::Internal(format!("password task panicked: {e}")))?
}

pub async fn hash_password_blocking(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|e| AppError::Internal(format!("password task panicked: {e}")))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("Secret@123").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password("Secret@123", &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn same_password_gets_different_salts() {
        let a = hash_password("Secret@123").unwrap();
        let b = hash_password("Secret@123").unwrap();
        assert_ne!(a, b);
    }
}
