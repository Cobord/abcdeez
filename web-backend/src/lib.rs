pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod monitoring;
pub mod services;
pub mod state;
pub mod tls;
pub mod utils;
pub mod websocket;

use std::sync::Arc;

use axum::{
    http::{header, Method},
    middleware as axum_middleware,
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

pub use state::AppState;

/// Build the application Router for use in tests or embedding
pub fn build_router(app_state: Arc<AppState>) -> Router {
    let cfg = app_state.config.clone();

    // API routes
    let api_routes = Router::new()
        // Authentication
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/auth/me", get(handlers::auth::me))
        // OAuth
        .route(
            "/auth/oauth/:provider/authorize",
            get(handlers::auth::oauth_authorization_url),
        )
        .route("/auth/oauth/callback", post(handlers::auth::oauth_callback))
        .route("/auth/apple/signin", post(handlers::auth::apple_signin))
        // Learners
        .route("/learners", post(handlers::learner::create))
        .route("/learners/:id", get(handlers::learner::get))
        .route("/learners/:id", patch(handlers::learner::update))
        .route("/learners/:id/stats", get(handlers::learner::stats))
        .route("/learners/:id/sessions", get(handlers::learner::sessions))
        .route("/learners/:id/export", get(handlers::learner::export))
        .route("/learners/:id", delete(handlers::learner::delete))
        // Sessions
        .route("/sessions", post(handlers::session::create))
        .route("/sessions/:id", get(handlers::session::get))
        .route(
            "/sessions/:id/responses",
            post(handlers::session::submit_response),
        )
        .route("/sessions/:id/responses", get(handlers::session::responses))
        .route("/sessions/:id/complete", post(handlers::session::complete))
        .route("/sessions/:id/replay", get(handlers::session::replay))
        // Simple task endpoints compatible with current handlers
        .route(
            "/tasks/generate",
            get(handlers::task_simple::generate_simple),
        )
        .route(
            "/tasks/difficulty",
            get(handlers::task_simple::get_difficulty),
        )
        .route("/tasks/hint", get(handlers::task_simple::generate_hint))
        // Analytics
        .route(
            "/analytics/population",
            get(handlers::analytics::population),
        )
        .route(
            "/analytics/bottlenecks",
            get(handlers::analytics::bottlenecks),
        )
        .route(
            "/analytics/strategies",
            get(handlers::analytics::strategies),
        )
        .route(
            "/analytics/learning-curves",
            get(handlers::analytics::learning_curves),
        )
        .route("/analytics/compare", post(handlers::analytics::compare))
        .route("/analytics/live", get(handlers::analytics::live))
        .route(
            "/analytics/response-time-analysis",
            get(handlers::analytics::response_time_analysis),
        )
        .route(
            "/analytics/learner/:id/performance",
            get(handlers::analytics::learner_performance_analysis),
        )
        .route(
            "/analytics/population/strategies",
            get(handlers::analytics::population_strategy_analysis),
        )
        .route(
            "/analytics/learner/:id/adaptive-difficulty",
            get(handlers::analytics::adaptive_difficulty_analysis),
        )
        // Permissions
        .layer(axum_middleware::from_fn(
            middleware::require_analytics_permission,
        ))
        // Admin
        .route("/admin/dashboard", get(handlers::admin::dashboard))
        .route("/admin/users", get(handlers::admin::list_users))
        .route("/admin/learners", get(handlers::admin::list_learners))
        .route("/admin/audit", get(handlers::admin::audit_trail))
        .route("/admin/jobs", get(handlers::admin::list_jobs))
        .route("/admin/jobs", post(handlers::admin::trigger_job))
        .route(
            "/admin/oauth-validation",
            post(handlers::admin::trigger_oauth_validation),
        )
        .route("/admin/config", post(handlers::admin::update_config))
        .route("/admin/audit-report", get(handlers::admin::audit_report))
        .layer(axum_middleware::from_fn(middleware::require_admin))
        // Auth middleware
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth_middleware,
        ))
        // Rate limit
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::rate_limit,
        ))
        // Content validation and IP blocking
        .layer(axum_middleware::from_fn(middleware::content_validation))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::ip_blocking,
        ));

    // Websocket routes
    let ws_routes = Router::new()
        .route("/sessions/:id/live", get(websocket::session_handler))
        .route("/analytics/ws", get(websocket::analytics_handler));

    // Health routes
    let health_routes = Router::new()
        // Provide a direct /health response (used by tests)
        .route(
            "/",
            get(|| async { axum::Json(serde_json::json!({"status": "healthy"})) }),
        )
        .route("/live", get(monitoring::health::simple_health_check))
        .route("/health", get(monitoring::health::detailed_health_check))
        .route("/metrics", get(monitoring::metrics::prometheus_metrics))
        .route("/metrics/json", get(monitoring::metrics::json_metrics))
        .route(
            "/performance",
            get(monitoring::performance::get_performance_metrics),
        )
        .route(
            "/performance/endpoints",
            get(monitoring::performance::get_endpoint_performance),
        );

    let cors = if cfg.cors_origin == "*" {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(
                cfg.cors_origin
                    .parse::<axum::http::HeaderValue>()
                    .expect("Invalid CORS origin"),
            )
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
    };

    let api_scope = Router::new().merge(api_routes).merge(ws_routes);

    Router::new()
        // Support both /api and /api/v1 prefixes for compatibility with tests
        .nest("/api", api_scope.clone())
        .nest("/api/v1", api_scope)
        .nest("/health", health_routes)
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::audit_middleware,
        ))
        .layer(axum_middleware::from_fn(
            monitoring::performance::performance_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::security_headers,
        ))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

/// Helper to create an AppState for tests using SQLite
pub async fn create_test_state_sqlite() -> Arc<AppState> {
    // Ensure required env defaults
    std::env::set_var(
        "JWT_SECRET",
        "test-secret-for-github-actions-minimum-32-chars",
    );
    std::env::set_var("DATABASE_URL", "sqlite:test.db");
    let cfg = config::Config::from_env().expect("Config from env");

    let pool = db::init_pool(&cfg.database_url)
        .await
        .expect("init sqlite pool");
    db::run_migrations(&pool).await.expect("run migrations");

    let cache_conn = cache::connection_manager();
    Arc::new(state::AppState::new(pool, cache_conn, Arc::new(cfg)))
}
