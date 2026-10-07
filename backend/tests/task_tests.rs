mod common;

use axum::http::StatusCode;
use common::*;
use serde_json::{json, Value};
use sqlx::PgPool;

/// Create the 5 validation tasks and return their ids.
async fn create_five_tasks(app: &TestApp, admin: &str) -> Vec<String> {
    let specs = [
        ("Infiltrate SPECTRE meeting", "high"),
        ("Collect gadgets from Q", "medium"),
        ("Brief M on findings", "low"),
        ("Audit MI6 budget", "medium"),
        ("Renew Aston Martin insurance", "low"),
    ];
    let mut ids = Vec::new();
    for (title, priority) in specs {
        let res = app.create_task(admin, title, priority).await;
        assert_eq!(res.status, StatusCode::CREATED, "{:?}", res.body);
        assert_eq!(res.body["status"], "todo");
        assert_eq!(res.body["priority"], priority);
        ids.push(res.body["id"].as_str().unwrap().to_string());
    }
    ids
}

async fn assign(app: &TestApp, admin: &str, ids: &[String], email: &str) -> (StatusCode, Value) {
    let res = app
        .post(
            "/tasks/assign",
            Some(admin),
            json!({ "task_ids": ids, "assignee_email": email }),
        )
        .await;
    (res.status, res.body)
}

#[sqlx::test]
async fn full_validation_workflow(pool: PgPool) {
    let app = TestApp::new(pool.clone()).await;
    let (admin, bond) = app.seeded_tokens().await;

    // Admin creates exactly 5 tasks.
    let ids = create_five_tasks(&app, &admin).await;
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM tasks")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(total, 5);

    // Admin assigns exactly 3 to James Bond.
    let (status, body) = assign(&app, &admin, &ids[..3], BOND_EMAIL).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    assert_eq!(body["assigned_count"], 3);

    // James Bond cannot create a task.
    let forbidden = app.create_task(&bond, "Go rogue", "high").await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);
    assert_eq!(forbidden.body["error"]["code"], "forbidden");

    // First view: from the database.
    let first = app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assert_eq!(first.status, StatusCode::OK);
    assert_eq!(first.body["user"]["email"], BOND_EMAIL);
    assert_eq!(first.body["user"]["role"], "staff");
    assert_eq!(first.body["summary"]["total_assigned_tasks"], 3);
    assert_eq!(first.body["cache"]["hit"], false);
    let tasks = first.body["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 3);
    let priorities: Vec<&str> = tasks
        .iter()
        .map(|t| t["priority"].as_str().unwrap())
        .collect();
    assert_eq!(priorities, ["high", "medium", "low"]);
    for task in tasks {
        assert_eq!(task["assigned_to"], BOND_EMAIL);
        assert_eq!(task["status"], "todo");
        assert!(ids[..3].contains(&task["id"].as_str().unwrap().to_string()));
    }

    // Second view: from cache, same data.
    let second = app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assert_eq!(second.status, StatusCode::OK);
    assert_eq!(second.body["cache"]["hit"], true);
    assert_eq!(second.body["tasks"], first.body["tasks"]);
}

#[sqlx::test]
async fn staff_cannot_assign_or_list_all_tasks(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let (admin, bond) = app.seeded_tokens().await;
    let ids = create_five_tasks(&app, &admin).await;

    let (status, _) = assign(&app, &bond, &ids[..1], BOND_EMAIL).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(
        app.get("/tasks", Some(&bond)).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.get("/users", Some(&bond)).await.status,
        StatusCode::FORBIDDEN
    );
}

#[sqlx::test]
async fn admin_sees_only_own_assigned_tasks_in_view_my_tasks(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let (admin, bond) = app.seeded_tokens().await;
    let ids = create_five_tasks(&app, &admin).await;
    assign(&app, &admin, &ids[..3], BOND_EMAIL).await;

    let admin_view = app.get("/tasks/view-my-tasks", Some(&admin)).await;
    assert_eq!(admin_view.body["summary"]["total_assigned_tasks"], 0);

    let bond_view = app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assert_eq!(bond_view.body["summary"]["total_assigned_tasks"], 3);
}

