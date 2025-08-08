# Performance Prediction

Performance prediction enables proactive adaptation by forecasting future learning outcomes. The system uses multiple models to predict accuracy, response time, and learning trajectories.

## Prediction Framework

```rust
pub struct PerformancePredictor {
    pub short_term_model: ShortTermPredictor,
    pub long_term_model: LongTermPredictor,
    pub ensemble: EnsemblePredictor,
}

impl PerformancePredictor {
    pub fn predict(&self, learner: &LearnerModel, task: &Task, horizon: TimeHorizon) -> Prediction {
        match horizon {
            TimeHorizon::Immediate => self.short_term_model.predict(learner, task),
            TimeHorizon::Session => self.ensemble.predict(learner, task),
            TimeHorizon::LongTerm => self.long_term_model.predict(learner, task),
        }
    }
}
```

## Item Response Theory (IRT) Models

### 3-Parameter Logistic Model

```rust
pub struct IRTModel {
    pub item_parameters: HashMap<String, ItemParams>,
    pub ability_estimates: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct ItemParams {
    pub difficulty: f64,    // b parameter
    pub discrimination: f64, // a parameter  
    pub guessing: f64,      // c parameter
}

impl IRTModel {
    pub fn predict_probability(&self, ability: f64, item: &ItemParams) -> f64 {
        let z = item.discrimination * (ability - item.difficulty);
        item.guessing + (1.0 - item.guessing) / (1.0 + (-z).exp())
    }
    
    pub fn predict_response_time(&self, ability: f64, item: &ItemParams) -> f64 {
        // Log-normal response time model
        let speed = ability - item.difficulty;
        let mean_log_rt = 7.0 - 0.5 * speed;
        let sd_log_rt = 0.3;
        
        sample_lognormal(mean_log_rt, sd_log_rt)
    }
}
```

## Learning Curve Prediction

### Power Law Model

```rust
pub struct LearningCurvePredictor {
    pub model_type: CurveModel,
    pub parameters: CurveParameters,
}

#[derive(Debug, Clone)]
pub enum CurveModel {
    PowerLaw { a: f64, b: f64, c: f64 },        // P(n) = a * n^(-b) + c
    Exponential { a: f64, b: f64, c: f64 },     // P(n) = a * (1 - exp(-b*n)) + c
    Hyperbolic { a: f64, b: f64, c: f64 },      // P(n) = a / (1 + b*n) + c
    Logistic { L: f64, k: f64, x0: f64 },       // P(n) = L / (1 + exp(-k*(n-x0)))
}

impl LearningCurvePredictor {
    pub fn fit(&mut self, data: &[(usize, f64)]) {
        match self.model_type {
            CurveModel::PowerLaw { .. } => self.fit_power_law(data),
            CurveModel::Exponential { .. } => self.fit_exponential(data),
            CurveModel::Hyperbolic { .. } => self.fit_hyperbolic(data),
            CurveModel::Logistic { .. } => self.fit_logistic(data),
        }
    }
    
    fn fit_power_law(&mut self, data: &[(usize, f64)]) {
        // Log-transform for linear regression
        let log_data: Vec<(f64, f64)> = data.iter()
            .filter(|(n, _)| *n > 0)
            .map(|(n, p)| ((*n as f64).ln(), p.ln()))
            .collect();
        
        let regression = linear_regression(
            &log_data.iter().map(|d| d.0).collect::<Vec<_>>(),
            &log_data.iter().map(|d| d.1).collect::<Vec<_>>()
        );
        
        if let CurveModel::PowerLaw { ref mut a, ref mut b, ref mut c } = self.model_type {
            *a = regression.intercept.exp();
            *b = -regression.slope;
            *c = self.estimate_asymptote(data);
        }
    }
    
    pub fn predict(&self, n: usize) -> f64 {
        match self.model_type {
            CurveModel::PowerLaw { a, b, c } => {
                a * (n as f64).powf(-b) + c
            }
            CurveModel::Exponential { a, b, c } => {
                a * (1.0 - (-b * n as f64).exp()) + c
            }
            CurveModel::Hyperbolic { a, b, c } => {
                a / (1.0 + b * n as f64) + c
            }
            CurveModel::Logistic { L, k, x0 } => {
                L / (1.0 + (-k * (n as f64 - x0)).exp())
            }
        }
    }
}
```

## Forgetting Curve Prediction

