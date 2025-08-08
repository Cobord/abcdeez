# Hierarchical Bayesian Models

Hierarchical Bayesian models capture both individual differences and population-level patterns, providing a principled framework for understanding learning across multiple learners.

## Model Architecture

The hierarchical structure separates individual and population parameters:

```
Population Level (Hyperparameters)
        ↓
Individual Level (Parameters)  
        ↓
Data Level (Observations)
```

## Mathematical Formulation

### Three-Level Hierarchy

```
Level 1 (Data):     y_ij | θ_i ~ p(y | θ_i)
Level 2 (Individual): θ_i | φ ~ p(θ | φ)
Level 3 (Population): φ ~ p(φ)
```

Where:
- y_ij = observation j from individual i
- θ_i = parameters for individual i
- φ = population hyperparameters

## Implementation

```rust
pub struct HierarchicalBayesianModel {
    // Population hyperparameters
    pub hyperparameters: Hyperparameters,
    
    // Individual parameters
    pub individual_models: HashMap<String, IndividualParameters>,
    
    // Data
    pub observations: Vec<ResponseData>,
}

#[derive(Debug, Clone)]
pub struct Hyperparameters {
    // Population ability distribution
    pub mu_theta: f64,      // Mean ability
    pub sigma_theta: f64,   // SD of ability
    
    // Learning rate distribution
    pub alpha_learn: f64,   // Shape parameter
    pub beta_learn: f64,    // Rate parameter
    
    // Response time distribution
    pub mu_rt: f64,         // Mean log(RT)
    pub sigma_rt: f64,      // SD log(RT)
    
    // Prior concentrations
    pub kappa: f64,         // Concentration parameter
}

#[derive(Debug, Clone)]
pub struct IndividualParameters {
    pub learner_id: String,
    pub ability: f64,
    pub learning_rate: f64,
    pub response_params: ResponseParams,
    pub strategy_weights: Vec<f64>,
}
```

## Inference via MCMC

### Gibbs Sampling

```rust
impl HierarchicalBayesianModel {
    pub fn gibbs_sampler(&mut self, n_iter: usize) -> MCMCChain {
        let mut chain = MCMCChain::new();
        
        for iter in 0..n_iter {
            // Sample hyperparameters given individuals
            self.sample_hyperparameters();
            
            // Sample individual parameters given hyperparameters and data
            for (id, _) in self.individual_models.clone() {
                self.sample_individual_parameters(&id);
            }
            
            // Store samples
            if iter > n_iter / 2 { // After burn-in
                chain.add_sample(self.get_current_state());
            }
        }
        
        chain
    }
    
    fn sample_hyperparameters(&mut self) {
        // Sample mu_theta from posterior
        let abilities: Vec<f64> = self.individual_models
            .values()
            .map(|m| m.ability)
            .collect();
        
        let n = abilities.len() as f64;
        let mean_ability = mean(&abilities);
        
        // Conjugate update for normal-normal model
        let precision_0 = 1.0 / self.hyperparameters.sigma_theta.powi(2);
        let precision_n = n / self.hyperparameters.sigma_theta.powi(2);
        
        let posterior_precision = precision_0 + precision_n;
        let posterior_mean = (precision_n * mean_ability) / posterior_precision;
        let posterior_std = (1.0 / posterior_precision).sqrt();
        
        self.hyperparameters.mu_theta = sample_normal(posterior_mean, posterior_std);
        
        // Sample sigma_theta from inverse-gamma posterior
        let sum_sq = abilities.iter()
            .map(|a| (a - self.hyperparameters.mu_theta).powi(2))
            .sum::<f64>();
        
        let alpha = n / 2.0 + 1.0;
        let beta = sum_sq / 2.0;
        
        self.hyperparameters.sigma_theta = sample_inverse_gamma(alpha, beta).sqrt();
    }
    
    fn sample_individual_parameters(&mut self, learner_id: &str) {
        let data = self.get_learner_data(learner_id);
        let hyperparams = &self.hyperparameters;
        
        // Sample ability using Metropolis-Hastings
        let current = self.individual_models[learner_id].ability;
        let proposal = sample_normal(current, 0.1);
        
        let log_ratio = self.log_posterior(proposal, &data, hyperparams)
                      - self.log_posterior(current, &data, hyperparams);
        
        if rand::random::<f64>() < log_ratio.exp().min(1.0) {
            self.individual_models.get_mut(learner_id).unwrap().ability = proposal;
        }
        
        // Sample learning rate
        self.sample_learning_rate(learner_id);
        
        // Sample strategy weights
        self.sample_strategy_weights(learner_id);
    }
}
```

