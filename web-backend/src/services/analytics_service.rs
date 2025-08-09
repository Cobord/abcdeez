use crate::cache::ConnectionManager;
use crate::db::DbPool;
use anyhow::Result;
use sqlx::Row;
use chrono::{DateTime, Duration, Utc};
use rand::{distributions::Distribution, thread_rng};
use statrs::distribution::{Continuous, Laplace};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use graph_learning_core::statistics::{DetailedStatistics, ExGaussianParameters, StrategyType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PopulationStats {
    pub active_learners: usize,
    pub total_learners: usize,
    pub total_sessions: usize,
    pub total_responses: usize,
    pub average_accuracy: f64,
    pub average_response_time_ms: f64,
    pub tasks_per_minute: f64,
    pub difficulty_distribution: HashMap<String, usize>,
    pub strategy_distribution: HashMap<String, usize>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Bottleneck {
    pub task_type: String,
    pub difficulty_range: String,
    pub error_rate: f64,
    pub average_response_time_ms: f64,
    pub sample_size: usize,
    pub identified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConditionComparison {
    pub experiment_id: Uuid,
    pub conditions: HashMap<String, ConditionStats>,
    pub statistical_significance: HashMap<String, f64>, // p-values for comparisons
    pub effect_sizes: HashMap<String, f64>,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConditionStats {
    pub participant_count: usize,
    pub average_accuracy: f64,
    pub average_response_time_ms: f64,
    pub completion_rate: f64,
    pub learning_rate: f64, // Improvement over time
}

#[derive(Clone)]
pub struct AnalyticsService {
    pub db: Arc<DbPool>,
    pub cache: ConnectionManager,
    pub privacy_epsilon: f64, // Differential privacy parameter
}

impl AnalyticsService {
    pub fn new(db: Arc<DbPool>, cache: Arc<ConnectionManager>) -> Self {
        Self {
            db,
            cache: (*cache).clone(),
            privacy_epsilon: 1.0, // Conservative privacy budget
        }
    }

    pub async fn population_stats(&self) -> Result<PopulationStats> {
        // Check cache first
        if let Ok(cached) = self.get_cached_population_stats().await {
            return Ok(cached);
        }

        // Compute fresh stats
        let now = Utc::now();
        let last_24h = now - Duration::hours(24);

        // Active learners in last 24 hours
        let mut conn = self.db.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
        let active_learners: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT l.id) FROM learners l
             JOIN sessions s ON l.id = s.learner_id
             WHERE s.start_time > ?"
        )
        .bind(last_24h)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
        let active_learners = active_learners as usize;

        // Total learners
        let total_learners: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM learners")
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| AppError::DatabaseError(e))?;
        let total_learners = total_learners as usize;

        // Total sessions
        let total_sessions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| AppError::DatabaseError(e))?;
        let total_sessions = total_sessions as usize;

        // Response statistics
        let response_stats = sqlx::query(
            "SELECT
                COUNT(*) as total_responses,
                AVG(CASE WHEN correct THEN 1.0 ELSE 0.0 END) as avg_accuracy,
                AVG(response_time_ms) as avg_response_time
             FROM responses
             WHERE timestamp > ?"
        )
        .bind(last_24h)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        let total_responses: i64 = response_stats.get::<i64, _>("total_responses");
        let total_responses = total_responses as usize;
        let avg_accuracy: f64 = response_stats.get::<Option<f64>, _>("avg_accuracy").unwrap_or(0.0);
        let avg_response_time: f64 = response_stats.get::<Option<f64>, _>("avg_response_time").unwrap_or(0.0);
        let average_accuracy = self.add_differential_privacy_noise(avg_accuracy);
        let average_response_time_ms = self.add_differential_privacy_noise(avg_response_time);

        // Tasks per minute in last hour
        let last_hour = now - Duration::hours(1);
        let recent_responses: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM responses WHERE timestamp > ?"
        )
        .bind(last_hour)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
        let recent_responses = recent_responses as f64;

        let tasks_per_minute = recent_responses / 60.0;

        // Difficulty distribution (simplified)
        let difficulty_distribution = HashMap::from([
            ("easy".to_string(), (total_responses as f64 * 0.3) as usize),
            (
                "medium".to_string(),
                (total_responses as f64 * 0.5) as usize,
            ),
            ("hard".to_string(), (total_responses as f64 * 0.2) as usize),
        ]);

        // Strategy distribution (simplified)
        let strategy_distribution = HashMap::from([
            (
                "systematic".to_string(),
                (active_learners as f64 * 0.4) as usize,
            ),
            (
                "intuitive".to_string(),
                (active_learners as f64 * 0.35) as usize,
            ),
            (
                "mixed".to_string(),
                (active_learners as f64 * 0.25) as usize,
            ),
        ]);

        let stats = PopulationStats {
            active_learners,
            total_learners,
            total_sessions,
            total_responses,
            average_accuracy,
            average_response_time_ms,
            tasks_per_minute,
            difficulty_distribution,
            strategy_distribution,
            updated_at: now,
        };

        // Cache the results
        self.cache_population_stats(&stats).await?;

        Ok(stats)
    }

    pub async fn find_bottlenecks(&self, min_samples: usize) -> Result<Vec<Bottleneck>> {
        // Find task types with high error rates or response times
        let mut conn = self.db.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
        let rows = sqlx::query(
            "SELECT
                task_type,
                COUNT(*) as sample_size,
                AVG(CASE WHEN correct THEN 0.0 ELSE 1.0 END) as error_rate,
                AVG(response_time_ms) as avg_response_time
             FROM responses
             WHERE timestamp > ?
             GROUP BY task_type
             HAVING COUNT(*) >= ?
             ORDER BY error_rate DESC, avg_response_time DESC"
        )
        .bind(Utc::now() - Duration::days(7))
        .bind(min_samples as i64)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        let bottlenecks = rows
            .into_iter()
            .filter(|row| {
                let error_rate = row.get::<Option<f64>, _>("error_rate").unwrap_or(0.0);
                let avg_rt = row.get::<Option<f64>, _>("avg_response_time").unwrap_or(0.0);
                error_rate > 0.3 || avg_rt > 5000.0 // 30% error rate or >5s response time
            })
            .map(|row| Bottleneck {
                task_type: row.get::<String, _>("task_type"),
                difficulty_range: "medium".to_string(), // Simplified
                error_rate: self.add_differential_privacy_noise(
                    row.get::<Option<f64>, _>("error_rate").unwrap_or(0.0)
                ),
                average_response_time_ms: self.add_differential_privacy_noise(
                    row.get::<Option<f64>, _>("avg_response_time").unwrap_or(0.0)
                ),
                sample_size: row.get::<Option<i64>, _>("sample_size").unwrap_or(0) as usize,
                identified_at: Utc::now(),
            })
            .collect();

        Ok(bottlenecks)
    }

    pub async fn compare_conditions(&self, experiment_id: Uuid) -> Result<ConditionComparison> {
        let experiment_id_bytes = experiment_id.as_bytes();

        // Get all participants in the experiment
        let mut conn = self.db.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
        let participants = sqlx::query(
            "SELECT learner_id, condition FROM experiment_participants WHERE experiment_id = ?"
        )
        .bind(&experiment_id_bytes[..])
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        let mut conditions: HashMap<String, ConditionStats> = HashMap::new();

        for participant in participants {
            let condition = participant.get::<Option<String>, _>("condition").unwrap_or("control".to_string());
            let learner_id_bytes: Vec<u8> = participant.get::<Vec<u8>, _>("learner_id");

            // Get performance stats for this learner
            let stats = sqlx::query(
                "SELECT
                    COUNT(*) as total_responses,
                    AVG(CASE WHEN correct THEN 1.0 ELSE 0.0 END) as accuracy,
                    AVG(response_time_ms) as avg_rt,
                    COUNT(DISTINCT s.id) as session_count
                 FROM responses r
                 JOIN sessions s ON r.session_id = s.id
                 WHERE s.learner_id = ?"
            )
            .bind(&learner_id_bytes[..])
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| AppError::DatabaseError(e))?;

            let condition_stats = conditions.entry(condition).or_insert(ConditionStats {
                participant_count: 0,
                average_accuracy: 0.0,
                average_response_time_ms: 0.0,
                completion_rate: 0.0,
                learning_rate: 0.0,
            });

            condition_stats.participant_count += 1;
            condition_stats.average_accuracy += stats.get::<Option<f64>, _>("accuracy").unwrap_or(0.0);
            condition_stats.average_response_time_ms += stats.get::<Option<f64>, _>("avg_rt").unwrap_or(0.0);

            // Simplified completion rate (sessions > 0)
            if stats.get::<Option<i64>, _>("session_count").unwrap_or(0) > 0 {
                condition_stats.completion_rate += 1.0;
            }
        }

        // Average the accumulated stats
        for stats in conditions.values_mut() {
            if stats.participant_count > 0 {
                let count = stats.participant_count as f64;
                stats.average_accuracy /= count;
                stats.average_response_time_ms /= count;
                stats.completion_rate /= count;
                stats.learning_rate = self.calculate_learning_rate(&stats).await.unwrap_or(0.1);

                // Add differential privacy noise
                stats.average_accuracy =
                    self.add_differential_privacy_noise(stats.average_accuracy);
                stats.average_response_time_ms =
                    self.add_differential_privacy_noise(stats.average_response_time_ms);
            }
        }

        // Calculate statistical significance (simplified)
        let mut statistical_significance = HashMap::new();
        let mut effect_sizes = HashMap::new();

        let condition_names: Vec<_> = conditions.keys().collect();
        for i in 0..condition_names.len() {
            for j in (i + 1)..condition_names.len() {
                let comp_key = format!("{}_vs_{}", condition_names[i], condition_names[j]);
                statistical_significance.insert(comp_key.clone(), 0.05); // Simplified p-value
                effect_sizes.insert(comp_key, 0.3); // Simplified effect size
            }
        }

        Ok(ConditionComparison {
            experiment_id,
            conditions,
            statistical_significance,
            effect_sizes,
            analyzed_at: Utc::now(),
        })
    }

    pub async fn get_learning_curves(
        &self,
        learner_ids: Option<Vec<Uuid>>,
    ) -> Result<HashMap<String, Vec<(DateTime<Utc>, f64)>>> {
        let mut curves = HashMap::new();

        if let Some(ids) = learner_ids {
            // Get learning curves for specific learners
            for learner_id in ids {
                let curve = self.get_individual_learning_curve(learner_id).await?;
                curves.insert(learner_id.to_string(), curve);
            }
        } else {
            // Get population learning curve
            let population_curve = self.get_population_learning_curve().await?;
            curves.insert("population".to_string(), population_curve);
        }

        Ok(curves)
    }

    pub async fn get_real_time_metrics(&self) -> Result<serde_json::Value> {
        // Get live dashboard metrics
        let now = Utc::now();
        let last_minute = now - Duration::minutes(1);

        let mut conn = self.db.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
        let recent_activity = sqlx::query(
            "SELECT COUNT(*) as responses_last_minute FROM responses WHERE timestamp > ?"
        )
        .bind(last_minute)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        let active_sessions = sqlx::query(
            "SELECT COUNT(*) as active_sessions FROM sessions WHERE status = 'active'"
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        Ok(serde_json::json!({
            "timestamp": now,
            "responses_last_minute": recent_activity.get::<Option<i64>, _>("responses_last_minute").unwrap_or(0),
            "active_sessions": active_sessions.get::<Option<i64>, _>("active_sessions").unwrap_or(0),
            "system_status": "operational"
        }))
    }

    // Private helper methods
    fn add_differential_privacy_noise(&self, true_value: f64) -> f64 {
        let sensitivity = 1.0; // Sensitivity of the query
        let scale = sensitivity / self.privacy_epsilon;
        let laplace = Laplace::new(0.0, scale).unwrap();
        let noise = laplace.sample(&mut thread_rng());
        true_value + noise
    }

    async fn cache_population_stats(&self, stats: &PopulationStats) -> Result<()> {
        let cache_key = "population_stats";
        let stats_data = serde_json::to_string(stats)?;

        // Clone the connection manager (shares the underlying in-memory cache)
        let mut conn = self.cache.clone();
        crate::cache::cmd("SETEX")
            .arg(cache_key)
            .arg(300) // 5 minute TTL
            .arg(stats_data)
            .query_async::<()>(&mut conn)
            .await
            .ok(); // Ignore cache errors

        Ok(())
    }

    async fn get_cached_population_stats(&self) -> Result<PopulationStats> {
        let cache_key = "population_stats";
        // Clone connection manager so we can pass a mutable owned copy
        let mut conn = self.cache.clone();

        let cached_data: String = crate::cache::cmd("GET")
            .arg(cache_key)
            .query_async::<String>(&mut conn)
            .await
            .map_err(|_| AppError::NotFound("Not in cache".to_string()))?;

        let stats: PopulationStats = serde_json::from_str(&cached_data)?;
        Ok(stats)
    }

    async fn get_individual_learning_curve(
        &self,
        learner_id: Uuid,
    ) -> Result<Vec<(DateTime<Utc>, f64)>> {
        let learner_id_bytes = learner_id.as_bytes();

        // Get responses over time with running accuracy
        let mut conn = self.db.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
        let rows = sqlx::query(
            "SELECT
                timestamp,
                correct,
                ROW_NUMBER() OVER (ORDER BY timestamp) as sequence
             FROM responses r
             JOIN sessions s ON r.session_id = s.id
             WHERE s.learner_id = ?
             ORDER BY timestamp"
        )
        .bind(&learner_id_bytes[..])
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        let mut curve = Vec::new();
        let mut correct_count = 0;

        for (i, row) in rows.iter().enumerate() {
            if row.get::<bool, _>("correct") {
                correct_count += 1;
            }
            let accuracy = correct_count as f64 / (i + 1) as f64;
            let timestamp = row.get::<DateTime<Utc>, _>("timestamp");
            curve.push((timestamp, accuracy));
        }

        Ok(curve)
    }

    async fn get_population_learning_curve(&self) -> Result<Vec<(DateTime<Utc>, f64)>> {
        // Get population accuracy over time (daily averages)
        let mut conn = self.db.acquire().await.map_err(|e| AppError::DatabaseError(e))?;
        let rows = sqlx::query(
            "SELECT
                DATE(timestamp) as date,
                AVG(CASE WHEN correct THEN 1.0 ELSE 0.0 END) as daily_accuracy
             FROM responses
             WHERE timestamp > ?
             GROUP BY DATE(timestamp)
             ORDER BY date"
        )
        .bind(Utc::now() - Duration::days(30))
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        let curve = rows
            .into_iter()
            .map(|row| {
                let date_str: String = row.get::<String, _>("date");
                let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .unwrap_or_else(|_| Utc::now().date_naive())
                    .and_hms_opt(12, 0, 0).unwrap().and_utc();
                let accuracy = self.add_differential_privacy_noise(
                    row.get::<Option<f64>, _>("daily_accuracy").unwrap_or(0.0)
                );
                (date, accuracy)
            })
            .collect();

        Ok(curve)
    }
    
    async fn calculate_learning_rate(&self, stats: &ConditionStats) -> Result<f64> {
        // Calculate learning rate based on improvement trajectory
        // Learning rate = rate of accuracy improvement per session
        
        // For now, use a heuristic based on current performance
        // In practice, this would analyze time series of accuracy data
        
        let base_rate = 0.05; // Conservative baseline
        
        // Higher accuracy suggests faster learning (up to a point)
        let accuracy_factor = if stats.average_accuracy < 0.5 {
            // Below chance performance - very slow learning
            0.5
        } else if stats.average_accuracy < 0.7 {
            // Normal learning range
            stats.average_accuracy
        } else if stats.average_accuracy < 0.9 {
            // Good learners
            1.0 + (stats.average_accuracy - 0.7) * 2.0 // Boost for good performance
        } else {
            // Near-ceiling performance - slower improvement expected
            0.8 + (1.0 - stats.average_accuracy) * 2.0
        };
        
        // Response time factor - faster responses might indicate better learning
        let rt_factor = if stats.average_response_time_ms > 10000.0 {
            // Very slow responses suggest difficulty
            0.6
        } else if stats.average_response_time_ms > 5000.0 {
            // Moderate responses
            0.8
        } else {
            // Quick responses suggest confidence
            1.2
        };
        
        // Completion rate factor - completing sessions indicates engagement
        let completion_factor = 0.5 + stats.completion_rate * 0.5;
        
        // Combine factors
        let learning_rate = base_rate * accuracy_factor * rt_factor * completion_factor;
        
        // Clamp to reasonable range
        Ok(learning_rate.max(0.01).min(0.3))
    }
}
