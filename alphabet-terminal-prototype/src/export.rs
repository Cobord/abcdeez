use crate::learner::LearnerModel;
use crate::tasks::{TaskResponse, TaskSession};
use crate::statistics::StrategyType;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerDataExport {
    pub learner_id: String,
    pub export_timestamp: DateTime<Utc>,
    pub sessions: Vec<SessionData>,
    pub performance_trajectories: Vec<PerformancePoint>,
    pub error_patterns: ErrorAnalysis,
    pub model_parameters: ModelSnapshot,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub topology_type: String,
    pub responses: Vec<TaskResponse>,
    pub summary: SessionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub total_tasks: usize,
    pub correct_count: usize,
    pub accuracy: f64,
    pub mean_rt_ms: f64,
    pub median_rt_ms: f64,
    pub strategy_detected: Option<StrategyType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePoint {
    pub trial_number: usize,
    pub timestamp: DateTime<Utc>,
    pub accuracy: f64,
    pub mean_rt: f64,
    pub task_type: String,
    pub difficulty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    pub total_errors: usize,
    pub error_rate: f64,
    pub common_confusions: Vec<(String, String, usize)>,
    pub error_by_task_type: HashMap<String, f64>,
    pub error_by_difficulty: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSnapshot {
    pub timestamp: DateTime<Utc>,
    pub node_embeddings: HashMap<String, NodeEmbeddingExport>,
    pub operation_proficiencies: HashMap<String, f64>,
    pub memory_strengths: HashMap<String, f64>,
    pub chunk_boundaries: Vec<ChunkBoundaryExport>,
    pub total_practice_time_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEmbeddingExport {
    pub position: f64,
    pub uncertainty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkBoundaryExport {
    pub position: usize,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub export_version: String,
    pub software_version: String,
    pub platform: String,
    pub experiment_id: Option<String>,
    pub notes: Option<String>,
}

impl LearnerDataExport {
    pub fn from_learner_model(
        model: &LearnerModel,
        sessions: Vec<TaskSession>,
        experiment_id: Option<String>,
    ) -> Self {
        let learner_id = model.learner_id.clone();
        let export_timestamp = Utc::now();
        
        // Convert sessions
        let session_data: Vec<SessionData> = sessions.iter().map(|session| {
            let responses = session.history.clone();
            let summary = Self::calculate_session_summary(&responses);
            
            SessionData {
                session_id: format!("session_{}", uuid::Uuid::new_v4()),
                start_time: responses.first()
                    .map(|r| r.timestamp)
                    .unwrap_or_else(Utc::now),
                end_time: responses.last().map(|r| r.timestamp),
                topology_type: "alphabet".to_string(), // TODO: get from session
                responses,
                summary,
            }
        }).collect();
        
        // Calculate performance trajectories
        let mut performance_trajectories = Vec::new();
        let mut running_correct = 0;
        let mut running_total = 0;
        
        for session in &session_data {
            for (i, response) in session.responses.iter().enumerate() {
                running_total += 1;
                if response.correct {
                    running_correct += 1;
                }
                
                performance_trajectories.push(PerformancePoint {
                    trial_number: running_total,
                    timestamp: response.timestamp,
                    accuracy: running_correct as f64 / running_total as f64,
                    mean_rt: response.response_time_ms as f64,
                    task_type: format!("{:?}", response.task.task_type),
                    difficulty: response.task.difficulty,
                });
            }
        }
        
        // Analyze errors
        let error_patterns = Self::analyze_errors(&session_data);
        
        // Create model snapshot
        let model_snapshot = ModelSnapshot {
            timestamp: export_timestamp,
            node_embeddings: model.node_embeddings.iter()
                .map(|(k, v)| (k.clone(), NodeEmbeddingExport {
                    position: v.position,
                    uncertainty: v.uncertainty,
                }))
                .collect(),
            operation_proficiencies: model.operation_proficiencies.iter()
                .map(|(k, v)| (k.clone(), sigmoid(v.theta)))
                .collect(),
            memory_strengths: model.memory_strengths.iter()
                .map(|(k, v)| (k.clone(), v.strength))
                .collect(),
            chunk_boundaries: model.chunk_boundaries.iter()
                .map(|b| ChunkBoundaryExport {
                    position: b.position,
                    strength: b.strength,
                })
                .collect(),
            total_practice_time_seconds: model.total_practice_time.as_secs(),
        };
        
        let metadata = ExportMetadata {
            export_version: "1.0.0".to_string(),
            software_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            experiment_id,
            notes: None,
        };
        
        LearnerDataExport {
            learner_id,
            export_timestamp,
            sessions: session_data,
            performance_trajectories,
            error_patterns,
            model_parameters: model_snapshot,
            metadata,
        }
    }
    
    fn calculate_session_summary(responses: &[TaskResponse]) -> SessionSummary {
        let total_tasks = responses.len();
        let correct_count = responses.iter().filter(|r| r.correct).count();
        let accuracy = if total_tasks > 0 {
            correct_count as f64 / total_tasks as f64
        } else {
            0.0
        };
        
        let rts: Vec<f64> = responses.iter()
            .map(|r| r.response_time_ms as f64)
            .collect();
        
        let mean_rt_ms = if !rts.is_empty() {
            rts.iter().sum::<f64>() / rts.len() as f64
        } else {
            0.0
        };
        
        let median_rt_ms = if !rts.is_empty() {
            let mut sorted_rts = rts.clone();
            sorted_rts.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted_rts[sorted_rts.len() / 2]
        } else {
            0.0
        };
        
        SessionSummary {
            total_tasks,
            correct_count,
            accuracy,
            mean_rt_ms,
            median_rt_ms,
            strategy_detected: None, // TODO: implement strategy detection
        }
    }
    
    fn analyze_errors(sessions: &[SessionData]) -> ErrorAnalysis {
        let mut total_errors = 0;
        let mut total_tasks = 0;
        let mut confusion_counts: HashMap<(String, String), usize> = HashMap::new();
        let mut error_by_task_type: HashMap<String, (usize, usize)> = HashMap::new();
        let mut error_by_difficulty: HashMap<String, Vec<bool>> = HashMap::new();
        
        for session in sessions {
            for response in &session.responses {
                total_tasks += 1;
                
                let task_type = format!("{:?}", response.task.task_type);
                let difficulty_bucket = format!("{:.1}", response.task.difficulty);
                
                error_by_task_type.entry(task_type.clone())
                    .or_insert((0, 0))
                    .1 += 1;
                
                error_by_difficulty.entry(difficulty_bucket)
                    .or_insert_with(Vec::new)
                    .push(response.correct);
                
                if !response.correct {
                    total_errors += 1;
                    
                    error_by_task_type.entry(task_type)
                        .or_insert((0, 0))
                        .0 += 1;
                    
                    let confusion = (
                        response.task.correct_answer.clone(),
                        response.user_answer.clone()
                    );
                    *confusion_counts.entry(confusion).or_insert(0) += 1;
                }
            }
        }
        
        let error_rate = if total_tasks > 0 {
            total_errors as f64 / total_tasks as f64
        } else {
            0.0
        };
        
        let mut common_confusions: Vec<(String, String, usize)> = confusion_counts
            .into_iter()
            .map(|((expected, actual), count)| (expected, actual, count))
            .collect();
        common_confusions.sort_by_key(|c| std::cmp::Reverse(c.2));
        common_confusions.truncate(10);
        
        let error_by_task_type_rates = error_by_task_type
            .into_iter()
            .map(|(k, (errors, total))| {
                (k, if total > 0 { errors as f64 / total as f64 } else { 0.0 })
            })
            .collect();
        
        let error_by_difficulty_rates: Vec<(f64, f64)> = error_by_difficulty
            .into_iter()
            .map(|(bucket, results)| {
                let errors = results.iter().filter(|&&c| !c).count();
                let rate = if !results.is_empty() {
                    errors as f64 / results.len() as f64
                } else {
                    0.0
                };
                (bucket.parse::<f64>().unwrap_or(0.0), rate)
            })
            .collect();
        
        ErrorAnalysis {
            total_errors,
            error_rate,
            common_confusions,
            error_by_task_type: error_by_task_type_rates,
            error_by_difficulty: error_by_difficulty_rates,
        }
    }
    
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();
        
        // Header
        csv.push_str("learner_id,session_id,trial,timestamp,task_type,correct,rt_ms,difficulty\n");
        
        // Data rows
        for session in &self.sessions {
            for (i, response) in session.responses.iter().enumerate() {
                csv.push_str(&format!(
                    "{},{},{},{},{:?},{},{},{}\n",
                    self.learner_id,
                    session.session_id,
                    i + 1,
                    response.timestamp.to_rfc3339(),
                    response.task.task_type,
                    response.correct,
                    response.response_time_ms,
                    response.task.difficulty
                ));
            }
        }
        
        csv
    }
    
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let json = self.to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
}

// Population analyzer for collaborative analysis
#[derive(Debug, Clone)]
pub struct PopulationAnalyzer {
    pub learners: Vec<LearnerDataExport>,
}

impl PopulationAnalyzer {
    pub fn new(learners: Vec<LearnerDataExport>) -> Self {
        PopulationAnalyzer { learners }
    }
    
    pub fn find_population_bottlenecks(&self) -> Vec<(String, String, f64)> {
        let mut transition_errors: HashMap<(String, String), (usize, usize)> = HashMap::new();
        
        for learner in &self.learners {
            for session in &learner.sessions {
                for window in session.responses.windows(2) {
                    if let [prev, curr] = window {
                        let transition = (
                            format!("{:?}", prev.task.task_type),
                            format!("{:?}", curr.task.task_type)
                        );
                        
                        let entry = transition_errors.entry(transition).or_insert((0, 0));
                        entry.1 += 1; // total
                        if !curr.correct {
                            entry.0 += 1; // errors
                        }
                    }
                }
            }
        }
        
        let mut bottlenecks: Vec<(String, String, f64)> = transition_errors
            .into_iter()
            .filter(|(_, (_, total))| *total >= 10) // Minimum sample size
            .map(|((from, to), (errors, total))| {
                (from, to, errors as f64 / total as f64)
            })
            .collect();
        
        bottlenecks.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        bottlenecks.truncate(20);
        
        bottlenecks
    }
    
    pub fn compute_item_difficulties(&self) -> HashMap<String, DifficultyNorm> {
        let mut item_performance: HashMap<String, Vec<bool>> = HashMap::new();
        
        for learner in &self.learners {
            for session in &learner.sessions {
                for response in &session.responses {
                    let item_key = format!("{:?}", response.task.task_type);
                    item_performance.entry(item_key)
                        .or_insert_with(Vec::new)
                        .push(response.correct);
                }
            }
        }
        
        item_performance
            .into_iter()
            .map(|(item, results)| {
                let success_rate = results.iter().filter(|&&c| c).count() as f64 
                    / results.len() as f64;
                
                let norm = DifficultyNorm {
                    item_id: item.clone(),
                    empirical_difficulty: 1.0 - success_rate,
                    sample_size: results.len(),
                    confidence_interval: Self::calculate_ci(success_rate, results.len()),
                };
                
                (item, norm)
            })
            .collect()
    }
    
    fn calculate_ci(p: f64, n: usize) -> (f64, f64) {
        // Wilson score interval
        let z = 1.96; // 95% confidence
        let n_f = n as f64;
        
        let denominator = 1.0 + z * z / n_f;
        let center = (p + z * z / (2.0 * n_f)) / denominator;
        let spread = z * (p * (1.0 - p) / n_f + z * z / (4.0 * n_f * n_f)).sqrt() / denominator;
        
        (
            (center - spread).max(0.0),
            (center + spread).min(1.0)
        )
    }
    
    pub fn cluster_by_strategy(&self) -> HashMap<StrategyType, Vec<String>> {
        let mut clusters: HashMap<StrategyType, Vec<String>> = HashMap::new();
        
        // Simplified strategy detection based on RT patterns
        for learner in &self.learners {
            let mut rt_by_distance: HashMap<usize, Vec<f64>> = HashMap::new();
            
            for session in &learner.sessions {
                for response in &session.responses {
                    // Estimate distance from task (simplified)
                    let distance = (response.task.difficulty * 10.0) as usize;
                    rt_by_distance.entry(distance)
                        .or_insert_with(Vec::new)
                        .push(response.response_time_ms as f64);
                }
            }
            
            // Calculate RT slope
            let distances: Vec<f64> = rt_by_distance.keys().map(|&d| d as f64).collect();
            let mean_rts: Vec<f64> = rt_by_distance.values()
                .map(|rts| rts.iter().sum::<f64>() / rts.len() as f64)
                .collect();
            
            let slope = if distances.len() >= 2 {
                Self::calculate_slope(&distances, &mean_rts)
            } else {
                0.0
            };
            
            let strategy = if slope > 100.0 {
                StrategyType::SerialScan
            } else if slope < 50.0 {
                StrategyType::DirectIndex
            } else {
                StrategyType::Mixed(50)
            };
            
            clusters.entry(strategy)
                .or_insert_with(Vec::new)
                .push(learner.learner_id.clone());
        }
        
        clusters
    }
    
    fn calculate_slope(x: &[f64], y: &[f64]) -> f64 {
        let n = x.len().min(y.len()) as f64;
        if n < 2.0 {
            return 0.0;
        }
        
        let sum_x: f64 = x.iter().take(n as usize).sum();
        let sum_y: f64 = y.iter().take(n as usize).sum();
        let sum_xy: f64 = x.iter().zip(y.iter())
            .take(n as usize)
            .map(|(a, b)| a * b)
            .sum();
        let sum_x2: f64 = x.iter().take(n as usize).map(|a| a * a).sum();
        
        (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyNorm {
    pub item_id: String,
    pub empirical_difficulty: f64,
    pub sample_size: usize,
    pub confidence_interval: (f64, f64),
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

// Add UUID support
pub use uuid;