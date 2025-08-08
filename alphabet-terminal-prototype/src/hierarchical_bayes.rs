use crate::bayesian::BayesianLearnerModel;
use crate::learner::{LearnerModel, OperationType};
use crate::topology::Topology;
use crate::tasks::Task;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use statrs::distribution::{Normal, Beta, Gamma, Continuous, ContinuousCDF};
use rand::prelude::*;

/// Hyperparameters for the hierarchical model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hyperparameters {
    // Population-level parameters
    pub mu_theta: f64,      // Mean of population ability
    pub sigma_theta: f64,   // Std of population ability
    pub alpha_rt: f64,      // Shape parameter for response time
    pub beta_rt: f64,       // Rate parameter for response time
    
    // Prior strength parameters
    pub kappa_ability: f64,     // Concentration for ability prior
    pub kappa_difficulty: f64,  // Concentration for difficulty prior
    pub kappa_strategy: f64,    // Concentration for strategy prior
}

/// Individual learner parameters in hierarchical model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualParameters {
    pub learner_id: String,
    pub ability: f64,           // Overall ability parameter
    pub operation_abilities: HashMap<OperationType, f64>,
    pub learning_rate: f64,
    pub strategy_weights: Vec<f64>,
    pub response_time_params: ResponseTimeParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeParams {
    pub mu: f64,     // Mean log response time
    pub sigma: f64,  // Std of log response time
    pub tau: f64,     // Ex-Gaussian tau parameter
}

/// Item (task) parameters in the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemParameters {
    pub task_id: String,
    pub difficulty: f64,
    pub discrimination: f64,
    pub guessing: f64,
    pub operation_type: OperationType,
}

/// Hierarchical Bayesian Learner Model
pub struct HierarchicalBayesianModel {
    pub hyperparameters: Hyperparameters,
    pub population_parameters: PopulationParameters,
    pub individual_models: HashMap<String, IndividualParameters>,
    pub item_bank: HashMap<String, ItemParameters>,
    pub data_points: Vec<ResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationParameters {
    pub mean_ability: f64,
    pub variance_ability: f64,
    pub mean_learning_rate: f64,
    pub variance_learning_rate: f64,
    pub operation_difficulties: HashMap<OperationType, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub learner_id: String,
    pub task_id: String,
    pub correct: bool,
    pub response_time: f64,
    pub timestamp: usize,
}

impl HierarchicalBayesianModel {
    pub fn new(topology: &Topology) -> Self {
        let hyperparameters = Hyperparameters {
            mu_theta: 0.0,
            sigma_theta: 1.0,
            alpha_rt: 2.0,
            beta_rt: 1.0,
            kappa_ability: 10.0,
            kappa_difficulty: 5.0,
            kappa_strategy: 3.0,
        };
        
        let population_parameters = PopulationParameters {
            mean_ability: 0.0,
            variance_ability: 1.0,
            mean_learning_rate: 0.1,
            variance_learning_rate: 0.01,
            operation_difficulties: Self::initialize_operation_difficulties(),
        };
        
        HierarchicalBayesianModel {
            hyperparameters,
            population_parameters,
            individual_models: HashMap::new(),
            item_bank: HashMap::new(),
            data_points: Vec::new(),
        }
    }
    
    fn initialize_operation_difficulties() -> HashMap<OperationType, f64> {
        let mut difficulties = HashMap::new();
        difficulties.insert(OperationType::Successor, 0.0);
        difficulties.insert(OperationType::Predecessor, 0.2);
        difficulties.insert(OperationType::KJump(2), 0.5);
        difficulties.insert(OperationType::PairwiseOrder, 0.3);
        difficulties.insert(OperationType::Segment(3, false), 0.4);
        difficulties
    }
    
