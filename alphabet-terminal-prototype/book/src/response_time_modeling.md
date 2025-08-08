# Response Time Modeling

Response time (RT) provides a window into cognitive processing. The system uses sophisticated statistical models to extract meaningful information from the temporal dynamics of responses.

## The Ex-Gaussian Model

Response times in cognitive tasks typically follow an Ex-Gaussian distribution—a convolution of Gaussian and exponential distributions:

```
f(x; μ, σ, τ) = (1/τ) × exp((μ - x)/τ + σ²/(2τ²)) × Φ((x - μ - σ²/τ)/σ)
```

Where:
- **μ**: Mean of the Gaussian component (typical response)
- **σ**: Standard deviation of the Gaussian component (variability)
- **τ**: Mean of the exponential component (slow responses)

### Cognitive Interpretation

Each parameter maps to cognitive processes:

```rust
pub struct ExGaussianParameters {
    pub mu: f64,    // Core processing time
    pub sigma: f64, // Motor/perceptual variability
    pub tau: f64,   // Attentional lapses/strategy shifts
}

impl ExGaussianParameters {
    pub fn interpret(&self) -> CognitiveProfile {
        CognitiveProfile {
            baseline_speed: 1.0 / self.mu,
            consistency: 1.0 / self.sigma,
            attention_stability: 1.0 / self.tau,
        }
    }
}
```

## Model Fitting

### Maximum Likelihood Estimation

```rust
impl ExGaussianModel {
    pub fn fit(data: &[f64]) -> Self {
        // Initial parameter estimates
        let mu_init = data.iter().sum::<f64>() / data.len() as f64;
        let sigma_init = statistical::std_dev(data);
        let tau_init = statistical::skewness(data).max(0.1) * sigma_init;
        
        // MLE optimization
        let params = optimize_mle(data, mu_init, sigma_init, tau_init);
        
        ExGaussianModel { params }
    }
    
    fn log_likelihood(data: &[f64], mu: f64, sigma: f64, tau: f64) -> f64 {
        data.iter()
            .map(|&x| self.log_pdf(x, mu, sigma, tau))
            .sum()
    }
    
    fn log_pdf(&self, x: f64, mu: f64, sigma: f64, tau: f64) -> f64 {
        if x < mu - 5.0 * sigma {
            return f64::NEG_INFINITY; // Impossible under model
        }
        
        let lambda = 1.0 / tau;
        let exp_arg = lambda * (mu - x + 0.5 * lambda * sigma * sigma);
        let norm_arg = (x - mu - lambda * sigma * sigma) / (sigma * SQRT_2);
        
        lambda.ln() + exp_arg + log_erfc(norm_arg)
    }
}
```

### Robust Estimation

Handle outliers and edge cases:

```rust
impl ExGaussianModel {
    pub fn fit_robust(data: &[f64]) -> Self {
        // Remove extreme outliers
        let cleaned = self.winsorize(data, 0.05);
        
        // Use method of moments for initial estimates
        let mu_init = percentile(&cleaned, 0.5); // Median
        let sigma_init = iqr(&cleaned) / 1.349; // Robust SD
        let tau_init = (mean(&cleaned) - mu_init).max(0.1);
        
        // Iteratively reweighted least squares
        let params = self.irls_fit(&cleaned, mu_init, sigma_init, tau_init);
        
        ExGaussianModel { params }
    }
    
    fn winsorize(&self, data: &[f64], alpha: f64) -> Vec<f64> {
        let lower = percentile(data, alpha);
        let upper = percentile(data, 1.0 - alpha);
        
        data.iter()
            .map(|&x| x.max(lower).min(upper))
            .collect()
    }
}
```

## Strategy-Dependent RT Models

Different cognitive strategies produce different RT signatures:

```rust
#[derive(Debug, Clone)]
pub enum RTModel {
    SerialScan {
        base_time: f64,
        scan_rate: f64,  // ms per item
    },
    DirectAccess {
        retrieval_time: f64,
        variance: f64,
    },
    Hybrid {
        serial_weight: f64,
        serial_model: Box<RTModel>,
        direct_model: Box<RTModel>,
    },
}

impl RTModel {
    pub fn predict(&self, task: &Task) -> f64 {
        match self {
            RTModel::SerialScan { base_time, scan_rate } => {
                let distance = self.compute_scan_distance(task);
                base_time + scan_rate * distance as f64
            }
            
            RTModel::DirectAccess { retrieval_time, variance } => {
                // Add noise for variability
                let noise = thread_rng().sample::<f64, _>(
                    Normal::new(0.0, *variance).unwrap()
                );
                retrieval_time + noise
            }
            
            RTModel::Hybrid { serial_weight, serial_model, direct_model } => {
                if thread_rng().gen::<f64>() < *serial_weight {
                    serial_model.predict(task)
                } else {
                    direct_model.predict(task)
                }
            }
        }
    }
}
```

