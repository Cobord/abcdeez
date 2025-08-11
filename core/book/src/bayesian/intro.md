# Bayesian Inference Framework

The Bayesian inference framework forms the mathematical foundation of ABCDeez Core's adaptive assessment system. By treating all learning parameters as probability distributions rather than fixed values, the system can quantify uncertainty, make principled inferences from limited data, and continuously update beliefs as evidence accumulates.

## Conceptual Foundation

### Why Bayesian Methods?

Traditional assessment systems make binary decisions: a learner either knows something or doesn't. Bayesian methods provide a more nuanced view:

1. **Uncertainty quantification**: We express how confident we are in our assessments
2. **Evidence accumulation**: Each response updates our beliefs systematically  
3. **Prior knowledge integration**: We can incorporate domain expertise and previous experience
4. **Principled decision making**: All inferences follow from probability theory
5. **Adaptive behavior**: The system becomes more confident and accurate over time

### Core Bayesian Concepts

#### Prior, Likelihood, and Posterior

The fundamental Bayesian equation governs all learning in ABCDeez Core:

**P(θ|data) = P(data|θ) × P(θ) / P(data)**

Where:
- **P(θ)**: Prior belief about parameter θ before seeing data
- **P(data|θ)**: Likelihood of observing the data given parameter θ
- **P(θ|data)**: Posterior belief about θ after incorporating the data
- **P(data)**: Marginal likelihood (normalizing constant)

#### Sequential Learning

Each new observation updates the posterior, which becomes the prior for the next observation:

```
Initial: P(θ)
After response 1: P(θ|r₁) ∝ P(r₁|θ) × P(θ)
After response 2: P(θ|r₁,r₂) ∝ P(r₂|θ) × P(θ|r₁)
...and so on
```

## Mathematical Framework

### Latent Variable Model

ABCDeez Core represents learner knowledge using latent variables:

```rust
// Core latent variables in the model
pub struct LatentVariables {
    pub theta: f64,              // Learner ability on logit scale
    pub theta_uncertainty: f64,  // Standard deviation of theta estimate
    pub item_difficulties: HashMap<String, f64>, // Item difficulty parameters
    pub item_discriminations: HashMap<String, f64>, // Item discrimination parameters
}

#[derive(Debug, Clone)]
pub struct BayesianModel {
    pub priors: PriorDistributions,
    pub hyperparameters: Hyperparameters,
    pub update_history: Vec<UpdateStep>,
}

#[derive(Debug, Clone)]
pub struct PriorDistributions {
    pub theta_prior_mean: f64,         // Prior mean for learner ability
    pub theta_prior_variance: f64,     // Prior variance for learner ability  
    pub difficulty_prior_mean: f64,    // Prior mean for item difficulties
    pub difficulty_prior_variance: f64, // Prior variance for item difficulties
    pub discrimination_prior_alpha: f64, // Prior α for discrimination (Gamma)
    pub discrimination_prior_beta: f64,  // Prior β for discrimination (Gamma)
}
```

### Item Response Theory Integration

The system uses a Bayesian version of the 2-parameter logistic (2PL) model:

**P(correct|θ, a, b) = 1 / (1 + exp(-a(θ - b)))**

Where:
- **θ**: Learner ability (latent trait)
- **a**: Item discrimination (how well the item differentiates ability levels)
- **b**: Item difficulty (ability level where P(correct) = 0.5)

```rust
pub struct TwoParameterLogisticModel {
    discrimination: f64,    // 'a' parameter
    difficulty: f64,        // 'b' parameter
}

impl TwoParameterLogisticModel {
    pub fn probability_correct(&self, theta: f64) -> f64 {
        1.0 / (1.0 + (-self.discrimination * (theta - self.difficulty)).exp())
    }
    
    pub fn log_likelihood(&self, theta: f64, response: bool) -> f64 {
        let p = self.probability_correct(theta);
        if response {
            p.ln()
        } else {
            (1.0 - p).ln()
        }
    }
    
    pub fn information(&self, theta: f64) -> f64 {
        // Fisher information: how much information this item provides at ability θ
        let p = self.probability_correct(theta);
        self.discrimination.powi(2) * p * (1.0 - p)
    }
}
```

