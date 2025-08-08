use std::sync::Arc;
use redis::aio::ConnectionManager;
use crate::{config::Config, db::DbPool};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub redis_conn: ConnectionManager,
    pub config: Config,
}

impl AppState {
    pub fn new(
        db_pool: DbPool,
        redis_conn: ConnectionManager,
        config: Config,
    ) -> Self {
        Self {
            db_pool,
            redis_conn,
            config,
        }
    }
}