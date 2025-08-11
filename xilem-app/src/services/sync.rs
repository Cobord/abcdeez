// Offline sync service for queueing and syncing data with the backend

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::{PendingResponse, Response};
use crate::services::api::ApiClient;
use crate::services::storage::StorageService;

const MAX_QUEUE_SIZE: usize = 1000;
const SYNC_BATCH_SIZE: usize = 50;

#[derive(Debug, Clone)]
pub struct SyncService {
    queue: Arc<Mutex<VecDeque<PendingResponse>>>,
    storage: Arc<StorageService>,
    api_client: Arc<Mutex<Option<ApiClient>>>,
    syncing: Arc<Mutex<bool>>,
    last_sync: Arc<Mutex<Option<DateTime<Utc>>>>,
}

impl SyncService {
    pub fn new(storage: Arc<StorageService>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_QUEUE_SIZE))),
            storage,
            api_client: Arc::new(Mutex::new(None)),
            syncing: Arc::new(Mutex::new(false)),
            last_sync: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_api_client(&self, client: ApiClient) {
        *self.api_client.lock().await = Some(client);
    }

    pub async fn add_to_queue(&self, response: PendingResponse) -> Result<()> {
        let mut queue = self.queue.lock().await;
        
        // Check queue size limit
        if queue.len() >= MAX_QUEUE_SIZE {
            // Remove oldest items if queue is full
            queue.pop_front();
        }
        
        queue.push_back(response.clone());
        
        // Also persist to storage for crash recovery
        self.storage.save_pending_response(&response).await?;
        
        Ok(())
    }

    pub async fn load_queue_from_storage(&self) -> Result<()> {
        let pending_responses = self.storage.get_all_pending_responses().await?;
        let mut queue = self.queue.lock().await;
        
        for response in pending_responses {
            if queue.len() < MAX_QUEUE_SIZE {
                queue.push_back(response);
            }
        }
        
        Ok(())
    }

    pub async fn sync_all(&self) -> Result<SyncResult> {
        // Check if already syncing
        let mut syncing = self.syncing.lock().await;
        if *syncing {
            return Ok(SyncResult {
                synced_count: 0,
                failed_count: 0,
                remaining_count: self.queue.lock().await.len(),
                errors: vec!["Sync already in progress".to_string()],
            });
        }
        *syncing = true;
        drop(syncing);

        let result = self.perform_sync().await;

        *self.syncing.lock().await = false;
        
        result
    }

    async fn perform_sync(&self) -> Result<SyncResult> {
        let api_client = self.api_client.lock().await;
        let api_client = match api_client.as_ref() {
            Some(client) => client,
            None => {
                return Ok(SyncResult {
                    synced_count: 0,
                    failed_count: 0,
                    remaining_count: self.queue.lock().await.len(),
                    errors: vec!["No API client configured".to_string()],
                });
            }
        };

        let mut synced_count = 0;
        let mut failed_count = 0;
        let mut errors = Vec::new();

        // Process queue in batches
        loop {
            let batch = self.get_next_batch().await;
            if batch.is_empty() {
                break;
            }

            match self.sync_batch(api_client, &batch).await {
                Ok(batch_result) => {
                    synced_count += batch_result.synced_count;
                    failed_count += batch_result.failed_count;
                    
                    // Remove successfully synced items from queue and storage
                    for response in &batch[..batch_result.synced_count] {
                        self.storage.delete_pending_response(&response.id).await?;
                    }
                    
                    if !batch_result.errors.is_empty() {
                        errors.extend(batch_result.errors);
                    }
                }
                Err(e) => {
                    errors.push(format!("Batch sync failed: {}", e));
                    failed_count += batch.len();
                    break; // Stop syncing on batch failure
                }
            }
        }

        *self.last_sync.lock().await = Some(Utc::now());

        Ok(SyncResult {
            synced_count,
            failed_count,
            remaining_count: self.queue.lock().await.len(),
            errors,
        })
    }

    async fn get_next_batch(&self) -> Vec<PendingResponse> {
        let mut queue = self.queue.lock().await;
        let mut batch = Vec::with_capacity(SYNC_BATCH_SIZE);
        
        for _ in 0..SYNC_BATCH_SIZE {
            if let Some(response) = queue.pop_front() {
                batch.push(response);
            } else {
                break;
            }
        }
        
        batch
    }

    async fn sync_batch(&self, api_client: &ApiClient, batch: &[PendingResponse]) -> Result<BatchSyncResult> {
        // Convert PendingResponse to Response for API
        let responses: Vec<Response> = batch
            .iter()
            .map(|pr| Response {
                id: pr.id,
                session_id: pr.session_id,
                sequence_number: pr.sequence_number,
                task_type: pr.task_type.clone(),
                task_data: pr.task_data.clone(),
                user_answer: pr.user_answer.clone(),
                correct: pr.correct,
                response_time_ms: pr.response_time_ms,
                hint_level: pr.hint_level,
                timestamp: pr.timestamp,
            })
            .collect();

        match api_client.sync_push(responses).await {
            Ok(sync_response) => Ok(BatchSyncResult {
                synced_count: sync_response.synced_count,
                failed_count: sync_response.failed_count,
                errors: sync_response.errors,
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn get_queue_size(&self) -> usize {
        self.queue.lock().await.len()
    }

    pub async fn get_last_sync(&self) -> Option<DateTime<Utc>> {
        *self.last_sync.lock().await
    }

    pub async fn is_syncing(&self) -> bool {
        *self.syncing.lock().await
    }

    pub async fn clear_queue(&self) -> Result<()> {
        let mut queue = self.queue.lock().await;
        let ids: Vec<Uuid> = queue.iter().map(|r| r.id).collect();
        queue.clear();
        
        // Also clear from storage
        for id in ids {
            self.storage.delete_pending_response(&id).await?;
        }
        
        Ok(())
    }

    // Sync with conflict resolution
    pub async fn sync_with_conflict_resolution(&self) -> Result<SyncResult> {
        // Pull remote changes first
        let api_client = self.api_client.lock().await;
        if let Some(client) = api_client.as_ref() {
            match client.sync_pull().await {
                Ok(pull_response) => {
                    // Process remote updates
                    for update in pull_response.updates {
                        // Handle remote updates (e.g., updated learner model, new achievements, etc.)
                        self.process_remote_update(update).await?;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to pull remote changes: {}", e);
                }
            }
        }

        // Then push local changes
        self.sync_all().await
    }

    async fn process_remote_update(&self, update: serde_json::Value) -> Result<()> {
        // Process different types of remote updates
        if let Ok(update_type) = update.get("type").and_then(|v| v.as_str()).ok_or(anyhow::anyhow!("Invalid update type")) {
            match update_type {
                "learner_model" => {
                    // Update local learner model
                    self.storage.save_json("learner_model", &update).await?;
                }
                "achievement" => {
                    // Update achievements
                    self.storage.save_json("achievements", &update).await?;
                }
                "session_summary" => {
                    // Save session summary
                    self.storage.save_json("session_summaries", &update).await?;
                }
                _ => {
                    tracing::debug!("Unknown update type: {}", update_type);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub synced_count: usize,
    pub failed_count: usize,
    pub remaining_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
struct BatchSyncResult {
    synced_count: usize,
    failed_count: usize,
    errors: Vec<String>,
}

// Background sync task that runs periodically
pub async fn start_background_sync(sync_service: Arc<SyncService>, interval_seconds: u64) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));
    
    loop {
        interval.tick().await;
        
        // Check if we have items to sync and are not already syncing
        if sync_service.get_queue_size().await > 0 && !sync_service.is_syncing().await {
            tracing::info!("Starting background sync...");
            match sync_service.sync_all().await {
                Ok(result) => {
                    tracing::info!(
                        "Background sync completed: {} synced, {} failed, {} remaining",
                        result.synced_count,
                        result.failed_count,
                        result.remaining_count
                    );
                }
                Err(e) => {
                    tracing::error!("Background sync failed: {}", e);
                }
            }
        }
    }
}