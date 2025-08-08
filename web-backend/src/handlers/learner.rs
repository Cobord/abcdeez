use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{Learner, CreateLearnerRequest, UpdateLearnerRequest, LearnerStats},
    state::AppState,
};

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateLearnerRequest>,
) -> AppResult<(StatusCode, Json<Learner>)> {
    // TODO: Implement learner creation
    Err(AppError::InternalServerError)
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Learner>> {
    // TODO: Implement learner retrieval
    Err(AppError::NotFound)
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLearnerRequest>,
) -> AppResult<Json<Learner>> {
    // TODO: Implement learner update
    Err(AppError::NotFound)
}

pub async fn stats(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<LearnerStats>> {
    // TODO: Implement stats calculation
    Err(AppError::NotFound)
}

pub async fn export(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Implement GDPR data export
    Err(AppError::NotFound)
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // TODO: Implement GDPR deletion
    Ok(StatusCode::NO_CONTENT)
}