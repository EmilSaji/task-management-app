use std::collections::HashSet;

use uuid::Uuid;

use super::normalize_email;
use crate::{
    auth::AuthUser,
    dto::task::{
        AssignTasksRequest, AssignTasksResponse, CacheInfo, CreateTaskRequest, MyTaskItem,
        MyTasksResponse, MyTasksUser, TaskResponse, TaskSummary, UpdateTaskRequest,
    },
    error::{AppError, AppResult},
    repositories::{
        tasks::{self, NewTask, TaskChanges},
        users,
    },
    state::AppState,
};

pub async fn create_task(
    state: &AppState,
    admin: &AuthUser,
    req: CreateTaskRequest,
) -> AppResult<TaskResponse> {
    let title = req.title.trim();
    if title.is_empty() {
        return Err(AppError::Validation("title: must not be blank".into()));
    }
    let description = req.description.as_deref().unwrap_or("").trim();

    let id = tasks::insert(
        &state.db,
        NewTask {
            title,
            description,
            status: req.status.unwrap_or_default(),
            priority: req.priority.unwrap_or_default(),
            created_by_id: admin.id,
        },
    )
    .await?;

    tracing::info!(task_id = %id, created_by = %admin.id, "task created");
    load_task(state, id).await
}

pub async fn list_tasks(state: &AppState) -> AppResult<Vec<TaskResponse>> {
    Ok(tasks::list_with_users(&state.db)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

/// Assign tasks to a user in one transaction, then invalidate the cache of
/// the new assignee and of anyone who previously held one of the tasks.
pub async fn assign_tasks(
    state: &AppState,
    req: AssignTasksRequest,
) -> AppResult<AssignTasksResponse> {
    let email = normalize_email(&req.assignee_email);
    let assignee = users::find_by_email(&state.db, &email)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No user with email {email}")))?;

    let mut seen = HashSet::new();
    let task_ids: Vec<Uuid> = req
        .task_ids
        .into_iter()
        .filter(|id| seen.insert(*id))
        .collect();

    let mut tx = state.db.begin().await?;
    let existing = tasks::lock_by_ids(&mut tx, &task_ids).await?;
    if existing.len() != task_ids.len() {
        let found: HashSet<Uuid> = existing.iter().map(|t| t.id).collect();
        let missing: Vec<String> = task_ids
            .iter()
            .filter(|id| !found.contains(id))
            .map(Uuid::to_string)
            .collect();
        return Err(AppError::NotFound(format!(
            "Tasks not found: {}",
            missing.join(", ")
        )));
    }
    tasks::assign(&mut tx, &task_ids, assignee.id).await?;
    tx.commit().await?;

    let mut affected: Vec<Uuid> = existing.iter().filter_map(|t| t.assigned_to_id).collect();
    affected.push(assignee.id);
    affected.sort();
    affected.dedup();
    invalidate(state, &affected).await;

    tracing::info!(assignee = %assignee.id, count = task_ids.len(), "tasks assigned");
    Ok(AssignTasksResponse {
        assigned_to: assignee.email,
        assigned_count: task_ids.len(),
        task_ids,
    })
}

/// Admins can change any field. The assignee may only change `status`.
pub async fn update_task(
    state: &AppState,
    user: &AuthUser,
    task_id: Uuid,
    req: UpdateTaskRequest,
) -> AppResult<TaskResponse> {
    if req.is_empty() {
        return Err(AppError::Validation(
            "at least one field must be provided".into(),
        ));
    }

    let task = tasks::find(&state.db, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;

    if !user.is_admin() {
        if task.assigned_to_id != Some(user.id) {
            return Err(AppError::NotFound("Task not found".into()));
        }
        if !req.only_status() {
            return Err(AppError::Forbidden(
                "Staff can only update the status of their own tasks".into(),
            ));
        }
    }

    let title = req.title.as_deref().map(str::trim);
    if title == Some("") {
        return Err(AppError::Validation("title: must not be blank".into()));
    }

    tasks::update(
        &state.db,
        task_id,
        TaskChanges {
            title,
            description: req.description.as_deref().map(str::trim),
            status: req.status,
            priority: req.priority,
        },
    )
    .await?;

    if let Some(assignee) = task.assigned_to_id {
        invalidate(state, &[assignee]).await;
    }
    load_task(state, task_id).await
}

/// Tasks assigned to the caller, served from Redis when possible.
pub async fn view_my_tasks(state: &AppState, user: &AuthUser) -> AppResult<MyTasksResponse> {
    match state.cache.get_my_tasks(user.id).await {
        Ok(Some(mut cached)) => {
            cached.cache = CacheInfo { hit: true };
            return Ok(cached);
        }
        Ok(None) => {}
        // Redis being down should degrade to slower responses, not errors.
        Err(e) => tracing::warn!(error = %e, "cache read failed, falling back to database"),
    }

    let db_user = users::find_by_id(&state.db, user.id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User no longer exists".into()))?;
    let rows = tasks::list_assigned_to(&state.db, user.id).await?;

    let tasks: Vec<MyTaskItem> = rows
        .into_iter()
        .map(|row| MyTaskItem {
            id: row.task.id,
            title: row.task.title,
            status: row.task.status,
            priority: row.task.priority,
            assigned_to: row.assigned_to_email.unwrap_or_default(),
        })
        .collect();

    let response = MyTasksResponse {
        user: MyTasksUser {
            email: db_user.email,
            role: db_user.role,
        },
        summary: TaskSummary {
            total_assigned_tasks: tasks.len(),
        },
        tasks,
        cache: CacheInfo { hit: false },
    };

    if let Err(e) = state.cache.set_my_tasks(user.id, &response).await {
        tracing::warn!(error = %e, "cache write failed");
    }
    Ok(response)
}

async fn load_task(state: &AppState, id: Uuid) -> AppResult<TaskResponse> {
    tasks::find_with_users(&state.db, id)
        .await?
        .map(Into::into)
        .ok_or_else(|| AppError::NotFound("Task not found".into()))
}

/// Invalidation runs after the DB commit. If Redis is unreachable we log
/// loudly; the entry's TTL bounds how long a stale list can be served.
async fn invalidate(state: &AppState, user_ids: &[Uuid]) {
    if let Err(e) = state.cache.invalidate(user_ids).await {
        tracing::error!(error = %e, ?user_ids, "failed to invalidate task cache");
    }
}
