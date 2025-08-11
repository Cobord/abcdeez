use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use sqlx::{Pool, Postgres, Sqlite};
use crate::error::{AppError, AppResult};
use crate::db::DbPool;
use tracing::{warn, error, info};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Circuit is open, rejecting requests
    HalfOpen,   // Testing if service recovered
}

/// Circuit breaker for database connections
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    failure_threshold: u64,
    success_threshold: u64,
    timeout_duration: Duration,
    half_open_max_requests: AtomicU64,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration: Duration::from_secs(30),
            half_open_max_requests: AtomicU64::new(3),
        }
    }

    pub async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        *state == CircuitState::Open
    }

    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        
        match *state {
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                
                if success_count >= self.success_threshold {
                    *state = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                    info!("Circuit breaker closed after successful recovery");
                }
            }
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        
        match *state {
            CircuitState::Closed if failure_count >= self.failure_threshold => {
                *state = CircuitState::Open;
                *self.last_failure_time.write().await = Some(Instant::now());
                error!("Circuit breaker opened after {} failures", failure_count);
            }
            CircuitState::HalfOpen => {
                *state = CircuitState::Open;
                *self.last_failure_time.write().await = Some(Instant::now());
                self.success_count.store(0, Ordering::SeqCst);
                warn!("Circuit breaker reopened after failure in half-open state");
            }
            _ => {}
        }
    }

    pub async fn check_state(&self) {
        let mut state = self.state.write().await;
        
        if *state == CircuitState::Open {
            if let Some(last_failure) = *self.last_failure_time.read().await {
                if last_failure.elapsed() >= self.timeout_duration {
                    *state = CircuitState::HalfOpen;
                    self.half_open_max_requests.store(3, Ordering::SeqCst);
                    info!("Circuit breaker entering half-open state for testing");
                }
            }
        }
    }

    pub async fn can_proceed(&self) -> bool {
        self.check_state().await;
        
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => {
                let remaining = self.half_open_max_requests.fetch_sub(1, Ordering::SeqCst);
                remaining > 0
            }
        }
    }
}

/// Enhanced database pool with monitoring and circuit breaker
pub struct MonitoredDbPool {
    pool: Arc<DbPool>,
    circuit_breaker: Arc<CircuitBreaker>,
    active_connections: AtomicU64,
    total_requests: AtomicU64,
    failed_requests: AtomicU64,
    slow_queries: AtomicU64,
    slow_query_threshold: Duration,
}