### Hamiltonian Monte Carlo

For more efficient sampling:

```rust
impl HierarchicalBayesianModel {
    pub fn hmc_sampler(&mut self, n_iter: usize, step_size: f64, n_steps: usize) -> MCMCChain {
        let mut chain = MCMCChain::new();
        
        for _ in 0..n_iter {
            // Sample momentum
            let momentum = self.sample_momentum();
            
            // Simulate Hamiltonian dynamics
            let (new_params, new_momentum) = self.leapfrog(
                self.get_params(),
                momentum,
                step_size,
                n_steps
            );
            
            // Accept/reject
            let current_h = self.hamiltonian(&self.get_params(), &momentum);
            let proposed_h = self.hamiltonian(&new_params, &new_momentum);
            
            let accept_prob = (current_h - proposed_h).exp().min(1.0);
            
            if rand::random::<f64>() < accept_prob {
                self.set_params(new_params);
            }
            
            chain.add_sample(self.get_current_state());
        }
        
        chain
    }
    
    fn leapfrog(&self, params: Parameters, momentum: Parameters, 
                epsilon: f64, n_steps: usize) -> (Parameters, Parameters) {
        let mut p = params;
        let mut m = momentum;
        
        // Half step for momentum
        let grad = self.gradient(&p);
        m = m + epsilon * 0.5 * grad;
        
        // Full steps
        for _ in 0..n_steps - 1 {
            p = p + epsilon * m;
            let grad = self.gradient(&p);
            m = m + epsilon * grad;
        }
        
        // Final position step
        p = p + epsilon * m;
        
        // Final half step for momentum
        let grad = self.gradient(&p);
        m = m + epsilon * 0.5 * grad;
        
        (p, -m) // Negate momentum for reversibility
    }
}
```

## Variational Inference

For faster approximate inference:

```rust
pub struct VariationalBayes {
    pub q_distributions: HashMap<String, VariationalDistribution>,
    pub elbo_history: Vec<f64>,
}

impl VariationalBayes {
    pub fn fit(&mut self, model: &HierarchicalBayesianModel, n_iter: usize) {
        for _ in 0..n_iter {
            // Update each variational distribution
            for param_name in self.q_distributions.keys() {
                self.update_q(param_name, model);
            }
            
            // Compute ELBO
            let elbo = self.compute_elbo(model);
            self.elbo_history.push(elbo);
            
            // Check convergence
            if self.converged() {
                break;
            }
        }
    }
    
    fn update_q(&mut self, param: &str, model: &HierarchicalBayesianModel) {
        // Coordinate ascent update
        let expected_log_joint = self.expected_log_joint(param, model);
        let entropy = self.q_distributions[param].entropy();
        
        // Find q that maximizes ELBO
        let optimal_q = self.optimize_q(expected_log_joint, entropy);
        self.q_distributions.insert(param.to_string(), optimal_q);
    }
    
    fn compute_elbo(&self, model: &HierarchicalBayesianModel) -> f64 {
        let expected_log_joint = self.expected_complete_log_likelihood(model);
        let entropy: f64 = self.q_distributions.values()
            .map(|q| q.entropy())
            .sum();
        
        expected_log_joint + entropy
    }
}
```

## Population-Level Analysis

### Shrinkage Estimation

Individual estimates are "shrunk" toward population mean:

