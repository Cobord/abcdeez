use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rand::prelude::*;
use rand_distr;

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
    /// Topology reference for node mapping
    topology: crate::topology::Topology,
    /// Confusability posteriors
    pub confusability: HashMap<(String, String), PosteriorDistribution>,
    /// Memory strength posteriors
    pub memory_strengths: HashMap<String, PosteriorDistribution>,
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
        // Differential entropy of a Gaussian: H = 0.5 * ln(2πeσ²) = 0.5 * ln(2πe) + ln(σ)
        // Simplified: H = 0.5 * ln(2π * e * σ²)
        if self.variance <= 0.0 {
            return 0.0;
        }
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

/// A sampled model from the posterior for Monte Carlo simulation
struct SampledModel {
    positions: HashMap<String, f64>,
    proficiencies: HashMap<String, f64>,
}

impl SampledModel {
    /// Predict success probability given sampled parameters
    fn predict_success_probability(&self, task: &crate::tasks::Task) -> f64 {
        // Get operation proficiency for this task
        let op_key = format!("{:?}", task.operation);
        let proficiency = self.proficiencies.get(&op_key).unwrap_or(&0.0);
        
        // Convert to probability using sigmoid
        1.0 / (1.0 + (-proficiency).exp())
    }
}

impl BayesianLearnerModel {
    pub fn new(topology: &crate::topology::Topology) -> Self {
        let mut node_positions = HashMap::new();
        let mut operation_proficiencies = HashMap::new();
        let mut memory_strengths = HashMap::new();
        
        // Initialize node position posteriors
        for node in &topology.nodes {
            node_positions.insert(
                node.id.clone(),
                PosteriorDistribution::new(node.position, 1.0)
            );
            
            // Initialize memory strength posteriors
            memory_strengths.insert(
                node.id.clone(),
                PosteriorDistribution::new(0.5, 0.25)
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
            topology: topology.clone(),
            confusability: HashMap::new(),
            memory_strengths,
        }
    }

    /// Calculate Expected Information Gain for a given task using Monte Carlo simulation
    /// EIG = E[KL(p(θ|D_t) || p(θ|D_t, Response to q))]
    pub fn calculate_eig(&self, task: &crate::tasks::Task) -> f64 {
        self.monte_carlo_eig(task, 1000)
    }
    
    /// Monte Carlo simulation for Expected Information Gain
    /// Samples from the posterior predictive distribution
    pub fn monte_carlo_eig(&self, task: &crate::tasks::Task, n_samples: usize) -> f64 {
        use rand::prelude::*;
        
        let mut rng = thread_rng();
        let mut total_eig = 0.0;
        
        for _ in 0..n_samples {
            // Sample from current posterior beliefs
            let sampled_model = self.sample_from_posterior(&mut rng);
            
            // Simulate response given sampled parameters
            let response_prob = sampled_model.predict_success_probability(task);
            let simulated_correct = rng.gen::<f64>() < response_prob;
            
            // Calculate KL divergence for this simulated outcome
            let kl = if simulated_correct {
                self.calculate_kl_if_correct_monte_carlo(task, &sampled_model)
            } else {
                self.calculate_kl_if_incorrect_monte_carlo(task, &sampled_model)
            };
            
            total_eig += kl;
        }
        
        total_eig / n_samples as f64
    }
    
    /// Sample a model from the current posterior distributions
    fn sample_from_posterior(&self, rng: &mut impl Rng) -> SampledModel {
        use rand_distr::Normal;
        
        let mut sampled_positions = HashMap::new();
        for (key, posterior) in &self.node_positions {
            let dist = Normal::new(posterior.mean, posterior.variance.sqrt())
                .unwrap_or(Normal::new(0.0, 1.0).unwrap());
            sampled_positions.insert(key.clone(), dist.sample(rng));
        }
        
        let mut sampled_proficiencies = HashMap::new();
        for (key, posterior) in &self.operation_proficiencies {
            let dist = Normal::new(posterior.mean, posterior.variance.sqrt())
                .unwrap_or(Normal::new(0.0, 1.0).unwrap());
            sampled_proficiencies.insert(key.clone(), dist.sample(rng));
        }
        
        SampledModel {
            positions: sampled_positions,
            proficiencies: sampled_proficiencies,
        }
    }
    
    /// Calculate KL divergence for correct response in Monte Carlo
    fn calculate_kl_if_correct_monte_carlo(&self, task: &crate::tasks::Task, _sampled: &SampledModel) -> f64 {
        // Create updated posterior given correct response
        let mut updated_model = self.clone();
        updated_model.update_with_response(ResponseData {
            task: task.clone(),
            correct: true,
            response_time: 1000.0, // Default for simulation
        });
        
        // Calculate KL divergence between current and updated posteriors
        self.kl_divergence_to(&updated_model)
    }
    
    /// Calculate KL divergence for incorrect response in Monte Carlo
    fn calculate_kl_if_incorrect_monte_carlo(&self, task: &crate::tasks::Task, _sampled: &SampledModel) -> f64 {
        // Create updated posterior given incorrect response
        let mut updated_model = self.clone();
        updated_model.update_with_response(ResponseData {
            task: task.clone(),
            correct: false,
            response_time: 2000.0, // Default for simulation
        });
        
        // Calculate KL divergence between current and updated posteriors
        self.kl_divergence_to(&updated_model)
    }
    
    /// Calculate total KL divergence to another model
    fn kl_divergence_to(&self, other: &BayesianLearnerModel) -> f64 {
        let mut total_kl = 0.0;
        
        // KL for node positions
        for (key, pos) in &self.node_positions {
            if let Some(other_pos) = other.node_positions.get(key) {
                total_kl += pos.kl_divergence(other_pos);
            }
        }
        
        // KL for operation proficiencies
        for (key, prof) in &self.operation_proficiencies {
            if let Some(other_prof) = other.operation_proficiencies.get(key) {
                total_kl += prof.kl_divergence(other_prof);
            }
        }
        
        total_kl
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

    fn get_node_position(&self, label: &str) -> Option<&PosteriorDistribution> {
        // Properly map label to node_id using topology
        if let Some(node) = self.topology.get_node_by_label(label) {
            self.node_positions.get(&node.id)
        } else {
            None
        }
    }

    fn calculate_boundary_kl(&self, start: &str, offset: usize) -> f64 {
        // Check if this crosses a chunk boundary
        if let Some(node) = self.topology.get_node_by_label(start) {
            let start_idx = self.topology.node_map.get(&node.id)
                .copied()
                .unwrap_or(0);
            let position = start_idx + offset;
            
            for boundary in &self.chunk_boundaries {
                if position == boundary.position {
                    // Crossing boundary would update our belief about its strength
                    let mut post = boundary.strength.clone();
                    post.variance *= 0.9;
                    return boundary.strength.kl_divergence(&post);
                }
            }
        }
        0.0
    }

    pub fn update_with_response(&mut self, response: ResponseData) {
        self.response_history.push(response.clone());
        
        // Calculate observation variance before mutable borrows
        let obs_variance = self.calculate_observation_variance(&response);
        
        // Update relevant posteriors based on response
        let op_key = format!("{:?}", response.task.operation);
        if let Some(prof) = self.operation_proficiencies.get_mut(&op_key) {
            let observation = if response.correct { 1.0 } else { 0.0 };
            prof.update(observation, obs_variance);
        }
        
        // Update node position posteriors and other parameters
        match &response.task.task_type {
            crate::tasks::TaskType::PairwiseOrder { a, b } => {
                // Update beliefs about relative positions
                self.update_pairwise_positions(a, b, response.correct);
            }
            crate::tasks::TaskType::Successor { item } | 
            crate::tasks::TaskType::Predecessor { item } => {
                self.update_adjacency_beliefs(item, &response.task.task_type, response.correct);
            }
            crate::tasks::TaskType::Segment { start, count, .. } => {
                self.update_segment_beliefs(start, *count, response.correct, response.response_time);
            }
            crate::tasks::TaskType::KJump { start, k } => {
                self.update_kjump_beliefs(start, *k, response.correct);
            }
            _ => {}
        }
    }

    fn update_pairwise_positions(&mut self, a: &str, b: &str, correct: bool) {
        // Update position posteriors based on comparison result
        if let (Some(node_a), Some(node_b)) = (self.topology.get_node_by_label(a), 
                                                 self.topology.get_node_by_label(b)) {
            // Avoid double mutable borrow by updating positions separately
            let node_a_id = node_a.id.clone();
            let node_b_id = node_b.id.clone();
            
            // Update position for node A
            if let Some(pos_a) = self.node_positions.get_mut(&node_a_id) {
                let obs_variance = 0.2;
                if correct {
                    pos_a.update(pos_a.mean, obs_variance * 0.8);
                } else {
                    pos_a.variance *= 1.1;
                }
            }
            
            // Update position for node B
            if let Some(pos_b) = self.node_positions.get_mut(&node_b_id) {
                let obs_variance = 0.2;
                if correct {
                    pos_b.update(pos_b.mean, obs_variance * 0.8);
                } else {
                    pos_b.variance *= 1.1;
                }
            }
            
            // Update confusability if incorrect
            if !correct {
                let key = if a < b { (a.to_string(), b.to_string()) } 
                          else { (b.to_string(), a.to_string()) };
                self.confusability.entry(key)
                    .or_insert(PosteriorDistribution::new(0.0, 1.0))
                    .update(1.0, 0.2);
            }
            
            // Update memory strengths
            if let Some(mem_a) = self.memory_strengths.get_mut(&node_a_id) {
                mem_a.update(if correct { 0.8 } else { 0.3 }, 0.1);
            }
            if let Some(mem_b) = self.memory_strengths.get_mut(&node_b_id) {
                mem_b.update(if correct { 0.8 } else { 0.3 }, 0.1);
            }
        }
    }
    
    /// Calculate adaptive observation variance based on response characteristics
    fn calculate_observation_variance(&self, response: &ResponseData) -> f64 {
        // Base variance
        let mut variance = 0.1;
        
        // Adjust based on response time (faster responses = more confidence)
        let rt_factor = (response.response_time / 1000.0).min(3.0).max(0.5);
        variance *= rt_factor / 1.5;
        
        // Adjust based on task difficulty
        let difficulty_factor = 0.5 + response.task.difficulty;
        variance *= difficulty_factor;
        
        // Adjust based on response history (more consistent = lower variance)
        if self.response_history.len() > 10 {
            let recent_accuracy = self.response_history.iter()
                .rev()
                .take(10)
                .filter(|r| r.correct)
                .count() as f64 / 10.0;
            variance *= 2.0 - recent_accuracy; // Higher accuracy = lower variance
        }
        
        variance.max(0.01).min(1.0)
    }
    
    /// Update beliefs for adjacency tasks (successor/predecessor)
    fn update_adjacency_beliefs(&mut self, item: &str, task_type: &crate::tasks::TaskType, correct: bool) {
        if let Some(node) = self.topology.get_node_by_label(item) {
            if let Some(pos) = self.node_positions.get_mut(&node.id) {
                let obs_variance = if correct { 0.05 } else { 0.15 };
                pos.update(pos.mean, obs_variance);
            }
            
            // Update memory strength
            if let Some(mem) = self.memory_strengths.get_mut(&node.id) {
                mem.update(if correct { 0.9 } else { 0.4 }, 0.1);
            }
            
            // Update adjacency-specific beliefs
            match task_type {
                crate::tasks::TaskType::Successor { .. } => {
                    if let Some(next_id) = self.topology.get_successor(&node.id) {
                        if let Some(next_mem) = self.memory_strengths.get_mut(&next_id) {
                            next_mem.update(if correct { 0.7 } else { 0.3 }, 0.15);
                        }
                    }
                }
                crate::tasks::TaskType::Predecessor { .. } => {
                    if let Some(prev_id) = self.topology.get_predecessor(&node.id) {
                        if let Some(prev_mem) = self.memory_strengths.get_mut(&prev_id) {
                            prev_mem.update(if correct { 0.7 } else { 0.3 }, 0.15);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    /// Update beliefs for segment tasks
    fn update_segment_beliefs(&mut self, start: &str, count: usize, correct: bool, response_time: f64) {
        if let Some(start_node) = self.topology.get_node_by_label(start) {
            let start_idx = self.topology.node_map.get(&start_node.id).copied().unwrap_or(0);
            
            // Update memory for all nodes in segment
            for i in 0..count {
                let idx = start_idx + i;
                if idx < self.topology.nodes.len() {
                    let node_id = &self.topology.nodes[idx].id;
                    if let Some(mem) = self.memory_strengths.get_mut(node_id) {
                        // Decay factor for distance from start
                        let distance_factor = 1.0 - (i as f64 / count as f64) * 0.3;
                        mem.update(if correct { 0.8 * distance_factor } else { 0.3 }, 0.12);
                    }
                }
            }
            
            // Update chunk boundaries if response time suggests difficulty
            if response_time > 2000.0 {
                for i in 1..count {
                    let boundary_pos = start_idx + i;
                    for boundary in &mut self.chunk_boundaries {
                        if boundary.position == boundary_pos {
                            // Slow response suggests chunk boundary
                            boundary.strength.update(0.7, 0.15);
                        }
                    }
                }
            }
        }
    }
    
    /// Update beliefs for k-jump tasks
    fn update_kjump_beliefs(&mut self, start: &str, k: i32, correct: bool) {
        if let Some(start_node) = self.topology.get_node_by_label(start) {
            let start_idx = self.topology.node_map.get(&start_node.id).copied().unwrap_or(0) as i32;
            let target_idx = (start_idx + k).max(0) as usize;
            
            // Update memory for start and target
            if let Some(mem) = self.memory_strengths.get_mut(&start_node.id) {
                mem.update(if correct { 0.85 } else { 0.4 }, 0.1);
            }
            
            if target_idx < self.topology.nodes.len() {
                let target_id = &self.topology.nodes[target_idx].id;
                if let Some(mem) = self.memory_strengths.get_mut(target_id) {
                    mem.update(if correct { 0.75 } else { 0.35 }, 0.12);
                }
                
                // Update position beliefs for large jumps
                if k.abs() > 2 {
                    if let Some(pos) = self.node_positions.get_mut(target_id) {
                        let obs_variance = if correct { 0.08 } else { 0.2 };
                        pos.update(pos.mean, obs_variance);
                    }
                }
            }
        }
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

