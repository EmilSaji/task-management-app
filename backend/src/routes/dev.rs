//! Development-only endpoints. Only mounted when `APP_ENV=development`.

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};

use crate::{
    dto::{
        dev::{DevEmail, LatestEmailQuery},
        user::{SeedQuery, SeedResponse},
    },
    error::{AppError, AppResult, ErrorBody},
    services::seed_service,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/seed/users", post(seed_users))
        .route("/dev/email-logs/latest", get(latest_email))
}

/// Create the Admin and James Bond users. `?reset=true` also clears tasks, challenges and cache.
#[utoipa::path(
    post, path = "/seed/users", tag = "dev",
    params(SeedQuery),
    responses((status = 200, body = SeedResponse))
)]
pub async fn seed_users(
    State(state): State<AppState>,
    Query(query): Query<SeedQuery>,
) -> AppResult<Json<SeedResponse>> {
    Ok(Json(seed_service::seed_users(&state, query.reset).await?))
}

/// Latest verification email captured by the dev mailer (includes the code).
#[utoipa::path(
    get, path = "/dev/email-logs/latest", tag = "dev",
    params(LatestEmailQuery),
    responses(
        (status = 200, body = DevEmail),
        (status = 404, description = "No email sent yet", body = ErrorBody),
    )
)]
pub async fn latest_email(
    State(state): State<AppState>,
    Query(query): Query<LatestEmailQuery>,
) -> AppResult<Json<DevEmail>> {
    state
        .outbox
        .latest(query.email.as_deref())
        .map(Json)
        .ok_or_else(|| AppError::NotFound("No verification email has been sent yet".into()))
}