    /// Add a new learner to the model
    pub fn add_learner(&mut self, learner_id: String) -> IndividualParameters {
        // Sample from population distribution
        let mut rng = thread_rng();
        let ability_dist = Normal::new(
            self.population_parameters.mean_ability,
            self.population_parameters.variance_ability.sqrt()
        ).unwrap();
        
        let learning_rate_dist = Normal::new(
            self.population_parameters.mean_learning_rate,
            self.population_parameters.variance_learning_rate.sqrt()
        ).unwrap();
        
        let ability = ability_dist.sample(&mut rng);
        let learning_rate = learning_rate_dist.sample(&mut rng).max(0.001).min(1.0);
        
        // Initialize operation-specific abilities
        let mut operation_abilities = HashMap::new();
        for (op, &pop_difficulty) in &self.population_parameters.operation_difficulties {
            let op_ability_dist = Normal::new(ability, 0.5).unwrap();
            operation_abilities.insert(op.clone(), op_ability_dist.sample(&mut rng) - pop_difficulty);
        }
        
        // Initialize strategy weights (Dirichlet prior)
        let n_strategies = 5;
        let alpha = vec![1.0; n_strategies];
        let strategy_weights = self.sample_dirichlet(&alpha);
        
        // Initialize response time parameters
        let response_time_params = ResponseTimeParams {
            mu: 0.0,
            sigma: 0.5,
            tau: 0.3,
        };
        
        let params = IndividualParameters {
            learner_id: learner_id.clone(),
            ability,
            operation_abilities,
            learning_rate,
            strategy_weights,
            response_time_params,
        };
        
        self.individual_models.insert(learner_id, params.clone());
        params
    }
    
    /// Add a task to the item bank
    pub fn add_item(&mut self, task: &Task) -> ItemParameters {
        let task_id = format!("{:?}_{}", task.task_type, task.prompt.len());
        
        // Initialize item parameters
        let difficulty = task.difficulty;
        let discrimination = 1.0; // Default discrimination
        let guessing = 0.25; // Default guessing parameter
        
        let params = ItemParameters {
            task_id: task_id.clone(),
            difficulty,
            discrimination,
            guessing,
            operation_type: task.operation.clone(),
        };
        
        self.item_bank.insert(task_id, params.clone());
        params
    }
    
    /// Update model with new response data
    pub fn update(&mut self, response: ResponseData) {
        self.data_points.push(response.clone());
        
        // Update individual learner parameters
        if let Some(learner) = self.individual_models.get_mut(&response.learner_id) {
            // Bayesian update of ability
            let prior_mean = learner.ability;
            let prior_var = 0.5; // Prior variance
            
            let likelihood_var = 1.0; // Observation variance
            let observed = if response.correct { 1.0 } else { 0.0 };
            
            // Posterior parameters (conjugate normal update)
            let posterior_var = 1.0 / (1.0 / prior_var + 1.0 / likelihood_var);
            let posterior_mean = posterior_var * (prior_mean / prior_var + observed / likelihood_var);
            
            // Update with learning rate
            learner.ability = learner.ability * (1.0 - learner.learning_rate) + 
                              posterior_mean * learner.learning_rate;
            
            // Update response time parameters
            let log_rt = response.response_time.ln();
            learner.response_time_params.mu = 
                learner.response_time_params.mu * 0.95 + log_rt * 0.05;
            
            let deviation = (log_rt - learner.response_time_params.mu).abs();
            learner.response_time_params.sigma = 
                learner.response_time_params.sigma * 0.95 + deviation * 0.05;
        }
        
        // Update item parameters - get item, modify it, then update in self
        let item_update = if let Some(item) = self.item_bank.get(&response.task_id) {
            let mut new_item = item.clone();
            
            // Update difficulty using empirical Bayes
            let observed = if response.correct { 1.0 } else { 0.0 };
            
            // Simple exponential smoothing update
            let alpha = 0.05; // Learning rate for item parameters
            new_item.difficulty = new_item.difficulty * (1.0 - alpha) + (1.0 - observed) * alpha;
            
            // Update discrimination based on response consistency
            if let Some(learner) = self.individual_models.get(&response.learner_id) {
                let expected = self.predict_probability(learner, &new_item);
                let residual = (observed - expected).abs();
                
                // Higher discrimination if predictions are accurate
                new_item.discrimination = new_item.discrimination * 0.95 + (1.0 - residual) * 2.0 * 0.05;
                new_item.discrimination = new_item.discrimination.max(0.1).min(3.0);
            }
            
            Some(new_item)
        } else {
            None
        };
        
        if let Some(updated_item) = item_update {
            self.item_bank.insert(response.task_id.clone(), updated_item);
        }
        
        // Periodically update population parameters
        if self.data_points.len() % 100 == 0 {
            self.update_population_params();
        }
    }
    
