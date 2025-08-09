use axum::http::StatusCode;
use axum::{body::Body, http::Request};
use serde_json::json;
use tower::ServiceExt;

mod common;
use common::*;

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_json(response).await;
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_user_registration() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "test@example.com",
                        "password": "SecurePass123!",
                        "display_name": "Test User"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_to_json(response).await;
    assert_eq!(body["email"], "test@example.com");
    assert_eq!(body["display_name"], "Test User");
    assert!(body["user_id"].is_string());
}

#[tokio::test]
async fn test_user_login() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;

    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_json(response).await;
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert_eq!(body["token_type"], "Bearer");
    assert!(body["expires_in"].is_number());
}

#[tokio::test]
async fn test_create_session() {
    let app = create_test_app().await;
    let auth = create_authenticated_user(&app).await;

    let response = app
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

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_to_json(response).await;
    assert!(body["session_id"].is_string());
    assert_eq!(body["domain"], "alphabet");
    assert_eq!(body["status"], "active");
}

#[tokio::test]
async fn test_get_next_task() {
    let app = create_test_app().await;
    let auth = create_authenticated_user(&app).await;
    let session = create_test_session(&app, &auth).await;

    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_json(response).await;
    assert!(body["task_id"].is_string());
    assert!(body["stimulus"].is_string());
    assert!(body["choices"].is_array());
}

#[tokio::test]
async fn test_submit_task_response() {
    let app = create_test_app().await;
    let auth = create_authenticated_user(&app).await;
    let session = create_test_session(&app, &auth).await;
    let task = get_next_task(&app, &auth, &session).await;

    let response = app
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
                        "response": "B",
                        "response_time": 1.234,
                        "client_timestamp": "2024-01-01T00:00:00Z"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_json(response).await;
    assert!(body["correct"].is_boolean());
    assert!(body["expected"].is_string());
    assert!(body["next_task_available"].is_boolean());
}

#[tokio::test]
async fn test_batch_trial_submission() {
    let app = create_test_app().await;
    let auth = create_authenticated_user(&app).await;
    let session = create_test_session(&app, &auth).await;

    let trials = vec![
        json!({
            "task_id": "task1",
            "stimulus": "A",
            "response": "B",
            "response_time": 1.234,
            "correct": true,
            "client_timestamp": "2024-01-01T00:00:00Z"
        }),
        json!({
            "task_id": "task2",
            "stimulus": "C",
            "response": "D",
            "response_time": 1.456,
            "correct": false,
            "client_timestamp": "2024-01-01T00:00:10Z"
        }),
    ];

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/batch/sessions/{}/trials", session.id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", auth.token))
                .body(Body::from(
                    json!({
                        "trials": trials,
                        "client_info": {
                            "app_version": "1.0.0",
                            "platform": "web",
                            "offline_duration": 3600
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_json(response).await;
    assert_eq!(body["accepted"], 2);
    assert!(body["session_metrics"].is_object());
}

#[tokio::test]
async fn test_get_user_analytics() {
    let app = create_test_app().await;
    let auth = create_authenticated_user(&app).await;

    // Create some sessions with data
    for _ in 0..3 {
        let session = create_test_session(&app, &auth).await;
        submit_test_trials(&app, &auth, &session, 10).await;
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/users/{}/analytics", auth.user_id))
                .header("authorization", format!("Bearer {}", auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_json(response).await;
    assert!(body["summary"]["total_sessions"].as_i64().unwrap() >= 3);
    assert!(body["summary"]["average_accuracy"].is_number());
    assert!(body["time_series"].is_array());
}

#[tokio::test]
async fn test_export_user_data() {
    let app = create_test_app().await;
    let auth = create_authenticated_user(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/users/{}/export?format=json", auth.user_id))
                .header("authorization", format!("Bearer {}", auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );
}

#[tokio::test]
async fn test_rate_limiting() {
    let app = create_test_app().await;

    // Send many requests quickly
    for _ in 0..150 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            // Rate limit hit
            assert!(response.headers().get("x-ratelimit-limit").is_some());
            assert!(response.headers().get("x-ratelimit-remaining").is_some());
            assert!(response.headers().get("x-ratelimit-reset").is_some());
            return;
        }
    }

    panic!("Rate limiting did not trigger");
}

#[tokio::test]
async fn test_session_completion() {
    let app = create_test_app().await;
    let auth = create_authenticated_user(&app).await;
    let session = create_test_session(&app, &auth).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/sessions/{}/end", session.id))
                .header("authorization", format!("Bearer {}", auth.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_json(response).await;
    assert_eq!(body["status"], "completed");
    assert!(body["ended_at"].is_string());
}

#[tokio::test]
async fn test_unauthorized_access() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "domain": "alphabet"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_input_validation() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "invalid-email",
                        "password": "weak"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = body_to_json(response).await;
    assert!(body["error"]["code"].is_string());
    assert!(body["error"]["message"].is_string());
}
