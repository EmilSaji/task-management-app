use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// A pending two-factor login. Only the HMAC of the code is stored.
#[derive(Debug, Clone, FromRow)]
pub struct LoginChallenge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code_hash: String,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
}

/// Metadata about an email that was sent (no secrets).
#[derive(Debug, Clone, FromRow)]
pub struct EmailLog {
    pub id: Uuid,
    pub recipient: String,
    pub subject: String,
    pub kind: String,
    pub challenge_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