    fn update_individual_params(&mut self, learner: &mut IndividualParameters, response: &ResponseData) {
        // Bayesian update of ability
        let prior_mean = learner.ability;
        let prior_var = 0.5; // Prior variance
        
        let likelihood_var = 1.0; // Observation variance
        let observed = if response.correct { 1.0 } else { 0.0 };
        
        // Posterior parameters (conjugate normal update)
        let posterior_var = 1.0 / (1.0 / prior_var + 1.0 / likelihood_var);
        let posterior_mean = posterior_var * (prior_mean / prior_var + observed / likelihood_var);
        
        // Update with learning rate
        learner.ability = learner.ability * (1.0 - learner.learning_rate) + 
                          posterior_mean * learner.learning_rate;
        
        // Update operation-specific ability
        if let Some(item) = self.item_bank.get(&response.task_id) {
            if let Some(op_ability) = learner.operation_abilities.get_mut(&item.operation_type) {
                *op_ability = *op_ability * (1.0 - learner.learning_rate) +
                             (observed - item.difficulty) * learner.learning_rate;
            }
        }
        
        // Update response time parameters
        let log_rt = response.response_time.ln();
        learner.response_time_params.mu = 
            learner.response_time_params.mu * 0.95 + log_rt * 0.05;
        
        let deviation = (log_rt - learner.response_time_params.mu).abs();
        learner.response_time_params.sigma = 
            learner.response_time_params.sigma * 0.95 + deviation * 0.05;
    }
    
    fn update_item_params(&mut self, item: &mut ItemParameters, response: &ResponseData) {
        // Update difficulty using empirical Bayes
        let observed = if response.correct { 1.0 } else { 0.0 };
        
        // Simple exponential smoothing update
        let alpha = 0.05; // Learning rate for item parameters
        item.difficulty = item.difficulty * (1.0 - alpha) + (1.0 - observed) * alpha;
        
        // Update discrimination based on response consistency
        if let Some(learner) = self.individual_models.get(&response.learner_id) {
            let expected = self.predict_probability(learner, item);
            let residual = (observed - expected).abs();
            
            // Higher discrimination if predictions are accurate
            item.discrimination = item.discrimination * 0.95 + (1.0 - residual) * 2.0 * 0.05;
            item.discrimination = item.discrimination.max(0.1).min(3.0);
        }
    }
    
    fn update_population_params(&mut self) {
        if self.individual_models.is_empty() {
            return;
        }
        
        // Update population mean and variance
        let abilities: Vec<f64> = self.individual_models.values()
            .map(|l| l.ability)
            .collect();
        
        self.population_parameters.mean_ability = 
            abilities.iter().sum::<f64>() / abilities.len() as f64;
        
        self.population_parameters.variance_ability = 
            abilities.iter()
                .map(|a| (a - self.population_parameters.mean_ability).powi(2))
                .sum::<f64>() / abilities.len() as f64;
        
        // Update learning rate parameters
        let learning_rates: Vec<f64> = self.individual_models.values()
            .map(|l| l.learning_rate)
            .collect();
        
        self.population_parameters.mean_learning_rate = 
            learning_rates.iter().sum::<f64>() / learning_rates.len() as f64;
        
        self.population_parameters.variance_learning_rate = 
            learning_rates.iter()
                .map(|lr| (lr - self.population_parameters.mean_learning_rate).powi(2))
                .sum::<f64>() / learning_rates.len() as f64;
        
        // Update operation difficulties
        for op_type in self.population_parameters.operation_difficulties.keys().cloned().collect::<Vec<_>>() {
            let op_abilities: Vec<f64> = self.individual_models.values()
                .filter_map(|l| l.operation_abilities.get(&op_type))
                .cloned()
                .collect();
            
            if !op_abilities.is_empty() {
                let mean_op_ability = op_abilities.iter().sum::<f64>() / op_abilities.len() as f64;
                self.population_parameters.operation_difficulties.insert(
                    op_type,
                    -mean_op_ability // Difficulty is negative of ability
                );
            }
        }
    }
    
