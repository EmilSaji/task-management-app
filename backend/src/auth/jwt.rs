use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::Role,
};

/// Claims carried in the access token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: Role,
    pub iat: i64,
    pub exp: i64,
}

/// HS256 signing / verification keys plus the token lifetime.
pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
    ttl: Duration,
}

impl JwtKeys {
    pub fn new(secret: &str, ttl_minutes: i64) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            ttl: Duration::minutes(ttl_minutes),
        }
    }

    pub fn ttl_seconds(&self) -> i64 {
        self.ttl.num_seconds()
    }

    pub fn issue(&self, user_id: Uuid, email: &str, role: Role) -> AppResult<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            role,
            iat: now.timestamp(),
            exp: (now + self.ttl).timestamp(),
        };
        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)
            .map_err(|e| AppError::Internal(format!("failed to sign JWT: {e}")))
    }

    pub fn verify(&self, token: &str) -> AppResult<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        decode::<Claims>(token, &self.decoding, &validation)
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized("Invalid or expired access token".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "unit-test-secret-unit-test-secret-1234";

    #[test]
    fn issue_and_verify_roundtrip() {
        let keys = JwtKeys::new(SECRET, 60);
        let id = Uuid::new_v4();
        let token = keys.issue(id, "a@example.com", Role::Admin).unwrap();
        let claims = keys.verify(&token).unwrap();
        assert_eq!(claims.sub, id);
        assert_eq!(claims.email, "a@example.com");
        assert_eq!(claims.role, Role::Admin);
    }

    #[test]
    fn rejects_token_signed_with_other_secret() {
        let token = JwtKeys::new(SECRET, 60)
            .issue(Uuid::new_v4(), "a@example.com", Role::Staff)
            .unwrap();
        let other = JwtKeys::new("another-secret-another-secret-123456", 60);
        assert!(other.verify(&token).is_err());
    }

    #[test]
    fn rejects_expired_token() {
        let keys = JwtKeys::new(SECRET, -1);
        let token = keys
            .issue(Uuid::new_v4(), "a@example.com", Role::Staff)
            .unwrap();
        assert!(keys.verify(&token).is_err());
    }
}