## Distance Effects

Response time often correlates with cognitive distance:

```rust
pub struct DistanceEffectAnalyzer {
    topology: Topology,
}

impl DistanceEffectAnalyzer {
    pub fn analyze(&self, responses: &[ResponseData]) -> DistanceEffect {
        let mut distances = Vec::new();
        let mut rts = Vec::new();
        
        for response in responses {
            if let Some(distance) = self.compute_distance(&response.task) {
                distances.push(distance as f64);
                rts.push(response.response_time);
            }
        }
        
        let correlation = pearson_correlation(&distances, &rts);
        let regression = linear_regression(&distances, &rts);
        
        DistanceEffect {
            correlation,
            slope: regression.slope,
            intercept: regression.intercept,
            r_squared: regression.r_squared,
            strategy: self.infer_strategy(correlation),
        }
    }
    
    fn infer_strategy(&self, correlation: f64) -> StrategyType {
        if correlation > 0.7 {
            StrategyType::SerialScan
        } else if correlation < 0.3 {
            StrategyType::DirectAccess
        } else {
            StrategyType::Hybrid
        }
    }
}
```

## Hierarchical RT Models

Model individual differences within population:

```rust
pub struct HierarchicalRTModel {
    // Population parameters
    pub mu_pop: Normal,      // Population mean RT
    pub sigma_pop: Gamma,    // Population RT variance
    pub tau_pop: Gamma,      // Population tau parameter
    
    // Individual parameters
    pub individual_params: HashMap<String, ExGaussianParameters>,
}

impl HierarchicalRTModel {
    pub fn fit_hierarchical(data: &[(String, Vec<f64>)]) -> Self {
        // Step 1: Get initial individual estimates
        let mut individual_estimates = HashMap::new();
        for (id, rts) in data {
            let model = ExGaussianModel::fit(rts);
            individual_estimates.insert(id.clone(), model.params);
        }
        
        // Step 2: Estimate population parameters
        let mus: Vec<f64> = individual_estimates.values()
            .map(|p| p.mu).collect();
        let mu_pop = Normal::new(mean(&mus), std_dev(&mus)).unwrap();
        
        let sigmas: Vec<f64> = individual_estimates.values()
            .map(|p| p.sigma).collect();
        let sigma_pop = fit_gamma(&sigmas);
        
        let taus: Vec<f64> = individual_estimates.values()
            .map(|p| p.tau).collect();
        let tau_pop = fit_gamma(&taus);
        
        // Step 3: Shrinkage estimation
        let individual_params = self.shrinkage_estimation(
            &individual_estimates,
            &mu_pop,
            &sigma_pop,
            &tau_pop
        );
        
        HierarchicalRTModel {
            mu_pop,
            sigma_pop,
            tau_pop,
            individual_params,
        }
    }
    
    fn shrinkage_estimation(
        &self,
        raw: &HashMap<String, ExGaussianParameters>,
        mu_pop: &Normal,
        sigma_pop: &Gamma,
        tau_pop: &Gamma,
    ) -> HashMap<String, ExGaussianParameters> {
        let mut shrunk = HashMap::new();
        
        for (id, params) in raw {
            // Shrink toward population mean
            let n = self.get_sample_size(id);
            let weight = n as f64 / (n as f64 + 10.0); // Shrinkage weight
            
            shrunk.insert(id.clone(), ExGaussianParameters {
                mu: weight * params.mu + (1.0 - weight) * mu_pop.mean().unwrap(),
                sigma: weight * params.sigma + (1.0 - weight) * sigma_pop.mean().unwrap(),
                tau: weight * params.tau + (1.0 - weight) * tau_pop.mean().unwrap(),
            });
        }
        
        shrunk
    }
}
```

## Speed-Accuracy Tradeoff

Model the relationship between speed and accuracy:

