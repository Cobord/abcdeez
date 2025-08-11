use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{error::AppError, models::preregistration::*, state::AppState};
use sqlx::Row;

/// Query parameters for listing pre-registrations
#[derive(Debug, Deserialize)]
pub struct ListPreRegistrationsQuery {
    pub experiment_id: Option<String>,
    pub researcher_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Create a new pre-registration
pub async fn create_preregistration(
    State(state): State<AppState>,
    Json(payload): Json<CreatePreRegistration>,
) -> Result<Json<PreRegistrationResponse>, AppError> {
    let id = format!("prereg_{}", Uuid::new_v4());
    let now = Utc::now();

    // Serialize complex fields to JSON
    let study_metadata_json = serde_json::to_string(&payload.study_metadata)?;
    let hypotheses_json = serde_json::to_string(&payload.hypotheses)?;
    let analysis_plan_json = serde_json::to_string(&payload.analysis_plan)?;
    let data_collection_json = serde_json::to_string(&payload.data_collection_plan)?;
    let exclusion_criteria_json = serde_json::to_string(&payload.exclusion_criteria)?;
    let decision_rules_json = serde_json::to_string(&payload.decision_rules)?;

    // Insert into database
    sqlx::query(
        r#"
        INSERT INTO preregistrations (
            id, experiment_id, researcher_id, title, description,
            registered_at, registration_hash, status,
            study_metadata, hypotheses, analysis_plan,
            data_collection_plan, exclusion_criteria, decision_rules,
            version, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8,
            ?9, ?10, ?11,
            ?12, ?13, ?14,
            ?15, ?16, ?17
        )
        "#,
    )
    .bind(&id)
    .bind(&payload.experiment_id)
    .bind("researcher_1") // TODO: Get from auth context
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(now)
    .bind("") // Hash will be set when finalized
    .bind("draft")
    .bind(study_metadata_json)
    .bind(hypotheses_json)
    .bind(analysis_plan_json)
    .bind(data_collection_json)
    .bind(exclusion_criteria_json)
    .bind(decision_rules_json)
    .bind(1i64)
    .bind(now)
    .bind(now)
    .execute(&state.db_pool)
    .await?;

    Ok(Json(PreRegistrationResponse {
        id: id.clone(),
        experiment_id: payload.experiment_id,
        title: payload.title,
        description: payload.description,
        status: "draft".to_string(),
        registered_at: None,
        registration_hash: None,
        can_edit: true,
        transparency_report: None,
    }))
}

/// Get a pre-registration by ID
pub async fn get_preregistration(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PreRegistrationResponse>, AppError> {
    let prereg = sqlx::query_as::<_, PreRegistrationDb>(
        "SELECT * FROM preregistrations WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(&state.db_pool)
    .await?;

    // Parse JSON fields
    let hypotheses: Hypotheses = serde_json::from_str(&prereg.hypotheses)?;
    let analysis_plan: AnalysisPlan = serde_json::from_str(&prereg.analysis_plan)?;

    // Get deviations count
    let deviations_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM preregistration_deviations WHERE preregistration_id = ?",
    )
    .bind(&id)
    .fetch_one(&state.db_pool)
    .await?;

    // Generate transparency report if registered
    let transparency_report = if prereg.status != "draft" {
        Some(TransparencyReport {
            registration_id: prereg.id.clone(),
            registered_at: prereg.registered_at,
            registration_hash: prereg.registration_hash.clone(),
            n_primary_hypotheses: hypotheses.primary.len(),
            n_secondary_hypotheses: hypotheses.secondary.len(),
            n_primary_analyses: analysis_plan.primary_analyses.len(),
            n_secondary_analyses: analysis_plan.secondary_analyses.len(),
            n_deviations: deviations_count as usize,
            deviation_descriptions: vec![], // TODO: Fetch actual deviations
            status: prereg.status.clone(),
        })
    } else {
        None
    };

    Ok(Json(PreRegistrationResponse {
        id: prereg.id,
        experiment_id: prereg.experiment_id,
        title: prereg.title,
        description: prereg.description,
        status: prereg.status.clone(),
        registered_at: if prereg.status != "draft" {
            Some(prereg.registered_at)
        } else {
            None
        },
        registration_hash: if prereg.status != "draft" {
            Some(prereg.registration_hash)
        } else {
            None
        },
        can_edit: prereg.status == "draft",
        transparency_report,
    }))
}

/// Update a pre-registration (only if in draft status)
pub async fn update_preregistration(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePreRegistration>,
) -> Result<Json<PreRegistrationResponse>, AppError> {
    // Check if pre-registration exists and is in draft status
    let current_status: String = sqlx::query_scalar(
        "SELECT status FROM preregistrations WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(&state.db_pool)
    .await?;

    if current_status != "draft" {
        return Err(AppError::BadRequest(
            "Cannot edit finalized pre-registration".to_string(),
        ));
    }

    // Update each field individually if provided
    if let Some(title) = &payload.title {
        sqlx::query("UPDATE preregistrations SET title = ? WHERE id = ?")
            .bind(title)
            .bind(&id)
            .execute(&state.db_pool)
            .await?;
    }

    if let Some(description) = &payload.description {
        sqlx::query("UPDATE preregistrations SET description = ? WHERE id = ?")
            .bind(description)
            .bind(&id)
            .execute(&state.db_pool)
            .await?;
    }

    if let Some(hypotheses) = &payload.hypotheses {
        let json = serde_json::to_string(hypotheses)?;
        sqlx::query("UPDATE preregistrations SET hypotheses = ? WHERE id = ?")
            .bind(json)
            .bind(&id)
            .execute(&state.db_pool)
            .await?;
    }

    // Update timestamp
    let updated_now = Utc::now();
    sqlx::query("UPDATE preregistrations SET updated_at = ? WHERE id = ?")
        .bind(updated_now)
        .bind(&id)
        .execute(&state.db_pool)
        .await?;

    get_preregistration(State(state), Path(id)).await
}

/// Finalize a pre-registration (lock it with hash)
pub async fn finalize_preregistration(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PreRegistrationResponse>, AppError> {
    // Fetch the current pre-registration
    let prereg = sqlx::query_as::<_, PreRegistrationDb>(
        "SELECT * FROM preregistrations WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(&state.db_pool)
    .await?;

    if prereg.status != "draft" {
        return Err(AppError::BadRequest(
            "Pre-registration already finalized".to_string(),
        ));
    }

    // Validate required fields
    let hypotheses: Hypotheses = serde_json::from_str(&prereg.hypotheses)?;
    let analysis_plan: AnalysisPlan = serde_json::from_str(&prereg.analysis_plan)?;
    let data_collection: DataCollectionPlan = serde_json::from_str(&prereg.data_collection_plan)?;

    if hypotheses.primary.is_empty() {
        return Err(AppError::BadRequest(
            "At least one primary hypothesis required".to_string(),
        ));
    }

    if analysis_plan.primary_analyses.is_empty() {
        return Err(AppError::BadRequest(
            "At least one primary analysis required".to_string(),
        ));
    }

    if data_collection.target_sample_size == 0 {
        return Err(AppError::BadRequest(
            "Target sample size must be specified".to_string(),
        ));
    }

    // Generate hash of the content
    let content_to_hash = format!(
        "{}{}{}{}{}{}{}",
        prereg.title,
        prereg.description,
        prereg.hypotheses,
        prereg.analysis_plan,
        prereg.data_collection_plan,
        prereg.exclusion_criteria,
        prereg.decision_rules
    );

    let mut hasher = Sha256::new();
    hasher.update(content_to_hash.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let now = Utc::now();

    // Update status and hash
    sqlx::query(
        "UPDATE preregistrations SET status = ?, registration_hash = ?, registered_at = ? WHERE id = ?",
    )
    .bind("registered")
    .bind(&hash)
    .bind(now)
    .bind(&id)
    .execute(&state.db_pool)
    .await?;

    get_preregistration(State(state), Path(id)).await
}

/// Record a deviation from the pre-registration
pub async fn record_deviation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<RecordDeviation>,
) -> Result<StatusCode, AppError> {
    let deviation_id = format!("dev_{}", Uuid::new_v4());

    let ts_now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO preregistration_deviations (
            id, preregistration_id, timestamp, description,
            justification, impact_assessment, created_by
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(&deviation_id)
    .bind(&id)
    .bind(ts_now)
    .bind(&payload.description)
    .bind(&payload.justification)
    .bind(&payload.impact_assessment)
    .bind("researcher_1") // TODO: Get from auth context
    .execute(&state.db_pool)
    .await?;

    Ok(StatusCode::CREATED)
}

/// Validate an analysis against the pre-registration
pub async fn validate_analysis(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ValidateAnalysis>,
) -> Result<Json<ValidationResponse>, AppError> {
    // Fetch the pre-registration
    let row = sqlx::query("SELECT analysis_plan FROM preregistrations WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db_pool)
        .await?;
    let analysis_plan_str: String = row.try_get("analysis_plan")?;
    let analysis_plan: AnalysisPlan = serde_json::from_str(&analysis_plan_str)?;

    // Check if analysis is pre-registered
    let is_primary = analysis_plan
        .primary_analyses
        .iter()
        .any(|a| a.name == payload.analysis_name);

    let is_secondary = analysis_plan
        .secondary_analyses
        .iter()
        .any(|a| a.name == payload.analysis_name);

    let (validation_result, is_exploratory, deviation_reason) = if is_primary {
        // Validate against primary analysis specification
        let planned = analysis_plan
            .primary_analyses
            .iter()
            .find(|a| a.name == payload.analysis_name)
            .unwrap();

        let matches = planned.statistical_model == payload.actual_test
            && planned.independent_variables == payload.actual_variables;

        if matches {
            ("valid".to_string(), false, None)
        } else {
            let reason = format!(
                "Deviation from plan. Expected: {} with {:?}, Got: {} with {:?}",
                planned.statistical_model,
                planned.independent_variables,
                payload.actual_test,
                payload.actual_variables
            );
            ("deviation".to_string(), false, Some(reason))
        }
    } else if is_secondary {
        ("valid".to_string(), false, None)
    } else {
        ("not_preregistered".to_string(), true, None)
    };

    // Record validation
    let validation_id = format!("val_{}", Uuid::new_v4());
    let variables_json = serde_json::to_string(&payload.actual_variables)?;

    sqlx::query(
        r#"
        INSERT INTO analysis_validations (
            id, preregistration_id, analysis_name, validation_result,
            actual_test, actual_variables, deviation_reason, is_exploratory
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&validation_id)
    .bind(&id)
    .bind(&payload.analysis_name)
    .bind(&validation_result)
    .bind(&payload.actual_test)
    .bind(&variables_json)
    .bind(&deviation_reason)
    .bind(is_exploratory)
    .execute(&state.db_pool)
    .await?;

    Ok(Json(ValidationResponse {
        is_preregistered: is_primary || is_secondary,
        is_primary,
        is_exploratory,
        validation_result: validation_result.clone(),
        deviation_reason,
    }))
}

/// List pre-registrations with filtering
pub async fn list_preregistrations(
    State(state): State<AppState>,
    Query(params): Query<ListPreRegistrationsQuery>,
) -> Result<Json<Vec<PreRegistrationResponse>>, AppError> {
    // Build parameterized query to prevent SQL injection
    let mut sql = String::from(
        "SELECT * FROM preregistrations WHERE 1=1"
    );
    let mut bindings: Vec<String> = Vec::new();
    let mut param_count = 0;

    if let Some(exp_id) = params.experiment_id {
        param_count += 1;
        sql.push_str(&format!(" AND experiment_id = ?{}", param_count));
        bindings.push(exp_id);
    }

    if let Some(researcher_id) = params.researcher_id {
        param_count += 1;
        sql.push_str(&format!(" AND researcher_id = ?{}", param_count));
        bindings.push(researcher_id);
    }

    if let Some(status) = params.status {
        param_count += 1;
        sql.push_str(&format!(" AND status = ?{}", param_count));
        bindings.push(status);
    }

    sql.push_str(" ORDER BY created_at DESC");

    // Add pagination with bounds checking
    let limit = params.limit.unwrap_or(100).min(1000).max(1);
    let offset = params.offset.unwrap_or(0).max(0);
    
    param_count += 1;
    sql.push_str(&format!(" LIMIT ?{}", param_count));
    bindings.push(limit.to_string());
    
    param_count += 1;
    sql.push_str(&format!(" OFFSET ?{}", param_count));
    bindings.push(offset.to_string());

    // Execute with parameterized query
    let mut query = sqlx::query_as::<_, PreRegistrationDb>(&sql);
    for binding in bindings {
        query = query.bind(binding);
    }
    
    let preregistrations = query
        .fetch_all(&state.db_pool)
        .await?;

    let responses: Vec<PreRegistrationResponse> = preregistrations
        .into_iter()
        .map(|p| PreRegistrationResponse {
            id: p.id,
            experiment_id: p.experiment_id,
            title: p.title,
            description: p.description,
            status: p.status.clone(),
            registered_at: if p.status != "draft" {
                Some(p.registered_at)
            } else {
                None
            },
            registration_hash: if p.status != "draft" {
                Some(p.registration_hash)
            } else {
                None
            },
            can_edit: p.status == "draft",
            transparency_report: None,
        })
        .collect();

    Ok(Json(responses))
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct RecordDeviation {
    pub description: String,
    pub justification: String,
    pub impact_assessment: String,
}

#[derive(Debug, Deserialize)]
pub struct ValidateAnalysis {
    pub analysis_name: String,
    pub actual_test: String,
    pub actual_variables: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub is_preregistered: bool,
    pub is_primary: bool,
    pub is_exploratory: bool,
    pub validation_result: String,
    pub deviation_reason: Option<String>,
}
