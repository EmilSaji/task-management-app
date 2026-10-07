//! Task management API: email 2FA login, JWT auth, role-based access,
//! task assignment and per-user Redis caching.

pub mod auth;
pub mod cache;
pub mod config;
pub mod dto;
pub mod error;
pub mod extract;
pub mod mail;
pub mod models;
pub mod openapi;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;

use axum::{
    http::{header, HeaderValue, Method},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub use state::AppState;

/// Build the full HTTP application.
pub fn build_app(state: AppState) -> Router {
    let mut api = Router::new()
        .merge(routes::health::router())
        .merge(routes::auth::router())
        .merge(routes::tasks::router())
        .merge(routes::users::router());

    if state.config.is_development() {
        api = api.merge(routes::dev::router());
    }

    let origins: Vec<HeaderValue> = state
        .config
        .cors_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    api.merge(
        SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
    )
    .layer(TraceLayer::new_for_http())
    .layer(cors)
    .with_state(state)
}
