use axum::{
    body::{self, Body},
    http::{Request, Response},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;
use web_backend::{build_router, create_test_state_sqlite};

pub struct TestUser {
    pub user_id: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
}

pub struct TestAuth {
    pub user_id: String,
    pub token: String,
    pub refresh_token: String,
}

pub struct TestSession {
    pub id: String,
    pub domain: String,
}

pub struct TestTask {
    pub id: String,
    pub stimulus: String,
    pub choices: Vec<String>,
}

pub async fn create_test_app() -> Router {
    let state: Arc<web_backend::state::AppState> = create_test_state_sqlite().await;
    build_router(state)
}

pub async fn create_test_user(app: &Router) -> TestUser {
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let password = "SecurePass123!";
    let display_name = "Test User";

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": &email,
                        "password": password,
                        "display_name": display_name
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_to_json(response).await;

    TestUser {
        user_id: body["user_id"].as_str().unwrap().to_string(),
        email,
        password: password.to_string(),
        display_name: display_name.to_string(),
    }
}

pub async fn create_authenticated_user(app: &Router) -> TestAuth {
    let user = create_test_user(app).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": user.email,
                        "password": user.password
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_to_json(response).await;

    TestAuth {
        user_id: user.user_id,
        token: body["access_token"].as_str().unwrap().to_string(),
        refresh_token: body["refresh_token"].as_str().unwrap().to_string(),
    }
}

pub async fn create_test_session(app: &Router, auth: &TestAuth) -> TestSession {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sessions")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", auth.token))
                .body(Body::from(
                    json!({
                        "domain": "alphabet",
                        "config": {
                            "difficulty": 0.5,
                            "hint_probability": 0.1,
                            "max_trials": 100
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_to_json(response).await;

    TestSession {
        id: body["session_id"].as_str().unwrap().to_string(),
        domain: "alphabet".to_string(),
    }
}

pub async fn get_next_task(app: &Router, auth: &TestAuth, session: &TestSession) -> TestTask {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/sessions/{}/tasks", session.id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_to_json(response).await;

    TestTask {
        id: body["task_id"].as_str().unwrap().to_string(),
        stimulus: body["stimulus"].as_str().unwrap().to_string(),
        choices: body["choices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect(),
    }
}

pub async fn submit_test_trials(
    app: &Router,
    auth: &TestAuth,
    session: &TestSession,
    count: usize,
) {
    for _ in 0..count {
        let task = get_next_task(app, auth, session).await;

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/sessions/{}/tasks/{}/response",
                        session.id, task.id
                    ))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", auth.token))
                    .body(Body::from(
                        json!({
                            "response": task.choices[0],
                            "response_time": 1.234,
                            "client_timestamp": chrono::Utc::now().to_rfc3339()
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
    }
}

pub async fn body_to_json(response: Response<Body>) -> Value {
    let body = body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn cleanup_test_data(_state: &web_backend::state::AppState) {
    // For SQLite tests: database file is ephemeral in CI; add cleanup here if needed
}
