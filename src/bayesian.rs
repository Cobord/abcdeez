use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use statrs::distribution::{Normal, ContinuousCDF};

/// Bayesian Expected Information Gain implementation for adaptive task selection
/// Based on the paper's equation: EIG = E[KL(p(θ|D_t) || p(θ|D_t, Response to q))]
#[derive(Debug, Clone)]
pub struct BayesianLearnerModel {
    /// Posterior distributions for node positions
    pub node_positions: HashMap<String, PosteriorDistribution>,
    /// Posterior distributions for operation proficiencies
    pub operation_proficiencies: HashMap<String, PosteriorDistribution>,
    /// Posterior for chunk boundaries
    pub chunk_boundaries: Vec<ChunkBoundaryPosterior>,
    /// Historical responses for updating posteriors
    pub response_history: Vec<ResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosteriorDistribution {
    pub mean: f64,
    pub variance: f64,
    pub confidence: f64,  // 1 - entropy/max_entropy
}

impl PosteriorDistribution {
    pub fn new(mean: f64, variance: f64) -> Self {
        let confidence = 1.0 / (1.0 + variance);
        PosteriorDistribution { mean, variance, confidence }
    }

    pub fn entropy(&self) -> f64 {
        // Entropy of a Gaussian: H = 0.5 * ln(2πe * σ²)
        0.5 * (2.0 * std::f64::consts::PI * std::f64::consts::E * self.variance).ln()
    }

    pub fn update(&mut self, observation: f64, observation_variance: f64) {
        // Bayesian update for Gaussian posterior
        let precision_prior = 1.0 / self.variance;
        let precision_obs = 1.0 / observation_variance;
        
        let precision_post = precision_prior + precision_obs;
        self.variance = 1.0 / precision_post;
        
        self.mean = (precision_prior * self.mean + precision_obs * observation) / precision_post;
        self.confidence = 1.0 / (1.0 + self.variance);
    }

    pub fn kl_divergence(&self, other: &PosteriorDistribution) -> f64 {
        // KL divergence between two Gaussians
        // KL(P||Q) = log(σ_Q/σ_P) + (σ_P² + (μ_P - μ_Q)²)/(2σ_Q²) - 1/2
        let sigma_p = self.variance.sqrt();
        let sigma_q = other.variance.sqrt();
        
        (sigma_q / sigma_p).ln() + 
        (self.variance + (self.mean - other.mean).powi(2)) / (2.0 * other.variance) - 0.5
    }
}

#[derive(Debug, Clone)]
pub struct ChunkBoundaryPosterior {
    pub position: usize,
    pub strength: PosteriorDistribution,
}

#[derive(Debug, Clone)]
pub struct ResponseData {
    pub task: crate::tasks::Task,
    pub correct: bool,
    pub response_time: f64,
}

impl BayesianLearnerModel {
    pub fn new(topology: &crate::topology::Topology) -> Self {
        let mut node_positions = HashMap::new();
        let mut operation_proficiencies = HashMap::new();
        
        // Initialize node position posteriors
        for node in &topology.nodes {
            node_positions.insert(
                node.id.clone(),
                PosteriorDistribution::new(node.position, 1.0)
            );
        }
        
        // Initialize operation proficiency posteriors
        let operations = vec![
            "Successor", "Predecessor", "PairwiseOrder", 
            "KJump", "Segment", "Index"
        ];
        
        for op in operations {
            operation_proficiencies.insert(
                op.to_string(),
                PosteriorDistribution::new(0.0, 1.0) // Start with neutral prior
            );
        }
        
        // Initialize chunk boundaries (for linear sequences)
        let chunk_boundaries = match topology.topology_type {
            crate::topology::TopologyType::Linear => {
                vec![6, 13, 19].into_iter().map(|pos| {
                    ChunkBoundaryPosterior {
                        position: pos,
                        strength: PosteriorDistribution::new(0.5, 0.25),
                    }
                }).collect()
            }
            _ => vec![],
        };
        
        BayesianLearnerModel {
            node_positions,
            operation_proficiencies,
            chunk_boundaries,
            response_history: vec![],
        }
    }