```rust
impl HierarchicalBayesianModel {
    pub fn shrinkage_estimates(&self) -> HashMap<String, ShrunkEstimate> {
        let mut shrunk = HashMap::new();
        
        for (id, params) in &self.individual_models {
            let n_obs = self.get_n_observations(id);
            
            // Use empirical Bayes or full Bayesian approach
            // Population variance is estimated, not known
            let tau2_estimate = self.estimate_population_variance();
            let tau2_se = self.population_variance_std_error();
            
            // Within-subject variance
            let sigma2 = self.within_subject_variance(id);
            
            // Shrinkage factor with uncertainty
            let shrinkage_mean = tau2_estimate / (tau2_estimate + sigma2 / n_obs as f64);
            
            // Propagate uncertainty through shrinkage calculation
            let shrinkage_var = self.delta_method_variance(
                tau2_estimate, tau2_se, sigma2, n_obs
            );
            
            // Shrunk estimate with uncertainty
            let raw = params.ability;
            let shrunk_ability = shrinkage_mean * raw + 
                               (1.0 - shrinkage_mean) * self.hyperparameters.mu_theta;
            
            // Standard error of shrunk estimate
            let shrunk_se = (
                shrinkage_var * raw.powi(2) +
                shrinkage_mean.powi(2) * self.individual_variance(id) +
                (1.0 - shrinkage_mean).powi(2) * self.population_mean_variance()
            ).sqrt();
            
            shrunk.insert(id.clone(), ShrunkEstimate {
                value: shrunk_ability,
                std_error: shrunk_se,
                shrinkage_factor: shrinkage_mean,
                effective_sample_size: n_obs as f64 * shrinkage_mean,
            });
        }
        
        shrunk
    }
    
    fn delta_method_variance(&self, tau2: f64, tau2_se: f64, sigma2: f64, n: usize) -> f64 {
        // Delta method for variance of g(tau2) = tau2/(tau2 + sigma2/n)
        let denominator = tau2 + sigma2 / n as f64;
        let derivative = (sigma2 / n as f64) / denominator.powi(2);
        derivative.powi(2) * tau2_se.powi(2)
    }
}

pub struct ShrunkEstimate {
    pub value: f64,
    pub std_error: f64,
    pub shrinkage_factor: f64,
    pub effective_sample_size: f64,
}
```

### Population Heterogeneity

Detect subgroups within population:

```rust
pub struct PopulationMixture {
    pub n_groups: usize,
    pub group_params: Vec<GroupParameters>,
    pub group_assignments: HashMap<String, usize>,
}

impl PopulationMixture {
    pub fn fit_mixture(&mut self, data: &HierarchicalData) {
        // Initialize with k-means
        self.initialize_groups(data);
        
        // EM algorithm
        for _ in 0..100 {
            // E-step: assign individuals to groups
            self.assign_to_groups(data);
            
            // M-step: update group parameters
            self.update_group_parameters(data);
            
            if self.converged() {
                break;
            }
        }
    }
    
    fn assign_to_groups(&mut self, data: &HierarchicalData) {
        for (id, individual_data) in data {
            let mut best_group = 0;
            let mut best_prob = f64::NEG_INFINITY;
            
            for (g, params) in self.group_params.iter().enumerate() {
                let prob = params.log_likelihood(individual_data);
                if prob > best_prob {
                    best_prob = prob;
                    best_group = g;
                }
            }
            
            self.group_assignments.insert(id.clone(), best_group);
        }
    }
}
```

## Model Comparison

### Deviance Information Criterion (DIC)

```rust
impl HierarchicalBayesianModel {
    pub fn compute_dic(&self, chain: &MCMCChain) -> f64 {
        // Mean deviance
        let deviances: Vec<f64> = chain.samples.iter()
            .map(|params| -2.0 * self.log_likelihood_at(params))
            .collect();
        let mean_deviance = mean(&deviances);
        
        // Deviance at posterior mean
        let posterior_mean = chain.posterior_mean();
        let deviance_at_mean = -2.0 * self.log_likelihood_at(&posterior_mean);
        
        // Effective parameters
        let p_d = mean_deviance - deviance_at_mean;
        
        // DIC
        mean_deviance + p_d
    }
}
```

### Widely Applicable Information Criterion (WAIC)

