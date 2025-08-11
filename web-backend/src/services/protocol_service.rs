use chrono::Utc;
use semver::Version;
use serde_json::{json, Value};
use sqlx::Row;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::Config,
    db::{DbPool, uuid_to_db, json_to_db},
    error::{AppError, AppResult},
    models::{federation::ChangeType, protocol::*},
};

/// Service for managing protocol versions and branching
pub struct ProtocolService {
    db_pool: Arc<DbPool>,
    config: Arc<Config>,
}

impl ProtocolService {
    pub fn new(db_pool: Arc<DbPool>, config: Arc<Config>) -> Self {
        Self { db_pool, config }
    }

    /// Create a new protocol with initial version
    pub async fn create_protocol(
        &self,
        request: CreateProtocolRequest,
        created_by: Uuid,
    ) -> AppResult<CreateProtocolResponse> {
        let protocol_id = Uuid::new_v4();
        let version_id = Uuid::new_v4();
        let initial_version = "1.0.0";
        let now = Utc::now();

        let mut tx = self.db_pool.begin().await?;

        // Create protocol
        sqlx::query(
            "INSERT INTO protocols
             (id, name, description, experiment_id, created_by, tags, is_public, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(uuid_to_db(protocol_id))
        .bind(&request.name)
        .bind(&request.description)
        .bind(uuid_to_db(request.experiment_id))
        .bind(uuid_to_db(created_by))
        .bind(json_to_db(&request.tags)?)
        .bind(request.is_public)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        // Create initial version
        sqlx::query(
            "INSERT INTO protocol_versions
             (id, protocol_id, version, parent_version_id, status, definition, changelog,
              validation_rules, created_by, created_at, updated_at)
             VALUES (?, ?, ?, NULL, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid_to_db(version_id))
        .bind(uuid_to_db(protocol_id))
        .bind(initial_version)
        .bind("Draft")
        .bind(json_to_db(&request.initial_definition)?)
        .bind("Initial version")
        .bind(json_to_db(&json!({
                "required_fields": ["task_type", "parameters"],
                "max_duration_minutes": 60
            }))?)
        .bind(uuid_to_db(created_by))
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        // Set as current version
        sqlx::query("UPDATE protocols SET current_version_id = ?, updated_at = ? WHERE id = ?")
            .bind(uuid_to_db(version_id))
            .bind(now)
            .bind(uuid_to_db(protocol_id))
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        info!(
            "Created protocol {} with initial version {}",
            protocol_id, initial_version
        );

        Ok(CreateProtocolResponse {
            protocol_id,
            initial_version_id: version_id,
            version: initial_version.to_string(),
        })
    }

    /// Create a new version of a protocol
    pub async fn create_version(
        &self,
        request: CreateVersionRequest,
        created_by: Uuid,
    ) -> AppResult<Uuid> {
        // Validate semantic version
        let _version = Version::parse(&request.version)
            .map_err(|_| AppError::ValidationError("Invalid semantic version".to_string()))?;

        let version_id = Uuid::new_v4();
        let now = Utc::now();
        let mut conn = self.db_pool.acquire().await?;

        // Check if version already exists
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM protocol_versions WHERE protocol_id = ? AND version = ?)",
        )
        .bind(uuid_to_db(request.protocol_id))
        .bind(&request.version)
        .fetch_one(&mut *conn)
        .await?;

        if exists {
            return Err(AppError::Conflict(format!(
                "Version {} already exists",
                request.version
            )));
        }

        // Validate parent version if specified
        if let Some(parent_id) = request.base_version_id {
            let parent_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM protocol_versions WHERE id = ? AND protocol_id = ?)",
            )
            .bind(uuid_to_db(parent_id))
            .bind(uuid_to_db(request.protocol_id))
            .fetch_one(&mut *conn)
            .await?;

            if !parent_exists {
                return Err(AppError::NotFound("Parent version not found".to_string()));
            }
        }