    /// Calculate Expected Information Gain for a given task
    pub fn calculate_eig(&self, task: &crate::tasks::Task) -> f64 {
        // Simulate possible outcomes (correct/incorrect)
        let p_correct = self.predict_accuracy(task);
        
        // Calculate expected KL divergence
        let eig_correct = p_correct * self.calculate_kl_if_correct(task);
        let eig_incorrect = (1.0 - p_correct) * self.calculate_kl_if_incorrect(task);
        
        eig_correct + eig_incorrect
    }

    fn predict_accuracy(&self, task: &crate::tasks::Task) -> f64 {
        // Get operation proficiency
        let op_key = format!("{:?}", task.operation);
        let proficiency = self.operation_proficiencies
            .get(&op_key)
            .map(|p| p.mean)
            .unwrap_or(0.0);
        
        // Sigmoid function for probability
        let z = proficiency - task.difficulty;
        1.0 / (1.0 + (-z).exp())
    }

    fn calculate_kl_if_correct(&self, task: &crate::tasks::Task) -> f64 {
        let mut total_kl = 0.0;
        
        // Calculate KL for affected parameters
        match &task.task_type {
            crate::tasks::TaskType::PairwiseOrder { a, b } => {
                // This task would reduce uncertainty about relative positions
                if let (Some(pos_a), Some(pos_b)) = 
                    (self.get_node_position(a), self.get_node_position(b)) {
                    
                    // Simulate posterior after correct response
                    let mut post_a = pos_a.clone();
                    let mut post_b = pos_b.clone();
                    
                    // Correct response confirms order, reduce variance
                    post_a.variance *= 0.8;
                    post_b.variance *= 0.8;
                    
                    total_kl += pos_a.kl_divergence(&post_a);
                    total_kl += pos_b.kl_divergence(&post_b);
                }
            }
            crate::tasks::TaskType::Successor { item } | 
            crate::tasks::TaskType::Predecessor { item } => {
                if let Some(pos) = self.get_node_position(item) {
                    let mut post = pos.clone();
                    post.variance *= 0.7; // Greater reduction for local adjacency
                    total_kl += pos.kl_divergence(&post);
                }
            }
            crate::tasks::TaskType::Segment { start, count, .. } => {
                // Segment tasks affect multiple nodes and chunk boundaries
                for i in 0..*count {
                    let boundary_kl = self.calculate_boundary_kl(start, i);
                    total_kl += boundary_kl;
                }
            }
            _ => {}
        }
        
        // Add operation proficiency KL
        let op_key = format!("{:?}", task.operation);
        if let Some(prof) = self.operation_proficiencies.get(&op_key) {
            let mut post_prof = prof.clone();
            post_prof.update(1.0, 0.1); // Update with success
            total_kl += prof.kl_divergence(&post_prof);
        }
        
        total_kl
    }

    fn calculate_kl_if_incorrect(&self, task: &crate::tasks::Task) -> f64 {
        let mut total_kl = 0.0;
        
        // Incorrect responses often increase uncertainty
        match &task.task_type {
            crate::tasks::TaskType::PairwiseOrder { a, b } => {
                if let (Some(pos_a), Some(pos_b)) = 
                    (self.get_node_position(a), self.get_node_position(b)) {
                    
                    let mut post_a = pos_a.clone();
                    let mut post_b = pos_b.clone();
                    
                    // Incorrect response increases uncertainty
                    post_a.variance *= 1.2;
                    post_b.variance *= 1.2;
                    
                    total_kl += pos_a.kl_divergence(&post_a);
                    total_kl += pos_b.kl_divergence(&post_b);
                }
            }
            _ => {}
        }
        
        // Update operation proficiency for failure
        let op_key = format!("{:?}", task.operation);
        if let Some(prof) = self.operation_proficiencies.get(&op_key) {
            let mut post_prof = prof.clone();
            post_prof.update(0.0, 0.1); // Update with failure
            total_kl += prof.kl_divergence(&post_prof);
        }
        
        total_kl
    }

    fn get_node_position(&self, _label: &str) -> Option<&PosteriorDistribution> {
        // This would need topology access to map label to node_id
        // For now, simplified implementation
        self.node_positions.values().next()
    }

    fn calculate_boundary_kl(&self, _start: &str, offset: usize) -> f64 {
        // Check if this crosses a chunk boundary
        for boundary in &self.chunk_boundaries {
            if offset == boundary.position {
                // Crossing boundary would update our belief about its strength
                let mut post = boundary.strength.clone();
                post.variance *= 0.9;
                return boundary.strength.kl_divergence(&post);
            }
        }
        0.0
    }

