use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    auth::{AdminUser, AuthUser},
    dto::task::{
        AssignTasksRequest, AssignTasksResponse, CreateTaskRequest, MyTasksResponse, TaskResponse,
        UpdateTaskRequest,
    },
    error::{AppResult, ErrorBody},
    extract::ValidJson,
    services::task_service,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tasks", post(create_task).get(list_tasks))
        .route("/tasks/assign", post(assign_tasks))
        .route("/tasks/view-my-tasks", get(view_my_tasks))
        .route("/tasks/{id}", patch(update_task))
}

/// Create a task (admin only).
#[utoipa::path(
    post, path = "/tasks", tag = "tasks",
    security(("bearer_auth" = [])),
    request_body = CreateTaskRequest,
    responses(
        (status = 201, body = TaskResponse),
        (status = 401, body = ErrorBody),
        (status = 403, description = "Caller is not an admin", body = ErrorBody),
        (status = 422, body = ErrorBody),
    )
)]
pub async fn create_task(
    State(state): State<AppState>,
    AdminUser(admin): AdminUser,
    ValidJson(req): ValidJson<CreateTaskRequest>,
) -> AppResult<(StatusCode, Json<TaskResponse>)> {
    let task = task_service::create_task(&state, &admin, req).await?;
    Ok((StatusCode::CREATED, Json(task)))
}

/// List all tasks with their assignees (admin only).
#[utoipa::path(
    get, path = "/tasks", tag = "tasks",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = [TaskResponse]),
        (status = 403, body = ErrorBody),
    )
)]
pub async fn list_tasks(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> AppResult<Json<Vec<TaskResponse>>> {
    Ok(Json(task_service::list_tasks(&state).await?))
}

/// Assign tasks to a user (admin only). Invalidates affected users' task caches.
#[utoipa::path(
    post, path = "/tasks/assign", tag = "tasks",
    security(("bearer_auth" = [])),
    request_body = AssignTasksRequest,
    responses(
        (status = 200, body = AssignTasksResponse),
        (status = 403, body = ErrorBody),
        (status = 404, description = "Unknown task id or assignee", body = ErrorBody),
    )
)]
pub async fn assign_tasks(
    State(state): State<AppState>,
    _admin: AdminUser,
    ValidJson(req): ValidJson<AssignTasksRequest>,
) -> AppResult<Json<AssignTasksResponse>> {
    Ok(Json(task_service::assign_tasks(&state, req).await?))
}

/// Update a task. Admins: any field. Assignee: status only. Invalidates the assignee's cache.
#[utoipa::path(
    patch, path = "/tasks/{id}", tag = "tasks",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Task id")),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, body = TaskResponse),
        (status = 403, body = ErrorBody),
        (status = 404, body = ErrorBody),
    )
)]
pub async fn update_task(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    ValidJson(req): ValidJson<UpdateTaskRequest>,
) -> AppResult<Json<TaskResponse>> {
    Ok(Json(
        task_service::update_task(&state, &user, id, req).await?,
    ))
}

/// Tasks assigned to the logged-in user. Cached per user in Redis (`cache.hit`).
#[utoipa::path(
    get, path = "/tasks/view-my-tasks", tag = "tasks",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = MyTasksResponse),
        (status = 401, body = ErrorBody),
    )
)]
pub async fn view_my_tasks(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<MyTasksResponse>> {
    Ok(Json(task_service::view_my_tasks(&state, &user).await?))
}
