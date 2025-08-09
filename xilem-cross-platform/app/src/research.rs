use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSession {
    pub id: String,
    pub participant_id: String,
    pub experiment_type: ExperimentType,
    pub condition: ExperimentCondition,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub data_points: Vec<DataPoint>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentType {
    LearningCurve,
    RetentionTest,
    InterferenceStudy,
    AdaptiveScheduling,
    CognitiveLoad,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentCondition {
    pub name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub control_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub trial_number: u32,
    pub stimulus: String,
    pub response: String,
    pub correct: bool,
    pub response_time_ms: u128,
    pub confidence: Option<f64>,
    pub eye_tracking: Option<EyeTrackingData>,
    pub physiological: Option<PhysiologicalData>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EyeTrackingData {
    pub fixation_count: u32,
    pub total_fixation_duration_ms: u128,
    pub saccade_count: u32,
    pub pupil_diameter: Option<f64>,
    pub gaze_path: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologicalData {
    pub heart_rate: Option<f64>,
    pub heart_rate_variability: Option<f64>,
    pub skin_conductance: Option<f64>,
    pub eeg_data: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchMetrics {
    pub learning_rate: f64,
    pub retention_score: f64,
    pub interference_index: f64,
    pub cognitive_load_estimate: f64,
    pub performance_consistency: f64,
    pub adaptation_effectiveness: f64,
}

#[derive(Debug, Clone)]
pub struct ResearchController {
    pub active_session: Option<ResearchSession>,
    pub sessions: Vec<ResearchSession>,
    pub participant_id: String,
    pub current_experiment: Option<ExperimentType>,
    pub data_collection_enabled: bool,
    pub privacy_mode: bool,
}

impl ResearchController {
    pub fn new(participant_id: String) -> Self {
        Self {
            active_session: None,
            sessions: Vec::new(),
            participant_id,
            current_experiment: None,
            data_collection_enabled: true,
            privacy_mode: false,
        }
    }

    pub fn start_experiment(
        &mut self,
        experiment_type: ExperimentType,
        condition: ExperimentCondition,
    ) -> Result<String, String> {
        if self.active_session.is_some() {
            return Err("An experiment is already in progress".to_string());
        }

        let session = ResearchSession {
            id: Uuid::new_v4().to_string(),
            participant_id: self.participant_id.clone(),
            experiment_type: experiment_type.clone(),
            condition,
            start_time: Utc::now(),
            end_time: None,
            data_points: Vec::new(),
            metadata: HashMap::new(),
        };

        let session_id = session.id.clone();
        self.active_session = Some(session);
        self.current_experiment = Some(experiment_type);
        
        Ok(session_id)
    }

    pub fn record_data_point(&mut self, data_point: DataPoint) -> Result<(), String> {
        if let Some(session) = &mut self.active_session {
            if self.data_collection_enabled && !self.privacy_mode {
                session.data_points.push(data_point);
                Ok(())
            } else {
                Err("Data collection is disabled or privacy mode is on".to_string())
            }
        } else {
            Err("No active research session".to_string())
        }
    }

    pub fn end_experiment(&mut self) -> Result<ResearchSession, String> {
        if let Some(mut session) = self.active_session.take() {
            session.end_time = Some(Utc::now());
            self.sessions.push(session.clone());
            self.current_experiment = None;
            Ok(session)
        } else {
            Err("No active experiment to end".to_string())
        }
    }

    pub fn calculate_metrics(&self, session: &ResearchSession) -> ResearchMetrics {
        let data_points = &session.data_points;
        
        if data_points.is_empty() {
            return ResearchMetrics {
                learning_rate: 0.0,
                retention_score: 0.0,
                interference_index: 0.0,
                cognitive_load_estimate: 0.0,
                performance_consistency: 0.0,
                adaptation_effectiveness: 0.0,
            };
        }

        let total_trials = data_points.len() as f64;
        let correct_trials = data_points.iter().filter(|dp| dp.correct).count() as f64;
        let accuracy = correct_trials / total_trials;

        let window_size = 5;
        let mut learning_curve = Vec::new();
        for window in data_points.windows(window_size) {
            let window_correct = window.iter().filter(|dp| dp.correct).count() as f64;
            learning_curve.push(window_correct / window_size as f64);
        }

        let learning_rate = if learning_curve.len() > 1 {
            let first_half = &learning_curve[..learning_curve.len()/2];
            let second_half = &learning_curve[learning_curve.len()/2..];
            let first_avg: f64 = first_half.iter().sum::<f64>() / first_half.len() as f64;
            let second_avg: f64 = second_half.iter().sum::<f64>() / second_half.len() as f64;
            (second_avg - first_avg).max(0.0)
        } else {
            0.0
        };

        let response_times: Vec<f64> = data_points
            .iter()
            .map(|dp| dp.response_time_ms as f64)
            .collect();
        
        let mean_rt = response_times.iter().sum::<f64>() / response_times.len() as f64;
        let variance = response_times
            .iter()
            .map(|rt| (rt - mean_rt).powi(2))
            .sum::<f64>() / response_times.len() as f64;
        let std_dev = variance.sqrt();
        let consistency = 1.0 - (std_dev / mean_rt).min(1.0);

        let cognitive_load = (mean_rt / 1000.0).min(10.0) / 10.0;

        ResearchMetrics {
            learning_rate,
            retention_score: accuracy,
            interference_index: 0.0,
            cognitive_load_estimate: cognitive_load,
            performance_consistency: consistency,
            adaptation_effectiveness: learning_rate * consistency,
        }
    }

    pub fn export_research_data(&self, format: ExportFormat) -> Result<String, String> {
        match format {
            ExportFormat::Json => {
                let data = serde_json::json!({
                    "participant_id": self.participant_id,
                    "sessions": self.sessions,
                    "total_sessions": self.sessions.len(),
                    "total_data_points": self.sessions.iter()
                        .map(|s| s.data_points.len())
                        .sum::<usize>(),
                });
                serde_json::to_string_pretty(&data)
                    .map_err(|e| format!("Failed to serialize: {}", e))
            }
            ExportFormat::Csv => {
                let mut csv_lines = vec![
                    "session_id,experiment_type,trial,stimulus,response,correct,rt_ms,timestamp".to_string()
                ];
                
                for session in &self.sessions {
                    for (i, dp) in session.data_points.iter().enumerate() {
                        csv_lines.push(format!(
                            "{},{},{},{},{},{},{},{}",
                            session.id,
                            format!("{:?}", session.experiment_type),
                            i + 1,
                            dp.stimulus,
                            dp.response,
                            dp.correct,
                            dp.response_time_ms,
                            dp.timestamp.to_rfc3339()
                        ));
                    }
                }
                
                Ok(csv_lines.join("\n"))
            }
            _ => Err("Unsupported export format".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Excel,
    Matlab,
    Python,
}

pub struct ResearchAnalyzer {
    sessions: Vec<ResearchSession>,
}

impl ResearchAnalyzer {
    pub fn new(sessions: Vec<ResearchSession>) -> Self {
        Self { sessions }
    }

    pub fn analyze_learning_curves(&self) -> HashMap<String, Vec<f64>> {
        let mut curves = HashMap::new();
        
        for session in &self.sessions {
            let window_size = 10;
            let mut session_curve = Vec::new();
            
            for window in session.data_points.windows(window_size) {
                let accuracy = window.iter().filter(|dp| dp.correct).count() as f64 
                    / window_size as f64;
                session_curve.push(accuracy);
            }
            
            curves.insert(session.id.clone(), session_curve);
        }
        
        curves
    }

    pub fn analyze_retention(&self, delay_minutes: u64) -> f64 {
        let mut retention_scores = Vec::new();
        
        for session in &self.sessions {
            if let Some(end_time) = session.end_time {
                let delayed_points: Vec<&DataPoint> = session.data_points
                    .iter()
                    .filter(|dp| {
                        let diff = dp.timestamp.signed_duration_since(end_time);
                        diff.num_minutes() >= delay_minutes as i64
                    })
                    .collect();
                
                if !delayed_points.is_empty() {
                    let accuracy = delayed_points.iter()
                        .filter(|dp| dp.correct)
                        .count() as f64 / delayed_points.len() as f64;
                    retention_scores.push(accuracy);
                }
            }
        }
        
        if retention_scores.is_empty() {
            0.0
        } else {
            retention_scores.iter().sum::<f64>() / retention_scores.len() as f64
        }
    }

    pub fn compare_conditions(&self) -> HashMap<String, ResearchMetrics> {
        let mut condition_metrics = HashMap::new();
        let mut condition_sessions: HashMap<String, Vec<&ResearchSession>> = HashMap::new();
        
        for session in &self.sessions {
            condition_sessions
                .entry(session.condition.name.clone())
                .or_insert_with(Vec::new)
                .push(session);
        }
        
        for (condition_name, sessions) in condition_sessions {
            let mut all_metrics = Vec::new();
            
            for session in sessions {
                let controller = ResearchController::new(session.participant_id.clone());
                all_metrics.push(controller.calculate_metrics(session));
            }
            
            if !all_metrics.is_empty() {
                let avg_metrics = ResearchMetrics {
                    learning_rate: all_metrics.iter().map(|m| m.learning_rate).sum::<f64>() 
                        / all_metrics.len() as f64,
                    retention_score: all_metrics.iter().map(|m| m.retention_score).sum::<f64>() 
                        / all_metrics.len() as f64,
                    interference_index: all_metrics.iter().map(|m| m.interference_index).sum::<f64>() 
                        / all_metrics.len() as f64,
                    cognitive_load_estimate: all_metrics.iter().map(|m| m.cognitive_load_estimate).sum::<f64>() 
                        / all_metrics.len() as f64,
                    performance_consistency: all_metrics.iter().map(|m| m.performance_consistency).sum::<f64>() 
                        / all_metrics.len() as f64,
                    adaptation_effectiveness: all_metrics.iter().map(|m| m.adaptation_effectiveness).sum::<f64>() 
                        / all_metrics.len() as f64,
                };
                
                condition_metrics.insert(condition_name, avg_metrics);
            }
        }
        
        condition_metrics
    }
}