impl MonitoredDbPool {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool: Arc::new(pool),
            circuit_breaker: Arc::new(CircuitBreaker::new()),
            active_connections: AtomicU64::new(0),
            total_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            slow_queries: AtomicU64::new(0),
            slow_query_threshold: Duration::from_secs(5),
        }
    }

    /// Acquire a connection with timeout and circuit breaker protection
    pub async fn acquire_with_timeout(&self, timeout: Duration) -> AppResult<sqlx::pool::PoolConnection<sqlx::Postgres>> {
        // Check circuit breaker
        if !self.circuit_breaker.can_proceed().await {
            metrics::increment_counter!("db.circuit_breaker.rejected");
            return Err(AppError::ServiceUnavailable("Database circuit breaker is open".into()));
        }

        // Track active connections
        self.active_connections.fetch_add(1, Ordering::SeqCst);
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        
        let start = Instant::now();
        
        // Try to acquire connection with timeout
        let result = match tokio::time::timeout(timeout, self.pool.acquire()).await {
            Ok(Ok(conn)) => {
                let elapsed = start.elapsed();
                
                // Track slow connections
                if elapsed > self.slow_query_threshold {
                    self.slow_queries.fetch_add(1, Ordering::SeqCst);
                    warn!("Slow database connection acquisition: {:?}", elapsed);
                    metrics::increment_counter!("db.slow_connection_acquisition");
                }
                
                self.circuit_breaker.record_success().await;
                metrics::histogram!("db.connection.acquisition_time", elapsed.as_millis() as f64);
                Ok(conn)
            },
            Ok(Err(e)) => {
                self.failed_requests.fetch_add(1, Ordering::SeqCst);
                self.circuit_breaker.record_failure().await;
                error!("Failed to acquire database connection: {}", e);
                metrics::increment_counter!("db.connection.acquisition_error");
                Err(AppError::DatabaseError(e))
            },
            Err(_) => {
                self.failed_requests.fetch_add(1, Ordering::SeqCst);
                self.circuit_breaker.record_failure().await;
                error!("Database connection acquisition timeout after {:?}", timeout);
                metrics::increment_counter!("db.connection.timeout");
                Err(AppError::ServiceUnavailable("Database connection timeout".into()))
            }
        };
        
        // Always decrement active connections
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
        
        // Record metrics
        metrics::gauge!("db.active_connections", self.active_connections.load(Ordering::SeqCst) as f64);
        metrics::gauge!("db.pool.size", self.pool.size() as f64);
        
        result
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            active_connections: self.active_connections.load(Ordering::SeqCst),
            total_requests: self.total_requests.load(Ordering::SeqCst),
            failed_requests: self.failed_requests.load(Ordering::SeqCst),
            slow_queries: self.slow_queries.load(Ordering::SeqCst),
            pool_size: self.pool.size() as u64,
            pool_idle: self.pool.num_idle() as u64,
        }
    }

    /// Health check for the database pool
    pub async fn health_check(&self) -> AppResult<HealthStatus> {
        let stats = self.get_stats();
        
        // Check if circuit breaker is open
        if self.circuit_breaker.is_open().await {
            return Ok(HealthStatus::Unhealthy {
                reason: "Circuit breaker is open".to_string(),
                stats,
            });
        }
        
        // Try a simple query with short timeout
        let health_check_timeout = Duration::from_secs(2);
        match self.acquire_with_timeout(health_check_timeout).await {
            Ok(mut conn) => {
                // Execute a simple health check query
                #[cfg(feature = "postgres")]
                let result = sqlx::query("SELECT 1").fetch_one(&mut *conn).await;
                #[cfg(feature = "sqlite")]
                let result = sqlx::query("SELECT 1").fetch_one(&mut *conn).await;
                
                match result {
                    Ok(_) => {
                        // Check pool utilization
                        let utilization = stats.active_connections as f64 / stats.pool_size as f64;
                        
                        if utilization > 0.9 {
                            Ok(HealthStatus::Degraded {
                                reason: format!("High pool utilization: {:.1}%", utilization * 100.0),
                                stats,
                            })
                        } else if stats.failed_requests > 100 {
                            Ok(HealthStatus::Degraded {
                                reason: format!("High failure rate: {} failed requests", stats.failed_requests),
                                stats,
                            })
                        } else {
                            Ok(HealthStatus::Healthy { stats })
                        }
                    }
                    Err(e) => Ok(HealthStatus::Unhealthy {
                        reason: format!("Health check query failed: {}", e),
                        stats,
                    }),
                }
            }
            Err(e) => Ok(HealthStatus::Unhealthy {
                reason: format!("Cannot acquire connection: {}", e),
                stats,
            }),
        }
    }

    /// Get the underlying pool (for compatibility)
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub active_connections: u64,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub slow_queries: u64,
    pub pool_size: u64,
    pub pool_idle: u64,
}

#[derive(Debug)]
pub enum HealthStatus {
    Healthy { stats: PoolStats },
    Degraded { reason: String, stats: PoolStats },
    Unhealthy { reason: String, stats: PoolStats },
}

// Export metrics module if not already available
mod metrics {
    pub fn increment_counter(name: &str) {
        // This would integrate with your metrics system
        tracing::debug!("Metric counter {}: +1", name);
    }
    
    pub fn histogram(name: &str, value: f64) {
        tracing::debug!("Metric histogram {}: {}", name, value);
    }
    
    pub fn gauge(name: &str, value: f64) {
        tracing::debug!("Metric gauge {}: {}", name, value);
    }
}