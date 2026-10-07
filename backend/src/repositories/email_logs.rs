use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::EmailLog;

pub async fn insert(
    db: impl PgExecutor<'_>,
    recipient: &str,
    subject: &str,
    kind: &str,
    challenge_id: Option<Uuid>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO email_logs (id, recipient, subject, kind, challenge_id)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(recipient)
    .bind(subject)
    .bind(kind)
    .bind(challenge_id)
    .execute(db)
    .await
    .map(|_| ())
}

pub async fn list_for_recipient(
    db: impl PgExecutor<'_>,
    recipient: &str,
) -> sqlx::Result<Vec<EmailLog>> {
    sqlx::query_as(
        "SELECT id, recipient, subject, kind, challenge_id, created_at
         FROM email_logs WHERE recipient = $1 ORDER BY created_at DESC",
    )
    .bind(recipient)
    .fetch_all(db)
    .await
}

pub async fn delete_all(db: impl PgExecutor<'_>) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM email_logs")
        .execute(db)
        .await
        .map(|_| ())
}
