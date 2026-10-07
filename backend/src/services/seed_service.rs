use crate::{
    auth::password,
    dto::user::{SeedResponse, SeededUser},
    error::AppResult,
    models::Role,
    repositories::{challenges, email_logs, tasks, users},
    state::AppState,
};

pub struct SeedUser {
    pub full_name: &'static str,
    pub email: &'static str,
    pub password: &'static str,
    pub role: Role,
}

pub const ADMIN: SeedUser = SeedUser {
    full_name: "Admin",
    email: "admin@example.com",
    password: "Admin@123",
    role: Role::Admin,
};

pub const JAMES_BOND: SeedUser = SeedUser {
    full_name: "James Bond",
    email: "jamesbond@example.com",
    password: "Bond@007",
    role: Role::Staff,
};

/// Create (or reset the credentials of) the two validation users.
/// With `reset`, also wipe tasks / challenges / email logs and the task cache.
pub async fn seed_users(state: &AppState, reset: bool) -> AppResult<SeedResponse> {
    let mut seeded = Vec::new();
    let mut tx = state.db.begin().await?;

    if reset {
        tasks::delete_all(&mut *tx).await?;
        email_logs::delete_all(&mut *tx).await?;
        challenges::delete_all(&mut *tx).await?;
    }

    for seed in [ADMIN, JAMES_BOND] {
        let hash = password::hash_password_blocking(seed.password.to_string()).await?;
        let user = users::upsert(&mut *tx, seed.full_name, seed.email, &hash, seed.role).await?;
        seeded.push(SeededUser {
            user: user.into(),
            password: seed.password.to_string(),
        });
    }
    tx.commit().await?;

    if reset {
        state.cache.clear_all().await?;
        state.outbox.clear();
    }

    tracing::info!(reset, "seeded validation users");
    Ok(SeedResponse {
        users: seeded,
        reset,
    })
}