#[sqlx::test]
async fn assignment_invalidates_cache(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let (admin, bond) = app.seeded_tokens().await;
    let ids = create_five_tasks(&app, &admin).await;
    assign(&app, &admin, &ids[..3], BOND_EMAIL).await;

    app.get("/tasks/view-my-tasks", Some(&bond)).await;
    let cached = app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assert_eq!(cached.body["cache"]["hit"], true);

    // Assigning another task must drop Bond's cached list.
    assign(&app, &admin, &ids[3..4], BOND_EMAIL).await;
    let fresh = app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assert_eq!(fresh.body["cache"]["hit"], false);
    assert_eq!(fresh.body["summary"]["total_assigned_tasks"], 4);

    // Re-assigning a task away from Bond also invalidates his cache.
    app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assign(&app, &admin, &ids[3..4], ADMIN_EMAIL).await;
    let after = app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assert_eq!(after.body["cache"]["hit"], false);
    assert_eq!(after.body["summary"]["total_assigned_tasks"], 3);
}

#[sqlx::test]
async fn task_update_invalidates_cache(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let (admin, bond) = app.seeded_tokens().await;
    let ids = create_five_tasks(&app, &admin).await;
    assign(&app, &admin, &ids[..3], BOND_EMAIL).await;

    app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assert_eq!(
        app.get("/tasks/view-my-tasks", Some(&bond)).await.body["cache"]["hit"],
        true
    );

    // Admin updates a task: cache dropped and new data visible.
    let res = app
        .patch(
            &format!("/tasks/{}", ids[0]),
            Some(&admin),
            json!({ "title": "Infiltrate SPECTRE (rescheduled)" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{:?}", res.body);
    let view = app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assert_eq!(view.body["cache"]["hit"], false);
    assert_eq!(
        view.body["tasks"][0]["title"],
        "Infiltrate SPECTRE (rescheduled)"
    );

    // Assignee updates status of his own task: allowed, and cache dropped.
    let res = app
        .patch(
            &format!("/tasks/{}", ids[0]),
            Some(&bond),
            json!({ "status": "in_progress" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    let view = app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assert_eq!(view.body["cache"]["hit"], false);
    assert_eq!(view.body["tasks"][0]["status"], "in_progress");
}

#[sqlx::test]
async fn staff_update_rules(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let (admin, bond) = app.seeded_tokens().await;
    let ids = create_five_tasks(&app, &admin).await;
    assign(&app, &admin, &ids[..1], BOND_EMAIL).await;

    // Can't change anything but status.
    let res = app
        .patch(
            &format!("/tasks/{}", ids[0]),
            Some(&bond),
            json!({ "title": "mine now" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);

    // Can't touch tasks not assigned to him.
    let res = app
        .patch(
            &format!("/tasks/{}", ids[1]),
            Some(&bond),
            json!({ "status": "done" }),
        )
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn assign_validates_input(pool: PgPool) {
    let app = TestApp::new(pool.clone()).await;
    let (admin, _) = app.seeded_tokens().await;
    let ids = create_five_tasks(&app, &admin).await;

    // Unknown task id: nothing is assigned (transactional).
    let unknown = uuid::Uuid::new_v4().to_string();
    let (status, _) = assign(&app, &admin, &[ids[0].clone(), unknown], BOND_EMAIL).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let assigned: i64 =
        sqlx::query_scalar("SELECT count(*) FROM tasks WHERE assigned_to_id IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(assigned, 0);

    // Unknown assignee.
    let (status, _) = assign(&app, &admin, &ids[..1], "nobody@example.com").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Empty list.
    let (status, _) = assign(&app, &admin, &[], BOND_EMAIL).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn create_task_validates_input(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let (admin, _) = app.seeded_tokens().await;

    let blank = app.create_task(&admin, "   ", "high").await;
    assert_eq!(blank.status, StatusCode::UNPROCESSABLE_ENTITY);

    let bad_priority = app.create_task(&admin, "x", "urgent").await;
    assert_eq!(bad_priority.status, StatusCode::BAD_REQUEST);

    let malformed = app
        .request(axum::http::Method::POST, "/tasks", Some(&admin), None)
        .await;
    assert!(malformed.status.is_client_error());
}

#[sqlx::test]
async fn seed_reset_clears_tasks_and_cache(pool: PgPool) {
    let app = TestApp::new(pool.clone()).await;
    let (admin, bond) = app.seeded_tokens().await;
    let ids = create_five_tasks(&app, &admin).await;
    assign(&app, &admin, &ids[..3], BOND_EMAIL).await;
    app.get("/tasks/view-my-tasks", Some(&bond)).await;

    let res = app.post("/seed/users?reset=true", None, json!({})).await;
    assert_eq!(res.status, StatusCode::OK);

    let tasks: i64 = sqlx::query_scalar("SELECT count(*) FROM tasks")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(tasks, 0);
    let view = app.get("/tasks/view-my-tasks", Some(&bond)).await;
    assert_eq!(view.body["cache"]["hit"], false);
    assert_eq!(view.body["summary"]["total_assigned_tasks"], 0);
}
