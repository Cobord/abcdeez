use std::{sync::Arc, time::Duration};
use tokio::time::interval;
use uuid::Uuid;
use sqlx::PgPool;
use redis::aio::ConnectionManager;
use anyhow::Result;

use crate::services::{AnalyticsService, LearnerService};

#[derive(Debug, Clone)]
pub enum JobType {
    StatisticsUpdate,
    LeaderboardRefresh,
    ModelSnapshot,
    DataCleanup,
    CacheWarmup,
}

impl JobType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "statistics_update" => Some(JobType::StatisticsUpdate),
            "leaderboard_refresh" => Some(JobType::LeaderboardRefresh),
            "model_snapshot" => Some(JobType::ModelSnapshot),
            "data_cleanup" => Some(JobType::DataCleanup),
            "cache_warmup" => Some(JobType::CacheWarmup),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            JobType::StatisticsUpdate => "statistics_update",
            JobType::LeaderboardRefresh => "leaderboard_refresh",
            JobType::ModelSnapshot => "model_snapshot",
            JobType::DataCleanup => "data_cleanup",
            JobType::CacheWarmup => "cache_warmup",
        }
    }
}

pub struct BatchJobService {
    db: Arc<PgPool>,
    redis: Arc<ConnectionManager>,
    analytics_service: AnalyticsService,
    learner_service: Arc<LearnerService>,
}

impl BatchJobService {
    pub fn new(
        db: Arc<PgPool>,
        redis: Arc<ConnectionManager>,
    ) -> Self {
        let analytics_service = AnalyticsService::new(db.clone(), redis.clone());
        let learner_service = Arc::new(LearnerService::new(db.clone(), redis.clone()));

        Self {
            db,
            redis: redis.clone(),
            analytics_service,
            learner_service,
        }
    }

