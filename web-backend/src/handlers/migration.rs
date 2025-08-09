use axum::{extract::{State, Path}, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    db::{MigrationManager, MigrationInfo, run_migrations_with_rollback},
    error::{AppError, AppResult},
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub current_version: Option<i64>,
    pub is_up_to_date: bool,
    pub applied_migrations: Vec<MigrationInfo>,
    pub validation_issues: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub target_version: i64,
    pub confirm: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackLastRequest {
    pub count: usize,
    pub confirm: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationResponse {
    pub success: bool,
    pub message: String,
    pub migrations_affected: Vec<MigrationInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupRequest {
    pub backup_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupResponse {
    pub backup_path: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Get current migration status
pub async fn get_migration_status(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<MigrationStatus>> {
    let manager = create_migration_manager(&state).await?;
    
    let applied_migrations = manager.get_migration_history().await
        .map_err(|e| AppError::InternalServerError)?;
    
    let current_version = applied_migrations.iter()
        .map(|m| m.version)
        .max();
    
    let is_up_to_date = manager.is_up_to_date().await
        .map_err(|e| AppError::InternalServerError)?;
    
    let validation_issues = manager.validate_migrations().await
        .map_err(|e| AppError::InternalServerError)?;

    let status = MigrationStatus {
        current_version,
        is_up_to_date,
        applied_migrations,
        validation_issues,
    };

    Ok(Json(status))
}

/// Run pending migrations
pub async fn run_migrations(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<MigrationResponse>> {
    tracing::info!("Admin triggered migration run");
    
    let manager = run_migrations_with_rollback(&state.db_pool).await
        .map_err(|e| AppError::InternalServerError)?;
    
    let applied_migrations = manager.get_migration_history().await
        .map_err(|e| AppError::InternalServerError)?;

    let response = MigrationResponse {
        success: true,
        message: format!("Successfully applied {} migrations", applied_migrations.len()),
        migrations_affected: applied_migrations,
    };

    Ok(Json(response))
}

/// Rollback to a specific migration version
pub async fn rollback_to_version(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RollbackRequest>,
) -> AppResult<Json<MigrationResponse>> {
    if !request.confirm {
        return Err(AppError::BadRequest(
            "Rollback must be confirmed with 'confirm: true'".to_string()
        ));
    }

    tracing::warn!(
        "Admin triggered rollback to version {}",
        request.target_version
    );

    let manager = create_migration_manager(&state).await?;
    
    match manager.rollback_to(request.target_version).await {
        Ok(rolled_back) => {
            let response = MigrationResponse {
                success: true,
                message: format!(
                    "Successfully rolled back {} migrations to version {}", 
                    rolled_back.len(), 
                    request.target_version
                ),
                migrations_affected: rolled_back,
            };
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Migration rollback failed: {}", e);
            Err(AppError::InternalServerError)
        }
    }
}

/// Rollback the last N migrations
pub async fn rollback_last_migrations(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RollbackLastRequest>,
) -> AppResult<Json<MigrationResponse>> {
    if !request.confirm {
        return Err(AppError::BadRequest(
            "Rollback must be confirmed with 'confirm: true'".to_string()
        ));
    }

    if request.count == 0 {
        return Err(AppError::BadRequest(
            "Count must be greater than 0".to_string()
        ));
    }

    tracing::warn!(
        "Admin triggered rollback of last {} migrations",
        request.count
    );

    let manager = create_migration_manager(&state).await?;
    
    match manager.rollback_last(request.count).await {
        Ok(rolled_back) => {
            let response = MigrationResponse {
                success: true,
                message: format!(
                    "Successfully rolled back {} migrations", 
                    rolled_back.len()
                ),
                migrations_affected: rolled_back,
            };
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Migration rollback failed: {}", e);
            Err(AppError::InternalServerError)
        }
    }
}

/// Validate migration integrity
pub async fn validate_migrations(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let manager = create_migration_manager(&state).await?;
    
    let issues = manager.validate_migrations().await
        .map_err(|e| AppError::InternalServerError)?;
    
    let applied_migrations = manager.get_migration_history().await
        .map_err(|e| AppError::InternalServerError)?;

    let response = serde_json::json!({
        "valid": issues.is_empty(),
        "total_migrations": applied_migrations.len(),
        "rollback_capable_count": applied_migrations.iter().filter(|m| m.can_rollback).count(),
        "issues": issues,
        "migrations": applied_migrations,
    });

    Ok(Json(response))
}

/// Create a database backup
pub async fn create_backup(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BackupRequest>,
) -> AppResult<Json<BackupResponse>> {
    let manager = create_migration_manager(&state).await?;
    
    let backup_path = manager.create_backup(&request.backup_name).await
        .map_err(|e| {
            tracing::error!("Backup creation failed: {}", e);
            AppError::InternalServerError
        })?;

    let response = BackupResponse {
        backup_path,
        timestamp: chrono::Utc::now(),
    };

    tracing::info!("Database backup created: {}", response.backup_path);
    Ok(Json(response))
}

/// Get detailed migration history
pub async fn get_migration_history(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<MigrationInfo>>> {
    let manager = create_migration_manager(&state).await?;
    
    let migrations = manager.get_migration_history().await
        .map_err(|e| AppError::InternalServerError)?;

    Ok(Json(migrations))
}

/// Create a dry-run rollback preview
pub async fn preview_rollback(
    State(state): State<Arc<AppState>>,
    Path(target_version): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    let manager = create_migration_manager(&state).await?;
    let applied_migrations = manager.get_migration_history().await
        .map_err(|e| AppError::InternalServerError)?;
    
    // Find migrations that would be rolled back
    let to_rollback: Vec<_> = applied_migrations
        .into_iter()
        .filter(|m| m.version > target_version)
        .collect();

    let can_rollback_all = to_rollback.iter().all(|m| m.can_rollback);
    let rollback_sql: Vec<_> = to_rollback.iter()
        .filter_map(|m| m.rollback_sql.as_ref().map(|sql| (m.version, sql)))
        .collect();

    let response = serde_json::json!({
        "target_version": target_version,
        "migrations_to_rollback": to_rollback,
        "can_rollback_all": can_rollback_all,
        "rollback_sql_count": rollback_sql.len(),
        "safe_to_proceed": can_rollback_all && !to_rollback.is_empty(),
        "warnings": if !can_rollback_all {
            vec!["Some migrations cannot be rolled back safely"]
        } else {
            vec![]
        }
    });

    Ok(Json(response))
}

// Helper function to create migration manager
async fn create_migration_manager(state: &AppState) -> AppResult<MigrationManager> {
    #[cfg(feature = "sqlite")]
    let migrations_path = "./migrations".to_string();
    #[cfg(feature = "postgres")]
    let migrations_path = "./migrations-postgres".to_string();
    
    let mut manager = MigrationManager::new(state.db_pool.clone(), migrations_path);
    
    // Register rollback scripts
    crate::db::register_rollback_scripts(&mut manager);
    
    Ok(manager)
}