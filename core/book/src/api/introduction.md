# API Reference Introduction

This section provides comprehensive API documentation for ABCDeez Core. Whether you're building educational applications, conducting research, or integrating cognitive assessment into your system, this reference will guide you through all available functionality.

## Quick Start Example

```rust
use abcdeez_core::prelude::*;

fn main() -> Result<(), Error> {
    // 1. Create a knowledge structure
    let topology = Topology::alphabet();
    
    // 2. Initialize a learner model
    let mut learner = LearnerModel::new("student_001".to_string(), &topology);
    
    // 3. Create Bayesian assessment model
    let mut bayesian = BayesianLearnerModel::new(&topology);
    
    // 4. Generate an adaptive task
    let task = bayesian.select_next_task(TaskType::Successor);
    
    // 5. Collect and process response
    let response = ResponseData {
        task: task.clone(),
        correct: true,
        response_time: 1250.0,
    };
    
    // 6. Update models
    learner.update_with_response(&response);
    bayesian.update_with_response(response);
    
    // 7. Get insights
    let metrics = LearnerMetrics::from_model(&learner);
    println!("Learning progress: {:?}", metrics);
    
    Ok(())
}
```

## Module Organization

ABCDeez Core is organized into four main modules:

### Core Module
Foundation types and configuration
```rust
use abcdeez_core::core::{
    config::{LearnerConfig, AdaptiveSchedulingConfig},
    topology::{Topology, Node, Edge, TopologyType},
    error::{Error, ErrorKind},
};
```

### Learning Module
Cognitive models and learning algorithms
```rust
use abcdeez_core::learning::{
    learner::{LearnerModel, OperationType, MemoryStrength},
    bayesian::{BayesianLearnerModel, PosteriorUpdate},
    scheduler::{AdaptiveScheduler, SchedulingStrategy},
};
```

### Statistics Module
Statistical analysis and validation
```rust
use abcdeez_core::statistics::{
    core::{ExGaussianDistribution, DetailedStatistics},
    analysis::{StrategyClassifier, ResponsePattern},
    validation::{CrossValidator, BootstrapCI},
    power_analysis::{PowerAnalyzer, EffectSize},
};
```

### Tasks Module
Assessment generation and selection
```rust
use abcdeez_core::tasks::{
    Task, TaskType, TaskGenerator,
    selection::{SelectionStrategy, InformationGain},
    difficulty::{DifficultyCalibrator, ItemParameters},
};
```

## Common Patterns

### Pattern 1: Basic Learning Session

```rust
pub fn run_learning_session(
    learner: &mut LearnerModel,
    topology: &Topology,
    n_trials: usize,
) -> Vec<ResponseData> {
    let mut responses = Vec::new();
    let mut bayesian = BayesianLearnerModel::new(topology);
    
    for _ in 0..n_trials {
        // Select optimal task
        let task = bayesian.select_next_task(TaskType::Adaptive);
        
        // Simulate or collect response
        let response = collect_user_response(&task);
        
        // Update models
        learner.update_with_response(&response);
        bayesian.update_with_response(response.clone());
        
        responses.push(response);
    }
    
    responses
}
```

### Pattern 2: Adaptive Assessment

```rust
pub fn adaptive_assessment(
    topology: &Topology,
    target_precision: f64,
) -> AssessmentResult {
    let mut model = BayesianLearnerModel::new(topology);
    let mut responses = Vec::new();
    
    while model.total_entropy() > target_precision {
        // Get most informative task
        let task = model.select_max_information_task();
        
        // Collect response
        let response = collect_response(&task);
        responses.push(response.clone());
        
        // Update posterior
        model.update_with_response(response);
        
        // Check stopping criteria
        if responses.len() >= MAX_ITEMS {
            break;
        }
    }
    
    AssessmentResult {
        ability_estimate: model.get_ability_estimate(),
        uncertainty: model.get_uncertainty(),
        responses,
    }
}
```

### Pattern 3: Performance Analysis

```rust
pub fn analyze_performance(
    responses: &[ResponseData],
) -> PerformanceAnalysis {
    // Fit response time distribution
    let rt_dist = ExGaussianDistribution::fit_mle(
        &responses.iter().map(|r| r.response_time).collect::<Vec<_>>()
    );
    
    // Classify strategy
    let strategy = StrategyClassifier::classify(responses);
    
    // Calculate detailed statistics
    let stats = DetailedStatistics::from_responses(responses);
    
    // Detect outliers
    let outliers = ResponseTimeDistribution::detect_outliers(
        &responses.iter().map(|r| r.response_time).collect::<Vec<_>>(),
        2.5
    );
    
    PerformanceAnalysis {
        response_time_model: rt_dist,
        strategy,
        statistics: stats,
        outlier_indices: outliers,
    }
}
```

## Error Handling

All fallible operations return `Result<T, Error>`:

```rust
use abcdeez_core::core::error::{Error, ErrorKind};

fn process_data(data: &[f64]) -> Result<Statistics, Error> {
    if data.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Cannot calculate statistics for empty data"
        ));
    }
    
    if data.iter().any(|x| !x.is_finite()) {
        return Err(Error::new(
            ErrorKind::NumericalInstability,
            "Data contains non-finite values"
        ));
    }
    
    Ok(calculate_statistics(data))
}
```