    /// Predict probability of correct response (3PL IRT model)
    pub fn predict_probability(&self, learner: &IndividualParameters, item: &ItemParameters) -> f64 {
        let ability = learner.operation_abilities
            .get(&item.operation_type)
            .unwrap_or(&learner.ability);
        
        // 3-parameter logistic model
        let z = item.discrimination * (ability - item.difficulty);
        item.guessing + (1.0 - item.guessing) / (1.0 + (-z).exp())
    }
    
    /// Predict response time
    pub fn predict_response_time(&self, learner: &IndividualParameters, item: &ItemParameters) -> f64 {
        let ability = learner.operation_abilities
            .get(&item.operation_type)
            .unwrap_or(&learner.ability);
        
        // Response time decreases with ability
        let base_rt = learner.response_time_params.mu;
        let ability_effect = -0.2 * (ability - item.difficulty);
        
        (base_rt + ability_effect).exp()
    }
    
    /// Sample from Dirichlet distribution
    fn sample_dirichlet(&self, alpha: &[f64]) -> Vec<f64> {
        let mut rng = thread_rng();
        let mut samples = Vec::new();
        
        // Sample from Gamma distributions
        for &a in alpha {
            let gamma = Gamma::new(a, 1.0).unwrap();
            samples.push(gamma.sample(&mut rng));
        }
        
        // Normalize
        let sum: f64 = samples.iter().sum();
        samples.iter().map(|s| s / sum).collect()
    }
    
    /// Perform MCMC sampling for posterior inference
    pub fn mcmc_sample(&mut self, n_iterations: usize) -> Vec<MCMCSample> {
        let mut samples = Vec::new();
        let mut rng = thread_rng();
        
        for _ in 0..n_iterations {
            // Sample hyperparameters
            self.sample_hyperparameters(&mut rng);
            
            // Sample population parameters
            self.sample_population_parameters(&mut rng);
            
            // Sample individual parameters
            for learner in self.individual_models.values_mut() {
                self.sample_individual_parameters(learner, &mut rng);
            }
            
            // Sample item parameters
            for item in self.item_bank.values_mut() {
                self.sample_item_parameters(item, &mut rng);
            }
            
            // Store sample
            samples.push(self.get_current_sample());
        }
        
        samples
    }
    
    fn sample_hyperparameters(&mut self, rng: &mut ThreadRng) {
        // Metropolis-Hastings for hyperparameters
        let proposal_std = 0.1;
        
        // Sample mu_theta
        let current = self.hyperparameters.mu_theta;
        let proposal = Normal::new(current, proposal_std).unwrap().sample(rng);
        let log_ratio = self.log_posterior_hyperparameter(proposal, "mu_theta") -
                       self.log_posterior_hyperparameter(current, "mu_theta");
        
        if rng.gen::<f64>().ln() < log_ratio {
            self.hyperparameters.mu_theta = proposal;
        }
        
        // Sample sigma_theta (must be positive)
        let current = self.hyperparameters.sigma_theta;
        let proposal = (Normal::new(current, proposal_std).unwrap().sample(rng)).abs().max(0.1);
        let log_ratio = self.log_posterior_hyperparameter(proposal, "sigma_theta") -
                       self.log_posterior_hyperparameter(current, "sigma_theta");
        
        if rng.gen::<f64>().ln() < log_ratio {
            self.hyperparameters.sigma_theta = proposal;
        }
    }
    
