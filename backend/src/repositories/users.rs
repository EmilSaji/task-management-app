use sqlx::PgExecutor;
use uuid::Uuid;

use crate::models::{Role, User};

const COLUMNS: &str = "id, full_name, email, hashed_password, role, created_at, updated_at";

pub async fn find_by_email(db: impl PgExecutor<'_>, email: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!("SELECT {COLUMNS} FROM users WHERE email = $1"))
        .bind(email)
        .fetch_optional(db)
        .await
}

pub async fn find_by_id(db: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<Option<User>> {
    sqlx::query_as::<_, User>(&format!("SELECT {COLUMNS} FROM users WHERE id = $1"))
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn list(db: impl PgExecutor<'_>, role: Option<Role>) -> sqlx::Result<Vec<User>> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {COLUMNS} FROM users
         WHERE ($1::user_role IS NULL OR role = $1)
         ORDER BY full_name"
    ))
    .bind(role)
    .fetch_all(db)
    .await
}

/// Insert a user, or update name/password/role if the email already exists.
pub async fn upsert(
    db: impl PgExecutor<'_>,
    full_name: &str,
    email: &str,
    hashed_password: &str,
    role: Role,
) -> sqlx::Result<User> {
    sqlx::query_as::<_, User>(&format!(
        "INSERT INTO users (id, full_name, email, hashed_password, role)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (email) DO UPDATE
            SET full_name = EXCLUDED.full_name,
                hashed_password = EXCLUDED.hashed_password,
                role = EXCLUDED.role
         RETURNING {COLUMNS}"
    ))
    .bind(Uuid::new_v4())
    .bind(full_name)
    .bind(email)
    .bind(hashed_password)
    .bind(role)
    .fetch_one(db)
    .await
}