### Bayesian Updating Algorithms

#### Maximum A Posteriori (MAP) Estimation

For point estimates of learner ability:

```rust
pub struct MAPEstimator {
    tolerance: f64,
    max_iterations: usize,
    learning_rate: f64,
}

impl MAPEstimator {
    pub fn estimate_theta(
        &self,
        responses: &[TaskResponse],
        items: &HashMap<String, TwoParameterLogisticModel>,
        prior: &NormalDistribution,
    ) -> Result<f64, String> {
        let mut theta = prior.mean; // Start at prior mean
        
        for _iteration in 0..self.max_iterations {
            let (gradient, hessian) = self.compute_derivatives(theta, responses, items, prior);
            
            // Newton-Raphson update
            let delta = gradient / hessian.abs();
            let new_theta = theta - self.learning_rate * delta;
            
            if (new_theta - theta).abs() < self.tolerance {
                return Ok(new_theta);
            }
            
            theta = new_theta;
        }
        
        Err("Failed to converge".to_string())
    }
    
    fn compute_derivatives(
        &self,
        theta: f64,
        responses: &[TaskResponse],
        items: &HashMap<String, TwoParameterLogisticModel>,
        prior: &NormalDistribution,
    ) -> (f64, f64) {
        let mut gradient = 0.0;
        let mut hessian = 0.0;
        
        // Contribution from likelihood
        for response in responses {
            if let Some(item) = items.get(&response.task.item_id) {
                let p = item.probability_correct(theta);
                let correct = response.correct as i32 as f64;
                
                gradient += item.discrimination * (correct - p);
                hessian -= item.discrimination.powi(2) * p * (1.0 - p);
            }
        }
        
        // Contribution from prior
        gradient -= (theta - prior.mean) / prior.variance;
        hessian -= 1.0 / prior.variance;
        
        (gradient, hessian)
    }
}
```

#### Full Bayesian Inference with MCMC

For uncertainty quantification, we use Markov Chain Monte Carlo:

```rust
pub struct MCMCsampler {
    pub chain_length: usize,
    pub burn_in: usize,
    pub thinning: usize,
    pub proposal_variance: f64,
}

impl MCMCsampler {
    pub fn sample_posterior(
        &self,
        responses: &[TaskResponse],
        items: &HashMap<String, TwoParameterLogisticModel>,
        prior: &NormalDistribution,
    ) -> Vec<f64> {
        let mut chain = Vec::with_capacity(self.chain_length);
        let mut current_theta = prior.mean;
        let mut current_log_prob = self.log_posterior(current_theta, responses, items, prior);
        
        for i in 0..self.chain_length + self.burn_in {
            // Propose new state
            let proposal = current_theta + rand::thread_rng().sample(
                Normal::new(0.0, self.proposal_variance).unwrap()
            );
            
            let proposal_log_prob = self.log_posterior(proposal, responses, items, prior);
            
            // Metropolis acceptance criterion
            let log_acceptance_ratio = proposal_log_prob - current_log_prob;
            let accept = log_acceptance_ratio > 0.0 || 
                        rand::random::<f64>() < log_acceptance_ratio.exp();
            
            if accept {
                current_theta = proposal;
                current_log_prob = proposal_log_prob;
            }
            
            // Store sample after burn-in
            if i >= self.burn_in && (i - self.burn_in) % self.thinning == 0 {
                chain.push(current_theta);
            }
        }
        
        chain
    }
    
    fn log_posterior(
        &self,
        theta: f64,
        responses: &[TaskResponse],
        items: &HashMap<String, TwoParameterLogisticModel>,
        prior: &NormalDistribution,
    ) -> f64 {
        let mut log_prob = 0.0;
        
        // Log-likelihood
        for response in responses {
            if let Some(item) = items.get(&response.task.item_id) {
                log_prob += item.log_likelihood(theta, response.correct);
            }
        }
        
        // Log-prior
        log_prob += prior.log_pdf(theta);
        
        log_prob
    }
}
```

