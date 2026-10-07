use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

#[utoipa::path(get, path = "/health", tag = "health", responses((status = 200)))]
pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
