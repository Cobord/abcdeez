use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::Claims,
    models::protocol::*,
    services::protocol_service::ProtocolService,
    state::AppState,
};

/// Create a new protocol with initial version
pub async fn create_protocol(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateProtocolRequest>,
) -> AppResult<(StatusCode, Json<CreateProtocolResponse>)> {
    let protocol_service =
        ProtocolService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let response = protocol_service
        .create_protocol(request, claims.sub)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// List all protocols
pub async fn list_protocols(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<Vec<Protocol>>> {
    let mut conn = state.db_pool.acquire().await?;

    let rows = sqlx::query(
        "SELECT id, name, description, experiment_id, current_version_id, created_by,
                tags, is_public, created_at, updated_at
         FROM protocols
         ORDER BY updated_at DESC",
    )
    .fetch_all(&mut *conn)
    .await?;

    let protocols: Vec<Protocol> = rows
        .into_iter()
        .map(|row| {
            let id_bytes: Vec<u8> = row.get("id");
            let experiment_bytes: Vec<u8> = row.get("experiment_id");
            let created_by_bytes: Vec<u8> = row.get("created_by");
            let tags_str: String = row.get("tags");

            Protocol {
                id: Uuid::from_slice(&id_bytes).unwrap(),
                name: row.get("name"),
                description: row.get("description"),
                experiment_id: Uuid::from_slice(&experiment_bytes).unwrap(),
                current_version_id: row
                    .get::<Option<Vec<u8>>, _>("current_version_id")
                    .and_then(|bytes| Uuid::from_slice(&bytes).ok()),
                created_by: Uuid::from_slice(&created_by_bytes).unwrap(),
                tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                is_public: row.get("is_public"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect();

    Ok(Json(protocols))
}

/// Get a specific protocol
pub async fn get_protocol(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(protocol_id): Path<Uuid>,
) -> AppResult<Json<Protocol>> {
    let mut conn = state.db_pool.acquire().await?;

    let row = sqlx::query(
        "SELECT id, name, description, experiment_id, current_version_id, created_by,
                tags, is_public, created_at, updated_at
         FROM protocols WHERE id = ?",
    )
    .bind(protocol_id.as_bytes().as_slice())
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(AppError::NotFound("Protocol not found".to_string()))?;

    let experiment_bytes: Vec<u8> = row.get("experiment_id");
    let created_by_bytes: Vec<u8> = row.get("created_by");
    let tags_str: String = row.get("tags");

    let protocol = Protocol {
        id: protocol_id,
        name: row.get("name"),
        description: row.get("description"),
        experiment_id: Uuid::from_slice(&experiment_bytes).unwrap(),
        current_version_id: row
            .get::<Option<Vec<u8>>, _>("current_version_id")
            .and_then(|bytes| Uuid::from_slice(&bytes).ok()),
        created_by: Uuid::from_slice(&created_by_bytes).unwrap(),
        tags: serde_json::from_str(&tags_str).unwrap_or_default(),
        is_public: row.get("is_public"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(protocol))
}

/// Create a new version of a protocol
pub async fn create_version(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(protocol_id): Path<Uuid>,
    Json(mut request): Json<CreateVersionRequest>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    request.protocol_id = protocol_id;

    let protocol_service =
        ProtocolService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let version_id = protocol_service.create_version(request, claims.sub).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "version_id": version_id,
            "protocol_id": protocol_id
        })),
    ))
}

/// List all versions of a protocol
pub async fn list_versions(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(protocol_id): Path<Uuid>,
) -> AppResult<Json<Vec<ProtocolVersion>>> {
    let mut conn = state.db_pool.acquire().await?;

    let rows = sqlx::query(
        "SELECT id, protocol_id, version, parent_version_id, status, definition, changelog,
                validation_rules, metadata, created_by, published_at, deprecated_at,
                created_at, updated_at
         FROM protocol_versions
         WHERE protocol_id = ?
         ORDER BY created_at DESC",
    )
    .bind(protocol_id.as_bytes().as_slice())
    .fetch_all(&mut *conn)
    .await?;

    let versions: Vec<ProtocolVersion> = rows
        .into_iter()
        .map(|row| {
            let id_bytes: Vec<u8> = row.get("id");
            let created_by_bytes: Vec<u8> = row.get("created_by");
            let definition_str: String = row.get("definition");
            let validation_str: Option<String> = row.get("validation_rules");
            let metadata_str: Option<String> = row.get("metadata");

            ProtocolVersion {
                id: Uuid::from_slice(&id_bytes).unwrap(),
                protocol_id,
                version: row.get("version"),
                parent_version_id: row
                    .get::<Option<Vec<u8>>, _>("parent_version_id")
                    .and_then(|bytes| Uuid::from_slice(&bytes).ok()),
                status: match row.get::<String, _>("status").as_str() {
                    "Draft" => ProtocolStatus::Draft,
                    "Review" => ProtocolStatus::Review,
                    "Published" => ProtocolStatus::Published,
                    "Deprecated" => ProtocolStatus::Deprecated,
                    "Archived" => ProtocolStatus::Archived,
                    _ => ProtocolStatus::Draft,
                },
                definition: serde_json::from_str(&definition_str).unwrap_or_default(),
                changelog: row.get("changelog"),
                validation_rules: validation_str.and_then(|s| serde_json::from_str(&s).ok()),
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
                created_by: Uuid::from_slice(&created_by_bytes).unwrap(),
                published_at: row.get("published_at"),
                deprecated_at: row.get("deprecated_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect();

    Ok(Json(versions))
}

/// Get a specific version
pub async fn get_version(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((protocol_id, version_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<ProtocolVersion>> {
    let mut conn = state.db_pool.acquire().await?;

    let row = sqlx::query(
        "SELECT id, protocol_id, version, parent_version_id, status, definition, changelog,
                validation_rules, metadata, created_by, published_at, deprecated_at,
                created_at, updated_at
         FROM protocol_versions
         WHERE id = ? AND protocol_id = ?",
    )
    .bind(version_id.as_bytes().as_slice())
    .bind(protocol_id.as_bytes().as_slice())
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(AppError::NotFound("Version not found".to_string()))?;

    let created_by_bytes: Vec<u8> = row.get("created_by");
    let definition_str: String = row.get("definition");
    let validation_str: Option<String> = row.get("validation_rules");
    let metadata_str: Option<String> = row.get("metadata");

    let version = ProtocolVersion {
        id: version_id,
        protocol_id,
        version: row.get("version"),
        parent_version_id: row
            .get::<Option<Vec<u8>>, _>("parent_version_id")
            .and_then(|bytes| Uuid::from_slice(&bytes).ok()),
        status: match row.get::<String, _>("status").as_str() {
            "Draft" => ProtocolStatus::Draft,
            "Review" => ProtocolStatus::Review,
            "Published" => ProtocolStatus::Published,
            "Deprecated" => ProtocolStatus::Deprecated,
            "Archived" => ProtocolStatus::Archived,
            _ => ProtocolStatus::Draft,
        },
        definition: serde_json::from_str(&definition_str).unwrap_or_default(),
        changelog: row.get("changelog"),
        validation_rules: validation_str.and_then(|s| serde_json::from_str(&s).ok()),
        metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
        created_by: Uuid::from_slice(&created_by_bytes).unwrap(),
        published_at: row.get("published_at"),
        deprecated_at: row.get("deprecated_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(version))
}

/// Publish a protocol version
pub async fn publish_version(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((protocol_id, version_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<PublishVersionRequest>,
) -> AppResult<StatusCode> {
    let protocol_service =
        ProtocolService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let mut full_request = request;
    full_request.version_id = version_id;

    protocol_service
        .publish_version(full_request, claims.sub)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Compare two versions
pub async fn compare_versions(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(protocol_id): Path<Uuid>,
    Json(request): Json<CompareVersionsRequest>,
) -> AppResult<Json<CompareVersionsResponse>> {
    let protocol_service =
        ProtocolService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let response = protocol_service.compare_versions(request).await?;

    Ok(Json(response))
}

/// Get version history
pub async fn version_history(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(protocol_id): Path<Uuid>,
    Json(request): Json<GetVersionHistoryRequest>,
) -> AppResult<Json<VersionHistoryResponse>> {
    let mut full_request = request;
    full_request.protocol_id = protocol_id;

    let protocol_service =
        ProtocolService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let response = protocol_service.get_version_history(full_request).await?;

    Ok(Json(response))
}

/// Create a protocol branch
pub async fn create_branch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(protocol_id): Path<Uuid>,
    Json(request): Json<serde_json::Value>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let name = request["name"]
        .as_str()
        .ok_or(AppError::ValidationError(
            "Branch name required".to_string(),
        ))?
        .to_string();

    let base_version_id: Uuid = request["base_version_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or(AppError::ValidationError(
            "Invalid base_version_id".to_string(),
        ))?;

    let description = request["description"]
        .as_str()
        .unwrap_or("Branch created")
        .to_string();

    let protocol_service =
        ProtocolService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let branch_id = protocol_service
        .create_branch(
            protocol_id,
            name.clone(),
            base_version_id,
            description,
            claims.sub,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "branch_id": branch_id,
            "protocol_id": protocol_id,
            "name": name
        })),
    ))
}

/// Validate a protocol
pub async fn validate_protocol(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((protocol_id, version_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<ProtocolValidationResult>> {
    let protocol_service =
        ProtocolService::new(Arc::new(state.db_pool.clone()), state.config.clone());

    let result = protocol_service
        .validate_protocol(protocol_id, version_id)
        .await?;

    Ok(Json(result))
}

/// Get protocol metrics
pub async fn get_metrics(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(protocol_id): Path<Uuid>,
) -> AppResult<Json<ProtocolMetrics>> {
    let mut conn = state.db_pool.acquire().await?;

    // Get current version
    let current_version_id: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT current_version_id FROM protocols WHERE id = ?")
            .bind(protocol_id.as_bytes().as_slice())
            .fetch_optional(&mut *conn)
            .await?;

    let version_id = current_version_id
        .and_then(|bytes| Uuid::from_slice(&bytes).ok())
        .ok_or(AppError::NotFound("No current version found".to_string()))?;

    // Calculate metrics
    let metrics = sqlx::query(
        "SELECT
            COUNT(DISTINCT s.id) as total_sessions,
            COUNT(DISTINCT s.learner_id) as active_learners,
            AVG(CASE WHEN s.status = 'completed' THEN 1.0 ELSE 0.0 END) as avg_completion_rate,
            AVG(CASE WHEN tr.correct = 1 THEN 1.0 ELSE 0.0 END) as avg_success_rate,
            MAX(s.created_at) as last_used
         FROM sessions s
         LEFT JOIN task_responses tr ON s.id = tr.session_id
         WHERE s.protocol_id = ?",
    )
    .bind(protocol_id.as_bytes().as_slice())
    .fetch_one(&mut *conn)
    .await?;

    let protocol_metrics = ProtocolMetrics {
        protocol_id,
        version_id,
        total_sessions: metrics.get::<Option<i64>, _>("total_sessions").unwrap_or(0) as i32,
        active_learners: metrics
            .get::<Option<i64>, _>("active_learners")
            .unwrap_or(0) as i32,
        avg_completion_rate: metrics
            .get::<Option<f64>, _>("avg_completion_rate")
            .unwrap_or(0.0),
        avg_success_rate: metrics
            .get::<Option<f64>, _>("avg_success_rate")
            .unwrap_or(0.0),
        last_used: metrics.get("last_used"),
        calculated_at: chrono::Utc::now(),
    };

    Ok(Json(protocol_metrics))
}