```rust
impl HierarchicalBayesianModel {
    pub fn compute_waic(&self, chain: &MCMCChain) -> f64 {
        let n_data = self.observations.len();
        let mut lppd = 0.0;
        let mut p_waic = 0.0;
        
        for i in 0..n_data {
            // Log pointwise predictive density
            let log_likes: Vec<f64> = chain.samples.iter()
                .map(|params| self.log_likelihood_single(&self.observations[i], params))
                .collect();
            
            lppd += log_sum_exp(&log_likes) - (chain.samples.len() as f64).ln();
            
            // Effective parameters
            p_waic += variance(&log_likes);
        }
        
        -2.0 * (lppd - p_waic)
    }
}
```

## Applications

### Learning Rate Heterogeneity

```rust
impl HierarchicalBayesianModel {
    pub fn analyze_learning_rates(&self) -> LearningRateAnalysis {
        let rates: Vec<f64> = self.individual_models.values()
            .map(|m| m.learning_rate)
            .collect();
        
        LearningRateAnalysis {
            population_mean: mean(&rates),
            population_sd: std_dev(&rates),
            fast_learners: self.identify_fast_learners(&rates),
            slow_learners: self.identify_slow_learners(&rates),
            optimal_grouping: self.cluster_by_learning_rate(&rates),
        }
    }
    
    fn identify_fast_learners(&self, rates: &[f64]) -> Vec<String> {
        let threshold = percentile(rates, 0.75);
        
        self.individual_models.iter()
            .filter(|(_, m)| m.learning_rate > threshold)
            .map(|(id, _)| id.clone())
            .collect()
    }
}
```

### Transfer Learning Prediction

```rust
impl HierarchicalBayesianModel {
    pub fn predict_transfer(&self, learner_id: &str, new_domain: &Domain) -> TransferPrediction {
        let individual = &self.individual_models[learner_id];
        let pop_mean = self.hyperparameters.mu_theta;
        
        // Combine individual and population information
        let weight = self.compute_shrinkage_weight(learner_id);
        let predicted_ability = weight * individual.ability + (1.0 - weight) * pop_mean;
        
        // Adjust for domain similarity
        let similarity = self.compute_domain_similarity(&individual.domain, new_domain);
        let adjusted_ability = predicted_ability * similarity;
        
        TransferPrediction {
            expected_performance: sigmoid(adjusted_ability),
            confidence_interval: self.compute_prediction_interval(adjusted_ability),
            recommended_start_difficulty: self.calibrate_difficulty(adjusted_ability),
        }
    }
}
```

## Diagnostics

### Convergence Assessment

```rust
impl MCMCChain {
    pub fn gelman_rubin(&self, other_chains: &[MCMCChain]) -> f64 {
        let m = other_chains.len() + 1; // Number of chains
        let n = self.samples.len();     // Length of each chain
        
        // Between-chain variance
        let chain_means: Vec<f64> = std::iter::once(self)
            .chain(other_chains.iter())
            .map(|chain| chain.mean())
            .collect();
        
        let grand_mean = mean(&chain_means);
        let b = n as f64 * variance(&chain_means);
        
        // Within-chain variance
        let within_vars: Vec<f64> = std::iter::once(self)
            .chain(other_chains.iter())
            .map(|chain| chain.variance())
            .collect();
        
        let w = mean(&within_vars);
        
        // Potential scale reduction factor
        let var_plus = ((n - 1) as f64 * w + b) / n as f64;
        (var_plus / w).sqrt()
    }
    
    pub fn effective_sample_size(&self) -> f64 {
        let autocorr = self.autocorrelation();
        let sum_autocorr: f64 = autocorr.iter().take_while(|&&ac| ac > 0.05).sum();
        self.samples.len() as f64 / (1.0 + 2.0 * sum_autocorr)
    }
}
```

## Summary

Hierarchical Bayesian models provide:
- **Population inference**: Learn about groups from individuals
- **Shrinkage estimation**: Borrow strength across individuals
- **Uncertainty quantification**: Full posterior distributions
- **Flexible inference**: MCMC, HMC, variational methods
- **Model comparison**: DIC, WAIC for model selection
- **Transfer learning**: Predict performance in new domains

This framework enables robust inference about learning at both individual and population levels.