### Hierarchical Bayesian Models

For modeling individual differences within populations:

```rust
#[derive(Debug, Clone)]
pub struct HierarchicalModel {
    pub population_mean: f64,        // μ: Population mean ability
    pub population_variance: f64,    // τ²: Population variance in ability
    pub individual_abilities: HashMap<String, f64>, // θᵢ: Individual abilities
    pub individual_variances: HashMap<String, f64>, // σᵢ²: Individual uncertainties
}

impl HierarchicalModel {
    pub fn update_population_parameters(
        &mut self,
        individual_estimates: &HashMap<String, (f64, f64)>,
    ) {
        // Update population mean (weighted by precision)
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        
        for (mean, variance) in individual_estimates.values() {
            let precision = 1.0 / variance;
            weighted_sum += precision * mean;
            weight_sum += precision;
        }
        
        if weight_sum > 0.0 {
            self.population_mean = weighted_sum / weight_sum;
        }
        
        // Update population variance
        let mut sum_squared_deviations = 0.0;
        for (mean, _) in individual_estimates.values() {
            sum_squared_deviations += (mean - self.population_mean).powi(2);
        }
        
        let n = individual_estimates.len() as f64;
        if n > 1.0 {
            self.population_variance = sum_squared_deviations / (n - 1.0);
        }
    }
    
    pub fn shrink_individual_estimate(
        &self,
        individual_mean: f64,
        individual_variance: f64,
        n_observations: usize,
    ) -> (f64, f64) {
        // James-Stein shrinkage toward population mean
        let population_precision = 1.0 / self.population_variance;
        let individual_precision = (n_observations as f64) / individual_variance;
        let posterior_precision = population_precision + individual_precision;
        
        let shrunk_mean = (population_precision * self.population_mean + 
                          individual_precision * individual_mean) / posterior_precision;
        let shrunk_variance = 1.0 / posterior_precision;
        
        (shrunk_mean, shrunk_variance)
    }
}
```

## Practical Implementation

### Bayesian Learning System

