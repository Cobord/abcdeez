use crate::cache::ConnectionManager;
use crate::db::DbPool;
use anyhow::Result;
use sqlx::Row;
use std::{sync::Arc, time::Duration};
use tokio::time::interval;
use uuid::Uuid;

use crate::config::Config;
use crate::services::oauth_service::{CredentialState, OAuthProvider, OAuthService};
use crate::services::{AnalyticsService, LearnerService};

#[derive(Debug, Clone)]
pub enum JobType {
    StatisticsUpdate,
    LeaderboardRefresh,
    ModelSnapshot,
    DataCleanup,
    CacheWarmup,
    OAuthCredentialValidation,
}

impl JobType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "statistics_update" => Some(JobType::StatisticsUpdate),
            "leaderboard_refresh" => Some(JobType::LeaderboardRefresh),
            "model_snapshot" => Some(JobType::ModelSnapshot),
            "data_cleanup" => Some(JobType::DataCleanup),
            "cache_warmup" => Some(JobType::CacheWarmup),
            "oauth_credential_validation" => Some(JobType::OAuthCredentialValidation),
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
            JobType::OAuthCredentialValidation => "oauth_credential_validation",
        }
    }
}

pub struct BatchJobService {
    db: Arc<DbPool>,
    redis: ConnectionManager,
    analytics_service: AnalyticsService,
    learner_service: Arc<LearnerService>,
    config: Arc<Config>,
}

