use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::models::{Role, User};

/// Public view of a user (no password hash).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            full_name: u.full_name,
            email: u.email,
            role: u.role,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListUsersQuery {
    /// Filter by role (`admin` or `staff`).
    pub role: Option<Role>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct SeedQuery {
    /// When true, also deletes all tasks, login challenges and email logs and
    /// clears the task cache, so the validation flow can be repeated from scratch.
    #[serde(default)]
    pub reset: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeededUser {
    #[serde(flatten)]
    pub user: UserDto,
    /// Development credentials, returned for convenience by the dev-only seed endpoint.
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeedResponse {
    pub users: Vec<SeededUser>,
    pub reset: bool,
}