```rust
pub struct BayesianLearningSystem {
    pub model: BayesianModel,
    pub estimator: MAPEstimator,
    pub mcmc_sampler: MCMCsampler,
    pub hierarchical_model: Option<HierarchicalModel>,
    pub item_bank: HashMap<String, TwoParameterLogisticModel>,
}

impl BayesianLearningSystem {
    pub fn new() -> Self {
        Self {
            model: BayesianModel::default(),
            estimator: MAPEstimator {
                tolerance: 0.001,
                max_iterations: 100,
                learning_rate: 1.0,
            },
            mcmc_sampler: MCMCsampler {
                chain_length: 1000,
                burn_in: 200,
                thinning: 2,
                proposal_variance: 0.5,
            },
            hierarchical_model: None,
            item_bank: HashMap::new(),
        }
    }
    
    pub fn update_learner_estimate(
        &mut self,
        learner_id: &str,
        new_response: TaskResponse,
    ) -> Result<LearnerEstimate, String> {
        // Get current estimate
        let current_estimate = self.get_current_estimate(learner_id)?;
        
        // Update with new response
        let responses = self.get_all_responses(learner_id);
        responses.push(new_response);
        
        // Compute new MAP estimate
        let new_theta = self.estimator.estimate_theta(
            &responses,
            &self.item_bank,
            &NormalDistribution {
                mean: current_estimate.theta,
                variance: current_estimate.uncertainty.powi(2),
            },
        )?;
        
        // Compute uncertainty (inverse of Fisher information)
        let information = self.calculate_total_information(learner_id, &responses);
        let new_uncertainty = if information > 0.0 {
            (1.0 / information).sqrt()
        } else {
            current_estimate.uncertainty
        };
        
        let new_estimate = LearnerEstimate {
            learner_id: learner_id.to_string(),
            theta: new_theta,
            uncertainty: new_uncertainty,
            n_responses: responses.len(),
            last_updated: Utc::now(),
        };
        
        // Apply hierarchical shrinkage if available
        if let Some(hierarchical) = &self.hierarchical_model {
            let (shrunk_theta, shrunk_variance) = hierarchical.shrink_individual_estimate(
                new_theta,
                new_uncertainty.powi(2),
                responses.len(),
            );
            
            new_estimate.theta = shrunk_theta;
            new_estimate.uncertainty = shrunk_variance.sqrt();
        }
        
        self.store_estimate(learner_id, &new_estimate);
        Ok(new_estimate)
    }
    
    fn calculate_total_information(
        &self,
        learner_id: &str,
        responses: &[TaskResponse],
    ) -> f64 {
        let current_theta = self.get_current_estimate(learner_id)
            .map(|est| est.theta)
            .unwrap_or(0.0);
            
        responses.iter()
            .filter_map(|response| self.item_bank.get(&response.task.item_id))
            .map(|item| item.information(current_theta))
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct LearnerEstimate {
    pub learner_id: String,
    pub theta: f64,           // Ability estimate
    pub uncertainty: f64,     // Standard error of estimate
    pub n_responses: usize,   // Number of responses
    pub last_updated: DateTime<Utc>,
}

impl LearnerEstimate {
    pub fn confidence_interval(&self, confidence_level: f64) -> (f64, f64) {
        // Calculate confidence interval for ability estimate
        let z_score = match confidence_level {
            0.90 => 1.645,
            0.95 => 1.96,
            0.99 => 2.576,
            _ => 1.96, // Default to 95%
        };
        
        let margin = z_score * self.uncertainty;
        (self.theta - margin, self.theta + margin)
    }
    
    pub fn probability_correct(&self, item: &TwoParameterLogisticModel) -> f64 {
        // Expected probability of correct response
        item.probability_correct(self.theta)
    }
    
    pub fn is_well_estimated(&self) -> bool {
        // Criteria for a well-estimated ability
        self.uncertainty < 0.5 && self.n_responses >= 10
    }
}
```

### Model Comparison and Selection

```rust
pub struct ModelComparison {
    candidate_models: Vec<BayesianModel>,
    comparison_metrics: Vec<String>,
}

impl ModelComparison {
    pub fn compute_model_evidence(
        &self,
        model: &BayesianModel,
        data: &[TaskResponse],
    ) -> f64 {
        // Compute marginal likelihood using importance sampling
        let n_samples = 10000;
        let mut log_evidence = 0.0;
        
        for _ in 0..n_samples {
            // Sample from prior
            let theta_sample = model.sample_from_prior();
            
            // Compute likelihood
            let log_likelihood = data.iter()
                .map(|response| {
                    if let Some(item) = model.get_item(&response.task.item_id) {
                        item.log_likelihood(theta_sample, response.correct)
                    } else {
                        0.0
                    }
                })
                .sum::<f64>();
                
            log_evidence += log_likelihood.exp();
        }
        
        (log_evidence / n_samples as f64).ln()
    }
    
    pub fn bayes_factor(&self, model1: &BayesianModel, model2: &BayesianModel, data: &[TaskResponse]) -> f64 {
        let evidence1 = self.compute_model_evidence(model1, data);
        let evidence2 = self.compute_model_evidence(model2, data);
        (evidence1 - evidence2).exp()
    }
    
    pub fn aic(&self, model: &BayesianModel, data: &[TaskResponse]) -> f64 {
        let log_likelihood = self.compute_log_likelihood(model, data);
        let n_parameters = model.count_parameters();
        -2.0 * log_likelihood + 2.0 * n_parameters as f64
    }
    
    pub fn dic(&self, model: &BayesianModel, data: &[TaskResponse]) -> f64 {
        // Deviance Information Criterion
        let posterior_samples = self.sample_posterior(model, data);
        
        // Mean deviance
        let mean_deviance: f64 = posterior_samples.iter()
            .map(|theta| -2.0 * self.compute_log_likelihood_at_theta(*theta, data))
            .sum::<f64>() / posterior_samples.len() as f64;
        
        // Effective number of parameters
        let mean_theta: f64 = posterior_samples.iter().sum::<f64>() / posterior_samples.len() as f64;
        let deviance_at_mean = -2.0 * self.compute_log_likelihood_at_theta(mean_theta, data);
        let p_eff = mean_deviance - deviance_at_mean;
        
        mean_deviance + p_eff
    }
}
```