    pub fn update_with_response(&mut self, response: ResponseData) {
        self.response_history.push(response.clone());
        
        // Update relevant posteriors based on response
        let op_key = format!("{:?}", response.task.operation);
        if let Some(prof) = self.operation_proficiencies.get_mut(&op_key) {
            let observation = if response.correct { 1.0 } else { 0.0 };
            prof.update(observation, 0.1);
        }
        
        // Update node position posteriors
        match &response.task.task_type {
            crate::tasks::TaskType::PairwiseOrder { a, b } => {
                // Update beliefs about relative positions
                self.update_pairwise_positions(a, b, response.correct);
            }
            _ => {}
        }
    }

    fn update_pairwise_positions(&mut self, _a: &str, _b: &str, _correct: bool) {
        // Implementation would update position posteriors based on comparison result
        // Simplified for now
    }

    /// Get tasks ranked by Expected Information Gain
    pub fn rank_tasks_by_eig(&self, tasks: Vec<crate::tasks::Task>) -> Vec<(crate::tasks::Task, f64)> {
        let mut ranked: Vec<(crate::tasks::Task, f64)> = tasks
            .into_iter()
            .map(|task| {
                let eig = self.calculate_eig(&task);
                (task, eig)
            })
            .collect();
        
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    /// Get current entropy of the model (uncertainty)
    pub fn total_entropy(&self) -> f64 {
        let mut entropy = 0.0;
        
        for pos in self.node_positions.values() {
            entropy += pos.entropy();
        }
        
        for prof in self.operation_proficiencies.values() {
            entropy += prof.entropy();
        }
        
        for boundary in &self.chunk_boundaries {
            entropy += boundary.strength.entropy();
        }
        
        entropy
    }
}

/// Monte Carlo estimation of Expected Information Gain
pub struct MonteCarloEIG {
    samples: usize,
}

impl MonteCarloEIG {
    pub fn new(samples: usize) -> Self {
        MonteCarloEIG { samples }
    }

    pub fn estimate_eig(
        &self, 
        model: &BayesianLearnerModel, 
        task: &crate::tasks::Task
    ) -> f64 {
        let mut total_gain = 0.0;
        
        for _ in 0..self.samples {
            // Sample from current posterior
            let sampled_params = self.sample_from_posterior(model);
            
            // Simulate response given sampled parameters
            let p_correct = self.simulate_response(&sampled_params, task);
            
            // Calculate information gain for this sample
            let gain = if rand::random::<f64>() < p_correct {
                model.calculate_kl_if_correct(task)
            } else {
                model.calculate_kl_if_incorrect(task)
            };
            
            total_gain += gain;
        }
        
        total_gain / self.samples as f64
    }

    fn sample_from_posterior(&self, model: &BayesianLearnerModel) -> SampledParameters {
        let mut sampled = SampledParameters::new();
        
        // Sample from each posterior distribution
        for (key, dist) in &model.node_positions {
            use rand_distr::{Distribution, Normal as RandNormal};
            let normal = RandNormal::new(dist.mean, dist.variance.sqrt()).unwrap();
            sampled.node_positions.insert(key.clone(), normal.sample(&mut rand::thread_rng()));
        }
        
        for (key, dist) in &model.operation_proficiencies {
            use rand_distr::{Distribution, Normal as RandNormal};
            let normal = RandNormal::new(dist.mean, dist.variance.sqrt()).unwrap();
            sampled.operation_proficiencies.insert(key.clone(), normal.sample(&mut rand::thread_rng()));
        }
        
        sampled
    }

    fn simulate_response(&self, params: &SampledParameters, task: &crate::tasks::Task) -> f64 {
        // Simulate response probability given sampled parameters
        let op_key = format!("{:?}", task.operation);
        let proficiency = params.operation_proficiencies
            .get(&op_key)
            .unwrap_or(&0.0);
        
        let z = proficiency - task.difficulty;
        1.0 / (1.0 + (-z).exp())
    }
}

struct SampledParameters {
    node_positions: HashMap<String, f64>,
    operation_proficiencies: HashMap<String, f64>,
}

impl SampledParameters {
    fn new() -> Self {
        SampledParameters {
            node_positions: HashMap::new(),
            operation_proficiencies: HashMap::new(),
        }
    }
}

