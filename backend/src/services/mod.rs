//! Business logic. Handlers stay thin: they extract input, call a service,
//! and serialize the result.

pub mod auth_service;
pub mod seed_service;
pub mod task_service;

pub(crate) fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}
