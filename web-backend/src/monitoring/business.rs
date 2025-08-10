use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Business metrics collector for learning platform KPIs
#[derive(Debug, Clone)]
pub struct BusinessMetricsCollector {
    // User engagement metrics
    pub daily_active_users: Arc<AtomicU64>,
    pub weekly_active_users: Arc<AtomicU64>,
    pub monthly_active_users: Arc<AtomicU64>,
    pub user_retention_rate: Arc<AtomicU64>, // Stored as percentage * 1000 for precision

    // Learning effectiveness metrics (stored as scaled integers)
    pub average_session_duration: Arc<AtomicU64>, // in minutes * 1000
    pub task_completion_rate: Arc<AtomicU64>,     // percentage * 1000
    pub accuracy_rate: Arc<AtomicU64>,            // percentage * 1000
    pub learning_velocity: Arc<AtomicU64>,        // tasks per hour * 1000

    // Content engagement metrics
    pub tasks_attempted: Arc<AtomicU64>,
    pub tasks_completed: Arc<AtomicU64>,
    pub hints_requested: Arc<AtomicU64>,
    pub interventions_triggered: Arc<AtomicU64>,

    // Conversion and growth metrics
    pub new_user_signups: Arc<AtomicU64>,
    pub trial_to_paid_conversion: Arc<AtomicU64>, // percentage * 1000
    pub user_churn_rate: Arc<AtomicU64>,          // percentage * 1000
    pub customer_lifetime_value: Arc<AtomicU64>,  // dollars * 100

    // Platform health metrics
    pub error_impact_score: Arc<AtomicU64>, // business impact * 1000
    pub feature_adoption_rate: Arc<AtomicU64>, // percentage * 1000
    pub user_satisfaction_score: Arc<AtomicU64>, // score * 1000

