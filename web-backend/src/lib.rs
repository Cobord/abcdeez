pub mod cache;
pub mod config;
pub mod db;
pub mod db_pool_monitor;
pub mod encryption;
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
    extract::State,
    http::{header, Method},
    middleware as axum_middleware,
    response::Html,
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::error;

pub use state::AppState;

/// Handler to serve the admin panel HTML
async fn serve_admin_panel() -> Result<Html<String>, error::AppError> {
    let admin_html = std::fs::read_to_string("static/admin.html").map_err(|e| {
        error!("Failed to read admin panel HTML: {}", e);
        error::AppError::InternalServerError
    })?;
    Ok(Html(admin_html))
}

/// Health check endpoint that verifies database and cache connectivity
async fn ready_check(State(state): State<Arc<AppState>>) -> Result<&'static str, error::AppError> {
    // Check database connection
    sqlx::query("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Database health check failed: {}", e);
            monitoring::global_metrics().record_db_query();
            error::AppError::InternalServerError
        })?;

    monitoring::global_metrics().record_db_query();

    // Check in-memory cache connection (PING)
    let mut conn = state.redis_conn.clone();
    cache::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| {
            error!("Cache health check failed: {}", e);
            monitoring::global_metrics().record_cache_access(false);
            error::AppError::InternalServerError
        })?;

    monitoring::global_metrics().record_cache_access(true);
    Ok("READY")
}

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
        // Experiment routes (protected)
        .route("/experiments", get(handlers::experiment::list))
        .route("/experiments", post(handlers::experiment::create))
        .route("/experiments/:id", get(handlers::experiment::get))
        .route("/experiments/:id/join", post(handlers::experiment::join))
        .route("/experiments/:id/results", get(handlers::experiment::results))
        .route("/experiments/:id/export", post(handlers::experiment::export))
        // Protocol versioning routes
        .route("/protocols", get(handlers::protocol::list_protocols))
        .route("/protocols", post(handlers::protocol::create_protocol))
        .route("/protocols/:id", get(handlers::protocol::get_protocol))
        .route("/protocols/:id/versions", post(handlers::protocol::create_version))
        .route("/protocols/:id/versions", get(handlers::protocol::list_versions))
        .route("/protocols/:id/versions/:version_id", get(handlers::protocol::get_version))
        .route("/protocols/:id/versions/:version_id/publish", post(handlers::protocol::publish_version))
        .route("/protocols/:id/versions/:version_id/validate", get(handlers::protocol::validate_protocol))
        .route("/protocols/:id/compare", post(handlers::protocol::compare_versions))
        .route("/protocols/:id/history", post(handlers::protocol::version_history))
        .route("/protocols/:id/branches", post(handlers::protocol::create_branch))
        .route("/protocols/:id/metrics", get(handlers::protocol::get_metrics))
        // Federation network routes
        .route("/federation/nodes", get(handlers::federation::list_nodes))
        .route("/federation/nodes", post(handlers::federation::register_node))
        .route("/federation/nodes/:id", get(handlers::federation::get_node))
        .route("/federation/heartbeat", post(handlers::federation::heartbeat))
        .route("/federation/share", post(handlers::federation::share_data))
        .route("/federation/sync/protocol", post(handlers::federation::sync_protocol))
        .route("/federation/compliance/verify", post(handlers::federation::verify_compliance))
        .route("/federation/stats", get(handlers::federation::network_stats))
        .route("/federation/agreements", get(handlers::federation::list_agreements))
        .route("/federation/agreements", post(handlers::federation::create_agreement))
        // Music domain routes
        .route("/music/scales", get(handlers::music::scales))
        .route("/music/progressions", get(handlers::music::progressions))
        .route("/music/tasks", post(handlers::music::tasks))
        .route("/music/audio/:note", get(handlers::music::audio))
        // Gamification routes
        .route("/gamification/profile", get(handlers::gamification::get_profile))
        .route("/gamification/achievements", get(handlers::gamification::get_achievements))
        .route("/gamification/achievements/:id/unlock", post(handlers::gamification::unlock_achievement))
        .route("/gamification/leaderboard", get(handlers::gamification::get_leaderboard))
        .route("/gamification/leaderboard/update", post(handlers::gamification::update_leaderboard_score))
        .route("/gamification/xp/add", post(handlers::gamification::add_xp))
        // Sync routes
        .route("/sync/devices", post(handlers::sync::register_device))
        .route("/sync/status", get(handlers::sync::sync_status))
        .route("/sync/pull", get(handlers::sync::sync_pull))
        .route("/sync/push", post(handlers::sync::sync_push))
        .route("/sync/conflicts/resolve", post(handlers::sync::resolve_conflict))
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
        // Business metrics routes
        .route("/admin/business/dashboard", get(handlers::business::get_business_dashboard))
        .route("/admin/business/daily-metrics", get(handlers::business::get_daily_metrics))
        .route("/admin/business/learning-effectiveness", get(handlers::business::get_learning_effectiveness))
        .route("/admin/business/user-journey", get(handlers::business::get_user_journey_analytics))
        .route("/admin/business/revenue", get(handlers::business::get_revenue_metrics))
        .route("/admin/business/export", get(handlers::business::export_business_data))
        // Migration management routes
        .route("/admin/migrations/status", get(handlers::migration::get_migration_status))
        .route("/admin/migrations/run", post(handlers::migration::run_migrations))
        .route("/admin/migrations/rollback", post(handlers::migration::rollback_to_version))
        .route("/admin/migrations/rollback-last", post(handlers::migration::rollback_last_migrations))
        .route("/admin/migrations/validate", get(handlers::migration::validate_migrations))
        .route("/admin/migrations/backup", post(handlers::migration::create_backup))
        .route("/admin/migrations/history", get(handlers::migration::get_migration_history))
        .route("/admin/migrations/preview/:version", get(handlers::migration::preview_rollback))
        // Audit retention management routes
        .route("/admin/audit/retention/statistics", get(handlers::audit_retention::get_retention_statistics))
        .route("/admin/audit/retention/cleanup", post(handlers::audit_retention::apply_retention_policies))
        .route("/admin/audit/retention/compliance-report", get(handlers::audit_retention::generate_compliance_report))
        .route("/admin/audit/retention/policies", get(handlers::audit_retention::get_retention_policies))
        .route("/admin/audit/retention/policies", axum::routing::put(handlers::audit_retention::update_retention_policy))
        .route("/admin/audit/retention/policies/:name", delete(handlers::audit_retention::delete_retention_policy))
        .route("/admin/audit/trail", get(handlers::audit_retention::get_audit_trail_with_retention))
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
        .route("/ready", get(ready_check))
        .route("/health", get(monitoring::health::detailed_health_check))
        .route("/health/enhanced", get(monitoring::health::enhanced_health_check))
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
    
    // Dashboard routes for metrics visualization
    let dashboard_routes = Router::new()
        .route("/dashboard", get(handlers::dashboard::metrics_dashboard_html))
        .route("/dashboard/data", get(handlers::dashboard::dashboard_data))
        .route("/dashboard/realtime", get(handlers::dashboard::realtime_metrics))
        .route("/dashboard/otel", get(handlers::dashboard::otel_metrics))
        .route("/dashboard/system", get(handlers::dashboard::system_info))
        .route("/dashboard/database", get(handlers::dashboard::database_report))
        .route("/dashboard/health", get(handlers::dashboard::detailed_health));
    
    // Static routes
    let static_routes = Router::new()
        .route("/admin", get(serve_admin_panel));

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
        .nest("/api", dashboard_routes)
        .merge(static_routes)
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::audit_middleware,
        ))
        .layer(axum_middleware::from_fn(
            monitoring::performance::performance_middleware,
        ))
        .layer(axum_middleware::from_fn(middleware::correlation_id_middleware))
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
