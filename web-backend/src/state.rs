use crate::{cache, cache::ConnectionManager, config::Config, db::DbPool};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    // For legacy code paths that referenced Redis explicitly.
    // We back this with the in-memory cache manager to avoid external deps.
    pub redis_conn: ConnectionManager,
    // New preferred field name for cache usage.
    pub cache_conn: ConnectionManager,
    pub config: Config,
}

impl AppState {
    // Primary constructor: provide a single cache connection manager
    // and it will be used for both `redis_conn` and `cache_conn` for compatibility.
    pub fn new(db_pool: DbPool, cache_conn: ConnectionManager, config: Config) -> Self {
        Self {
            db_pool,
            redis_conn: cache_conn.clone(),
            cache_conn,
            config,
        }
    }

    // Convenience constructor for local/dev: creates an in-memory cache manager.
    pub fn new_in_memory(db_pool: DbPool, config: Config) -> Self {
        let conn = cache::connection_manager();
        Self::new(db_pool, conn, config)
    }
}