    // Time-series data storage
    pub daily_snapshots: Arc<RwLock<Vec<DailyBusinessMetrics>>>,
    pub user_journey_data: Arc<RwLock<HashMap<Uuid, UserJourneyMetrics>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyBusinessMetrics {
    pub date: DateTime<Utc>,
    pub active_users: u64,
    pub new_signups: u64,
    pub total_sessions: u64,
    pub average_session_duration_minutes: f64,
    pub task_completion_rate: f64,
    pub user_retention_rate: f64,
    pub revenue_metrics: RevenueMetrics,
    pub engagement_metrics: EngagementMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueMetrics {
    pub mrr: f64, // Monthly Recurring Revenue
    pub arr: f64, // Annual Recurring Revenue
    pub new_revenue: f64,
    pub churn_revenue: f64,
    pub expansion_revenue: f64,
    pub customer_acquisition_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementMetrics {
    pub tasks_per_session: f64,
    pub bounce_rate: f64,
    pub feature_usage: HashMap<String, f64>,
    pub user_feedback_score: f64,
    pub support_ticket_volume: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJourneyMetrics {
    pub user_id: Uuid,
    pub first_session: DateTime<Utc>,
    pub last_session: DateTime<Utc>,
    pub total_sessions: u64,
    pub total_tasks_attempted: u64,
    pub total_tasks_completed: u64,
    pub current_streak: u64,
    pub longest_streak: u64,
    pub learning_path_progress: f64,
    pub skill_improvements: HashMap<String, f64>,
    pub subscription_tier: String,
    pub lifecycle_stage: UserLifecycleStage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserLifecycleStage {
    Trial,
    Onboarding,
    Active,
    AtRisk,
    Churned,
    Reactivated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEffectivenessMetrics {
    pub skill_progression_velocity: f64,
    pub knowledge_retention_rate: f64,
    pub adaptive_difficulty_effectiveness: f64,
    pub personalization_impact: f64,
    pub intervention_success_rate: f64,
}

impl Default for BusinessMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl BusinessMetricsCollector {
    pub fn new() -> Self {
        Self {
            daily_active_users: Arc::new(AtomicU64::new(0)),
            weekly_active_users: Arc::new(AtomicU64::new(0)),
            monthly_active_users: Arc::new(AtomicU64::new(0)),
            user_retention_rate: Arc::new(AtomicU64::new(0)),
            average_session_duration: Arc::new(AtomicU64::new(0)),
            task_completion_rate: Arc::new(AtomicU64::new(0)),
            accuracy_rate: Arc::new(AtomicU64::new(0)),
            learning_velocity: Arc::new(AtomicU64::new(0)),
            tasks_attempted: Arc::new(AtomicU64::new(0)),
            tasks_completed: Arc::new(AtomicU64::new(0)),
            hints_requested: Arc::new(AtomicU64::new(0)),
            interventions_triggered: Arc::new(AtomicU64::new(0)),
            new_user_signups: Arc::new(AtomicU64::new(0)),
            trial_to_paid_conversion: Arc::new(AtomicU64::new(0)),
            user_churn_rate: Arc::new(AtomicU64::new(0)),
            customer_lifetime_value: Arc::new(AtomicU64::new(0)),
            error_impact_score: Arc::new(AtomicU64::new(0)),
            feature_adoption_rate: Arc::new(AtomicU64::new(0)),
            user_satisfaction_score: Arc::new(AtomicU64::new(0)),
            daily_snapshots: Arc::new(RwLock::new(Vec::new())),
            user_journey_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a new user signup
    pub async fn record_user_signup(&self, user_id: Uuid, trial: bool) {
        self.new_user_signups.fetch_add(1, Ordering::Relaxed);

        // Initialize user journey tracking
        let mut user_journeys = self.user_journey_data.write().await;
        user_journeys.insert(
            user_id,
            UserJourneyMetrics {
                user_id,
                first_session: Utc::now(),
                last_session: Utc::now(),
                total_sessions: 0,
                total_tasks_attempted: 0,
                total_tasks_completed: 0,
                current_streak: 0,
                longest_streak: 0,
                learning_path_progress: 0.0,
                skill_improvements: HashMap::new(),
                subscription_tier: if trial {
                    "trial".to_string()
                } else {
                    "free".to_string()
                },
                lifecycle_stage: UserLifecycleStage::Trial,
            },
        );

        tracing::info!(
            user_id = %user_id,
            trial = trial,
            "New user signup recorded"
        );
    }

    /// Record a learning session
    pub async fn record_learning_session(
        &self,
        user_id: Uuid,
        duration_minutes: f64,
        tasks_attempted: u64,
        tasks_completed: u64,
    ) {
        // Update global metrics
        self.tasks_attempted
            .fetch_add(tasks_attempted, Ordering::Relaxed);
        self.tasks_completed
            .fetch_add(tasks_completed, Ordering::Relaxed);

        // Update running averages
        let current_avg = f64::from_bits(self.average_session_duration.load(Ordering::Relaxed));
        let new_avg = (current_avg + duration_minutes) / 2.0; // Simplified moving average
        self.average_session_duration
            .store(new_avg.to_bits(), Ordering::Relaxed);

        if tasks_attempted > 0 {
            let completion_rate = tasks_completed as f64 / tasks_attempted as f64;
            let current_rate = f64::from_bits(self.task_completion_rate.load(Ordering::Relaxed));
            let new_rate = (current_rate + completion_rate) / 2.0;
            self.task_completion_rate
                .store(new_rate.to_bits(), Ordering::Relaxed);
        }

        // Update user journey data
        let mut user_journeys = self.user_journey_data.write().await;
        if let Some(journey) = user_journeys.get_mut(&user_id) {
            journey.last_session = Utc::now();
            journey.total_sessions += 1;
            journey.total_tasks_attempted += tasks_attempted;
            journey.total_tasks_completed += tasks_completed;

            if tasks_completed > 0 {
                journey.current_streak += 1;
                journey.longest_streak = journey.longest_streak.max(journey.current_streak);
            } else if tasks_attempted > 0 {
                journey.current_streak = 0; // Reset streak on failed attempts
            }

            // Update lifecycle stage based on activity
            journey.lifecycle_stage = self.determine_lifecycle_stage(journey);
        }

        tracing::debug!(
            user_id = %user_id,
            duration_minutes = duration_minutes,
            tasks_attempted = tasks_attempted,
            tasks_completed = tasks_completed,
            "Learning session recorded"
        );
    }

    /// Record task performance
    pub async fn record_task_performance(
        &self,
        user_id: Uuid,
        task_type: &str,
        correct: bool,
        response_time_ms: u64,
        hint_used: bool,
    ) {
        if correct {
            self.tasks_completed.fetch_add(1, Ordering::Relaxed);
        }
        self.tasks_attempted.fetch_add(1, Ordering::Relaxed);

        if hint_used {
            self.hints_requested.fetch_add(1, Ordering::Relaxed);
        }

        // Update accuracy rate
        let total_completed = self.tasks_completed.load(Ordering::Relaxed) as f64;
        let total_attempted = self.tasks_attempted.load(Ordering::Relaxed) as f64;
        if total_attempted > 0.0 {
            let accuracy = total_completed / total_attempted;
            self.accuracy_rate
                .store(accuracy.to_bits(), Ordering::Relaxed);
        }

        // Update learning velocity (tasks per hour)
        let velocity = total_attempted / (Utc::now().timestamp() as f64 / 3600.0); // Simplified
        self.learning_velocity
            .store(velocity.to_bits(), Ordering::Relaxed);

        // Update user-specific skill tracking
        let mut user_journeys = self.user_journey_data.write().await;
        if let Some(journey) = user_journeys.get_mut(&user_id) {
            let skill_improvement = if correct {
                if response_time_ms < 3000 {
                    0.1
                } else {
                    0.05
                }
            } else {
                -0.02
            };

            *journey
                .skill_improvements
                .entry(task_type.to_string())
                .or_insert(0.0) += skill_improvement;
        }

        tracing::debug!(
            user_id = %user_id,
            task_type = task_type,
            correct = correct,
            response_time_ms = response_time_ms,
            hint_used = hint_used,
            "Task performance recorded"
        );
    }

    /// Record business event (subscription, churn, etc.)
    pub async fn record_business_event(
        &self,
        user_id: Uuid,
        event_type: BusinessEventType,
        value: f64,
    ) {
        match event_type {
            BusinessEventType::Subscription { tier } => {
                let mut user_journeys = self.user_journey_data.write().await;
                if let Some(journey) = user_journeys.get_mut(&user_id) {
                    journey.subscription_tier = tier;
                    journey.lifecycle_stage = UserLifecycleStage::Active;
                }

                // Update conversion rate
                let total_signups = self.new_user_signups.load(Ordering::Relaxed) as f64;
                if total_signups > 0.0 {
                    let current_conversions =
                        f64::from_bits(self.trial_to_paid_conversion.load(Ordering::Relaxed))
                            * total_signups;
                    let new_conversion_rate = (current_conversions + 1.0) / total_signups;
                    self.trial_to_paid_conversion
                        .store(new_conversion_rate.to_bits(), Ordering::Relaxed);
                }
            }
            BusinessEventType::Churn => {
                let mut user_journeys = self.user_journey_data.write().await;
                if let Some(journey) = user_journeys.get_mut(&user_id) {
                    journey.lifecycle_stage = UserLifecycleStage::Churned;
                }

                // Update churn rate (simplified)
                let current_churn = f64::from_bits(self.user_churn_rate.load(Ordering::Relaxed));
                let new_churn = (current_churn + 0.01).min(1.0); // Increment churn
                self.user_churn_rate
                    .store(new_churn.to_bits(), Ordering::Relaxed);
            }
            BusinessEventType::Revenue => {
                // Update customer lifetime value
                let current_clv =
                    f64::from_bits(self.customer_lifetime_value.load(Ordering::Relaxed));
                let new_clv = current_clv + value;
                self.customer_lifetime_value
                    .store(new_clv.to_bits(), Ordering::Relaxed);
            }
            BusinessEventType::SatisfactionScore => {
                // Update user satisfaction (value should be 1-5 scale)
                let current_score =
                    f64::from_bits(self.user_satisfaction_score.load(Ordering::Relaxed));
                let new_score = (current_score + value) / 2.0; // Moving average
                self.user_satisfaction_score
                    .store(new_score.to_bits(), Ordering::Relaxed);
            }
        }

        tracing::info!(
            user_id = %user_id,
            event_type = ?event_type,
            value = value,
            "Business event recorded"
        );
    }

    /// Generate daily business metrics snapshot
    pub async fn generate_daily_snapshot(&self) -> DailyBusinessMetrics {
        let daily_active = self.calculate_daily_active_users().await;
        let total_sessions = self.calculate_daily_sessions().await;

        DailyBusinessMetrics {
            date: Utc::now(),
            active_users: daily_active,
            new_signups: self.new_user_signups.load(Ordering::Relaxed),
            total_sessions,
            average_session_duration_minutes: f64::from_bits(
                self.average_session_duration.load(Ordering::Relaxed),
            ),
            task_completion_rate: f64::from_bits(self.task_completion_rate.load(Ordering::Relaxed)),
            user_retention_rate: f64::from_bits(self.user_retention_rate.load(Ordering::Relaxed)),
            revenue_metrics: self.calculate_revenue_metrics().await,
            engagement_metrics: self.calculate_engagement_metrics().await,
        }
    }

    /// Get current business KPI dashboard
    pub async fn get_business_dashboard(&self) -> BusinessDashboard {
        BusinessDashboard {
            kpis: BusinessKPIs {
                daily_active_users: self.daily_active_users.load(Ordering::Relaxed),
                weekly_active_users: self.weekly_active_users.load(Ordering::Relaxed),
                monthly_active_users: self.monthly_active_users.load(Ordering::Relaxed),
                user_retention_rate: f64::from_bits(
                    self.user_retention_rate.load(Ordering::Relaxed),
                ),
                task_completion_rate: f64::from_bits(
                    self.task_completion_rate.load(Ordering::Relaxed),
                ),
                learning_velocity: f64::from_bits(self.learning_velocity.load(Ordering::Relaxed)),
                customer_lifetime_value: f64::from_bits(
                    self.customer_lifetime_value.load(Ordering::Relaxed),
                ),
                trial_to_paid_conversion: f64::from_bits(
                    self.trial_to_paid_conversion.load(Ordering::Relaxed),
                ),
                user_satisfaction_score: f64::from_bits(
                    self.user_satisfaction_score.load(Ordering::Relaxed),
                ),
            },
            trends: self.calculate_trends().await,
            alerts: self.generate_business_alerts().await,
            last_updated: Utc::now(),
        }
    }

    // Helper methods
    fn determine_lifecycle_stage(&self, journey: &UserJourneyMetrics) -> UserLifecycleStage {
        let days_since_last_session = (Utc::now() - journey.last_session).num_days();

        match journey.subscription_tier.as_str() {
            "trial" => {
                if days_since_last_session > 7 {
                    UserLifecycleStage::AtRisk
                } else if journey.total_sessions < 3 {
                    UserLifecycleStage::Onboarding
                } else {
                    UserLifecycleStage::Trial
                }
            }
            "free" => {
                if days_since_last_session > 14 {
                    UserLifecycleStage::Churned
                } else if journey.total_tasks_completed > 50 {
                    UserLifecycleStage::Active
                } else {
                    UserLifecycleStage::Active
                }
            }
            _ => {
                if days_since_last_session > 30 {
                    UserLifecycleStage::Churned
                } else if days_since_last_session > 7 {
                    UserLifecycleStage::AtRisk
                } else {
                    UserLifecycleStage::Active
                }
            }
        }
    }

    async fn calculate_daily_active_users(&self) -> u64 {
        let user_journeys = self.user_journey_data.read().await;
        let today = Utc::now().date_naive();

        user_journeys
            .values()
            .filter(|journey| journey.last_session.date_naive() == today)
            .count() as u64
    }

    async fn calculate_daily_sessions(&self) -> u64 {
        let user_journeys = self.user_journey_data.read().await;
        user_journeys
            .values()
            .map(|journey| journey.total_sessions)
            .sum()
    }

    async fn calculate_revenue_metrics(&self) -> RevenueMetrics {
        // Simplified revenue calculations
        let user_journeys = self.user_journey_data.read().await;
        let paid_users = user_journeys
            .values()
            .filter(|j| j.subscription_tier != "trial" && j.subscription_tier != "free")
            .count() as f64;

        RevenueMetrics {
            mrr: paid_users * 29.99, // Assuming $29.99/month
            arr: paid_users * 29.99 * 12.0,
            new_revenue: paid_users * 29.99,
            churn_revenue: 0.0,              // Would need historical data
            expansion_revenue: 0.0,          // Would need upgrade tracking
            customer_acquisition_cost: 50.0, // Assumed CAC
        }
    }

    async fn calculate_engagement_metrics(&self) -> EngagementMetrics {
        let user_journeys = self.user_journey_data.read().await;
        let total_users = user_journeys.len() as f64;

        if total_users == 0.0 {
            return EngagementMetrics {
                tasks_per_session: 0.0,
                bounce_rate: 0.0,
                feature_usage: HashMap::new(),
                user_feedback_score: 0.0,
                support_ticket_volume: 0,
            };
        }

        let avg_tasks_per_session = user_journeys
            .values()
            .map(|j| {
                if j.total_sessions > 0 {
                    j.total_tasks_attempted as f64 / j.total_sessions as f64
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / total_users;

        let single_session_users = user_journeys
            .values()
            .filter(|j| j.total_sessions <= 1)
            .count() as f64;

        EngagementMetrics {
            tasks_per_session: avg_tasks_per_session,
            bounce_rate: single_session_users / total_users,
            feature_usage: HashMap::new(), // Would need feature tracking
            user_feedback_score: f64::from_bits(
                self.user_satisfaction_score.load(Ordering::Relaxed),
            ),
            support_ticket_volume: 0, // Would need support system integration
        }
    }

    async fn calculate_trends(&self) -> Vec<TrendData> {
        // Simplified trend calculation - would use historical data in practice
        vec![
            TrendData {
                metric: "Daily Active Users".to_string(),
                current_value: self.daily_active_users.load(Ordering::Relaxed) as f64,
                previous_value: (self.daily_active_users.load(Ordering::Relaxed) as f64) * 0.9, // Mock
                change_percent: 10.0,
                trend_direction: TrendDirection::Up,
            },
            TrendData {
                metric: "Task Completion Rate".to_string(),
                current_value: f64::from_bits(self.task_completion_rate.load(Ordering::Relaxed)),
                previous_value: f64::from_bits(self.task_completion_rate.load(Ordering::Relaxed))
                    * 0.95,
                change_percent: 5.0,
                trend_direction: TrendDirection::Up,
            },
        ]
    }

    async fn generate_business_alerts(&self) -> Vec<BusinessAlert> {
        let mut alerts = Vec::new();

        // Check for concerning metrics
        let churn_rate = f64::from_bits(self.user_churn_rate.load(Ordering::Relaxed));
        if churn_rate > 0.1 {
            alerts.push(BusinessAlert {
                severity: AlertSeverity::High,
                metric: "User Churn Rate".to_string(),
                current_value: churn_rate,
                threshold: 0.1,
                message: "User churn rate is above acceptable threshold".to_string(),
                created_at: Utc::now(),
            });
        }

        let completion_rate = f64::from_bits(self.task_completion_rate.load(Ordering::Relaxed));
        if completion_rate < 0.5 {
            alerts.push(BusinessAlert {
                severity: AlertSeverity::Medium,
                metric: "Task Completion Rate".to_string(),
                current_value: completion_rate,
                threshold: 0.5,
                message: "Task completion rate is below target".to_string(),
                created_at: Utc::now(),
            });
        }

        alerts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusinessEventType {
    Subscription { tier: String },
    Churn,
    Revenue,
    SatisfactionScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessKPIs {
    pub daily_active_users: u64,
    pub weekly_active_users: u64,
    pub monthly_active_users: u64,
    pub user_retention_rate: f64,
    pub task_completion_rate: f64,
    pub learning_velocity: f64,
    pub customer_lifetime_value: f64,
    pub trial_to_paid_conversion: f64,
    pub user_satisfaction_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDashboard {
    pub kpis: BusinessKPIs,
    pub trends: Vec<TrendData>,
    pub alerts: Vec<BusinessAlert>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub metric: String,
    pub current_value: f64,
    pub previous_value: f64,
    pub change_percent: f64,
    pub trend_direction: TrendDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessAlert {
    pub severity: AlertSeverity,
    pub metric: String,
    pub current_value: f64,
    pub threshold: f64,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Global business metrics instance
static BUSINESS_METRICS: once_cell::sync::Lazy<BusinessMetricsCollector> =
    once_cell::sync::Lazy::new(BusinessMetricsCollector::new);

pub fn global_business_metrics() -> &'static BusinessMetricsCollector {
    &BUSINESS_METRICS
}
