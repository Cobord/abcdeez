use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, Acquire};
use tokio::time::timeout;
use uuid::Uuid;

use crate::{
    cache::{self, ConnectionManager},
    db::DbPool,
    error::{AppError, AppResult},
    config::Config,
};

/// Job status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    DeadLetter,
    Cancelled,
}

/// Dead Letter Queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    pub job_id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub failure_reason: String,
    pub retry_count: i32,
    pub original_created_at: DateTime<Utc>,
    pub moved_to_dlq_at: DateTime<Utc>,
    pub last_error: String,
    pub stack_trace: Option<String>,
}

/// Enhanced batch job with DLQ support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub retry_count: i32,
    pub max_retries: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub backoff_seconds: u64,
}

/// Distributed lock for job processing
pub struct DistributedLock {
    cache: ConnectionManager,
    lock_prefix: String,
    ttl_seconds: u64,
}

impl DistributedLock {
    pub fn new(cache: ConnectionManager) -> Self {
        Self {
            cache,
            lock_prefix: "job_lock:".to_string(),
            ttl_seconds: 300, // 5 minutes default
        }
    }
    
    /// Try to acquire a lock for a specific job
    pub async fn try_acquire(&mut self, job_id: &Uuid, worker_id: &str) -> AppResult<bool> {
        let lock_key = format!("{}{}", self.lock_prefix, job_id);
        
        // Use SET NX (set if not exists) with expiration
        let result = cache::cmd("SET")
            .arg(&lock_key)
            .arg(worker_id)
            .arg("NX") // Only set if not exists
            .arg("EX")
            .arg(self.ttl_seconds)
            .query_async::<String>(&mut self.cache)
            .await;
        
        match result {
            Ok(response) if response == "OK" => Ok(true),
            _ => Ok(false),
        }
    }
    
    /// Release a lock (only if we own it)
    pub async fn release(&mut self, job_id: &Uuid, worker_id: &str) -> AppResult<()> {
        let lock_key = format!("{}{}", self.lock_prefix, job_id);
        
        // Check if we own the lock
        let current_owner = cache::cmd("GET")
            .arg(&lock_key)
            .query_async::<Option<String>>(&mut self.cache)
            .await
            .unwrap_or(None);
        
        if current_owner.as_deref() == Some(worker_id) {
            cache::cmd("DEL")
                .arg(&lock_key)
                .query_async::<()>(&mut self.cache)
                .await
                .ok();
        }
        
        Ok(())
    }
    
