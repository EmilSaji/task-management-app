use axum::{extract::State, routing::get, routing::post, Json, Router};

use crate::{
    auth::AuthUser,
    dto::{
        auth::{LoginRequest, LoginResponse, TokenResponse, VerifyTwoFactorRequest},
        user::UserDto,
    },
    error::{AppError, AppResult, ErrorBody},
    extract::ValidJson,
    repositories::users,
    services::auth_service,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/verify-2fa", post(verify_two_factor))
        .route("/auth/me", get(me))
}

/// Start login: validates credentials and emails a one-time code. Returns a challenge, not a JWT.
#[utoipa::path(
    post, path = "/auth/login", tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "2FA challenge created and code emailed", body = LoginResponse),
        (status = 401, description = "Invalid email or password", body = ErrorBody),
        (status = 422, description = "Validation error", body = ErrorBody),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    ValidJson(req): ValidJson<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    Ok(Json(auth_service::login(&state, req).await?))
}

/// Complete login with the emailed code. Codes expire after 5 minutes and are single-use.
#[utoipa::path(
    post, path = "/auth/verify-2fa", tag = "auth",
    request_body = VerifyTwoFactorRequest,
    responses(
        (status = 200, description = "Verified; JWT issued", body = TokenResponse),
        (status = 401, description = "Invalid, expired or already-used code", body = ErrorBody),
        (status = 429, description = "Too many incorrect attempts", body = ErrorBody),
    )
)]
pub async fn verify_two_factor(
    State(state): State<AppState>,
    ValidJson(req): ValidJson<VerifyTwoFactorRequest>,
) -> AppResult<Json<TokenResponse>> {
    Ok(Json(auth_service::verify_two_factor(&state, req).await?))
}

/// The currently authenticated user.
#[utoipa::path(
    get, path = "/auth/me", tag = "auth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = UserDto),
        (status = 401, body = ErrorBody),
    )
)]
pub async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<UserDto>> {
    let user = users::find_by_id(&state.db, user.id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User no longer exists".into()))?;
    Ok(Json(user.into()))
}
