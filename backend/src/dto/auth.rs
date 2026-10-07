use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::user::UserDto;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "must be a valid email address"))]
    #[schema(example = "admin@example.com")]
    pub email: String,
    #[validate(length(min = 1, max = 256, message = "is required"))]
    #[schema(example = "Admin@123")]
    pub password: String,
}

/// Step 1 of login: no token yet, only a challenge to complete with the emailed code.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub login_challenge_id: Uuid,
    pub two_factor_required: bool,
    pub expires_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct VerifyTwoFactorRequest {
    pub login_challenge_id: Uuid,
    #[validate(custom(function = "validate_code"))]
    #[schema(example = "123456")]
    pub code: String,
}

fn validate_code(code: &str) -> Result<(), validator::ValidationError> {
    let code = code.trim();
    if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("code").with_message("must be a 6-digit code".into()))
    }
}

/// Step 2 of login: issued only after a successful 2FA verification.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    /// Lifetime of the access token in seconds.
    pub expires_in: i64,
    pub user: UserDto,
}
