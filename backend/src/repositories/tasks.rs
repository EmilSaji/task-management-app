use sqlx::{PgConnection, PgExecutor};
use uuid::Uuid;

use crate::models::{Task, TaskPriority, TaskStatus, TaskWithUsers};

const SELECT_WITH_USERS: &str = "
    SELECT t.id, t.title, t.description, t.status, t.priority,
           t.created_by_id, t.assigned_to_id, t.created_at, t.updated_at,
           c.email AS created_by_email,
           a.email AS assigned_to_email
    FROM tasks t
    JOIN users c ON c.id = t.created_by_id
    LEFT JOIN users a ON a.id = t.assigned_to_id";

pub struct NewTask<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_by_id: Uuid,
}

#[derive(Default)]
pub struct TaskChanges<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
}

pub async fn insert(db: impl PgExecutor<'_>, new: NewTask<'_>) -> sqlx::Result<Uuid> {
    sqlx::query_scalar(
        "INSERT INTO tasks (id, title, description, status, priority, created_by_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(new.title)
    .bind(new.description)
    .bind(new.status)
    .bind(new.priority)
    .bind(new.created_by_id)
    .fetch_one(db)
    .await
}

pub async fn find_with_users(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> sqlx::Result<Option<TaskWithUsers>> {
    sqlx::query_as(&format!("{SELECT_WITH_USERS} WHERE t.id = $1"))
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn list_with_users(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<TaskWithUsers>> {
    sqlx::query_as(&format!("{SELECT_WITH_USERS} ORDER BY t.created_at, t.id"))
        .fetch_all(db)
        .await
}

/// Tasks assigned to a user, highest priority first.
pub async fn list_assigned_to(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> sqlx::Result<Vec<TaskWithUsers>> {
    sqlx::query_as(&format!(
        "{SELECT_WITH_USERS} WHERE t.assigned_to_id = $1
         ORDER BY t.priority DESC, t.created_at, t.id"
    ))
    .bind(user_id)
    .fetch_all(db)
    .await
}

/// Lock the given tasks for the rest of the transaction and return their current state.
pub async fn lock_by_ids(conn: &mut PgConnection, ids: &[Uuid]) -> sqlx::Result<Vec<Task>> {
    sqlx::query_as(
        "SELECT id, title, description, status, priority, created_by_id, assigned_to_id,
                created_at, updated_at
         FROM tasks WHERE id = ANY($1) FOR UPDATE",
    )
    .bind(ids)
    .fetch_all(conn)
    .await
}

pub async fn assign(conn: &mut PgConnection, ids: &[Uuid], user_id: Uuid) -> sqlx::Result<u64> {
    sqlx::query("UPDATE tasks SET assigned_to_id = $1 WHERE id = ANY($2)")
        .bind(user_id)
        .bind(ids)
        .execute(conn)
        .await
        .map(|r| r.rows_affected())
}

/// Apply a partial update; `None` fields keep their current value.
pub async fn update(
    db: impl PgExecutor<'_>,
    id: Uuid,
    changes: TaskChanges<'_>,
) -> sqlx::Result<bool> {
    sqlx::query(
        "UPDATE tasks SET
            title       = COALESCE($2, title),
            description = COALESCE($3, description),
            status      = COALESCE($4, status),
            priority    = COALESCE($5, priority)
         WHERE id = $1",
    )
    .bind(id)
    .bind(changes.title)
    .bind(changes.description)
    .bind(changes.status)
    .bind(changes.priority)
    .execute(db)
    .await
    .map(|r| r.rows_affected() == 1)
}

pub async fn find(db: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<Option<Task>> {
    sqlx::query_as(
        "SELECT id, title, description, status, priority, created_by_id, assigned_to_id,
                created_at, updated_at
         FROM tasks WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn delete_all(db: impl PgExecutor<'_>) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM tasks")
        .execute(db)
        .await
        .map(|_| ())
}