```rust
pub struct ForgettingPredictor {
    pub retention_model: RetentionModel,
    pub individual_params: HashMap<String, ForgettingParams>,
}

impl ForgettingPredictor {
    pub fn predict_retention(&self, learner_id: &str, item: &str, time: Duration) -> f64 {
        let params = &self.individual_params[learner_id];
        let hours = time.num_hours() as f64;
        
        match self.retention_model {
            RetentionModel::Ebbinghaus => {
                // R(t) = exp(-t/S)
                (-hours / params.stability).exp()
            }
            RetentionModel::PowerForgetting => {
                // R(t) = (1 + t)^(-decay)
                (1.0 + hours).powf(-params.decay_rate)
            }
            RetentionModel::ACTRModel => {
                // R(t) = sum(t_i^-d) for all practice times
                self.compute_actr_activation(item, hours, params)
            }
        }
    }
    
    fn compute_actr_activation(&self, item: &str, current_time: f64, params: &ForgettingParams) -> f64 {
        let practice_times = self.get_practice_times(item);
        let decay = params.decay_rate;
        
        practice_times.iter()
            .map(|t| (current_time - t).powf(-decay))
            .sum()
    }
    
    pub fn optimal_review_time(&self, learner_id: &str, item: &str, target_retention: f64) -> Duration {
        let params = &self.individual_params[learner_id];
        
        // Solve for t when R(t) = target_retention
        let hours = match self.retention_model {
            RetentionModel::Ebbinghaus => {
                -params.stability * target_retention.ln()
            }
            RetentionModel::PowerForgetting => {
                (target_retention.powf(-1.0 / params.decay_rate) - 1.0)
            }
            _ => self.numerical_solve_review_time(learner_id, item, target_retention)
        };
        
        Duration::hours(hours as i64)
    }
}
```

## Schedule Optimization

```rust
pub struct ScheduleOptimizer {
    pub objective: OptimizationObjective,
    pub constraints: ScheduleConstraints,
}

impl ScheduleOptimizer {
    pub fn optimize_schedule(&self, learner: &LearnerModel, items: &[Item], duration: Duration) -> Schedule {
        // Dynamic programming approach
        let n = items.len();
        let time_slots = (duration.num_minutes() / 5) as usize; // 5-minute slots
        
        // dp[i][t] = max performance using items 0..i in time t
        let mut dp = vec![vec![0.0; time_slots + 1]; n + 1];
        let mut choices = vec![vec![Decision::Skip; time_slots + 1]; n + 1];
        
        for i in 1..=n {
            for t in 0..=time_slots {
                // Option 1: Skip item i
                dp[i][t] = dp[i-1][t];
                choices[i][t] = Decision::Skip;
                
                // Option 2: Practice item i
                let practice_time = self.estimate_practice_time(&items[i-1]);
                if t >= practice_time {
                    let value = self.compute_value(&items[i-1], learner) + dp[i-1][t - practice_time];
                    if value > dp[i][t] {
                        dp[i][t] = value;
                        choices[i][t] = Decision::Practice(practice_time);
                    }
                }
            }
        }
        
        // Reconstruct optimal schedule
        self.reconstruct_schedule(&choices, items)
    }
    
    fn compute_value(&self, item: &Item, learner: &LearnerModel) -> f64 {
        match self.objective {
            OptimizationObjective::MaximizeLearning => {
                let current = learner.get_mastery(item);
                let predicted = self.predict_after_practice(item, learner);
                predicted - current
            }
            OptimizationObjective::MinimizeForgetting => {
                let retention = learner.get_retention(item);
                1.0 - retention // Higher value for items about to be forgotten
            }
            OptimizationObjective::BalancedMastery => {
                // Prefer items at medium difficulty
                let mastery = learner.get_mastery(item);
                4.0 * mastery * (1.0 - mastery) // Quadratic with peak at 0.5
            }
        }
    }
}
```

## Ensemble Methods

Combine multiple predictors:

```rust
pub struct EnsemblePredictor {
    pub models: Vec<Box<dyn Predictor>>,
    pub weights: Vec<f64>,
    pub aggregation: AggregationMethod,
}

impl EnsemblePredictor {
    pub fn predict(&self, learner: &LearnerModel, task: &Task) -> Prediction {
        let predictions: Vec<Prediction> = self.models.iter()
            .map(|model| model.predict(learner, task))
            .collect();
        
        match self.aggregation {
            AggregationMethod::WeightedAverage => {
                self.weighted_average(&predictions)
            }
            AggregationMethod::Stacking => {
                self.stacking_prediction(&predictions)
            }
            AggregationMethod::BayesianModelAverage => {
                self.bayesian_average(&predictions)
            }
        }
    }
    
    fn bayesian_average(&self, predictions: &[Prediction]) -> Prediction {
        // Compute posterior model probabilities
        let log_evidences: Vec<f64> = self.models.iter()
            .enumerate()
            .map(|(i, model)| model.log_evidence())
            .collect();
        
        let log_sum = log_sum_exp(&log_evidences);
        let posteriors: Vec<f64> = log_evidences.iter()
            .map(|le| (le - log_sum).exp())
            .collect();
        
        // Weighted combination
        let mut combined = Prediction::default();
        for (pred, weight) in predictions.iter().zip(posteriors.iter()) {
            combined.accuracy += weight * pred.accuracy;
            combined.response_time += weight * pred.response_time;
            combined.confidence += weight * pred.confidence;
        }
        
        combined
    }
}
```

