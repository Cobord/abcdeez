use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentNodeEmbedding {
    pub node_id: String,
    pub position: f64,
    pub uncertainty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Successor,
    Predecessor,
    PairwiseOrder,
    KJump(i32),
    Segment(usize, bool),
    Index,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProficiency {
    pub operation: OperationType,
    pub theta: f64,
    pub practice_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStrength {
    pub node_id: String,
    pub strength: f64,
    pub last_practice: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkBoundary {
    pub position: usize,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerModel {
    pub learner_id: String,
    pub node_embeddings: HashMap<String, LatentNodeEmbedding>,
    pub operation_proficiencies: HashMap<String, OperationProficiency>,
    pub memory_strengths: HashMap<String, MemoryStrength>,
    pub confusability_matrix: HashMap<(String, String), f64>,
    pub chunk_boundaries: Vec<ChunkBoundary>,
    pub total_practice_time: std::time::Duration,
    pub session_count: usize,
}

impl LearnerModel {
    pub fn new(learner_id: String, topology: &crate::topology::Topology) -> Self {
        let mut node_embeddings = HashMap::new();
        let mut memory_strengths = HashMap::new();

        for node in &topology.nodes {
            node_embeddings.insert(
                node.id.clone(),
                LatentNodeEmbedding {
                    node_id: node.id.clone(),
                    position: node.position + rand::random::<f64>() * 0.5 - 0.25,
                    uncertainty: 1.0,
                },
            );

            memory_strengths.insert(
                node.id.clone(),
                MemoryStrength {
                    node_id: node.id.clone(),
                    strength: 0.0,
                    last_practice: chrono::Utc::now(),
                },
            );
        }

        let mut operation_proficiencies = HashMap::new();
        let operations = vec![
            OperationType::Successor,
            OperationType::Predecessor,
            OperationType::PairwiseOrder,
            OperationType::KJump(2),
            OperationType::KJump(3),
            OperationType::Segment(3, false),
            OperationType::Segment(3, true),
            OperationType::Index,
        ];

        for op in operations {
            let key = format!("{:?}", op);
            operation_proficiencies.insert(
                key,
                OperationProficiency {
                    operation: op,
                    theta: -1.0,
                    practice_count: 0,
                },
            );
        }

        let chunk_boundaries = match topology.topology_type {
            crate::topology::TopologyType::Linear => {
                vec![
                    ChunkBoundary {
                        position: 6,
                        strength: 0.8,
                    },
                    ChunkBoundary {
                        position: 13,
                        strength: 0.8,
                    },
                    ChunkBoundary {
                        position: 19,
                        strength: 0.8,
                    },
                ]
            }
            crate::topology::TopologyType::Cyclic => vec![],
        };

        LearnerModel {
            learner_id,
            node_embeddings,
            operation_proficiencies,
            memory_strengths,
            confusability_matrix: HashMap::new(),
            chunk_boundaries,
            total_practice_time: std::time::Duration::new(0, 0),
            session_count: 0,
        }
    }

    pub fn update_node_embedding(&mut self, node_id: &str, new_position: f64, reduce_uncertainty: f64) {
        if let Some(embedding) = self.node_embeddings.get_mut(node_id) {
            embedding.position = 0.7 * embedding.position + 0.3 * new_position;
            embedding.uncertainty *= (1.0 - reduce_uncertainty).max(0.1);
        }
    }

    pub fn update_operation_proficiency(&mut self, operation: &OperationType, success: bool) {
        let key = format!("{:?}", operation);
        if let Some(prof) = self.operation_proficiencies.get_mut(&key) {
            prof.practice_count += 1;
            
            let learning_rate = 0.1;
            if success {
                prof.theta += learning_rate * (1.0 - sigmoid(prof.theta));
            } else {
                prof.theta -= learning_rate * sigmoid(prof.theta);
            }
        }
    }

    pub fn update_memory_strength(&mut self, node_id: &str, correct: bool) {
        if let Some(mem) = self.memory_strengths.get_mut(node_id) {
            let now = chrono::Utc::now();
            let time_since = now.signed_duration_since(mem.last_practice);
            let hours_since = time_since.num_hours() as f64;
            
            let decay_rate = 0.05;
            let decayed_strength = mem.strength * (-decay_rate * hours_since).exp();
            
            if correct {
                mem.strength = (decayed_strength + 0.2).min(1.0);
            } else {
                mem.strength = (decayed_strength - 0.1).max(0.0);
            }
            
            mem.last_practice = now;
        }
    }

    pub fn update_confusability(&mut self, node_a: &str, node_b: &str, confused: bool) {
        let key = if node_a < node_b {
            (node_a.to_string(), node_b.to_string())
        } else {
            (node_b.to_string(), node_a.to_string())
        };

        let current = self.confusability_matrix.get(&key).unwrap_or(&0.0);
        let new_value = if confused {
            (current + 0.1).min(1.0)
        } else {
            (current * 0.9).max(0.0)
        };

        self.confusability_matrix.insert(key, new_value);
    }

    pub fn get_probability_correct(&self, operation: &OperationType, difficulty: f64) -> f64 {
        let key = format!("{:?}", operation);
        let prof = self.operation_proficiencies.get(&key);
        
        let theta = prof.map(|p| p.theta).unwrap_or(-1.0);
        
        sigmoid(theta - difficulty)
    }

    pub fn predict_response_time(&self, operation: &OperationType, distance: usize) -> f64 {
        let key = format!("{:?}", operation);
        let prof = self.operation_proficiencies.get(&key);
        
        let base_rt = 1000.0;
        let distance_penalty = 100.0 * distance as f64;
        
        let proficiency_bonus = prof.map(|p| 200.0 * sigmoid(p.theta)).unwrap_or(0.0);
        
        let boundary_penalty = self.chunk_boundaries.iter()
            .filter(|b| b.position < distance)
            .map(|b| 200.0 * b.strength)
            .sum::<f64>();

        (base_rt + distance_penalty + boundary_penalty - proficiency_bonus).max(300.0)
    }

    pub fn get_bidirectionality_index(&self) -> f64 {
        let forward = self.operation_proficiencies.get(&format!("{:?}", OperationType::Successor));
        let backward = self.operation_proficiencies.get(&format!("{:?}", OperationType::Predecessor));

        match (forward, backward) {
            (Some(f), Some(b)) => (f.theta - b.theta).abs(),
            _ => 1.0,
        }
    }

    pub fn get_symbolic_distance_slope(&self) -> f64 {
        let distances = vec![1, 2, 3, 4, 5];
        let mut rts = Vec::new();

        for d in &distances {
            let rt = self.predict_response_time(&OperationType::PairwiseOrder, *d);
            rts.push(rt);
        }

        let n = distances.len() as f64;
        let sum_x: f64 = distances.iter().sum::<usize>() as f64;
        let sum_y: f64 = rts.iter().sum();
        let sum_xy: f64 = distances.iter().zip(rts.iter())
            .map(|(x, y)| *x as f64 * y)
            .sum();
        let sum_x2: f64 = distances.iter().map(|x| (*x * *x) as f64).sum();

        (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
    }

    pub fn get_chunk_boundary_penalty(&self) -> f64 {
        self.chunk_boundaries.iter()
            .map(|b| b.strength)
            .sum::<f64>() / self.chunk_boundaries.len().max(1) as f64
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerMetrics {
    pub bidirectionality_index: f64,
    pub symbolic_distance_slope: f64,
    pub chunk_boundary_penalty: f64,
    pub avg_memory_strength: f64,
    pub operation_proficiencies: HashMap<String, f64>,
}

impl LearnerMetrics {
    pub fn from_model(model: &LearnerModel) -> Self {
        let avg_memory_strength = model.memory_strengths.values()
            .map(|m| m.strength)
            .sum::<f64>() / model.memory_strengths.len().max(1) as f64;

        let mut operation_proficiencies = HashMap::new();
        for (key, prof) in &model.operation_proficiencies {
            operation_proficiencies.insert(key.clone(), sigmoid(prof.theta));
        }

        LearnerMetrics {
            bidirectionality_index: model.get_bidirectionality_index(),
            symbolic_distance_slope: model.get_symbolic_distance_slope(),
            chunk_boundary_penalty: model.get_chunk_boundary_penalty(),
            avg_memory_strength,
            operation_proficiencies,
        }
    }
}