## Advanced Features

### Adaptive Prior Selection

```rust
pub struct AdaptivePriorSelection {
    candidate_priors: Vec<PriorDistribution>,
    prior_weights: Vec<f64>,
    update_frequency: usize,
}

impl AdaptivePriorSelection {
    pub fn update_prior_weights(&mut self, new_data: &[TaskResponse]) {
        let mut new_weights = Vec::new();
        
        for (i, prior) in self.candidate_priors.iter().enumerate() {
            // Compute marginal likelihood under this prior
            let marginal_likelihood = self.compute_marginal_likelihood(prior, new_data);
            
            // Update weight using Bayes rule
            let updated_weight = self.prior_weights[i] * marginal_likelihood;
            new_weights.push(updated_weight);
        }
        
        // Normalize weights
        let total_weight: f64 = new_weights.iter().sum();
        for weight in &mut new_weights {
            *weight /= total_weight;
        }
        
        self.prior_weights = new_weights;
    }
    
    pub fn get_mixture_prior(&self) -> MixturePrior {
        MixturePrior {
            components: self.candidate_priors.clone(),
            weights: self.prior_weights.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MixturePrior {
    components: Vec<PriorDistribution>,
    weights: Vec<f64>,
}

impl MixturePrior {
    pub fn sample(&self) -> f64 {
        // Select component based on weights
        let u: f64 = rand::random();
        let mut cumsum = 0.0;
        
        for (i, weight) in self.weights.iter().enumerate() {
            cumsum += weight;
            if u <= cumsum {
                return self.components[i].sample();
            }
        }
        
        // Fallback to last component
        self.components.last().unwrap().sample()
    }
    
    pub fn pdf(&self, x: f64) -> f64 {
        self.components.iter()
            .zip(self.weights.iter())
            .map(|(component, weight)| weight * component.pdf(x))
            .sum()
    }
}
```

### Online Learning with Forgetting

```rust
pub struct OnlineBayesianLearner {
    forgetting_rate: f64,
    minimum_information: f64,
    recency_weight: f64,
}

impl OnlineBayesianLearner {
    pub fn update_with_forgetting(
        &mut self,
        current_estimate: &LearnerEstimate,
        new_response: &TaskResponse,
        time_elapsed: f64,
    ) -> LearnerEstimate {
        // Apply forgetting to current estimate
        let information_loss = self.forgetting_rate * time_elapsed;
        let degraded_precision = 1.0 / current_estimate.uncertainty.powi(2) - information_loss;
        let degraded_precision = degraded_precision.max(1.0 / self.minimum_information);
        let degraded_uncertainty = (1.0 / degraded_precision).sqrt();
        
        // Weight new information more heavily
        let recency_adjusted_information = self.calculate_information(new_response) * 
                                         (1.0 + self.recency_weight);
        
        // Combine degraded prior with new information
        let new_precision = degraded_precision + recency_adjusted_information;
        let new_uncertainty = (1.0 / new_precision).sqrt();
        
        // Update mean (simplified)
        let likelihood_contribution = if new_response.correct { 1.0 } else { -1.0 };
        let new_theta = (degraded_precision * current_estimate.theta + 
                        recency_adjusted_information * likelihood_contribution) / new_precision;
        
        LearnerEstimate {
            learner_id: current_estimate.learner_id.clone(),
            theta: new_theta,
            uncertainty: new_uncertainty,
            n_responses: current_estimate.n_responses + 1,
            last_updated: Utc::now(),
        }
    }
}
```

