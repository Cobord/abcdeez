use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{Session, CreateSessionRequest, TaskResponse},
    state::AppState,
};

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSessionRequest>,
) -> AppResult<(StatusCode, Json<Session>)> {
    // TODO: Create session and generate topology
    Err(AppError::InternalServerError)
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Session>> {
    // TODO: Get session details
    Err(AppError::NotFound)
}

pub async fn submit_response(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(response): Json<TaskResponse>,
) -> AppResult<StatusCode> {
    // TODO: Process response and update learner model
    Ok(StatusCode::NO_CONTENT)
}

pub async fn complete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // TODO: Mark session as completed
    Ok(StatusCode::NO_CONTENT)
}

pub async fn replay(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<TaskResponse>>> {
    // TODO: Get all responses for replay
    Ok(Json(vec![]))
}