mod config;
mod db;
mod handlers;
mod models;
mod services;
mod state;
mod websocket;
mod middleware;
mod error;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post, patch, delete},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::state::AppState;
use crate::handlers::{auth, learner, session, task, analytics, experiment, music};
use crate::middleware::{auth_middleware, rate_limit};

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
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;
    info!("Database migrations completed");

    // Initialize Redis
    let redis_client = redis::Client::open(config.redis_url.clone())?;
    let redis_conn = redis_client
        .get_connection_manager()
        .await?;
    info!("Redis connection established");

    // Create app state
    let app_state = Arc::new(AppState::new(
        db_pool,
        redis_conn,
        config.clone(),
    ));

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
        .route("/learners/:id/export", get(learner::export))
        .route("/learners/:id", delete(learner::delete))
        
        // Session routes (protected)
        .route("/sessions", post(session::create))
        .route("/sessions/:id", get(session::get))
        .route("/sessions/:id/responses", post(session::submit_response))
        .route("/sessions/:id/complete", post(session::complete))
        .route("/sessions/:id/replay", get(session::replay))
        
        // Task routes (protected)
        .route("/tasks/next", get(task::next))
        .route("/tasks/generate", post(task::generate))
        .route("/tasks/difficulty", get(task::difficulty))
        .route("/hints/request", post(task::request_hint))
        
        // Analytics routes (protected with different permissions)
        .route("/analytics/population", get(analytics::population))
        .route("/analytics/bottlenecks", get(analytics::bottlenecks))
        .route("/analytics/strategies", get(analytics::strategies))
        .route("/analytics/learning-curves", get(analytics::learning_curves))
        .route("/analytics/compare", post(analytics::compare))
        
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
        
        // Apply auth middleware to protected routes
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        // Apply rate limiting
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            rate_limit,
        ));

    // WebSocket routes (separate as they need different handling)
    let ws_routes = Router::new()
        .route("/sessions/:id/live", get(websocket::session_handler))
        .route("/analytics/live", get(websocket::analytics_handler));

    // Health check routes
    let health_routes = Router::new()
        .route("/live", get(health_check))
        .route("/ready", get(ready_check))
        .route("/metrics", get(metrics));

    // Combine all routes
    let app = Router::new()
        .nest("/api", api_routes)
        .nest("/api", ws_routes)
        .nest("/health", health_routes)
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
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
            error::AppError::InternalServerError
        })?;
    
    // Check Redis connection
    let mut conn = state.redis_conn.clone();
    redis::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| {
            error!("Redis health check failed: {}", e);
            error::AppError::InternalServerError
        })?;
    
    Ok("READY")
}

async fn metrics() -> String {
    // TODO: Implement Prometheus metrics export
    "# HELP tasks_completed_total Total number of tasks completed\n\
     # TYPE tasks_completed_total counter\n\
     tasks_completed_total 0\n"
        .to_string()
}