## Testing and Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bayesian_updating() {
        let mut system = BayesianLearningSystem::new();
        
        // Add a test item
        system.item_bank.insert("item1".to_string(), TwoParameterLogisticModel {
            discrimination: 1.0,
            difficulty: 0.0,
        });
        
        // Start with neutral prior
        let initial_estimate = LearnerEstimate {
            learner_id: "learner1".to_string(),
            theta: 0.0,
            uncertainty: 1.0,
            n_responses: 0,
            last_updated: Utc::now(),
        };
        
        system.store_estimate("learner1", &initial_estimate);
        
        // Simulate correct response
        let correct_response = TaskResponse {
            task: Task {
                item_id: "item1".to_string(),
                // ... other fields
            },
            correct: true,
            response_time_ms: 1500.0,
            // ... other fields
        };
        
        let updated_estimate = system.update_learner_estimate("learner1", correct_response).unwrap();
        
        // Ability should increase after correct response
        assert!(updated_estimate.theta > initial_estimate.theta);
        // Uncertainty should decrease with more information
        assert!(updated_estimate.uncertainty < initial_estimate.uncertainty);
    }
    
    #[test]
    fn test_confidence_intervals() {
        let estimate = LearnerEstimate {
            learner_id: "test".to_string(),
            theta: 1.0,
            uncertainty: 0.5,
            n_responses: 20,
            last_updated: Utc::now(),
        };
        
        let (lower, upper) = estimate.confidence_interval(0.95);
        
        // 95% CI should span about 4 standard errors
        let expected_width = 2.0 * 1.96 * 0.5;
        let actual_width = upper - lower;
        assert!((actual_width - expected_width).abs() < 0.01);
    }
    
    #[test]
    fn test_hierarchical_shrinkage() {
        let mut hierarchical = HierarchicalModel {
            population_mean: 0.0,
            population_variance: 1.0,
            individual_abilities: HashMap::new(),
            individual_variances: HashMap::new(),
        };
        
        // Individual with extreme estimate but low precision
        let (shrunk_mean, shrunk_variance) = hierarchical.shrink_individual_estimate(
            2.0,  // Extreme individual estimate
            4.0,  // High individual variance
            5,    // Few observations
        );
        
        // Should shrink toward population mean
        assert!(shrunk_mean < 2.0);
        assert!(shrunk_mean > 0.0);
        
        // Shrinkage should reduce variance
        assert!(shrunk_variance < 4.0);
    }
}
```

## Best Practices

1. **Choose appropriate priors**: Balance informativeness with objectivity
2. **Monitor convergence**: Check MCMC diagnostics and estimate stability
3. **Validate assumptions**: Test model assumptions with posterior predictive checks
4. **Handle extreme responses**: Implement robust procedures for unusual patterns
5. **Computational efficiency**: Use analytical solutions when available, approximations when necessary
6. **Uncertainty propagation**: Always carry forward uncertainty through calculations

## Common Pitfalls

- **Overconfident priors**: Too-narrow priors can bias estimates
- **Computational issues**: MCMC may not converge or mix poorly
- **Model misspecification**: Wrong likelihood function or missing parameters
- **Ignoring model uncertainty**: Focusing only on parameter uncertainty
- **Scale confusion**: Mixing up different parameterizations (logit vs. probability)

## Applications

- **Computerized Adaptive Testing**: Optimal item selection using information theory
- **Learning Analytics**: Tracking knowledge growth over time
- **Diagnostic Assessment**: Identifying specific skill deficits with uncertainty
- **Personalized Learning**: Adapting content difficulty to learner ability
- **Educational Research**: Making inferences about learning processes

## Next Steps

- Explore [Bayesian Architecture](./architecture.md) for system design details
- Learn about [Bayesian Updates](./updates.md) for implementation specifics  
- Understand [Model Comparison](./comparison.md) for selecting optimal models
- See [Statistical Methods](../statistics/core.md) for complementary techniques