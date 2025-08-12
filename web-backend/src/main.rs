use web_backend::{build_router, cache, config, db, monitoring, AppState};

use std::net::SocketAddr;
use std::sync::Arc;

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "web_backend=debug,tower_http=debug,axum=info,abcdeez_core=debug,sqlx=warn,hyper=warn".into()
                }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
        )
        .init();

    info!("Tracing initialized with structured logging");

    // Load configuration
    let config = config::Config::from_env()?;

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
    let cache_conn = cache::connection_manager();

    // Create app state
    let app_state = Arc::new(AppState::new(db_pool, cache_conn, Arc::new(config.clone())));

    // Build the app router using the centralized function from lib.rs
    let app = build_router(app_state.clone());

    // Start background monitoring tasks
    let health_monitor_state = app_state.clone();
    let performance_monitor_state = app_state.clone();

    tokio::spawn(async move {
        monitoring::health::start_health_monitor(health_monitor_state).await;
    });

    tokio::spawn(async move {
        monitoring::performance::start_performance_monitor(performance_monitor_state).await;
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

    // Server address (HTTP)
    let http_addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    info!(
        "Starting server (HTTP only) on {}",
        http_addr
    );

    // Start server without TLS
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
