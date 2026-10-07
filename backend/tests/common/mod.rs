//! Shared helpers for integration tests.
//!
//! Each test gets a fresh Postgres database from `#[sqlx::test]` (migrations
//! applied) and a unique Redis key prefix, so tests can run in parallel
//! against the docker-compose services.

#![allow(dead_code)]

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use task_api::{
    build_app,
    cache::TaskCache,
    config::{AppEnv, Config},
    AppState,
};
use tower::ServiceExt;
use uuid::Uuid;

pub const ADMIN_EMAIL: &str = "admin@example.com";
pub const ADMIN_PASSWORD: &str = "Admin@123";
pub const BOND_EMAIL: &str = "jamesbond@example.com";
pub const BOND_PASSWORD: &str = "Bond@007";

pub struct TestApp {
    pub router: Router,
    pub state: AppState,
}

pub struct TestResponse {
    pub status: StatusCode,
    pub body: Value,
}

impl TestApp {
    pub async fn new(pool: PgPool) -> Self {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
        let prefix = format!("test:{}:", Uuid::new_v4());
        let config = Config {
            database_url: String::new(),
            redis_url: redis_url.clone(),
            bind_addr: "127.0.0.1:0".into(),
            app_env: AppEnv::Development,
            jwt_secret: "integration-test-jwt-secret-0123456789".into(),
            otp_secret: "integration-test-otp-secret-0123456789".into(),
            jwt_ttl_minutes: 60,
            otp_ttl_seconds: 300,
            otp_max_attempts: 5,
            cache_ttl_seconds: 60,
            cache_key_prefix: prefix.clone(),
            cors_origins: vec!["http://localhost:5173".into()],
        };
        let cache = TaskCache::connect(&redis_url, prefix, 60)
            .await
            .expect("Redis must be running (docker compose up -d)");
        let state = AppState::new(pool, cache, config);
        Self {
            router: build_app(state.clone()),
            state,
        }
    }

    pub async fn request(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> TestResponse {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let request = match body {
            Some(json) => builder
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json.to_string())),
            None => builder.body(Body::empty()),
        }
        .unwrap();

        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        };
        TestResponse { status, body }
    }

    pub async fn get(&self, uri: &str, token: Option<&str>) -> TestResponse {
        self.request(Method::GET, uri, token, None).await
    }

    pub async fn post(&self, uri: &str, token: Option<&str>, body: Value) -> TestResponse {
        self.request(Method::POST, uri, token, Some(body)).await
    }

    pub async fn patch(&self, uri: &str, token: Option<&str>, body: Value) -> TestResponse {
        self.request(Method::PATCH, uri, token, Some(body)).await
    }

    pub async fn seed(&self) {
        let res = self.post("/seed/users", None, json!({})).await;
        assert_eq!(res.status, StatusCode::OK, "seed failed: {:?}", res.body);
    }

    /// Step 1 of login. Returns the challenge id.
    pub async fn start_login(&self, email: &str, password: &str) -> Uuid {
        let res = self
            .post(
                "/auth/login",
                None,
                json!({ "email": email, "password": password }),
            )
            .await;
        assert_eq!(res.status, StatusCode::OK, "login failed: {:?}", res.body);
        res.body["login_challenge_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap()
    }

    /// Read the code the dev mailer "sent" to `email`.
    pub async fn latest_code(&self, email: &str) -> String {
        let res = self
            .get(&format!("/dev/email-logs/latest?email={email}"), None)
            .await;
        assert_eq!(res.status, StatusCode::OK, "no email: {:?}", res.body);
        res.body["code"].as_str().unwrap().to_string()
    }

    pub async fn verify(&self, challenge_id: Uuid, code: &str) -> TestResponse {
        self.post(
            "/auth/verify-2fa",
            None,
            json!({ "login_challenge_id": challenge_id, "code": code }),
        )
        .await
    }

    /// Full login (password + 2FA). Returns the JWT.
    pub async fn login(&self, email: &str, password: &str) -> String {
        let challenge = self.start_login(email, password).await;
        let code = self.latest_code(email).await;
        let res = self.verify(challenge, &code).await;
        assert_eq!(res.status, StatusCode::OK, "verify failed: {:?}", res.body);
        res.body["access_token"].as_str().unwrap().to_string()
    }

    pub async fn create_task(&self, token: &str, title: &str, priority: &str) -> TestResponse {
        self.post(
            "/tasks",
            Some(token),
            json!({ "title": title, "description": "", "priority": priority }),
        )
        .await
    }

    /// Seed, log both users in, and return (admin_token, bond_token).
    pub async fn seeded_tokens(&self) -> (String, String) {
        self.seed().await;
        let admin = self.login(ADMIN_EMAIL, ADMIN_PASSWORD).await;
        let bond = self.login(BOND_EMAIL, BOND_PASSWORD).await;
        (admin, bond)
    }
}