impl BatchJobService {
    pub fn new(db: Arc<DbPool>, redis: ConnectionManager, config: Arc<Config>) -> Self {
        let analytics_service = AnalyticsService::new_with_config(
            db.clone(),
            Arc::new(redis.clone()),
            config.clone(),
        );
        let learner_service = Arc::new(LearnerService::new(db.clone(), redis.clone()));

        Self {
            db,
            redis: redis.clone(),
            analytics_service,
            learner_service,
            config,
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
        let mut conn = self.db.acquire().await?;
        let pending_jobs = sqlx::query(
            "SELECT id, job_type, payload, retry_count
             FROM job_queue
             WHERE status = 'pending'
             ORDER BY created_at ASC
             LIMIT 10",
        )
        .fetch_all(&mut *conn)
        .await?;

        for job in pending_jobs {
            let job_id_bytes: Vec<u8> = job.get::<Vec<u8>, _>("id");
            let job_id = Uuid::from_bytes(job_id_bytes.clone().try_into().unwrap_or_default());
            let job_type_str: String = job.get::<String, _>("job_type");
            let payload: String = job.get::<String, _>("payload");
            let retry_count: i32 = job.get::<i32, _>("retry_count");
            let job_type = JobType::from_str(&job_type_str);

            // Mark job as running
            self.update_job_status(job_id, "running", None).await?;

            let result = if let Some(job_type) = job_type {
                self.execute_job(job_type, &payload).await
            } else {
                Err(anyhow::anyhow!("Unknown job type: {}", job_type_str))
            };

            match result {
                Ok(_) => {
                    self.update_job_status(job_id, "completed", None).await?;
                    tracing::info!("Job {} completed successfully", job_id);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let retry_count = retry_count + 1;

                    if retry_count < 3 {
                        // Retry the job
                        sqlx::query(
                            "UPDATE job_queue SET status = 'pending', retry_count = ?, error_message = ? WHERE id = ?"
                        )
                        .bind(retry_count)
                        .bind(&error_msg)
                        .bind(&job_id_bytes)
                        .execute(&mut *conn)
                        .await?;
                        tracing::warn!(
                            "Job {} failed, retrying (attempt {}): {}",
                            job_id,
                            retry_count,
                            error_msg
                        );
                    } else {
                        // Mark as failed permanently
                        self.update_job_status(job_id, "failed", Some(error_msg))
                            .await?;
                        tracing::error!(
                            "Job {} failed permanently after {} retries",
                            job_id,
                            retry_count
                        );
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
            JobType::OAuthCredentialValidation => self.validate_oauth_credentials().await,
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
        crate::cache::cmd("SETEX")
            .arg("cached_population_stats")
            .arg(3600) // 1 hour TTL
            .arg(stats_json)
            .query_async::<()>(&mut conn)
            .await?;

        // Update bottlenecks analysis
        let bottlenecks = self.analytics_service.find_bottlenecks(50).await?;
        let bottlenecks_json = serde_json::to_string(&bottlenecks)?;
        crate::cache::cmd("SETEX")
            .arg("cached_bottlenecks")
            .arg(1800) // 30 minutes TTL
            .arg(bottlenecks_json)
            .query_async::<()>(&mut conn)
            .await?;

        tracing::info!("Statistics update job completed");
        Ok(())
    }

    // Refresh leaderboards
    async fn refresh_leaderboards(&self) -> Result<()> {
        tracing::info!("Starting leaderboard refresh job");

        // Get top performers by accuracy
        let mut conn = self.db.acquire().await?;
        let top_accuracy = sqlx::query(
            "SELECT l.id, l.display_name, u.username,
                    AVG(CASE WHEN r.correct THEN 1.0 ELSE 0.0 END) as accuracy,
                    COUNT(r.id) as total_responses
             FROM learners l
             LEFT JOIN users u ON l.user_id = u.id
             LEFT JOIN sessions s ON l.id = s.learner_id
             LEFT JOIN responses r ON s.id = r.session_id
             WHERE r.timestamp > ?
             GROUP BY l.id, l.display_name, u.username
             HAVING COUNT(r.id) >= 100
             ORDER BY accuracy DESC
             LIMIT 50",
        )
        .bind(chrono::Utc::now() - chrono::Duration::days(30))
        .fetch_all(&mut *conn)
        .await?;

        // Get top performers by practice time
        let top_practice_time = sqlx::query(
            "SELECT l.id, l.display_name, u.username, l.total_practice_time_seconds
             FROM learners l
             LEFT JOIN users u ON l.user_id = u.id
             WHERE l.total_practice_time_seconds > 0
             ORDER BY l.total_practice_time_seconds DESC
             LIMIT 50",
        )
        .fetch_all(&mut *conn)
        .await?;

        // Cache leaderboards
        let accuracy_leaderboard: Result<Vec<serde_json::Value>, sqlx::Error> = top_accuracy
            .into_iter()
            .enumerate()
            .map(|(rank, row)| {
                let id_bytes: Vec<u8> = row.get::<Vec<u8>, _>("id");
                let display_name: Option<String> = row.get::<Option<String>, _>("display_name");
                let username: Option<String> = row.get::<Option<String>, _>("username");
                let accuracy: Option<f64> = row.get::<Option<f64>, _>("accuracy");
                let total_responses: Option<i64> = row.get::<Option<i64>, _>("total_responses");

                Ok(serde_json::json!({
                    "rank": rank + 1,
                    "learner_id": Uuid::from_bytes(id_bytes.try_into().unwrap_or_default()),
                    "display_name": display_name,
                    "username": username,
                    "accuracy": accuracy.unwrap_or(0.0),
                    "total_responses": total_responses.unwrap_or(0)
                }))
            })
            .collect();
        let accuracy_leaderboard = accuracy_leaderboard?;

        let practice_leaderboard: Result<Vec<serde_json::Value>, sqlx::Error> = top_practice_time
            .into_iter()
            .enumerate()
            .map(|(rank, row)| {
                let id_bytes: Vec<u8> = row.get::<Vec<u8>, _>("id");
                let display_name: Option<String> = row.get::<Option<String>, _>("display_name");
                let username: Option<String> = row.get::<Option<String>, _>("username");
                let total_practice_time_seconds: i32 =
                    row.get::<i32, _>("total_practice_time_seconds");

                Ok(serde_json::json!({
                    "rank": rank + 1,
                    "learner_id": Uuid::from_bytes(id_bytes.try_into().unwrap_or_default()),
                    "display_name": display_name,
                    "username": username,
                    "total_practice_hours": total_practice_time_seconds as f64 / 3600.0
                }))
            })
            .collect();
        let practice_leaderboard = practice_leaderboard?;

        let mut conn = self.redis.clone();
        crate::cache::cmd("SETEX")
            .arg("leaderboard_accuracy")
            .arg(3600) // 1 hour TTL
            .arg(serde_json::to_string(&accuracy_leaderboard)?)
            .query_async::<()>(&mut conn)
            .await?;

        crate::cache::cmd("SETEX")
            .arg("leaderboard_practice_time")
            .arg(3600) // 1 hour TTL
            .arg(serde_json::to_string(&practice_leaderboard)?)
            .query_async::<()>(&mut conn)
            .await?;

        tracing::info!("Leaderboard refresh job completed");
        Ok(())
    }

    // Create model snapshots for analysis
    async fn create_model_snapshots(&self) -> Result<()> {
        tracing::info!("Starting model snapshot job");

        // Get active learners
        let mut conn = self.db.acquire().await?;
        let active_learners = sqlx::query(
            "SELECT DISTINCT l.id
             FROM learners l
             JOIN sessions s ON l.id = s.learner_id
             WHERE s.start_time > ?",
        )
        .bind(chrono::Utc::now() - chrono::Duration::hours(24))
        .fetch_all(&mut *conn)
        .await?;

        let mut snapshots_created = 0;

        for learner_row in active_learners {
            let id_bytes: Vec<u8> = learner_row.get::<Vec<u8>, _>("id");
            let learner_id = Uuid::from_bytes(id_bytes.try_into().unwrap_or_default());

            if let Ok(learner) = self.learner_service.get_learner(learner_id).await {
                // Create snapshot (this would normally be done by the service)
                let snapshot_id = Uuid::new_v4();
                let learner_id_bytes = learner_id.as_bytes();
                let snapshot_id_bytes = snapshot_id.as_bytes();

                let parameters = serde_json::to_string(&learner.learning_model)?;
                let metrics = serde_json::to_string(
                    &graph_learning_core::LearnerMetrics::from_model(&learner.learning_model),
                )?;

                sqlx::query(
                    "INSERT INTO model_snapshots (id, learner_id, timestamp, parameters, metrics)
                     VALUES (?, ?, ?, ?, ?)",
                )
                .bind(&snapshot_id_bytes[..])
                .bind(&learner_id_bytes[..])
                .bind(chrono::Utc::now())
                .bind(parameters)
                .bind(metrics)
                .execute(&mut *conn)
                .await?;

                snapshots_created += 1;
            }
        }

        tracing::info!(
            "Model snapshot job completed: {} snapshots created",
            snapshots_created
        );
        Ok(())
    }

    // Cleanup old data
    async fn cleanup_old_data(&self) -> Result<()> {
        tracing::info!("Starting data cleanup job");

        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(90);

        // Clean up old audit logs (keep 90 days)
        let mut conn = self.db.acquire().await?;
        let deleted_audit = sqlx::query("DELETE FROM audit_log WHERE timestamp < ?")
            .bind(cutoff_date)
            .execute(&mut *conn)
            .await?;

        // Clean up old job queue entries
        let deleted_jobs = sqlx::query(
            "DELETE FROM job_queue WHERE created_at < ? AND status IN ('completed', 'failed')",
        )
        .bind(cutoff_date)
        .execute(&mut *conn)
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
        crate::cache::cmd("SETEX")
            .arg("warmed_population_stats")
            .arg(1800) // 30 minutes TTL
            .arg(serde_json::to_string(&stats)?)
            .query_async::<()>(&mut conn)
            .await?;

        // Warmup common learner models (most active learners)
        let mut db_conn = self.db.acquire().await?;
        let active_learners = sqlx::query(
            "SELECT l.id, COUNT(r.id) as response_count
             FROM learners l
             JOIN sessions s ON l.id = s.learner_id
             JOIN responses r ON s.id = r.session_id
             WHERE r.timestamp > ?
             GROUP BY l.id
             ORDER BY response_count DESC
             LIMIT 20",
        )
        .bind(chrono::Utc::now() - chrono::Duration::days(7))
        .fetch_all(&mut *db_conn)
        .await?;

        for learner_row in active_learners {
            let id_bytes: Vec<u8> = learner_row.get::<Vec<u8>, _>("id");
            let learner_id = Uuid::from_bytes(id_bytes.try_into().unwrap_or_default());

            // This will cache the learner model if it's not already cached
            if let Err(_) = self.learner_service.get_learner(learner_id).await {
                continue; // Skip if learner can't be loaded
            }
        }

        tracing::info!("Cache warmup job completed");
        Ok(())
    }

    // Validate OAuth credentials for all OAuth users
    async fn validate_oauth_credentials(&self) -> Result<()> {
        tracing::info!("Starting OAuth credential validation job");

        let mut conn = self.db.acquire().await?;

        // Get all OAuth users who haven't been checked in the last 24 hours
        let oauth_users = sqlx::query(
            "SELECT u.id, u.username, u.apple_user_id, u.github_user_id, u.auth_provider,
                    COALESCE(occ.last_check_time, '1970-01-01') as last_check_time
             FROM users u
             LEFT JOIN oauth_credential_checks occ ON u.id = occ.user_id AND occ.provider = u.auth_provider
             WHERE u.auth_provider IN ('apple', 'github')
             AND (occ.last_check_time IS NULL OR occ.last_check_time < datetime('now', '-1 day'))
             ORDER BY COALESCE(occ.last_check_time, '1970-01-01') ASC
             LIMIT 100"
        )
        .fetch_all(&mut *conn)
        .await?;

        let oauth_service = OAuthService::new(self.config.clone());
        let mut validated_count = 0;
        let mut failed_count = 0;

        for user_row in oauth_users {
            let user_id_bytes: Vec<u8> = user_row.get("id");
            let user_id = Uuid::from_bytes(user_id_bytes.try_into().unwrap_or_default());
            let username: String = user_row.get("username");
            let auth_provider: String = user_row.get("auth_provider");

            // Parse provider and get provider user ID
            let provider = match auth_provider.as_str() {
                "apple" => {
                    let apple_user_id: Option<String> = user_row.get("apple_user_id");
                    if let Some(provider_user_id) = apple_user_id {
                        (OAuthProvider::Apple, provider_user_id)
                    } else {
                        tracing::warn!("Apple user {} missing apple_user_id", username);
                        continue;
                    }
                }
                "github" => {
                    let github_user_id: Option<String> = user_row.get("github_user_id");
                    if let Some(provider_user_id) = github_user_id {
                        (OAuthProvider::GitHub, provider_user_id)
                    } else {
                        tracing::warn!("GitHub user {} missing github_user_id", username);
                        continue;
                    }
                }
                _ => {
                    tracing::warn!(
                        "Unknown auth provider for user {}: {}",
                        username,
                        auth_provider
                    );
                    continue;
                }
            };

            let (oauth_provider, provider_user_id) = provider;

            // For this implementation, we'll create a minimal app state to use the OAuth service
            // In a real implementation, you might want to inject the AppState or modify the OAuth service
            let dummy_app_state = crate::state::AppState::new(
                (*self.db).clone(),
                self.redis.clone(),
                self.config.clone(),
            );

            // Validate credentials (this is a simplified approach)
            let validation_result: Result<CredentialState, anyhow::Error> = match oauth_provider {
                OAuthProvider::Apple => {
                    // Apple doesn't provide direct token validation
                    // We'll record as "unknown" for now
                    Ok(CredentialState::Unknown)
                }
                OAuthProvider::GitHub => {
                    // For GitHub, we would need an access token to validate
                    // Since we don't store access tokens for security reasons,
                    // we'll mark as unknown and let the user re-authenticate when needed
                    Ok(CredentialState::Unknown)
                }
            };

            // Record the validation result
            match validation_result {
                Ok(credential_state) => {
                    oauth_service
                        .record_credential_check(
                            &dummy_app_state,
                            user_id,
                            oauth_provider,
                            &provider_user_id,
                            credential_state.clone(),
                            None,
                        )
                        .await?;

                    validated_count += 1;
                    tracing::debug!(
                        "Validated {} user {}: {:?}",
                        oauth_provider,
                        username,
                        credential_state
                    );
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    oauth_service
                        .record_credential_check(
                            &dummy_app_state,
                            user_id,
                            oauth_provider,
                            &provider_user_id,
                            CredentialState::Unknown,
                            Some(&error_msg),
                        )
                        .await?;

                    failed_count += 1;
                    tracing::warn!(
                        "Failed to validate {} user {}: {}",
                        oauth_provider,
                        username,
                        error_msg
                    );
                }
            }

            // Small delay to avoid overwhelming external APIs
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        tracing::info!(
            "OAuth credential validation job completed: {} validated, {} failed",
            validated_count,
            failed_count
        );
        Ok(())
    }

    // Update job status
    async fn update_job_status(
        &self,
        job_id: Uuid,
        status: &str,
        error_message: Option<String>,
    ) -> Result<()> {
        let job_id_bytes = job_id.as_bytes();
        let now = chrono::Utc::now();

        let mut conn = self.db.acquire().await?;
        if status == "running" {
            sqlx::query("UPDATE job_queue SET status = ?, started_at = ? WHERE id = ?")
                .bind(status)
                .bind(now)
                .bind(&job_id_bytes[..])
                .execute(&mut *conn)
                .await?;
        } else {
            sqlx::query(
                "UPDATE job_queue SET status = ?, completed_at = ?, error_message = ? WHERE id = ?",
            )
            .bind(status)
            .bind(now)
            .bind(error_message)
            .bind(&job_id_bytes[..])
            .execute(&mut *conn)
            .await?;
        }

        Ok(())
    }

    // Schedule a new job
    pub async fn schedule_job(
        &self,
        job_type: JobType,
        payload: serde_json::Value,
    ) -> Result<Uuid> {
        let job_id = Uuid::new_v4();
        let job_id_bytes = job_id.as_bytes();

        let mut conn = self.db.acquire().await?;
        sqlx::query(
            "INSERT INTO job_queue (id, job_type, payload, status) VALUES (?, ?, ?, 'pending')",
        )
        .bind(&job_id_bytes[..])
        .bind(job_type.as_str())
        .bind(payload.to_string())
        .execute(&mut *conn)
        .await?;

        Ok(job_id)
    }

    /// Schedule OAuth credential validation job
    pub async fn schedule_oauth_validation(&self) -> Result<Uuid> {
        self.schedule_job(JobType::OAuthCredentialValidation, serde_json::json!({}))
            .await
    }

    /// Start periodic OAuth credential validation (runs every 6 hours)
    pub async fn start_oauth_validation_scheduler(&self) {
        let mut interval = interval(Duration::from_secs(6 * 3600)); // Every 6 hours

        loop {
            interval.tick().await;

            match self.schedule_oauth_validation().await {
                Ok(job_id) => {
                    tracing::info!("Scheduled OAuth credential validation job: {}", job_id);
                }
                Err(e) => {
                    tracing::error!("Failed to schedule OAuth credential validation: {}", e);
                }
            }
        }
    }
}
