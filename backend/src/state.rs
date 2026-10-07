use std::{sync::Arc, time::Duration};

use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{
    auth::jwt::JwtKeys,
    cache::TaskCache,
    config::Config,
    mail::{ConsoleMailer, DevOutbox, Mailer},
};

/// Shared application state, cloned cheaply into every handler.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cache: TaskCache,
    pub mailer: Arc<dyn Mailer>,
    /// In-memory record of sent emails, readable through the dev-only endpoint.
    pub outbox: Arc<DevOutbox>,
    pub jwt: Arc<JwtKeys>,
    pub config: Arc<Config>,
}

impl AppState {
    /// Assemble state from already-connected resources (used by tests).
    pub fn new(db: PgPool, cache: TaskCache, config: Config) -> Self {
        let outbox = Arc::new(DevOutbox::default());
        let mailer: Arc<dyn Mailer> = Arc::new(ConsoleMailer::new(outbox.clone()));
        let jwt = Arc::new(JwtKeys::new(&config.jwt_secret, config.jwt_ttl_minutes));
        Self {
            db,
            cache,
            mailer,
            outbox,
            jwt,
            config: Arc::new(config),
        }
    }

    /// Connect to Postgres and Redis and run pending migrations.
    pub async fn connect(config: Config) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&config.database_url)
            .await
            .context("failed to connect to Postgres")?;

        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .context("failed to run database migrations")?;

        let cache = TaskCache::connect(
            &config.redis_url,
            config.cache_key_prefix.clone(),
            config.cache_ttl_seconds,
        )
        .await
        .context("failed to connect to Redis")?;

        Ok(Self::new(db, cache, config))
    }
}
