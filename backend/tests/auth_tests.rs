mod common;

use axum::http::StatusCode;
use common::*;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn seed_creates_admin_and_james_bond(pool: PgPool) {
    let app = TestApp::new(pool.clone()).await;
    let res = app.post("/seed/users", None, json!({})).await;
    assert_eq!(res.status, StatusCode::OK);

    let users = res.body["users"].as_array().unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0]["email"], ADMIN_EMAIL);
    assert_eq!(users[0]["role"], "admin");
    assert_eq!(users[1]["email"], BOND_EMAIL);
    assert_eq!(users[1]["role"], "staff");
    assert!(users[0].get("hashed_password").is_none());

    // Seeding twice is idempotent.
    app.seed().await;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);

    // Passwords are stored as Argon2 hashes.
    let hash: String = sqlx::query_scalar("SELECT hashed_password FROM users WHERE email = $1")
        .bind(ADMIN_EMAIL)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(hash.starts_with("$argon2"));
}

#[sqlx::test]
async fn login_creates_challenge_without_returning_jwt(pool: PgPool) {
    let app = TestApp::new(pool.clone()).await;
    app.seed().await;

    let res = app
        .post(
            "/auth/login",
            None,
            json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(res.body["login_challenge_id"].is_string());
    assert_eq!(res.body["two_factor_required"], true);
    assert!(res.body.get("access_token").is_none(), "no JWT before 2FA");

    // An email event was recorded, and the code is not stored in plain text.
    let code = app.latest_code(ADMIN_EMAIL).await;
    let stored_hash: String = sqlx::query_scalar("SELECT code_hash FROM login_challenges")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_ne!(stored_hash, code);
    assert!(!stored_hash.contains(&code));
    let logs: i64 = sqlx::query_scalar("SELECT count(*) FROM email_logs WHERE recipient = $1")
        .bind(ADMIN_EMAIL)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(logs, 1);
}

#[sqlx::test]
async fn login_rejects_bad_credentials(pool: PgPool) {
    let app = TestApp::new(pool).await;
    app.seed().await;

    let wrong_password = app
        .post(
            "/auth/login",
            None,
            json!({ "email": ADMIN_EMAIL, "password": "nope" }),
        )
        .await;
    assert_eq!(wrong_password.status, StatusCode::UNAUTHORIZED);

    let unknown = app
        .post(
            "/auth/login",
            None,
            json!({ "email": "nobody@example.com", "password": "nope" }),
        )
        .await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown.body, wrong_password.body, "no user enumeration");

    let invalid = app
        .post(
            "/auth/login",
            None,
            json!({ "email": "not-an-email", "password": "x" }),
        )
        .await;
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn correct_code_returns_jwt(pool: PgPool) {
    let app = TestApp::new(pool).await;
    app.seed().await;

    let challenge = app.start_login(ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = app.latest_code(ADMIN_EMAIL).await;
    let res = app.verify(challenge, &code).await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["token_type"], "Bearer");
    assert_eq!(res.body["user"]["role"], "admin");
    let token = res.body["access_token"].as_str().unwrap();

    let me = app.get("/auth/me", Some(token)).await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.body["email"], ADMIN_EMAIL);
}

#[sqlx::test]
async fn incorrect_code_is_rejected(pool: PgPool) {
    let app = TestApp::new(pool).await;
    app.seed().await;

    let challenge = app.start_login(ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = app.latest_code(ADMIN_EMAIL).await;
    let wrong = if code == "000000" { "111111" } else { "000000" };

    let res = app.verify(challenge, wrong).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"]["code"], "invalid_code");

    // The right code still works after one wrong attempt.
    assert_eq!(app.verify(challenge, &code).await.status, StatusCode::OK);
}

#[sqlx::test]
async fn expired_code_is_rejected(pool: PgPool) {
    let app = TestApp::new(pool.clone()).await;
    app.seed().await;

    let challenge = app.start_login(ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = app.latest_code(ADMIN_EMAIL).await;

    // Simulate the 5-minute window passing.
    sqlx::query(
        "UPDATE login_challenges SET expires_at = now() - interval '1 second' WHERE id = $1",
    )
    .bind(challenge)
    .execute(&pool)
    .await
    .unwrap();

    let res = app.verify(challenge, &code).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"]["code"], "code_expired");
}

#[sqlx::test]
async fn challenge_expires_after_five_minutes(pool: PgPool) {
    let app = TestApp::new(pool.clone()).await;
    app.seed().await;

    let challenge = app.start_login(ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let ttl_seconds: f64 = sqlx::query_scalar(
        "SELECT EXTRACT(EPOCH FROM (expires_at - created_at))::float8 FROM login_challenges WHERE id = $1",
    )
    .bind(challenge)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!((ttl_seconds - 300.0).abs() < 5.0, "ttl was {ttl_seconds}");
}

#[sqlx::test]
async fn reused_code_is_rejected(pool: PgPool) {
    let app = TestApp::new(pool).await;
    app.seed().await;

    let challenge = app.start_login(ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = app.latest_code(ADMIN_EMAIL).await;

    assert_eq!(app.verify(challenge, &code).await.status, StatusCode::OK);
    let reused = app.verify(challenge, &code).await;
    assert_eq!(reused.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reused.body["error"]["code"], "code_already_used");
}

#[sqlx::test]
async fn too_many_wrong_attempts_lock_the_challenge(pool: PgPool) {
    let app = TestApp::new(pool).await;
    app.seed().await;

    let challenge = app.start_login(ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let code = app.latest_code(ADMIN_EMAIL).await;
    let wrong = if code == "000000" { "111111" } else { "000000" };

    for _ in 0..5 {
        assert_eq!(
            app.verify(challenge, wrong).await.status,
            StatusCode::UNAUTHORIZED
        );
    }
    // Even the correct code is refused now.
    let res = app.verify(challenge, &code).await;
    assert_eq!(res.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(res.body["error"]["code"], "too_many_attempts");
}

#[sqlx::test]
async fn new_login_supersedes_previous_challenge(pool: PgPool) {
    let app = TestApp::new(pool).await;
    app.seed().await;

    let first = app.start_login(ADMIN_EMAIL, ADMIN_PASSWORD).await;
    let first_code = app.latest_code(ADMIN_EMAIL).await;
    let _second = app.start_login(ADMIN_EMAIL, ADMIN_PASSWORD).await;

    assert_eq!(
        app.verify(first, &first_code).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test]
async fn unknown_challenge_and_bad_tokens_are_rejected(pool: PgPool) {
    let app = TestApp::new(pool).await;
    app.seed().await;

    let res = app.verify(Uuid::new_v4(), "123456").await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"]["code"], "invalid_challenge");

    assert_eq!(
        app.get("/tasks/view-my-tasks", None).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        app.get("/tasks/view-my-tasks", Some("garbage"))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}