        // Validate protocol definition
        let validation_result = self
            .validate_definition(&request.definition, &request.validation_rules)
            .await?;
        if !validation_result.is_valid {
            return Err(AppError::ValidationError(format!(
                "Protocol definition validation failed: {:?}",
                validation_result.errors
            )));
        }

        // Create version
        sqlx::query(
            "INSERT INTO protocol_versions
             (id, protocol_id, version, parent_version_id, status, definition, changelog,
              validation_rules, metadata, created_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid_to_db(version_id))
        .bind(uuid_to_db(request.protocol_id))
        .bind(&request.version)
        .bind(request.base_version_id.map(|id| uuid_to_db(id)))
        .bind("Draft")
        .bind(json_to_db(&request.definition)?)
        .bind(&request.changelog)
        .bind(match &request.validation_rules {
            Some(rules) => Some(json_to_db(rules)?),
            None => None
        })
        .bind(json_to_db(&json!({
                "created_from": request.base_version_id,
                "validation_result": validation_result
            }))?)
        .bind(uuid_to_db(created_by))
        .bind(now)
        .bind(now)
        .execute(&mut *conn)
        .await?;

        // Record change
        self.record_change(
            &mut conn,
            request.protocol_id,
            ChangeType::Created,
            &request.version,
            &request.changelog,
            &request.definition,
            created_by,
        )
        .await?;

        info!(
            "Created protocol version {} for protocol {}",
            request.version, request.protocol_id
        );

        Ok(version_id)
    }

    /// Publish a protocol version
    pub async fn publish_version(
        &self,
        request: PublishVersionRequest,
        published_by: Uuid,
    ) -> AppResult<()> {
        let now = Utc::now();
        let mut tx = self.db_pool.begin().await?;

        // Get version info
        let version_info = sqlx::query(
            "SELECT protocol_id, version, status, definition FROM protocol_versions WHERE id = ?",
        )
        .bind(uuid_to_db(request.version_id))
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("Version not found".to_string()))?;

        let protocol_id: Vec<u8> = version_info.get("protocol_id");
        let protocol_id = Uuid::from_slice(&protocol_id)?;
        let version: String = version_info.get("version");
        let status: String = version_info.get("status");
        let definition: String = version_info.get("definition");

        if status == "Published" {
            return Err(AppError::Conflict("Version already published".to_string()));
        }

        // Update version status
        sqlx::query(
            "UPDATE protocol_versions
             SET status = 'Published', published_at = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(now)
        .bind(now)
        .bind(uuid_to_db(request.version_id))
        .execute(&mut *tx)
        .await?;

        // Set as current version if requested
        if request.set_as_current {
            sqlx::query(
                "UPDATE protocols
                 SET current_version_id = ?, updated_at = ?
                 WHERE id = ?",
            )
            .bind(uuid_to_db(request.version_id))
            .bind(now)
            .bind(uuid_to_db(protocol_id))
            .execute(&mut *tx)
            .await?;
        }

        // Record change
        self.record_change(
            &mut *tx,
            protocol_id,
            ChangeType::Published,
            &version,
            "Version published",
            &serde_json::from_str(&definition)?,
            published_by,
        )
        .await?;

        tx.commit().await?;

        // Notify federation if requested
        if request.notify_federation {
            self.notify_federation_of_version(protocol_id, request.version_id)
                .await?;
        }

        info!(
            "Published protocol version {} for protocol {}",
            version, protocol_id
        );

        Ok(())
    }

