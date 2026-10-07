use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams)]
pub struct LatestEmailQuery {
    /// Only return the latest email sent to this address.
    pub email: Option<String>,
}

/// A verification email captured by the development mailer.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DevEmail {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub code: String,
    pub login_challenge_id: Uuid,
    pub sent_at: DateTime<Utc>,
}
