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
#[cfg(test)]
mod tests;
mod tls;
mod utils;
mod websocket;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    http::{header, Method},
    middleware as axum_middleware,
    response::Html,
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::handlers::{
    admin, analytics, auth, dashboard, experiment, gamification, learner, music, session, sync,
    task_simple,
};
use crate::middleware::{
    audit_middleware, auth_middleware, content_validation, ip_blocking, rate_limit, require_admin,
    require_analytics_permission, security_headers,
};
use crate::monitoring::{health, metrics, performance};
use crate::state::AppState;

// Handler to serve the admin panel HTML
async fn serve_admin_panel() -> Result<Html<String>, error::AppError> {
    let admin_html = std::fs::read_to_string("static/admin.html").map_err(|e| {
        error!("Failed to read admin panel HTML: {}", e);
        error::AppError::InternalServerError
    })?;
    Ok(Html(admin_html))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "web_backend=debug,tower_http=debug,axum=info,graph_learning_core=debug,sqlx=warn,hyper=warn".into()
                }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .json() // Use structured JSON logging for better observability
        )
        .init();
    
    info!("Tracing initialized with structured logging");

    // Load configuration
    let config = Config::from_env()?;

    // Validate production safety
    if let Err(e) = config.validate_production_safety() {
        error!("Production safety validation failed: {}", e);
        return Err(e.into());
    }

    info!(
        "Starting web backend - Environment: {:?}, Port: {}",
        config.environment, config.port
    );

    // Initialize database
    let db_pool = match db::init_pool(&config.database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to initialize database pool: {}", e);
            return Err(e.into());
        }
    };

    // Run migrations
    match db::run_migrations(&db_pool).await {
        Ok(_) => info!("Database migrations completed"),
        Err(e) => {
            error!("Failed to run database migrations: {}", e);
            return Err(e.into());
        }
    }

    // Initialize in-memory cache manager (no external Redis required)
    let cache_conn = crate::cache::connection_manager();

    // Create app state
    let app_state = Arc::new(AppState::new(db_pool, cache_conn, Arc::new(config.clone())));

    // Build API router
    let api_routes = Router::new()
        // Authentication routes
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        // OAuth routes
        .route(
            "/auth/oauth/:provider/authorize",
            get(auth::oauth_authorization_url),
        )
        .route("/auth/oauth/callback", post(auth::oauth_callback))
        .route("/auth/apple/signin", post(auth::apple_signin))
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
        .route(
            "/analytics/response-time-analysis",
            get(analytics::response_time_analysis),
        )
        .route(
            "/analytics/learner/:id/performance",
            get(analytics::learner_performance_analysis),
        )
        .route(
            "/analytics/population/strategies",
            get(analytics::population_strategy_analysis),
        )
        .route(
            "/analytics/learner/:id/adaptive-difficulty",
            get(analytics::adaptive_difficulty_analysis),
        )
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
        // Gamification routes (protected)
        .route("/gamification/profile", get(gamification::get_profile))
        .route(
            "/gamification/achievements",
            get(gamification::get_achievements),
        )
        .route(
            "/gamification/achievements/:id/unlock",
            post(gamification::unlock_achievement),
        )
        .route(
            "/gamification/leaderboard",
            get(gamification::get_leaderboard),
        )
        .route(
            "/gamification/leaderboard/update",
            post(gamification::update_leaderboard_score),
        )
        .route("/gamification/xp/add", post(gamification::add_xp))
        // Sync routes (protected)
        .route("/sync/devices", post(sync::register_device))
        .route("/sync/status", get(sync::sync_status))
        .route("/sync/pull", get(sync::sync_pull))
        .route("/sync/push", post(sync::sync_push))
        .route("/sync/conflicts/resolve", post(sync::resolve_conflict))
        // Admin routes (protected with admin permissions)
        .route("/admin/dashboard", get(admin::dashboard))
        .route("/admin/users", get(admin::list_users))
        .route("/admin/learners", get(admin::list_learners))
        .route("/admin/audit", get(admin::audit_trail))
        .route("/admin/jobs", get(admin::list_jobs))
        .route("/admin/jobs", post(admin::trigger_job))
        .route(
            "/admin/oauth-validation",
            post(admin::trigger_oauth_validation),
        )
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
        .route("/analytics/ws", get(websocket::analytics_handler));

    // Health check and monitoring routes
    let health_routes = Router::new()
        .route("/live", get(health::simple_health_check))
        .route("/ready", get(ready_check))
        .route("/health", get(health::detailed_health_check))
        .route("/metrics", get(metrics::prometheus_metrics))
        .route("/metrics/json", get(metrics::json_metrics))
        .route("/performance", get(performance::get_performance_metrics))
        .route(
            "/performance/endpoints",
            get(performance::get_endpoint_performance),
        );

    // Dashboard routes for metrics visualization
    let dashboard_routes = Router::new()
        .route("/dashboard", get(dashboard::metrics_dashboard_html))
        .route("/dashboard/data", get(dashboard::dashboard_data))
        .route("/dashboard/realtime", get(dashboard::realtime_metrics))
        .route("/dashboard/otel", get(dashboard::otel_metrics))
        .route("/dashboard/system", get(dashboard::system_info))
        .route("/dashboard/database", get(dashboard::database_report))
        .route("/dashboard/health", get(dashboard::detailed_health));

    // Static routes
    let static_routes = Router::new().route("/admin", get(serve_admin_panel));

    // Combine all routes
    let app = Router::new()
        .nest("/api", api_routes)
        .nest("/api", ws_routes)
        .nest("/health", health_routes)
        .nest("/api", dashboard_routes)
        .merge(static_routes)
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            audit_middleware,
        ))
        .layer(axum_middleware::from_fn(
            performance::performance_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            security_headers,
        ))
        .layer({
            let cors = if config.cors_origin == "*" {
                CorsLayer::permissive()
            } else {
                CorsLayer::new()
                    .allow_origin(
                        config
                            .cors_origin
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
            cors
        })
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

    // Start background job workers
    let batch_job_worker_state = app_state.clone();
    let oauth_validator_state = app_state.clone();

    // Start main batch job worker
    tokio::spawn(async move {
        batch_job_worker_state
            .batch_job_service
            .start_worker()
            .await;
    });

    // Start OAuth credential validation scheduler
    tokio::spawn(async move {
        oauth_validator_state
            .batch_job_service
            .start_oauth_validation_scheduler()
            .await;
    });

    // Setup TLS if configured
    let tls_manager = crate::tls::TlsManager::new(Arc::new(config.clone()));
    let tls_acceptor = tls_manager.create_tls_acceptor().await?;

    // Start certificate renewal scheduler
    let renewal_config = Arc::new(config.clone());
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400)); // Check daily
        loop {
            interval.tick().await;
            let renewal_manager = crate::tls::TlsManager::new(renewal_config.clone());
            if let Err(e) = renewal_manager.check_certificate_renewal().await {
                error!("Certificate renewal check failed: {}", e);
            }
        }
    });

    // Server addresses
    let http_addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let https_addr = SocketAddr::from(([0, 0, 0, 0], config.tls_port));

    // Add ACME challenge route to the app
    let app = app.route(
        "/.well-known/acme-challenge/:token",
        axum::routing::get(crate::tls::handle_acme_challenge),
    );

    info!(
        "Starting server with TLS support - HTTP: {}, HTTPS: {}",
        http_addr, https_addr
    );

    // Start server with TLS support
    crate::tls::serve_with_tls(app, http_addr, https_addr, tls_acceptor).await?;

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