## Type Safety

The API leverages Rust's type system for safety:

```rust
// Operations are strongly typed
pub enum OperationType {
    Successor,
    Predecessor,
    PairwiseOrder,
    KJump(i32),
    Segment(usize, bool),
}

// Task types are exhaustive
pub enum TaskType {
    Successor { item: String },
    Predecessor { item: String },
    PairwiseOrder { item_a: String, item_b: String },
    // ...
}

// Strategies are explicit
pub enum Strategy {
    SerialScanning,
    DirectAccess,
    Hybrid,
}
```

## Builder Patterns

Complex objects use builder patterns:

```rust
let config = LearnerConfigBuilder::new()
    .initial_memory_strength(0.5)
    .memory_decay_rate(0.05)
    .learning_rate(0.2)
    .edge_emphasis(0.1)
    .validate()?
    .build();

let learner = LearnerModel::new_with_config(
    "student_001".to_string(),
    &topology,
    config
);
```

## Async Support

For integration with async systems:

```rust
use abcdeez_core::async_api::*;

async fn async_assessment(
    model: Arc<Mutex<BayesianLearnerModel>>,
    task_stream: impl Stream<Item = Task>,
) -> Result<Vec<ResponseData>, Error> {
    let mut responses = Vec::new();
    
    pin_mut!(task_stream);
    
    while let Some(task) = task_stream.next().await {
        let response = collect_response_async(&task).await?;
        
        let mut model = model.lock().await;
        model.update_with_response(response.clone());
        
        responses.push(response);
    }
    
    Ok(responses)
}
```

## Serialization

All major types support serialization:

```rust
use serde_json;

// Serialize learner state
let learner_json = serde_json::to_string(&learner)?;

// Deserialize for persistence
let learner: LearnerModel = serde_json::from_str(&learner_json)?;

// Export metrics
let metrics = LearnerMetrics::from_model(&learner);
let metrics_json = serde_json::to_string_pretty(&metrics)?;
```

## Performance Considerations

### Memory Management

```rust
// Efficient batch processing
impl LearnerModel {
    pub fn update_batch(&mut self, responses: &[ResponseData]) {
        // Pre-allocate space
        self.response_history.reserve(responses.len());
        
        // Process in single pass
        for response in responses {
            self.update_with_response_internal(response);
        }
        
        // Batch constraint updates
        self.apply_identifiability_constraints();
    }
}
```

### Parallel Processing

```rust
use rayon::prelude::*;

pub fn parallel_analysis(
    participants: Vec<ParticipantData>,
) -> Vec<AnalysisResult> {
    participants
        .par_iter()
        .map(|participant| {
            let mut model = LearnerModel::new(
                participant.id.clone(),
                &participant.topology
            );
            
            for response in &participant.responses {
                model.update_with_response(response);
            }
            
            AnalysisResult::from_model(&model)
        })
        .collect()
}
```

## Testing Utilities

The API includes testing utilities:

```rust
#[cfg(test)]
mod tests {
    use abcdeez_core::testing::*;
    
    #[test]
    fn test_learning_progression() {
        // Generate synthetic data
        let data = generate_synthetic_responses(100, 0.7);
        
        // Create test topology
        let topology = create_test_topology(26);
        
        // Run simulation
        let results = simulate_learning_session(
            &topology,
            &data,
            SimulationParams::default()
        );
        
        // Validate results
        assert_convergence(&results, 0.8, 50);
    }
}
```

## Logging and Debugging

Enable detailed logging:

```rust
use log::{debug, info, warn};

impl LearnerModel {
    pub fn update_with_logging(&mut self, response: &ResponseData) {
        info!("Processing response for task: {:?}", response.task);
        
        let initial_strength = self.get_memory_strength(&response.task.item);
        debug!("Initial memory strength: {}", initial_strength);
        
        self.update_with_response(response);
        
        let final_strength = self.get_memory_strength(&response.task.item);
        info!("Memory strength updated: {} -> {}", 
              initial_strength, final_strength);
        
        if final_strength < 0.3 {
            warn!("Low memory strength detected for item: {}", 
                  response.task.item);
        }
    }
}
```

## Version Compatibility

```rust
use abcdeez_core::version::*;

// Check compatibility
if !is_compatible_version(required_version) {
    panic!("Incompatible ABCDeez Core version");
}

// Migration support
let old_data = load_v1_data()?;
let migrated = migrate_to_current(old_data)?;
```

## Best Practices

1. **Always validate input data**
2. **Handle errors explicitly**
3. **Use appropriate precision for statistics**
4. **Consider memory usage for large datasets**
5. **Enable logging in development**
6. **Write tests using provided utilities**
7. **Profile performance for production use**

## Getting Help

- Check individual module documentation
- Review examples in the `examples/` directory
- Consult test files for usage patterns
- Open issues on GitHub for bugs
- Join discussions for questions

## Next Steps

- Explore [Core Module API](./core.md)
- Learn [Learning Module API](./learning.md)
- Study [Statistics Module API](./statistics.md)
- Review [Tasks Module API](./tasks.md)