```rust
pub struct SpeedAccuracyModel {
    pub threshold: f64,      // Decision threshold
    pub drift_rate: f64,     // Evidence accumulation rate
    pub non_decision: f64,   // Non-decision time
}

impl SpeedAccuracyModel {
    pub fn fit_sat(responses: &[ResponseData]) -> Self {
        // Group by speed emphasis
        let fast = responses.iter().filter(|r| r.response_time < 1000.0);
        let slow = responses.iter().filter(|r| r.response_time >= 1000.0);
        
        let acc_fast = fast.clone().filter(|r| r.correct).count() as f64 
                     / fast.count() as f64;
        let acc_slow = slow.clone().filter(|r| r.correct).count() as f64 
                     / slow.count() as f64;
        
        // Fit diffusion model parameters
        let threshold = self.estimate_threshold(acc_slow - acc_fast);
        let drift_rate = self.estimate_drift(acc_slow, mean_rt_slow);
        let non_decision = self.estimate_non_decision(responses);
        
        SpeedAccuracyModel {
            threshold,
            drift_rate,
            non_decision,
        }
    }
    
    pub fn predict(&self, emphasis: SpeedEmphasis) -> (f64, f64) {
        let threshold = match emphasis {
            SpeedEmphasis::Speed => self.threshold * 0.7,
            SpeedEmphasis::Balanced => self.threshold,
            SpeedEmphasis::Accuracy => self.threshold * 1.3,
        };
        
        // Predict RT and accuracy
        let rt = self.non_decision + threshold / self.drift_rate;
        let accuracy = 1.0 / (1.0 + (-self.drift_rate * threshold).exp());
        
        (rt, accuracy)
    }
}
```

## Learning Effects on RT

Track how response times change with learning:

```rust
pub struct RTLearningCurve {
    pub initial_rt: f64,
    pub asymptotic_rt: f64,
    pub learning_rate: f64,
}

impl RTLearningCurve {
    pub fn fit(trial_data: &[(usize, f64)]) -> Self {
        // Power law of practice: RT(n) = a * n^(-b) + c
        
        let log_data: Vec<(f64, f64)> = trial_data.iter()
            .map(|(n, rt)| ((*n as f64).ln(), rt.ln()))
            .collect();
        
        let regression = linear_regression(
            &log_data.iter().map(|p| p.0).collect::<Vec<_>>(),
            &log_data.iter().map(|p| p.1).collect::<Vec<_>>()
        );
        
        RTLearningCurve {
            initial_rt: regression.intercept.exp(),
            learning_rate: -regression.slope,
            asymptotic_rt: self.estimate_asymptote(trial_data),
        }
    }
    
    pub fn predict(&self, trial: usize) -> f64 {
        let a = self.initial_rt - self.asymptotic_rt;
        let b = self.learning_rate;
        let c = self.asymptotic_rt;
        
        a * (trial as f64).powf(-b) + c
    }
}
```

## Mixture Models

Handle multiple strategies within individuals:

```rust
pub struct MixtureRTModel {
    pub components: Vec<ExGaussianModel>,
    pub weights: Vec<f64>,
}

impl MixtureRTModel {
    pub fn fit_em(data: &[f64], k: usize) -> Self {
        // Initialize with k-means
        let initial_clusters = kmeans_1d(data, k);
        let mut components = Vec::new();
        let mut weights = vec![1.0 / k as f64; k];
        
        for cluster in initial_clusters {
            components.push(ExGaussianModel::fit(&cluster));
        }
        
        // EM algorithm
        for _ in 0..100 {
            // E-step: compute responsibilities
            let responsibilities = self.compute_responsibilities(data, &components, &weights);
            
            // M-step: update parameters
            for (i, component) in components.iter_mut().enumerate() {
                let weighted_data = self.weight_data(data, &responsibilities[i]);
                *component = ExGaussianModel::fit(&weighted_data);
                weights[i] = responsibilities[i].iter().sum::<f64>() / data.len() as f64;
            }
            
            // Check convergence
            if self.converged() {
                break;
            }
        }
        
        MixtureRTModel { components, weights }
    }
    
    pub fn classify(&self, rt: f64) -> usize {
        // Assign to most likely component
        self.components.iter()
            .enumerate()
            .map(|(i, comp)| (i, self.weights[i] * comp.pdf(rt)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap()
    }
}
```

## Real-Time Adaptation

Update RT models online:

```rust
impl ExGaussianModel {
    pub fn update_online(&mut self, new_rt: f64, learning_rate: f64) {
        // Stochastic gradient update
        let gradient = self.compute_gradient(new_rt);
        
        self.params.mu -= learning_rate * gradient.mu;
        self.params.sigma -= learning_rate * gradient.sigma;
        self.params.tau -= learning_rate * gradient.tau;
        
        // Ensure valid parameters
        self.params.sigma = self.params.sigma.max(0.01);
        self.params.tau = self.params.tau.max(0.01);
    }
    
    pub fn detect_outlier(&self, rt: f64) -> bool {
        let log_prob = self.log_pdf(rt);
        log_prob < -10.0 // Very unlikely under current model
    }
}
```

## Summary

Response time modeling provides:
- **Ex-Gaussian fitting**: Captures RT distribution shape
- **Strategy detection**: Infers cognitive strategies from temporal patterns
- **Hierarchical models**: Account for individual differences
- **Learning curves**: Track improvement over time
- **Mixture models**: Handle strategy switching
- **Online adaptation**: Real-time model updates

These models transform raw timing data into deep insights about cognitive processing.