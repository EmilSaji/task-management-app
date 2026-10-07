use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};

use crate::{
    auth::AdminUser,
    dto::user::{ListUsersQuery, UserDto},
    error::{AppResult, ErrorBody},
    repositories::users,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/users", get(list_users))
}

/// List users, optionally filtered by role (admin only; feeds the assignment UI).
#[utoipa::path(
    get, path = "/users", tag = "users",
    security(("bearer_auth" = [])),
    params(ListUsersQuery),
    responses(
        (status = 200, body = [UserDto]),
        (status = 403, body = ErrorBody),
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(query): Query<ListUsersQuery>,
) -> AppResult<Json<Vec<UserDto>>> {
    let users = users::list(&state.db, query.role).await?;
    Ok(Json(users.into_iter().map(Into::into).collect()))
}
