# Strategy Detection

Cognitive strategies reveal how learners mentally represent and navigate knowledge structures. The system detects strategies through statistical analysis of response patterns, particularly the relationship between response time and cognitive distance.

## Core Strategy Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StrategyType {
    SerialScan,      // Sequential mental search
    DirectAccess,    // Direct retrieval from memory
    Hybrid,          // Mixed strategy
    ChunkBased,      // Hierarchical chunk navigation
    Associative,     // Association-based retrieval
}
```

## Serial Scan Strategy

Learners mentally "scan" through the sequence:

```rust
pub struct SerialScanDetector {
    pub base_time: f64,      // Time to initiate scan
    pub scan_rate: f64,      // Time per item scanned
    pub correlation_threshold: f64,
}

impl SerialScanDetector {
    pub fn detect(&self, responses: &[ResponseData]) -> DetectionResult {
        // Extract RT and distance for each response
        let mut rts = Vec::new();
        let mut distances = Vec::new();
        
        for response in responses {
            if let Some(distance) = self.compute_distance(&response.task) {
                rts.push(response.response_time);
                distances.push(distance as f64);
            }
        }
        
        // Compute correlation
        let correlation = pearson_correlation(&distances, &rts);
        
        // Fit linear model: RT = base_time + scan_rate * distance
        let regression = linear_regression(&distances, &rts);
        
        DetectionResult {
            strategy: if correlation > self.correlation_threshold {
                StrategyType::SerialScan
            } else {
                StrategyType::Unknown
            },
            confidence: correlation.abs(),
            parameters: SerialScanParams {
                base_time: regression.intercept,
                scan_rate: regression.slope,
            },
        }
    }
    
    fn compute_distance(&self, task: &Task) -> Option<usize> {
        match &task.task_type {
            TaskType::Successor { item } => Some(1),
            TaskType::KJump { start, k } => Some(k.abs() as usize),
            TaskType::PairwiseOrder { a, b } => {
                self.topology.calculate_distance(a, b)
            }
            _ => None,
        }
    }
}
```

### Mathematical Foundation

For serial scan, RT follows:

```
RT(d) = β₀ + β₁ × d + ε
```

Where:
- d = cognitive distance
- β₀ = base processing time
- β₁ = scan rate
- ε ~ N(0, σ²) = noise

## Direct Access Strategy

Items retrieved directly without sequential search:

```rust
pub struct DirectAccessDetector {
    pub mean_rt: f64,
    pub variance: f64,
    pub independence_threshold: f64,
}

impl DirectAccessDetector {
    pub fn detect(&self, responses: &[ResponseData]) -> DetectionResult {
        let (rts, distances) = self.extract_data(responses);
        
        // Test independence of RT from distance
        let correlation = pearson_correlation(&distances, &rts);
        let chi_squared = self.independence_test(&rts, &distances);
        
        // Check if RT distribution is consistent
        let cv = coefficient_of_variation(&rts); // σ/μ
        
        DetectionResult {
            strategy: if correlation.abs() < 0.3 && cv < 0.5 {
                StrategyType::DirectAccess
            } else {
                StrategyType::Unknown
            },
            confidence: 1.0 - correlation.abs(),
            parameters: DirectAccessParams {
                mean_rt: mean(&rts),
                sd_rt: std_dev(&rts),
            },
        }
    }
    
    fn independence_test(&self, rts: &[f64], distances: &[f64]) -> f64 {
        // Chi-squared test for independence
        let contingency_table = self.build_contingency_table(rts, distances);
        chi_squared_statistic(&contingency_table)
    }
}
```

## Hybrid Strategy Detection

Many learners switch between strategies:

```rust
pub struct HybridStrategyDetector {
    serial_detector: SerialScanDetector,
    direct_detector: DirectAccessDetector,
    mixture_model: MixtureModel,
}

impl HybridStrategyDetector {
    pub fn detect(&self, responses: &[ResponseData]) -> DetectionResult {
        // Fit mixture model
        let components = self.fit_mixture(responses);
        
        // Classify each response
        let classifications = responses.iter()
            .map(|r| self.classify_response(r, &components))
            .collect::<Vec<_>>();
        
        // Compute proportions
        let serial_prop = classifications.iter()
            .filter(|c| **c == StrategyType::SerialScan)
            .count() as f64 / classifications.len() as f64;
        
        DetectionResult {
            strategy: StrategyType::Hybrid,
            confidence: self.mixture_model.bic_score(),
            parameters: HybridParams {
                serial_weight: serial_prop,
                serial_params: components[0].clone(),
                direct_params: components[1].clone(),
            },
        }
    }
    
