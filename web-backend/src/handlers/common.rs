use crate::error::{AppError, AppResult};
use crate::db::DbPool;
use std::sync::Arc;

/// Common database operations and error handling helpers
pub struct DbHelper;

impl DbHelper {
    /// Execute a database query with consistent error handling
    pub async fn execute_query<F, T>(
        pool: &DbPool,
        operation_name: &str,
        query_fn: F,
    ) -> AppResult<T>
    where
        F: for<'a> FnOnce(&'a mut sqlx::Transaction<'static, sqlx::Sqlite>) -> Result<T, sqlx::Error> + Send,
        T: Send,
    {
        let mut tx = pool
            .begin()
            .await
            .map_err(AppError::DatabaseError)?;

        let result = query_fn(&mut tx)
            .map_err(|e| {
                tracing::error!("Database operation '{}' failed: {}", operation_name, e);
                AppError::DatabaseError(e)
            })?;

        tx.commit()
            .await
            .map_err(AppError::DatabaseError)?;

        Ok(result)
    }

    /// Acquire a connection with consistent error handling
    pub async fn acquire_connection(
        pool: &DbPool,
    ) -> AppResult<sqlx::pool::PoolConnection<sqlx::Sqlite>> {
        pool.acquire()
            .await
            .map_err(AppError::DatabaseError)
    }

    /// Check if a resource exists by ID with consistent error handling
    pub async fn resource_exists(
        pool: &DbPool,
        table: &str,
        id_column: &str,
        id_value: &str,
    ) -> AppResult<bool> {
        let query = format!("SELECT 1 FROM {} WHERE {} = ? LIMIT 1", table, id_column);
        
        let mut conn = Self::acquire_connection(pool).await?;
        
        let exists = sqlx::query(&query)
            .bind(id_value)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::DatabaseError)?
            .is_some();

        Ok(exists)
    }
}

/// Common validation helpers
pub struct ValidationHelper;

impl ValidationHelper {
    /// Validate UUID format
    pub fn validate_uuid(uuid_str: &str, field_name: &str) -> AppResult<uuid::Uuid> {
        uuid::Uuid::parse_str(uuid_str)
            .map_err(|_| AppError::ValidationError(format!("Invalid {} format", field_name)))
    }

    /// Validate string length
    pub fn validate_string_length(
        value: &str,
        field_name: &str,
        min_len: usize,
        max_len: usize,
    ) -> AppResult<()> {
        if value.len() < min_len || value.len() > max_len {
            return Err(AppError::ValidationError(format!(
                "{} must be between {} and {} characters",
                field_name, min_len, max_len
            )));
        }
        Ok(())
    }

    /// Validate required field is not empty
    pub fn validate_not_empty(value: &str, field_name: &str) -> AppResult<()> {
        if value.trim().is_empty() {
            return Err(AppError::ValidationError(format!(
                "{} cannot be empty",
                field_name
            )));
        }
        Ok(())
    }
}

/// Common response helpers
pub struct ResponseHelper;

impl ResponseHelper {
    /// Create a standardized success response
    pub fn success_response<T: serde::Serialize>(
        data: T,
        status: axum::http::StatusCode,
    ) -> AppResult<(axum::http::StatusCode, axum::Json<T>)> {
        Ok((status, axum::Json(data)))
    }

    /// Create a standardized created response
    pub fn created_response<T: serde::Serialize>(
        data: T,
    ) -> AppResult<(axum::http::StatusCode, axum::Json<T>)> {
        Self::success_response(data, axum::http::StatusCode::CREATED)
    }

    /// Create a standardized ok response
    pub fn ok_response<T: serde::Serialize>(
        data: T,
    ) -> AppResult<(axum::http::StatusCode, axum::Json<T>)> {
        Self::success_response(data, axum::http::StatusCode::OK)
    }
}