    fn sample_population_parameters(&mut self, rng: &mut ThreadRng) {
        // Gibbs sampling for population parameters
        if self.individual_models.is_empty() {
            return;
        }
        
        let abilities: Vec<f64> = self.individual_models.values()
            .map(|l| l.ability)
            .collect();
        
        // Sample mean ability (conjugate normal)
        let n = abilities.len() as f64;
        let sample_mean = abilities.iter().sum::<f64>() / n;
        
        let posterior_var = 1.0 / (n / self.population_parameters.variance_ability + 
                                   1.0 / self.hyperparameters.sigma_theta.powi(2));
        let posterior_mean = posterior_var * 
            (n * sample_mean / self.population_parameters.variance_ability +
             self.hyperparameters.mu_theta / self.hyperparameters.sigma_theta.powi(2));
        
        let dist = Normal::new(posterior_mean, posterior_var.sqrt()).unwrap();
        self.population_parameters.mean_ability = dist.sample(rng);
        
        // Sample variance (inverse gamma)
        let alpha = n / 2.0 + 1.0;
        let beta = abilities.iter()
            .map(|a| (a - self.population_parameters.mean_ability).powi(2))
            .sum::<f64>() / 2.0;
        
        let gamma = Gamma::new(alpha, 1.0 / beta).unwrap();
        self.population_parameters.variance_ability = 1.0 / gamma.sample(rng);
    }
    
    fn sample_individual_parameters(&mut self, learner: &mut IndividualParameters, rng: &mut ThreadRng) {
        // Sample ability
        let responses: Vec<_> = self.data_points.iter()
            .filter(|r| r.learner_id == learner.learner_id)
            .collect();
        
        if !responses.is_empty() {
            // Calculate likelihood
            let mut log_likelihood = 0.0;
            for response in &responses {
                if let Some(item) = self.item_bank.get(&response.task_id) {
                    let p = self.predict_probability(learner, item);
                    log_likelihood += if response.correct { p.ln() } else { (1.0 - p).ln() };
                }
            }
            
            // Prior
            let prior_dist = Normal::new(
                self.population_parameters.mean_ability,
                self.population_parameters.variance_ability.sqrt()
            ).unwrap();
            
            // Metropolis step
            let proposal_std = 0.2;
            let current = learner.ability;
            let proposal = Normal::new(current, proposal_std).unwrap().sample(rng);
            
            let log_prior_ratio = prior_dist.ln_pdf(proposal) - prior_dist.ln_pdf(current);
            
            // Calculate likelihood ratio
            let old_ability = learner.ability;
            learner.ability = proposal;
            let mut new_log_likelihood = 0.0;
            for response in &responses {
                if let Some(item) = self.item_bank.get(&response.task_id) {
                    let p = self.predict_probability(learner, item);
                    new_log_likelihood += if response.correct { p.ln() } else { (1.0 - p).ln() };
                }
            }
            
            let log_ratio = new_log_likelihood - log_likelihood + log_prior_ratio;
            
            if rng.gen::<f64>().ln() >= log_ratio {
                learner.ability = old_ability; // Reject proposal
            }
        }
        
        // Sample strategy weights (Dirichlet)
        let alpha = learner.strategy_weights.iter().map(|w| w * 10.0).collect::<Vec<_>>();
        learner.strategy_weights = self.sample_dirichlet(&alpha);
    }
    
