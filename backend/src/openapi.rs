use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::routes;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Task Management API",
        description = "Rust/Axum API with email 2FA, JWT auth, role-based access and Redis caching."
    ),
    paths(
        routes::health::health,
        routes::auth::login,
        routes::auth::verify_two_factor,
        routes::auth::me,
        routes::tasks::create_task,
        routes::tasks::list_tasks,
        routes::tasks::assign_tasks,
        routes::tasks::update_task,
        routes::tasks::view_my_tasks,
        routes::users::list_users,
        routes::dev::seed_users,
        routes::dev::latest_email,
    ),
    modifiers(&BearerAuth),
    tags(
        (name = "auth", description = "Login with email 2FA"),
        (name = "tasks", description = "Task management"),
        (name = "users", description = "Users"),
        (name = "dev", description = "Development-only helpers"),
    )
)]
pub struct ApiDoc;

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}
