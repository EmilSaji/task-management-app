//! Request / response bodies (the API contract). Kept separate from DB models
//! so the database schema can evolve without changing the public API.

pub mod auth;
pub mod dev;
pub mod task;
pub mod user;