    /// Extend lock TTL (for long-running jobs)
    pub async fn extend(&mut self, job_id: &Uuid, worker_id: &str) -> AppResult<bool> {
        let lock_key = format!("{}{}", self.lock_prefix, job_id);
        
        // Check if we own the lock
        let current_owner = cache::cmd("GET")
            .arg(&lock_key)
            .query_async::<Option<String>>(&mut self.cache)
            .await
            .unwrap_or(None);
        
        if current_owner.as_deref() == Some(worker_id) {
            cache::cmd("EXPIRE")
                .arg(&lock_key)
                .arg(self.ttl_seconds)
                .query_async::<()>(&mut self.cache)
                .await
                .ok();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Enhanced Batch Job Service with DLQ and distributed locking
pub struct EnhancedBatchJobService {
    db: Arc<DbPool>,
    cache: ConnectionManager,
    config: Arc<Config>,
    worker_id: String,
    max_retries: i32,
    dlq_enabled: bool,
}

impl EnhancedBatchJobService {
    pub fn new(db: Arc<DbPool>, cache: ConnectionManager, config: Arc<Config>) -> Self {
        let worker_id = format!("worker_{}", Uuid::new_v4());
        
        Self {
            db,
            cache,
            config,
            worker_id,
            max_retries: 3,
            dlq_enabled: true,
        }
    }
    
    /// Process a single job with all safety mechanisms
    pub async fn process_job_with_dlq(&self, job: BatchJob) -> AppResult<()> {
        // Try to acquire distributed lock
        let mut lock = DistributedLock::new(self.cache.clone());
        
        if !lock.try_acquire(&job.id, &self.worker_id).await? {
            tracing::debug!("Job {} already being processed by another worker", job.id);
            return Ok(());
        }
        
        // Process job with timeout
        let job_timeout = Duration::from_secs(300); // 5 minutes max per job
        let process_result = timeout(job_timeout, self.execute_job(&job)).await;
        
        match process_result {
            Ok(Ok(_)) => {
                // Job succeeded
                self.mark_job_complete(&job.id).await?;
                tracing::info!("Job {} completed successfully", job.id);
            }
            Ok(Err(_)) | Err(_) => {
                // Job failed or timed out
                let error_msg = match &process_result {
                    Err(_) => "Job execution timeout".to_string(),
                    Ok(Err(e)) => e.to_string(),
                    _ => "Unknown error".to_string(),
                };
                
                tracing::error!("Job {} failed: {}", job.id, error_msg);
                
                // Check if we should retry or move to DLQ
                if job.retry_count >= self.max_retries {
                    if self.dlq_enabled {
                        self.move_to_dlq(&job, &error_msg).await?;
                        tracing::warn!("Job {} moved to dead letter queue after {} retries", job.id, job.retry_count);
                    } else {
                        self.mark_job_failed(&job.id, &error_msg).await?;
                    }
                } else {
                    // Schedule retry with exponential backoff
                    let backoff = self.calculate_backoff(job.retry_count);
                    self.schedule_retry(&job.id, backoff).await?;
                    tracing::info!("Job {} scheduled for retry in {} seconds", job.id, backoff);
                }
            }
        }
        
        // Release lock
        lock.release(&job.id, &self.worker_id).await?;
        
        Ok(())
    }
    
    /// Move job to dead letter queue
    async fn move_to_dlq(&self, job: &BatchJob, error: &str) -> AppResult<()> {
        let mut conn = self.db.acquire().await.map_err(AppError::DatabaseError)?;
        let mut tx = conn.begin().await.map_err(AppError::DatabaseError)?;
        
        // Insert into DLQ table
        let dlq_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO dead_letter_queue (
                id, job_id, job_type, payload, failure_reason, 
                retry_count, original_created_at, moved_to_dlq_at, last_error
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(crate::db::uuid_to_db(dlq_id))
        .bind(crate::db::uuid_to_db(job.id))
        .bind(&job.job_type)
        .bind(serde_json::to_string(&job.payload).unwrap())
        .bind("Max retries exceeded")
        .bind(job.retry_count)
        .bind(job.created_at)
        .bind(Utc::now())
        .bind(error)
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;
        
        // Update original job status
        sqlx::query(
            "UPDATE batch_jobs SET status = 'dead_letter', updated_at = ? WHERE id = ?"
        )
        .bind(Utc::now())
        .bind(crate::db::uuid_to_db(job.id))
        .execute(&mut *tx)
        .await
        .map_err(AppError::DatabaseError)?;
        
        tx.commit().await.map_err(AppError::DatabaseError)?;
        
        // Send alert about DLQ entry
        self.alert_on_dlq_entry(job, error).await?;
        
        Ok(())
    }
    
    /// Calculate exponential backoff for retries
    fn calculate_backoff(&self, retry_count: i32) -> u64 {
        let base: u64 = 2;
        let max_backoff = 3600; // 1 hour max
        
        std::cmp::min(
            base.pow(retry_count as u32) * 10, // 10s, 20s, 40s, 80s...
            max_backoff
        )
    }
    
    /// Schedule a job retry with backoff
    async fn schedule_retry(&self, job_id: &Uuid, backoff_seconds: u64) -> AppResult<()> {
        let mut conn = self.db.acquire().await.map_err(AppError::DatabaseError)?;
        
        let scheduled_at = Utc::now() + chrono::Duration::seconds(backoff_seconds as i64);
        
        sqlx::query(
            "UPDATE batch_jobs 
             SET status = 'pending', 
                 retry_count = retry_count + 1,
                 scheduled_at = ?,
                 updated_at = ?
             WHERE id = ?"
        )
        .bind(scheduled_at)
        .bind(Utc::now())
        .bind(crate::db::uuid_to_db(*job_id))
        .execute(&mut *conn)
        .await
        .map_err(AppError::DatabaseError)?;
        
        Ok(())
    }
    
    /// Mark job as complete
    async fn mark_job_complete(&self, job_id: &Uuid) -> AppResult<()> {
        let mut conn = self.db.acquire().await.map_err(AppError::DatabaseError)?;
        
        sqlx::query(
            "UPDATE batch_jobs 
             SET status = 'completed', 
                 completed_at = ?,
                 updated_at = ?
             WHERE id = ?"
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(crate::db::uuid_to_db(*job_id))
        .execute(&mut *conn)
        .await
        .map_err(AppError::DatabaseError)?;
        
        Ok(())
    }
    
    /// Mark job as failed
    async fn mark_job_failed(&self, job_id: &Uuid, error: &str) -> AppResult<()> {
        let mut conn = self.db.acquire().await.map_err(AppError::DatabaseError)?;
        
        sqlx::query(
            "UPDATE batch_jobs 
             SET status = 'failed', 
                 error_message = ?,
                 updated_at = ?
             WHERE id = ?"
        )
        .bind(error)
        .bind(Utc::now())
        .bind(crate::db::uuid_to_db(*job_id))
        .execute(&mut *conn)
        .await
        .map_err(AppError::DatabaseError)?;
        
        Ok(())
    }
    
    /// Execute the actual job logic
    async fn execute_job(&self, job: &BatchJob) -> AppResult<()> {
        // This would contain the actual job execution logic
        // For now, just a placeholder
        tracing::info!("Executing job {} of type {}", job.id, job.job_type);
        
        // Simulate some work
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Return success or error based on job type
        Ok(())
    }
    
    /// Send alert when job enters DLQ
    async fn alert_on_dlq_entry(&self, job: &BatchJob, error: &str) -> AppResult<()> {
        // In production, this would send to alerting service (PagerDuty, Slack, etc.)
        tracing::error!(
            "ALERT: Job {} (type: {}) moved to DLQ after {} retries. Error: {}",
            job.id,
            job.job_type,
            job.retry_count,
            error
        );
        
        // Could also increment metrics
        // metrics::increment_counter!("jobs.dlq.entries", 1);
        
        Ok(())
    }
    
    /// Process jobs from the DLQ (manual intervention or automated retry)
    pub async fn process_dlq(&self) -> AppResult<()> {
        let mut conn = self.db.acquire().await.map_err(AppError::DatabaseError)?;
        
        // Get DLQ entries older than 1 hour (give time for investigation)
        let dlq_entries = sqlx::query(
            "SELECT * FROM dead_letter_queue 
             WHERE moved_to_dlq_at < ? 
             AND retry_attempted = false
             LIMIT 10"
        )
        .bind(Utc::now() - chrono::Duration::hours(1))
        .fetch_all(&mut *conn)
        .await
        .map_err(AppError::DatabaseError)?;
        
        for entry in dlq_entries {
            let job_id_bytes: Vec<u8> = entry.get("job_id");
            let job_id = Uuid::from_slice(&job_id_bytes).unwrap_or_default();
            
            tracing::info!("Attempting to reprocess DLQ job {}", job_id);
            
            // Mark as retry attempted
            sqlx::query(
                "UPDATE dead_letter_queue SET retry_attempted = true WHERE job_id = ?"
            )
            .bind(&job_id_bytes)
            .execute(&mut *conn)
            .await
            .ok();
            
            // Resubmit job with reset retry count
            // Implementation depends on your job submission logic
        }
        
        Ok(())
    }
}

/// Federation sync with distributed locking
pub struct FederationSyncService {
    db: Arc<DbPool>,
    cache: ConnectionManager,
}

impl FederationSyncService {
    pub fn new(db: Arc<DbPool>, cache: ConnectionManager) -> Self {
        Self { db, cache }
    }
    
    /// Sync with a federation node using distributed lock
    pub async fn sync_with_lock(&self, node_id: Uuid) -> AppResult<()> {
        let lock_key = format!("federation_sync:{}", node_id);
        let lock_value = Uuid::new_v4().to_string();
        let lock_ttl = 300; // 5 minutes
        
        // Try to acquire lock using atomic SET NX operation
        let mut cache_conn = self.cache.clone();
        let acquired = cache::cmd("SET")
            .arg(&lock_key)
            .arg(&lock_value)
            .arg("NX")
            .arg("EX")
            .arg(lock_ttl)
            .query_async::<String>(&mut cache_conn)
            .await;
        
        if acquired.is_err() || acquired.unwrap() != "OK" {
            return Err(AppError::Conflict("Federation sync already in progress".into()));
        }
        
        // Perform sync
        let sync_result = self.perform_sync(node_id).await;
        
        // Release lock (only if we own it)
        let current_value = cache::cmd("GET")
            .arg(&lock_key)
            .query_async::<Option<String>>(&mut cache_conn)
            .await
            .unwrap_or(None);
        
        if current_value.as_deref() == Some(&lock_value) {
            cache::cmd("DEL")
                .arg(&lock_key)
                .query_async::<()>(&mut cache_conn)
                .await
                .ok();
        }
        
        sync_result
    }
    
    async fn perform_sync(&self, node_id: Uuid) -> AppResult<()> {
        // Actual federation sync logic would go here
        tracing::info!("Performing federation sync with node {}", node_id);
        
        // Simulate sync work
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        Ok(())
    }
}