    /// Compare two protocol versions
    pub async fn compare_versions(
        &self,
        request: CompareVersionsRequest,
    ) -> AppResult<CompareVersionsResponse> {
        let mut conn = self.db_pool.acquire().await?;

        // Get both versions
        let version_a = self
            .get_version_summary(&mut conn, request.version_a_id)
            .await?;
        let version_b = self
            .get_version_summary(&mut conn, request.version_b_id)
            .await?;

        // Get definitions for comparison
        let (def_a, def_b) = if request.include_definition_diff {
            let def_a: String =
                sqlx::query_scalar("SELECT definition FROM protocol_versions WHERE id = ?")
                    .bind(uuid_to_db(request.version_a_id))
                    .fetch_one(&mut *conn)
                    .await?;

            let def_b: String =
                sqlx::query_scalar("SELECT definition FROM protocol_versions WHERE id = ?")
                    .bind(uuid_to_db(request.version_b_id))
                    .fetch_one(&mut *conn)
                    .await?;

            (
                Some(serde_json::from_str(&def_a)?),
                Some(serde_json::from_str(&def_b)?),
            )
        } else {
            (None, None)
        };

        // Calculate differences
        let differences = if let (Some(def_a), Some(def_b)) = (&def_a, &def_b) {
            self.calculate_differences(def_a, def_b)
        } else {
            vec![]
        };

        // Generate definition diff if requested
        let definition_diff = if request.include_definition_diff {
            def_a.zip(def_b).map(|(a, b)| {
                json!({
                    "version_a": a,
                    "version_b": b,
                    "diff_summary": self.generate_diff_summary(&a, &b)
                })
            })
        } else {
            None
        };

        Ok(CompareVersionsResponse {
            version_a,
            version_b,
            differences,
            definition_diff,
        })
    }