    // Start the background job worker
    pub async fn start_worker(&self) {
        let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.process_pending_jobs().await {
                tracing::error!("Error processing batch jobs: {}", e);
            }
        }
    }

    // Process pending jobs from the queue
    async fn process_pending_jobs(&self) -> Result<()> {
        // Get pending jobs
        let pending_jobs = sqlx::query!(
            "SELECT id, job_type, payload, retry_count 
             FROM job_queue 
             WHERE status = 'pending' 
             ORDER BY created_at ASC 
             LIMIT 10"
        )
        .fetch_all(&**self.db)
        .await?;

        for job in pending_jobs {
            let job_id = Uuid::from_bytes(job.id.try_into().unwrap_or_default());
            let job_type = JobType::from_str(&job.job_type);

            // Mark job as running
            self.update_job_status(job_id, "running", None).await?;

            let result = if let Some(job_type) = job_type {
                self.execute_job(job_type, &job.payload).await
            } else {
                Err(anyhow::anyhow!("Unknown job type: {}", job.job_type))
            };

            match result {
                Ok(_) => {
                    self.update_job_status(job_id, "completed", None).await?;
                    tracing::info!("Job {} completed successfully", job_id);
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    let retry_count = job.retry_count + 1;
                    
                    if retry_count < 3 {
                        // Retry the job
                        sqlx::query!(
                            "UPDATE job_queue SET status = 'pending', retry_count = $1, error_message = $2 WHERE id = $3",
                            retry_count,
                            error_msg,
                            job.id
                        )
                        .execute(&**self.db)
                        .await?;
                        tracing::warn!("Job {} failed, retrying (attempt {}): {}", job_id, retry_count, error_msg);
                    } else {
                        // Mark as failed permanently
                        self.update_job_status(job_id, "failed", Some(error_msg)).await?;
                        tracing::error!("Job {} failed permanently after {} retries", job_id, retry_count);
                    }
                }
            }
        }

        Ok(())
    }

    // Execute a specific job type
    async fn execute_job(&self, job_type: JobType, payload: &str) -> Result<()> {
        match job_type {
            JobType::StatisticsUpdate => self.update_statistics().await,
            JobType::LeaderboardRefresh => self.refresh_leaderboards().await,
            JobType::ModelSnapshot => self.create_model_snapshots().await,
            JobType::DataCleanup => self.cleanup_old_data().await,
            JobType::CacheWarmup => self.warmup_cache().await,
        }
    }

    // Update population statistics
    async fn update_statistics(&self) -> Result<()> {
        tracing::info!("Starting statistics update job");

        // Update population stats and cache them
        let stats = self.analytics_service.population_stats().await?;
        
        // Cache the updated stats
        let stats_json = serde_json::to_string(&stats)?;
        let mut conn = self.redis.clone();
        redis::cmd("SETEX")
            .arg("cached_population_stats")
            .arg(3600) // 1 hour TTL
            .arg(stats_json)
            .query_async(&mut conn)
            .await?;

        // Update bottlenecks analysis
        let bottlenecks = self.analytics_service.find_bottlenecks(50).await?;
        let bottlenecks_json = serde_json::to_string(&bottlenecks)?;
        redis::cmd("SETEX")
            .arg("cached_bottlenecks")
            .arg(1800) // 30 minutes TTL
            .arg(bottlenecks_json)
            .query_async(&mut conn)
            .await?;

        tracing::info!("Statistics update job completed");
        Ok(())
    }

    // Refresh leaderboards
    async fn refresh_leaderboards(&self) -> Result<()> {
        tracing::info!("Starting leaderboard refresh job");

        // Get top performers by accuracy
        let top_accuracy = sqlx::query!(
            "SELECT l.id, l.display_name, u.username,
                    AVG(CASE WHEN r.correct THEN 1.0 ELSE 0.0 END) as accuracy,
                    COUNT(r.id) as total_responses
             FROM learners l
             LEFT JOIN users u ON l.user_id = u.id
             LEFT JOIN sessions s ON l.id = s.learner_id
             LEFT JOIN responses r ON s.id = r.session_id
             WHERE r.timestamp > $1
             GROUP BY l.id, l.display_name, u.username
             HAVING COUNT(r.id) >= 100
             ORDER BY accuracy DESC
             LIMIT 50",
            chrono::Utc::now() - chrono::Duration::days(30)
        )
        .fetch_all(&**self.db)
        .await?;

        // Get top performers by practice time
        let top_practice_time = sqlx::query!(
            "SELECT l.id, l.display_name, u.username, l.total_practice_time_seconds
             FROM learners l
             LEFT JOIN users u ON l.user_id = u.id
             WHERE l.total_practice_time_seconds > 0
             ORDER BY l.total_practice_time_seconds DESC
             LIMIT 50"
        )
        .fetch_all(&**self.db)
        .await?;

        // Cache leaderboards
        let accuracy_leaderboard: Vec<serde_json::Value> = top_accuracy
            .into_iter()
            .enumerate()
            .map(|(rank, row)| serde_json::json!({
                "rank": rank + 1,
                "learner_id": Uuid::from_bytes(row.id.try_into().unwrap_or_default()),
                "display_name": row.display_name,
                "username": row.username,
                "accuracy": row.accuracy.unwrap_or(0.0),
                "total_responses": row.total_responses.unwrap_or(0)
            }))
            .collect();

        let practice_leaderboard: Vec<serde_json::Value> = top_practice_time
            .into_iter()
            .enumerate()
            .map(|(rank, row)| serde_json::json!({
                "rank": rank + 1,
                "learner_id": Uuid::from_bytes(row.id.try_into().unwrap_or_default()),
                "display_name": row.display_name,
                "username": row.username,
                "total_practice_hours": row.total_practice_time_seconds as f64 / 3600.0
            }))
            .collect();

        let mut conn = self.redis.clone();
        redis::cmd("SETEX")
            .arg("leaderboard_accuracy")
            .arg(3600) // 1 hour TTL
            .arg(serde_json::to_string(&accuracy_leaderboard)?)
            .query_async(&mut conn)
            .await?;

        redis::cmd("SETEX")
            .arg("leaderboard_practice_time")
            .arg(3600) // 1 hour TTL
            .arg(serde_json::to_string(&practice_leaderboard)?)
            .query_async(&mut conn)
            .await?;

        tracing::info!("Leaderboard refresh job completed");
        Ok(())
    }

    // Create model snapshots for analysis
    async fn create_model_snapshots(&self) -> Result<()> {
        tracing::info!("Starting model snapshot job");

        // Get active learners
        let active_learners = sqlx::query!(
            "SELECT DISTINCT l.id 
             FROM learners l 
             JOIN sessions s ON l.id = s.learner_id 
             WHERE s.start_time > $1",
            chrono::Utc::now() - chrono::Duration::hours(24)
        )
        .fetch_all(&**self.db)
        .await?;

        let mut snapshots_created = 0;
        
        for learner_row in active_learners {
            let learner_id = Uuid::from_bytes(learner_row.id.try_into().unwrap_or_default());
            
            if let Ok(learner) = self.learner_service.get_learner(learner_id).await {
                // Create snapshot (this would normally be done by the service)
                let snapshot_id = Uuid::new_v4();
                let learner_id_bytes = learner_id.as_bytes();
                let snapshot_id_bytes = snapshot_id.as_bytes();
                
                let parameters = serde_json::to_string(&learner.core_model)?;
                let metrics = serde_json::to_string(&graph_learning_core::LearnerMetrics::from_model(&learner.core_model))?;

                sqlx::query!(
                    "INSERT INTO model_snapshots (id, learner_id, timestamp, parameters, metrics) 
                     VALUES ($1, $2, $3, $4, $5)",
                    snapshot_id_bytes,
                    learner_id_bytes,
                    chrono::Utc::now(),
                    parameters,
                    metrics
                )
                .execute(&**self.db)
                .await?;

                snapshots_created += 1;
            }
        }

        tracing::info!("Model snapshot job completed: {} snapshots created", snapshots_created);
        Ok(())
    }

    // Cleanup old data
    async fn cleanup_old_data(&self) -> Result<()> {
        tracing::info!("Starting data cleanup job");

        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(90);

        // Clean up old audit logs (keep 90 days)
        let deleted_audit = sqlx::query!(
            "DELETE FROM audit_log WHERE timestamp < $1",
            cutoff_date
        )
        .execute(&**self.db)
        .await?;

        // Clean up old job queue entries
        let deleted_jobs = sqlx::query!(
            "DELETE FROM job_queue WHERE created_at < $1 AND status IN ('completed', 'failed')",
            cutoff_date
        )
        .execute(&**self.db)
        .await?;

        tracing::info!(
            "Data cleanup job completed: {} audit logs, {} job records deleted",
            deleted_audit.rows_affected(),
            deleted_jobs.rows_affected()
        );

        Ok(())
    }

    // Warmup frequently accessed cache entries
    async fn warmup_cache(&self) -> Result<()> {
        tracing::info!("Starting cache warmup job");

        // Warmup population statistics
        let stats = self.analytics_service.population_stats().await?;
        let mut conn = self.redis.clone();
        redis::cmd("SETEX")
            .arg("warmed_population_stats")
            .arg(1800) // 30 minutes TTL
            .arg(serde_json::to_string(&stats)?)
            .query_async(&mut conn)
            .await?;

        // Warmup common learner models (most active learners)
        let active_learners = sqlx::query!(
            "SELECT l.id, COUNT(r.id) as response_count
             FROM learners l
             JOIN sessions s ON l.id = s.learner_id
             JOIN responses r ON s.id = r.session_id
             WHERE r.timestamp > $1
             GROUP BY l.id
             ORDER BY response_count DESC
             LIMIT 20",
            chrono::Utc::now() - chrono::Duration::days(7)
        )
        .fetch_all(&**self.db)
        .await?;

        for learner_row in active_learners {
            let learner_id = Uuid::from_bytes(learner_row.id.try_into().unwrap_or_default());
            
            // This will cache the learner model if it's not already cached
            if let Err(_) = self.learner_service.get_learner(learner_id).await {
                continue; // Skip if learner can't be loaded
            }
        }

        tracing::info!("Cache warmup job completed");
        Ok(())
    }

    // Update job status
    async fn update_job_status(&self, job_id: Uuid, status: &str, error_message: Option<String>) -> Result<()> {
        let job_id_bytes = job_id.as_bytes();
        let now = chrono::Utc::now();

        if status == "running" {
            sqlx::query!(
                "UPDATE job_queue SET status = $1, started_at = $2 WHERE id = $3",
                status,
                now,
                job_id_bytes
            )
            .execute(&**self.db)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE job_queue SET status = $1, completed_at = $2, error_message = $3 WHERE id = $4",
                status,
                now,
                error_message,
                job_id_bytes
            )
            .execute(&**self.db)
            .await?;
        }

        Ok(())
    }

    // Schedule a new job
    pub async fn schedule_job(&self, job_type: JobType, payload: serde_json::Value) -> Result<Uuid> {
        let job_id = Uuid::new_v4();
        let job_id_bytes = job_id.as_bytes();

        sqlx::query!(
            "INSERT INTO job_queue (id, job_type, payload, status) VALUES ($1, $2, $3, 'pending')",
            job_id_bytes,
            job_type.as_str(),
            payload.to_string()
        )
        .execute(&**self.db)
        .await?;

        Ok(job_id)
    }
}