    fn fit_mixture(&self, responses: &[ResponseData]) -> Vec<StrategyParams> {
        // EM algorithm for mixture of strategies
        let mut params = vec![
            self.initialize_serial_params(responses),
            self.initialize_direct_params(responses),
        ];
        
        for _ in 0..100 {
            // E-step: compute responsibilities
            let responsibilities = self.compute_responsibilities(responses, &params);
            
            // M-step: update parameters
            params = self.update_parameters(responses, &responsibilities);
            
            if self.converged(&params) {
                break;
            }
        }
        
        params
    }
}
```

## Chunk-Based Strategy

Learners organize sequence into chunks:

```rust
pub struct ChunkStrategyDetector {
    pub chunk_boundaries: Vec<usize>,
    pub within_chunk_rate: f64,
    pub between_chunk_penalty: f64,
}

impl ChunkStrategyDetector {
    pub fn detect(&self, responses: &[ResponseData]) -> DetectionResult {
        // Detect chunk boundaries from RT patterns
        let boundaries = self.detect_boundaries(responses);
        
        // Separate within vs between chunk transitions
        let (within, between) = self.separate_transitions(responses, &boundaries);
        
        // Compare RT distributions
        let t_statistic = two_sample_t_test(&within, &between);
        
        DetectionResult {
            strategy: if t_statistic > 2.0 {
                StrategyType::ChunkBased
            } else {
                StrategyType::Unknown
            },
            confidence: t_statistic / 10.0, // Normalized
            parameters: ChunkParams {
                boundaries,
                within_rate: mean(&within),
                between_penalty: mean(&between) - mean(&within),
            },
        }
    }
    
    fn detect_boundaries(&self, responses: &[ResponseData]) -> Vec<usize> {
        // Use RT peaks to identify chunk boundaries
        let mut boundaries = Vec::new();
        let window = 3;
        
        for i in window..responses.len()-window {
            let local_rts = &responses[i-window..i+window];
            let center_rt = responses[i].response_time;
            
            if self.is_local_maximum(center_rt, local_rts) {
                boundaries.push(i);
            }
        }
        
        boundaries
    }
}
```

## Strategy Transitions

Track how strategies evolve:

```rust
pub struct StrategyTransitionAnalyzer {
    pub window_size: usize,
    pub transition_matrix: Matrix<f64>,
}

impl StrategyTransitionAnalyzer {
    pub fn analyze(&mut self, responses: &[ResponseData]) -> TransitionAnalysis {
        let mut strategies = Vec::new();
        
        // Detect strategy in sliding windows
        for window in responses.windows(self.window_size) {
            let strategy = self.detect_strategy(window);
            strategies.push(strategy);
        }
        
        // Build transition matrix
        for i in 0..strategies.len()-1 {
            let from = &strategies[i];
            let to = &strategies[i+1];
            self.update_transition(from, to);
        }
        
        // Analyze transitions
        TransitionAnalysis {
            stable_strategy: self.find_stable_strategy(),
            transition_rate: self.compute_transition_rate(),
            learning_trajectory: strategies,
        }
    }
    
    fn find_stable_strategy(&self) -> StrategyType {
        // Find eigenvector for eigenvalue 1 (stationary distribution)
        let eigenvector = self.transition_matrix.stationary_distribution();
        self.strategy_from_distribution(eigenvector)
    }
}
```

## Multi-Level Strategy Detection

Strategies can operate at multiple levels:

```rust
pub struct MultiLevelStrategyDetector {
    pub micro_level: MicroStrategyDetector,   // Individual items
    pub meso_level: MesoStrategyDetector,     // Chunks
    pub macro_level: MacroStrategyDetector,   // Overall structure
}

