use std::env;

use anyhow::{bail, Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

/// Runtime configuration, loaded from environment variables (see `.env.example`).
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub bind_addr: String,
    pub app_env: AppEnv,
    pub jwt_secret: String,
    pub otp_secret: String,
    pub jwt_ttl_minutes: i64,
    pub otp_ttl_seconds: i64,
    pub otp_max_attempts: i32,
    pub cache_ttl_seconds: u64,
    /// Prepended to every Redis key so several apps / test runs can share one Redis.
    pub cache_key_prefix: String,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let app_env = match env_or("APP_ENV", "development").to_lowercase().as_str() {
            "development" | "dev" | "local" => AppEnv::Development,
            "production" | "prod" => AppEnv::Production,
            other => bail!("APP_ENV must be 'development' or 'production', got '{other}'"),
        };

        let config = Self {
            database_url: required("DATABASE_URL")?,
            redis_url: env_or("REDIS_URL", "redis://localhost:6379"),
            bind_addr: env_or("BIND_ADDR", "127.0.0.1:8080"),
            app_env,
            jwt_secret: required("JWT_SECRET")?,
            otp_secret: required("OTP_SECRET")?,
            jwt_ttl_minutes: parsed("JWT_TTL_MINUTES", 60)?,
            otp_ttl_seconds: parsed("OTP_TTL_SECONDS", 300)?,
            otp_max_attempts: parsed("OTP_MAX_ATTEMPTS", 5)?,
            cache_ttl_seconds: parsed("CACHE_TTL_SECONDS", 300)?,
            cache_key_prefix: env_or("CACHE_KEY_PREFIX", "taskapp:"),
            cors_origins: env_or("CORS_ORIGINS", "http://localhost:5173")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        };

        if config.jwt_secret.len() < 32 {
            bail!("JWT_SECRET must be at least 32 characters");
        }
        if config.otp_secret.len() < 32 {
            bail!("OTP_SECRET must be at least 32 characters");
        }
        Ok(config)
    }

    pub fn is_development(&self) -> bool {
        self.app_env == AppEnv::Development
    }
}

fn required(key: &str) -> anyhow::Result<String> {
    env::var(key).with_context(|| format!("missing required environment variable {key}"))
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn parsed<T: std::str::FromStr>(key: &str, default: T) -> anyhow::Result<T> {
    match env::var(key) {
        Ok(raw) => raw.parse().map_err(|_| {
            anyhow::anyhow!("environment variable {key} has an invalid value '{raw}'")
        }),
        Err(_) => Ok(default),
    }
}