    fn sample_item_parameters(&mut self, item: &mut ItemParameters, rng: &mut ThreadRng) {
        // Sample difficulty
        let responses: Vec<_> = self.data_points.iter()
            .filter(|r| r.task_id == item.task_id)
            .collect();
        
        if responses.len() > 5 { // Only update if we have enough data
            let n_correct = responses.iter().filter(|r| r.correct).count() as f64;
            let n_total = responses.len() as f64;
            
            // Beta-binomial conjugate update
            let alpha = n_correct + 1.0;
            let beta = n_total - n_correct + 1.0;
            
            let beta_dist = Beta::new(alpha, beta).unwrap();
            let p_correct = beta_dist.sample(rng);
            
            // Convert to difficulty (logit scale)
            item.difficulty = -(p_correct / (1.0 - p_correct)).ln();
            item.difficulty = item.difficulty.max(-3.0).min(3.0);
        }
    }
    
    fn log_posterior_hyperparameter(&self, value: f64, param_name: &str) -> f64 {
        // Log prior (weakly informative)
        let log_prior = match param_name {
            "mu_theta" => {
                let prior = Normal::new(0.0, 10.0).unwrap();
                prior.ln_pdf(value)
            }
            "sigma_theta" => {
                if value <= 0.0 {
                    return f64::NEG_INFINITY;
                }
                let prior = Gamma::new(2.0, 2.0).unwrap();
                prior.ln_pdf(value)
            }
            _ => 0.0,
        };
        
        // Log likelihood (how well it explains population parameters)
        let log_likelihood = if !self.individual_models.is_empty() {
            let dist = Normal::new(
                self.hyperparameters.mu_theta,
                self.hyperparameters.sigma_theta
            ).unwrap();
            
            self.individual_models.values()
                .map(|l| dist.ln_pdf(l.ability))
                .sum::<f64>()
        } else {
            0.0
        };
        
        log_prior + log_likelihood
    }
    
    fn get_current_sample(&self) -> MCMCSample {
        MCMCSample {
            hyperparameters: self.hyperparameters.clone(),
            population_mean_ability: self.population_parameters.mean_ability,
            population_variance_ability: self.population_parameters.variance_ability,
            individual_abilities: self.individual_models.iter()
                .map(|(id, params)| (id.clone(), params.ability))
                .collect(),
            item_difficulties: self.item_bank.iter()
                .map(|(id, params)| (id.clone(), params.difficulty))
                .collect(),
        }
    }
    
    /// Calculate DIC (Deviance Information Criterion)
    pub fn calculate_dic(&self, samples: &[MCMCSample]) -> f64 {
        // Calculate mean deviance
        let mean_deviance = samples.iter()
            .map(|s| self.calculate_deviance(s))
            .sum::<f64>() / samples.len() as f64;
        
        // Calculate deviance at posterior mean
        let posterior_mean = self.calculate_posterior_mean(samples);
        let deviance_at_mean = self.calculate_deviance(&posterior_mean);
        
        // DIC = mean(deviance) + (mean(deviance) - deviance(mean))
        2.0 * mean_deviance - deviance_at_mean
    }
    
    fn calculate_deviance(&self, sample: &MCMCSample) -> f64 {
        let mut log_likelihood = 0.0;
        
        for response in &self.data_points {
            if let (Some(learner_ability), Some(item_difficulty)) = (
                sample.individual_abilities.get(&response.learner_id),
                sample.item_difficulties.get(&response.task_id)
            ) {
                // Simple 2PL model for deviance calculation
                let z = 1.0 * (learner_ability - item_difficulty);
                let p = 1.0 / (1.0 + (-z).exp());
                
                log_likelihood += if response.correct {
                    p.ln()
                } else {
                    (1.0 - p).ln()
                };
            }
        }
        
        -2.0 * log_likelihood
    }
    