impl MultiLevelStrategyDetector {
    pub fn detect(&self, responses: &[ResponseData]) -> MultiLevelResult {
        // Detect strategies at each level
        let micro = self.micro_level.detect(responses);
        let meso = self.meso_level.detect(responses);
        let macro = self.macro_level.detect(responses);
        
        // Integrate across levels
        MultiLevelResult {
            dominant_level: self.find_dominant_level(&micro, &meso, &macro),
            micro_strategy: micro.strategy,
            meso_strategy: meso.strategy,
            macro_strategy: macro.strategy,
            coherence: self.compute_coherence(&micro, &meso, &macro),
        }
    }
    
    fn compute_coherence(&self, micro: &DetectionResult, 
                        meso: &DetectionResult, 
                        macro: &DetectionResult) -> f64 {
        // Measure agreement across levels
        let agreements = [
            self.agreement(micro, meso),
            self.agreement(meso, macro),
            self.agreement(micro, macro),
        ];
        
        agreements.iter().sum::<f64>() / 3.0
    }
}
```

## Bayesian Strategy Classification

Use Bayesian inference for robust detection:

```rust
pub struct BayesianStrategyClassifier {
    pub prior: HashMap<StrategyType, f64>,
    pub likelihoods: HashMap<StrategyType, Box<dyn Likelihood>>,
}

impl BayesianStrategyClassifier {
    pub fn classify(&self, responses: &[ResponseData]) -> Classification {
        let mut posteriors = HashMap::new();
        
        for (strategy, prior) in &self.prior {
            let likelihood = self.likelihoods[strategy].compute(responses);
            posteriors.insert(strategy.clone(), prior * likelihood);
        }
        
        // Normalize
        let total: f64 = posteriors.values().sum();
        for value in posteriors.values_mut() {
            *value /= total;
        }
        
        // Find MAP estimate
        let map = posteriors.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(s, _)| s.clone())
            .unwrap();
        
        Classification {
            strategy: map,
            posterior_probabilities: posteriors,
            bayes_factor: self.compute_bayes_factor(&posteriors),
        }
    }
}
```

## Real-Time Strategy Monitoring

```rust
pub struct OnlineStrategyMonitor {
    pub detector: StrategyDetector,
    pub buffer: CircularBuffer<ResponseData>,
    pub current_strategy: StrategyType,
}

impl OnlineStrategyMonitor {
    pub fn update(&mut self, response: ResponseData) -> Option<StrategyChange> {
        self.buffer.push(response);
        
        if self.buffer.len() >= self.min_window {
            let new_strategy = self.detector.detect(&self.buffer.to_vec());
            
            if new_strategy != self.current_strategy {
                let change = StrategyChange {
                    from: self.current_strategy.clone(),
                    to: new_strategy.clone(),
                    confidence: self.compute_change_confidence(),
                    timestamp: Utc::now(),
                };
                
                self.current_strategy = new_strategy;
                return Some(change);
            }
        }
        
        None
    }
}
```

## Validation Methods

Ensure strategy detection is reliable:

```rust
impl StrategyDetector {
    pub fn cross_validate(&self, responses: &[ResponseData], k: usize) -> f64 {
        let fold_size = responses.len() / k;
        let mut accuracies = Vec::new();
        
        for i in 0..k {
            let test_start = i * fold_size;
            let test_end = (i + 1) * fold_size;
            
            let train = [&responses[..test_start], &responses[test_end..]].concat();
            let test = &responses[test_start..test_end];
            
            let strategy = self.detect(&train);
            let accuracy = self.evaluate_prediction(strategy, test);
            accuracies.push(accuracy);
        }
        
        mean(&accuracies)
    }
    
    pub fn bootstrap_confidence(&self, responses: &[ResponseData], n: usize) -> (f64, f64) {
        let mut detections = Vec::new();
        
        for _ in 0..n {
            let sample = self.bootstrap_sample(responses);
            let result = self.detect(&sample);
            detections.push(result.confidence);
        }
        
        let ci_lower = percentile(&detections, 0.025);
        let ci_upper = percentile(&detections, 0.975);
        
        (ci_lower, ci_upper)
    }
}
```

## Summary

Strategy detection provides:
- **Multiple strategy types**: Serial, direct, hybrid, chunk-based
- **Statistical robustness**: Correlation, regression, mixture models
- **Multi-level analysis**: Micro, meso, macro strategies
- **Bayesian classification**: Principled uncertainty quantification
- **Online monitoring**: Real-time strategy tracking
- **Validation methods**: Cross-validation, bootstrapping

Understanding cognitive strategies enables personalized, effective learning experiences.