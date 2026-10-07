use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::LoginChallenge;

pub async fn insert(
    db: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
    code_hash: &str,
    expires_at: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO login_challenges (id, user_id, code_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(user_id)
    .bind(code_hash)
    .bind(expires_at)
    .execute(db)
    .await
    .map(|_| ())
}

pub async fn find(db: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<Option<LoginChallenge>> {
    sqlx::query_as(
        "SELECT id, user_id, code_hash, expires_at, consumed_at, attempts, created_at
         FROM login_challenges WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn increment_attempts(db: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE login_challenges SET attempts = attempts + 1 WHERE id = $1")
        .bind(id)
        .execute(db)
        .await
        .map(|_| ())
}

/// Atomically mark a challenge as used. Returns `false` if it was already
/// consumed or has expired, so two concurrent verifications can't both succeed.
pub async fn consume(db: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<bool> {
    sqlx::query(
        "UPDATE login_challenges SET consumed_at = now()
         WHERE id = $1 AND consumed_at IS NULL AND expires_at > now()",
    )
    .bind(id)
    .execute(db)
    .await
    .map(|r| r.rows_affected() == 1)
}

/// Invalidate any still-pending challenges for a user (a new login supersedes them).
pub async fn revoke_pending_for_user(db: impl PgExecutor<'_>, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE login_challenges SET consumed_at = now()
         WHERE user_id = $1 AND consumed_at IS NULL",
    )
    .bind(user_id)
    .execute(db)
    .await
    .map(|_| ())
}

pub async fn delete_all(db: impl PgExecutor<'_>) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM login_challenges")
        .execute(db)
        .await
        .map(|_| ())
}