    fn calculate_posterior_mean(&self, samples: &[MCMCSample]) -> MCMCSample {
        let n = samples.len() as f64;
        
        let mut mean_sample = MCMCSample {
            hyperparameters: self.hyperparameters.clone(),
            population_mean_ability: 0.0,
            population_variance_ability: 0.0,
            individual_abilities: HashMap::new(),
            item_difficulties: HashMap::new(),
        };
        
        // Average population parameters
        mean_sample.population_mean_ability = samples.iter()
            .map(|s| s.population_mean_ability)
            .sum::<f64>() / n;
        
        mean_sample.population_variance_ability = samples.iter()
            .map(|s| s.population_variance_ability)
            .sum::<f64>() / n;
        
        // Average individual abilities
        for learner_id in self.individual_models.keys() {
            let mean_ability = samples.iter()
                .filter_map(|s| s.individual_abilities.get(learner_id))
                .sum::<f64>() / n;
            mean_sample.individual_abilities.insert(learner_id.clone(), mean_ability);
        }
        
        // Average item difficulties
        for item_id in self.item_bank.keys() {
            let mean_difficulty = samples.iter()
                .filter_map(|s| s.item_difficulties.get(item_id))
                .sum::<f64>() / n;
            mean_sample.item_difficulties.insert(item_id.clone(), mean_difficulty);
        }
        
        mean_sample
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCMCSample {
    pub hyperparameters: Hyperparameters,
    pub population_mean_ability: f64,
    pub population_variance_ability: f64,
    pub individual_abilities: HashMap<String, f64>,
    pub item_difficulties: HashMap<String, f64>,
}

/// Multi-level model for nested data structures
pub struct MultiLevelModel {
    pub levels: Vec<Level>,
    pub cross_level_interactions: Vec<Interaction>,
    pub random_effects: HashMap<String, RandomEffect>,
}

#[derive(Debug, Clone)]
pub struct Level {
    pub name: String,
    pub units: Vec<String>,
    pub fixed_effects: HashMap<String, f64>,
    pub variance_components: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct Interaction {
    pub level1: String,
    pub level2: String,
    pub coefficient: f64,
}

#[derive(Debug, Clone)]
pub struct RandomEffect {
    pub level: String,
    pub unit: String,
    pub effect: f64,
    pub variance: f64,
}

impl MultiLevelModel {
    pub fn new() -> Self {
        MultiLevelModel {
            levels: Vec::new(),
            cross_level_interactions: Vec::new(),
            random_effects: HashMap::new(),
        }
    }
    
    pub fn add_level(&mut self, name: String, units: Vec<String>) {
        let level = Level {
            name,
            units,
            fixed_effects: HashMap::new(),
            variance_components: HashMap::new(),
        };
        self.levels.push(level);
    }
    
    pub fn fit(&mut self, data: &[ResponseData]) {
        // Simplified fitting procedure
        // In practice, would use REML or full Bayesian inference
        
        // Estimate variance components
        for level in &mut self.levels {
            level.variance_components.insert("intercept".to_string(), 1.0);
            level.variance_components.insert("slope".to_string(), 0.5);
        }
        
        // Estimate fixed effects
        for level in &mut self.levels {
            level.fixed_effects.insert("intercept".to_string(), 0.0);
            level.fixed_effects.insert("time".to_string(), 0.1);
        }
        
        // Estimate random effects (BLUPs)
        for response in data {
            let key = format!("{}_{}", response.learner_id, response.task_id);
            self.random_effects.insert(key, RandomEffect {
                level: "learner".to_string(),
                unit: response.learner_id.clone(),
                effect: 0.0,
                variance: 0.1,
            });
        }
    }
    
    pub fn predict(&self, learner_id: &str, task_id: &str) -> f64 {
        // Combine fixed and random effects
        let mut prediction = 0.0;
        
        // Add fixed effects
        for level in &self.levels {
            if let Some(&intercept) = level.fixed_effects.get("intercept") {
                prediction += intercept;
            }
        }
        
        // Add random effects
        let key = format!("{}_{}", learner_id, task_id);
        if let Some(random_effect) = self.random_effects.get(&key) {
            prediction += random_effect.effect;
        }
        
        // Apply logistic transformation for probability
        1.0 / (1.0 + (-prediction).exp())
    }
}