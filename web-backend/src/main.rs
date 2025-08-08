mod cache;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod monitoring;
mod services;
mod state;
mod websocket;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::handlers::{admin, analytics, auth, experiment, learner, music, session, task, task_simple};
use crate::middleware::{audit_middleware, auth_middleware, content_validation, ip_blocking, rate_limit, require_admin, require_analytics_permission, security_headers};
use crate::monitoring::{health, metrics, performance};
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "web_backend=debug,tower_http=debug,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    info!("Starting web backend with config: {:?}", config);

    // Initialize database
    let db_pool = db::init_pool(&config.database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    info!("Database migrations completed");

    // Initialize in-memory cache manager (no external Redis required)
    let cache_conn = crate::cache::connection_manager();

    // Create app state
    let app_state = Arc::new(AppState::new(db_pool, cache_conn, config.clone()));

    // Build API router
    let api_routes = Router::new()
        // Authentication routes
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        // Learner routes (protected)
        .route("/learners", post(learner::create))
        .route("/learners/:id", get(learner::get))
        .route("/learners/:id", patch(learner::update))
        .route("/learners/:id/stats", get(learner::stats))
        .route("/learners/:id/sessions", get(learner::sessions))
        .route("/learners/:id/export", get(learner::export))
        .route("/learners/:id", delete(learner::delete))
        // Session routes (protected)
        .route("/sessions", post(session::create))
        .route("/sessions/:id", get(session::get))
        .route("/sessions/:id/responses", post(session::submit_response))
        .route("/sessions/:id/responses", get(session::responses))
        .route("/sessions/:id/complete", post(session::complete))
        .route("/sessions/:id/replay", get(session::replay))
        // Task routes (protected) - using simple handlers that work with current Axum
        .route("/tasks/generate", get(task_simple::generate_simple))
        .route("/tasks/difficulty", get(task_simple::get_difficulty))
        .route("/tasks/hint", get(task_simple::generate_hint))
        // Analytics routes (protected with analytics permissions)  
        .route("/analytics/population", get(analytics::population))
        .route("/analytics/bottlenecks", get(analytics::bottlenecks))
        .route("/analytics/strategies", get(analytics::strategies))
        .route(
            "/analytics/learning-curves",
            get(analytics::learning_curves),
        )
        .route("/analytics/compare", post(analytics::compare))
        .route("/analytics/live", get(analytics::live))
        // Advanced analytics routes with core statistics
        .route("/analytics/response-time-analysis", get(analytics::response_time_analysis))
        .route("/analytics/learner/:id/performance", get(analytics::learner_performance_analysis))
        .route("/analytics/population/strategies", get(analytics::population_strategy_analysis))
        .route("/analytics/learner/:id/adaptive-difficulty", get(analytics::adaptive_difficulty_analysis))
        // Apply analytics permission middleware to analytics routes
        .layer(axum_middleware::from_fn(require_analytics_permission))
        // Experiment routes (protected)
        .route("/experiments", get(experiment::list))
        .route("/experiments", post(experiment::create))
        .route("/experiments/:id", get(experiment::get))
        .route("/experiments/:id/join", post(experiment::join))
        .route("/experiments/:id/results", get(experiment::results))
        .route("/experiments/:id/export", post(experiment::export))
        // Music domain routes
        .route("/music/scales", get(music::scales))
        .route("/music/progressions", get(music::progressions))
        .route("/music/tasks", post(music::tasks))
        .route("/music/audio/:note", get(music::audio))
        // Admin routes (protected with admin permissions)
        .route("/admin/dashboard", get(admin::dashboard))
        .route("/admin/users", get(admin::list_users))
        .route("/admin/learners", get(admin::list_learners))
        .route("/admin/audit", get(admin::audit_trail))
        .route("/admin/jobs", get(admin::list_jobs))
        .route("/admin/jobs", post(admin::trigger_job))
        .route("/admin/config", post(admin::update_config))
        .route("/admin/audit-report", get(admin::audit_report))
        // Apply admin-only middleware to admin routes
        .layer(axum_middleware::from_fn(require_admin))
        // Apply auth middleware to protected routes
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        // Apply rate limiting
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            rate_limit,
        ))
        // Add content validation middleware
        .layer(axum_middleware::from_fn(content_validation))
        // Add IP blocking middleware
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            ip_blocking,
        ));

    // WebSocket routes (separate as they need different handling)
    let ws_routes = Router::new()
        .route("/sessions/:id/live", get(websocket::session_handler))
        .route("/analytics/live", get(websocket::analytics_handler));

    // Health check and monitoring routes
    let health_routes = Router::new()
        .route("/live", get(health::simple_health_check))
        .route("/ready", get(ready_check))
        .route("/health", get(health::detailed_health_check))
        .route("/metrics", get(metrics::prometheus_metrics))
        .route("/metrics/json", get(metrics::json_metrics))
        .route("/performance", get(performance::get_performance_metrics))
        .route("/performance/endpoints", get(performance::get_endpoint_performance));

    // Combine all routes
    let app = Router::new()
        .nest("/api", api_routes)
        .nest("/api", ws_routes)
        .nest("/health", health_routes)
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            audit_middleware,
        ))
        .layer(axum_middleware::from_fn(performance::performance_middleware))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            security_headers,
        ))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state.clone());

    // Start background monitoring tasks
    let health_monitor_state = app_state.clone();
    let performance_monitor_state = app_state.clone();
    
    tokio::spawn(async move {
        health::start_health_monitor(health_monitor_state).await;
    });
    
    tokio::spawn(async move {
        performance::start_performance_monitor(performance_monitor_state).await;
    });

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn ready_check(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<&'static str, error::AppError> {
    // Check database connection
    sqlx::query("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Database health check failed: {}", e);
            crate::monitoring::global_metrics().record_db_query();
            error::AppError::InternalServerError
        })?;

    crate::monitoring::global_metrics().record_db_query();

    // Check in-memory cache connection (PING)
    let mut conn = state.redis_conn.clone();
    crate::cache::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| {
            error!("Cache health check failed: {}", e);
            crate::monitoring::global_metrics().record_cache_access(false);
            error::AppError::InternalServerError
        })?;

    crate::monitoring::global_metrics().record_cache_access(true);
    Ok("READY")
}