## Confidence Intervals

Quantify prediction uncertainty:

```rust
impl PerformancePredictor {
    pub fn predict_with_confidence(&self, learner: &LearnerModel, task: &Task) -> PredictionWithCI {
        // Bootstrap confidence intervals
        let n_bootstrap = 1000;
        let mut predictions = Vec::new();
        
        for _ in 0..n_bootstrap {
            let sampled_learner = self.bootstrap_learner_model(learner);
            let pred = self.predict(&sampled_learner, task, TimeHorizon::Immediate);
            predictions.push(pred.accuracy);
        }
        
        predictions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        PredictionWithCI {
            point_estimate: median(&predictions),
            ci_lower: percentile(&predictions, 0.025),
            ci_upper: percentile(&predictions, 0.975),
            prediction_interval: self.compute_prediction_interval(&predictions),
        }
    }
    
    fn compute_prediction_interval(&self, predictions: &[f64]) -> (f64, f64) {
        // Account for both parameter uncertainty and inherent variability
        let param_uncertainty = std_dev(predictions);
        let inherent_variability = 0.1; // Domain-specific
        
        let total_sd = (param_uncertainty.powi(2) + inherent_variability.powi(2)).sqrt();
        let mean = mean(predictions);
        
        (mean - 1.96 * total_sd, mean + 1.96 * total_sd)
    }
}
```

## Real-Time Adaptation

Update predictions online:

```rust
pub struct OnlinePredictor {
    pub base_model: Box<dyn Predictor>,
    pub adaptation_rate: f64,
    pub error_buffer: CircularBuffer<f64>,
}

impl OnlinePredictor {
    pub fn predict_and_update(&mut self, learner: &LearnerModel, task: &Task, actual: f64) -> f64 {
        // Make prediction
        let prediction = self.base_model.predict(learner, task);
        
        // Observe error
        let error = actual - prediction.accuracy;
        self.error_buffer.push(error);
        
        // Adapt model
        if self.should_adapt() {
            self.adapt_model(error);
        }
        
        // Bias-corrected prediction
        let bias = self.error_buffer.mean();
        prediction.accuracy + self.adaptation_rate * bias
    }
    
    fn adapt_model(&mut self, error: f64) {
        // Gradient-based update
        let gradient = self.base_model.compute_gradient(error);
        self.base_model.update_parameters(gradient, self.adaptation_rate);
        
        // Adjust adaptation rate
        if self.error_buffer.variance() < 0.01 {
            self.adaptation_rate *= 0.95; // Reduce if stable
        } else {
            self.adaptation_rate *= 1.05; // Increase if volatile
        }
    }
}
```

## Multi-Step Prediction

Predict entire learning trajectories:

```rust
pub struct TrajectoryPredictor {
    pub state_transition: StateTransitionModel,
    pub observation_model: ObservationModel,
}

impl TrajectoryPredictor {
    pub fn predict_trajectory(&self, initial: &LearnerState, tasks: &[Task], horizon: usize) -> Trajectory {
        let mut states = vec![initial.clone()];
        let mut predictions = Vec::new();
        
        for t in 0..horizon {
            let current_state = &states[t];
            let task = &tasks[t % tasks.len()];
            
            // Predict performance at current state
            let performance = self.observation_model.predict(current_state, task);
            predictions.push(performance);
            
            // Predict next state
            let next_state = self.state_transition.predict(current_state, task, performance);
            states.push(next_state);
        }
        
        Trajectory {
            states,
            predictions,
            confidence_bands: self.compute_confidence_bands(&states),
        }
    }
    
    fn compute_confidence_bands(&self, states: &[LearnerState]) -> Vec<(f64, f64)> {
        states.iter()
            .enumerate()
            .map(|(t, state)| {
                let uncertainty = self.state_transition.uncertainty_at_time(t);
                let mean = state.expected_performance();
                (mean - 2.0 * uncertainty, mean + 2.0 * uncertainty)
            })
            .collect()
    }
}
```

## Summary

Performance prediction enables:
- **IRT models**: Principled probability estimates
- **Learning curves**: Power law and exponential fits
- **Forgetting curves**: Retention and optimal review timing
- **Schedule optimization**: Dynamic programming for task selection
- **Ensemble methods**: Combine multiple predictors
- **Confidence intervals**: Quantify uncertainty
- **Online adaptation**: Real-time model updates
- **Trajectory prediction**: Multi-step forecasting

These predictive capabilities enable proactive, personalized learning experiences.