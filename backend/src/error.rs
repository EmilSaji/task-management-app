use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

/// Reasons a 2FA verification can fail. Each maps to a stable error code
/// so clients (and tests) can tell them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwoFactorError {
    UnknownChallenge,
    InvalidCode,
    Expired,
    AlreadyUsed,
    TooManyAttempts,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("two-factor verification failed: {0:?}")]
    TwoFactor(TwoFactorError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Cache(#[from] redis::RedisError),
    #[error("{0}")]
    Internal(String),
}

/// JSON error envelope: `{"error": {"code": "...", "message": "..."}}`
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

impl AppError {
    fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m.clone()),
            Self::Validation(m) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error",
                m.clone(),
            ),
            Self::Unauthorized(m) => (StatusCode::UNAUTHORIZED, "unauthorized", m.clone()),
            Self::Forbidden(m) => (StatusCode::FORBIDDEN, "forbidden", m.clone()),
            Self::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m.clone()),
            Self::Conflict(m) => (StatusCode::CONFLICT, "conflict", m.clone()),
            Self::TwoFactor(reason) => {
                let (status, code, message) = match reason {
                    TwoFactorError::UnknownChallenge => (
                        StatusCode::UNAUTHORIZED,
                        "invalid_challenge",
                        "Login challenge not found. Please log in again.",
                    ),
                    TwoFactorError::InvalidCode => (
                        StatusCode::UNAUTHORIZED,
                        "invalid_code",
                        "The verification code is incorrect.",
                    ),
                    TwoFactorError::Expired => (
                        StatusCode::UNAUTHORIZED,
                        "code_expired",
                        "The verification code has expired. Please log in again.",
                    ),
                    TwoFactorError::AlreadyUsed => (
                        StatusCode::UNAUTHORIZED,
                        "code_already_used",
                        "The verification code has already been used. Please log in again.",
                    ),
                    TwoFactorError::TooManyAttempts => (
                        StatusCode::TOO_MANY_REQUESTS,
                        "too_many_attempts",
                        "Too many incorrect attempts. Please log in again.",
                    ),
                };
                (status, code, message.to_string())
            }
            // Never leak internals to the client; details are logged instead.
            Self::Database(_) | Self::Cache(_) | Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "An unexpected error occurred.".to_string(),
            ),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();
        if status.is_server_error() {
            tracing::error!(error = ?self, "request failed");
        } else {
            tracing::debug!(%status, code, %message, "request rejected");
        }
        let body = ErrorBody {
            error: ErrorDetail {
                code: code.to_string(),
                message,
            },
        };
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