    /// Get version history for a protocol
    pub async fn get_version_history(
        &self,
        request: GetVersionHistoryRequest,
    ) -> AppResult<VersionHistoryResponse> {
        let mut conn = self.db_pool.acquire().await?;

        // Build query based on filters
        let mut query = String::from(
            "SELECT id, version, status, parent_version_id, changelog, created_by,
                    created_at, published_at
             FROM protocol_versions
             WHERE protocol_id = ?",
        );

        if !request.include_deprecated {
            query.push_str(" AND status != 'Deprecated'");
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = request.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        // Get current version ID
        let current_version_id: Option<Vec<u8>> =
            sqlx::query_scalar("SELECT current_version_id FROM protocols WHERE id = ?")
                .bind(uuid_to_db(request.protocol_id))
                .fetch_optional(&mut *conn)
                .await?;

        let current_version_id = current_version_id.and_then(|bytes| Uuid::from_slice(&bytes).ok());

        // Get versions
        let rows = sqlx::query(&query)
            .bind(uuid_to_db(request.protocol_id))
            .fetch_all(&mut *conn)
            .await?;

        let versions: Vec<VersionHistoryItem> = rows
            .into_iter()
            .map(|row| {
                let id_bytes: Vec<u8> = row.get("id");
                let id = Uuid::from_slice(&id_bytes).unwrap();

                VersionHistoryItem {
                    id,
                    version: row.get("version"),
                    status: serde_json::from_str(row.get("status")).unwrap(),
                    parent_version_id: row
                        .get::<Option<Vec<u8>>, _>("parent_version_id")
                        .and_then(|bytes| Uuid::from_slice(&bytes).ok()),
                    changelog: row.get("changelog"),
                    created_by: Uuid::from_slice(&row.get::<Vec<u8>, _>("created_by")).unwrap(),
                    created_at: row.get("created_at"),
                    published_at: row.get("published_at"),
                    is_current: Some(id) == current_version_id,
                }
            })
            .collect();

        // Get branches if requested
        let branches = if request.include_branches {
            Some(
                self.get_protocol_branches(&mut conn, request.protocol_id)
                    .await?,
            )
        } else {
            None
        };

        // Get total count
        let total_versions: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM protocol_versions WHERE protocol_id = ?")
                .bind(uuid_to_db(request.protocol_id))
                .fetch_one(&mut *conn)
                .await?;

        Ok(VersionHistoryResponse {
            protocol_id: request.protocol_id,
            versions,
            branches,
            total_versions: total_versions as i32,
        })
    }

    /// Create a protocol branch
    pub async fn create_branch(
        &self,
        protocol_id: Uuid,
        name: String,
        base_version_id: Uuid,
        description: String,
        created_by: Uuid,
    ) -> AppResult<Uuid> {
        let branch_id = Uuid::new_v4();
        let now = Utc::now();
        let mut conn = self.db_pool.acquire().await?;

        sqlx::query(
            "INSERT INTO protocol_branches
             (id, protocol_id, name, base_version_id, description, created_by, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid_to_db(branch_id))
        .bind(uuid_to_db(protocol_id))
        .bind(&name)
        .bind(uuid_to_db(base_version_id))
        .bind(&description)
        .bind(uuid_to_db(created_by))
        .bind(now)
        .execute(&mut *conn)
        .await?;

        info!("Created branch {} for protocol {}", name, protocol_id);

        Ok(branch_id)
    }

    /// Validate a protocol definition
    pub async fn validate_protocol(
        &self,
        protocol_id: Uuid,
        version_id: Uuid,
    ) -> AppResult<ProtocolValidationResult> {
        let mut conn = self.db_pool.acquire().await?;

        // Get version definition and rules
        let version = sqlx::query(
            "SELECT definition, validation_rules FROM protocol_versions WHERE id = ? AND protocol_id = ?"
        )
        .bind(uuid_to_db(version_id))
        .bind(uuid_to_db(protocol_id))
        .fetch_optional(&mut *conn)
        .await?
        .ok_or(AppError::NotFound("Version not found".to_string()))?;

        let definition: String = version.get("definition");
        let validation_rules: Option<String> = version.get("validation_rules");

        let definition: Value = serde_json::from_str(&definition)?;
        let validation_rules: Option<Value> = validation_rules
            .as_ref()
            .and_then(|r| serde_json::from_str(r).ok());

        self.validate_definition(&definition, &validation_rules)
            .await
    }

    // Helper methods

    async fn validate_definition(
        &self,
        definition: &Value,
        rules: &Option<Value>,
    ) -> AppResult<ProtocolValidationResult> {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Check required fields
        if let Some(rules) = rules {
            if let Some(required_fields) = rules["required_fields"].as_array() {
                for field in required_fields {
                    if let Some(field_name) = field.as_str() {
                        if definition[field_name].is_null() {
                            errors.push(ValidationError {
                                field: field_name.to_string(),
                                error_type: "missing_required".to_string(),
                                message: format!("Required field '{}' is missing", field_name),
                            });
                        }
                    }
                }
            }
        }

        // Check task type
        if let Some(task_type) = definition["task_type"].as_str() {
            if ![
                "recognition",
                "discrimination",
                "production",
                "comprehension",
            ]
            .contains(&task_type)
            {
                warnings.push(ValidationWarning {
                    field: "task_type".to_string(),
                    warning_type: "unknown_type".to_string(),
                    message: format!("Unknown task type: {}", task_type),
                });
            }
        }

        // Additional validation logic...

        Ok(ProtocolValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            validated_at: Utc::now(),
        })
    }

    async fn get_version_summary(
        &self,
        conn: &mut crate::db::DbConnection,
        version_id: Uuid,
    ) -> AppResult<VersionSummary> {
        let row = sqlx::query(
            "SELECT id, version, status, created_at, published_at
             FROM protocol_versions WHERE id = ?",
        )
        .bind(uuid_to_db(version_id))
        .fetch_optional(conn)
        .await?
        .ok_or(AppError::NotFound("Version not found".to_string()))?;

        Ok(VersionSummary {
            id: version_id,
            version: row.get("version"),
            status: serde_json::from_str(row.get("status")).unwrap(),
            created_at: row.get("created_at"),
            published_at: row.get("published_at"),
        })
    }

    fn calculate_differences(&self, def_a: &Value, def_b: &Value) -> Vec<VersionDifference> {
        let mut differences = vec![];

        // Compare top-level fields
        if let (Some(obj_a), Some(obj_b)) = (def_a.as_object(), def_b.as_object()) {
            for (key, value_a) in obj_a {
                if let Some(value_b) = obj_b.get(key) {
                    if value_a != value_b {
                        differences.push(VersionDifference {
                            field: key.clone(),
                            change_type: DiffChangeType::Modified,
                            old_value: Some(value_a.clone()),
                            new_value: Some(value_b.clone()),
                            description: format!("Field '{}' was modified", key),
                        });
                    }
                } else {
                    differences.push(VersionDifference {
                        field: key.clone(),
                        change_type: DiffChangeType::Removed,
                        old_value: Some(value_a.clone()),
                        new_value: None,
                        description: format!("Field '{}' was removed", key),
                    });
                }
            }

            for (key, value_b) in obj_b {
                if !obj_a.contains_key(key) {
                    differences.push(VersionDifference {
                        field: key.clone(),
                        change_type: DiffChangeType::Added,
                        old_value: None,
                        new_value: Some(value_b.clone()),
                        description: format!("Field '{}' was added", key),
                    });
                }
            }
        }

        differences
    }

    fn generate_diff_summary(&self, def_a: &Value, def_b: &Value) -> Value {
        json!({
            "fields_added": self.count_added_fields(def_a, def_b),
            "fields_removed": self.count_removed_fields(def_a, def_b),
            "fields_modified": self.count_modified_fields(def_a, def_b),
        })
    }

    fn count_added_fields(&self, def_a: &Value, def_b: &Value) -> usize {
        if let (Some(obj_a), Some(obj_b)) = (def_a.as_object(), def_b.as_object()) {
            obj_b.keys().filter(|k| !obj_a.contains_key(*k)).count()
        } else {
            0
        }
    }

    fn count_removed_fields(&self, def_a: &Value, def_b: &Value) -> usize {
        if let (Some(obj_a), Some(obj_b)) = (def_a.as_object(), def_b.as_object()) {
            obj_a.keys().filter(|k| !obj_b.contains_key(*k)).count()
        } else {
            0
        }
    }

    fn count_modified_fields(&self, def_a: &Value, def_b: &Value) -> usize {
        if let (Some(obj_a), Some(obj_b)) = (def_a.as_object(), def_b.as_object()) {
            obj_a
                .keys()
                .filter(|k| obj_b.contains_key(*k) && obj_a[*k] != obj_b[*k])
                .count()
        } else {
            0
        }
    }

    async fn record_change(
        &self,
        conn: &mut crate::db::DbConnection,
        protocol_id: Uuid,
        change_type: ChangeType,
        version: &str,
        description: &str,
        data: &Value,
        changed_by: Uuid,
    ) -> AppResult<()> {
        let change_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO protocol_changes
             (id, protocol_id, change_type, version, description, data, changed_at, changed_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uuid_to_db(change_id))
        .bind(uuid_to_db(protocol_id))
        .bind(format!("{:?}", change_type))
        .bind(version)
        .bind(description)
        .bind(json_to_db(&data)?)
        .bind(Utc::now())
        .bind(changed_by)
        .execute(conn)
        .await?;

        Ok(())
    }

    async fn get_protocol_branches(
        &self,
        conn: &mut crate::db::DbConnection,
        protocol_id: Uuid,
    ) -> AppResult<Vec<BranchInfo>> {
        let rows = sqlx::query(
            "SELECT b.id, b.name, v.version as base_version, b.created_at,
                    b.merged_into_version_id
             FROM protocol_branches b
             JOIN protocol_versions v ON b.base_version_id = v.id
             WHERE b.protocol_id = ?
             ORDER BY b.created_at DESC",
        )
        .bind(uuid_to_db(protocol_id))
        .fetch_all(conn)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let id_bytes: Vec<u8> = row.get("id");
                BranchInfo {
                    id: Uuid::from_slice(&id_bytes).unwrap(),
                    name: row.get("name"),
                    base_version: row.get("base_version"),
                    created_at: row.get("created_at"),
                    is_merged: row
                        .get::<Option<Vec<u8>>, _>("merged_into_version_id")
                        .is_some(),
                }
            })
            .collect())
    }

    async fn notify_federation_of_version(
        &self,
        protocol_id: Uuid,
        version_id: Uuid,
    ) -> AppResult<()> {
        // TODO: Implement federation notification
        info!(
            "Would notify federation of new version {} for protocol {}",
            version_id, protocol_id
        );
        Ok(())
    }
}
