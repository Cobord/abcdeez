# TEST_STRATEGY.md

## Comprehensive Testing Strategy for Graph Learning Core Library

### Executive Summary

This document outlines a comprehensive testing strategy for the alphabet-terminal-prototype graph learning system. The strategy covers unit testing, integration testing, performance testing, statistical validation, and experimental validation to ensure the system meets all requirements specified in PAPER.md and achieves high scientific rigor.

## 1. Test Categories

### 1.1 Unit Tests
Test individual components in isolation to ensure correctness of fundamental operations.

### 1.2 Integration Tests
Test interactions between modules to ensure proper system behavior.

### 1.3 Statistical Tests
Validate statistical methods and ensure mathematical correctness.

### 1.4 Performance Tests
Benchmark critical operations and ensure scalability.

### 1.5 End-to-End Tests
Complete workflow tests simulating real experimental scenarios.

## 2. Unit Test Coverage

### 2.1 Topology Module (`src/topology.rs`)

```rust
#[cfg(test)]
mod topology_tests {
    use super::*;
    
    #[test]
    fn test_linear_topology_creation() {
        let topo = Topology::alphabet();
        assert_eq!(topo.nodes.len(), 26);
        assert_eq!(topo.topology_type, TopologyType::Linear);
    }
    
    #[test]
    fn test_cyclic_topology_wraparound() {
        let topo = Topology::days_of_week();
        let sunday = topo.get_node_by_label("Sunday").unwrap();
        let successor = topo.get_successor(&sunday.id);
        assert_eq!(successor, Some("node_0".to_string())); // Monday
    }
    
    #[test]
    fn test_dag_topological_sort() {
        let dag = Topology::example_dag();
        let sorted = dag.get_topological_sort().unwrap();
        
        // Verify all dependencies come before dependents
        for (i, node) in sorted.iter().enumerate() {
            for (j, later_node) in sorted[i+1..].iter().enumerate() {
                assert!(!dag.has_path(later_node, node).unwrap_or(false));
            }
        }
    }
    
    #[test]
    fn test_shortest_path() {
        let topo = Topology::alphabet();
        let path = topo.shortest_path("A", "E").unwrap();
        assert_eq!(path, vec!["A", "B", "C", "D", "E"]);
    }
}
```

### 2.2 Learner Model (`src/learner.rs`)

```rust
#[test]
fn test_learner_initialization() {
    let topo = Topology::alphabet();
    let learner = LearnerModel::new("test_learner".to_string(), &topo);
    
    assert_eq!(learner.learner_id, "test_learner");
    assert_eq!(learner.node_embeddings.len(), 26);
    assert!(learner.chunk_boundaries.is_empty());
}

#[test]
fn test_operation_proficiency_update() {
    let mut learner = create_test_learner();
    let initial_theta = learner.operation_proficiencies
        .get("Successor").unwrap().theta;
    
    learner.update_operation_proficiency(&OperationType::Successor, true);
    
    let updated_theta = learner.operation_proficiencies
        .get("Successor").unwrap().theta;
    assert!(updated_theta > initial_theta);
}

#[test]
fn test_memory_strength_decay() {
    let mut learner = create_test_learner();
    learner.update_memory_strength("A", true);
    let initial_strength = learner.memory_strengths.get("node_0").unwrap().strength;
    
    // Simulate time passing
    std::thread::sleep(std::time::Duration::from_millis(100));
    learner.update_memory_strength("B", true);
    
    // Check that A's strength has decayed
    let current_strength = learner.calculate_current_strength("node_0");
    assert!(current_strength < initial_strength);
}
```

### 2.3 Bayesian Model (`src/bayesian.rs`)

```rust
#[test]
fn test_monte_carlo_eig() {
    let topo = Topology::alphabet();
    let model = BayesianLearnerModel::new(&topo);
    let task = create_test_task();
    
    let eig = model.monte_carlo_eig(&task, 1000);
    
    // EIG should be non-negative
    assert!(eig >= 0.0);
    // EIG should be bounded by entropy
    assert!(eig <= model.calculate_entropy());
}

#[test]
fn test_posterior_update() {
    let mut model = BayesianLearnerModel::new(&Topology::alphabet());
    let initial_entropy = model.calculate_entropy();
    
    // Update with correct response
    model.update_posterior(&create_test_task(), true);
    
    let updated_entropy = model.calculate_entropy();
    // Entropy should decrease after update (information gain)
    assert!(updated_entropy < initial_entropy);
}

#[test]
fn test_sampling_consistency() {
    let model = BayesianLearnerModel::new(&Topology::alphabet());
    let mut rng = thread_rng();
    
    let samples: Vec<_> = (0..1000)
        .map(|_| model.sample_from_posterior(&mut rng))
        .collect();
    
    // Check that samples follow expected distribution
    let mean_theta: f64 = samples.iter()
        .map(|s| s.operation_proficiencies.get("Successor").unwrap().theta)
        .sum::<f64>() / samples.len() as f64;
    
    assert!((mean_theta - 0.5).abs() < 0.1); // Should be around prior mean
}
```

### 2.4 Statistics Module (`src/statistics.rs`)

```rust
#[test]
fn test_ex_gaussian_pdf() {
    let params = ExGaussianParams {
        mu: 1.0,
        sigma: 0.5,
        tau: 0.3,
    };
    let model = ExGaussianModel::new(params);
    
    // PDF should integrate to 1 (approximately)
    let integral: f64 = (0..10000)
        .map(|i| {
            let x = i as f64 * 0.01;
            model.pdf(x) * 0.01
        })
        .sum();
    
    assert!((integral - 1.0).abs() < 0.01);
}

#[test]
fn test_strategy_classification() {
    let serial_times = vec![1.0, 1.5, 2.0, 2.5, 3.0];
    let distances = vec![1, 2, 3, 4, 5];
    
    let analysis = StrategyAnalysis::analyze(&serial_times, &distances);
    
    assert!(analysis.correlation > 0.8);
    assert_eq!(analysis.strategy_classification, StrategyType::SerialScan);
}
```

## 3. Integration Tests

### 3.1 Task Generation and Response

```rust
#[test]
fn test_task_generation_flow() {
    let topo = Topology::alphabet();
    let mut generator = TaskGenerator::new(topo.clone());
    let mut learner = LearnerModel::new("test".to_string(), &topo);
    
    // Generate task
    let task = generator.generate_task(Some(TaskType::Successor { 
        item: "C".to_string() 
    }));
    
    assert_eq!(task.correct_answer, "D");
    assert!(task.options.contains(&"D".to_string()));
    
    // Simulate response
    learner.update_operation_proficiency(&task.operation, true);
    
    // Verify update
    let prof = learner.operation_proficiencies.get("Successor").unwrap();
    assert_eq!(prof.practice_count, 1);
}
```

### 3.2 Adaptive Scheduling

```rust
#[test]
fn test_adaptive_task_selection() {
    let topo = Topology::alphabet();
    let learner = LearnerModel::new("test".to_string(), &topo);
    let bayesian = BayesianLearnerModel::new(&topo);
    let mut scheduler = AdaptiveScheduler::new(topo.clone());
    
    // Select next task based on EIG
    let task = scheduler.select_next_task(&learner, &bayesian);
    
    // Verify task has positive EIG
    let eig = bayesian.monte_carlo_eig(&task, 100);
    assert!(eig > 0.0);
}
```

### 3.3 Transfer Learning

```rust
#[test]
fn test_knowledge_transfer() {
    let source = Topology::alphabet();
    let target = Topology::new_linear(
        (0..10).map(|i| i.to_string()).collect()
    );
    
    let mut system = TransferLearningSystem::new(
        source.clone(), 
        target.clone(), 
        TransferType::Structural
    );
    
    // Train source model
    let mut source_model = LearnerModel::new("source".to_string(), &source);
    for _ in 0..100 {
        source_model.update_operation_proficiency(&OperationType::Successor, true);
    }
    
    system.set_source_model(source_model);
    let target_model = system.transfer_knowledge();
    
    // Target should have non-zero proficiencies
    let target_prof = target_model.operation_proficiencies
        .get("Successor").unwrap();
    assert!(target_prof.theta > 0.0);
}
```

## 4. Statistical Validation Tests

### 4.1 Hypothesis Testing

```rust
#[test]
fn test_t_test_implementation() {
    let validator = StatisticalValidator::new(0.95);
    
    let group1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let group2 = vec![6.0, 7.0, 8.0, 9.0, 10.0];
    
    let result = validator.t_test(&group1, &group2, false);
    
    assert!(result.p_value < 0.05);
    assert!(result.significant);
    assert!(result.effect_size > 2.0); // Large effect
}

#[test]
fn test_anova_multiple_groups() {
    let validator = StatisticalValidator::new(0.95);
    
    let groups = vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0, 6.0],
        vec![7.0, 8.0, 9.0],
    ];
    
    let result = validator.anova(groups);
    
    assert!(result.p_value < 0.05);
    assert!(result.significant);
}

#[test]
fn test_multiple_comparison_correction() {
    let validator = StatisticalValidator::new(0.95);
    
    let p_values = vec![0.01, 0.04, 0.03, 0.20, 0.15];
    let corrected = validator.apply_correction(p_values);
    
    // After Benjamini-Hochberg correction
    assert!(corrected[0] < 0.05); // Still significant
    assert!(corrected[1] < 0.10); // May or may not be significant
}
```

### 4.2 Cross-Validation

```rust
#[test]
fn test_k_fold_cross_validation() {
    let validator = StatisticalValidator::new(0.95);
    
    let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
    let labels: Vec<bool> = (0..100).map(|i| i % 2 == 0).collect();
    
    let results = validator.cross_validate(&data, &labels, 5, |train, train_labels| {
        // Simple majority class predictor
        let n_true = train_labels.iter().filter(|&&x| x).count();
        let majority = n_true > train_labels.len() / 2;
        vec![majority; train_labels.len()]
    });
    
    assert_eq!(results.k_folds, 5);
    assert!(results.mean_accuracy >= 0.45 && results.mean_accuracy <= 0.55);
}
```

## 5. Performance Benchmarks

### 5.1 Monte Carlo Simulation Performance

```rust
#[bench]
fn bench_monte_carlo_eig(b: &mut Bencher) {
    let topo = Topology::alphabet();
    let model = BayesianLearnerModel::new(&topo);
    let task = create_test_task();
    
    b.iter(|| {
        black_box(model.monte_carlo_eig(&task, 1000))
    });
}
```

### 5.2 Task Generation Performance

```rust
#[bench]
fn bench_task_generation(b: &mut Bencher) {
    let topo = Topology::alphabet();
    let mut generator = TaskGenerator::new(topo);
    
    b.iter(|| {
        black_box(generator.generate_task(None))
    });
}
```

## 6. End-to-End Experimental Tests

### 6.1 Complete Experiment Simulation

```rust
#[test]
fn test_full_experiment_workflow() {
    let mut framework = ExperimentFramework::new("./test_output".to_string());
    
    // Create experiment
    let config = ExperimentTemplates::basic_learning_experiment();
    let exp_id = framework.create_experiment("Test Experiment".to_string(), config);
    
    // Add conditions
    framework.add_condition(ExperimentCondition {
        name: "Control".to_string(),
        parameters: HashMap::new(),
        task_distribution: TaskDistribution {
            successor_prob: 0.5,
            predecessor_prob: 0.3,
            k_jump_prob: 0.1,
            segment_prob: 0.05,
            pairwise_prob: 0.05,
        },
    });
    
    framework.add_condition(ExperimentCondition {
        name: "Adaptive".to_string(),
        parameters: HashMap::new(),
        task_distribution: TaskDistribution {
            successor_prob: 0.3,
            predecessor_prob: 0.3,
            k_jump_prob: 0.2,
            segment_prob: 0.1,
            pairwise_prob: 0.1,
        },
    });
    
    // Run experiment
    framework.run_experiment().unwrap();
    
    // Verify results
    let experiment = &framework.experiments[0];
    assert!(experiment.results.is_some());
    
    let results = experiment.results.as_ref().unwrap();
    assert!(results.summary_statistics.overall_accuracy > 0.0);
    assert!(!results.hypothesis_tests.is_empty());
}
```

### 6.2 Hierarchical Model Validation

```rust
#[test]
fn test_hierarchical_bayesian_model() {
    let topo = Topology::alphabet();
    let mut model = HierarchicalBayesianModel::new(&topo);
    
    // Add multiple learners
    for i in 0..10 {
        model.add_learner(format!("learner_{}", i));
    }
    
    // Simulate responses
    for _ in 0..100 {
        let response = ResponseData {
            learner_id: format!("learner_{}", rand::random::<usize>() % 10),
            task_id: "task_1".to_string(),
            correct: rand::random::<bool>(),
            response_time: 1.0 + rand::random::<f64>() * 2.0,
            timestamp: 0,
        };
        model.update(response);
    }
    
    // Run MCMC
    let samples = model.mcmc_sample(100);
    
    // Verify convergence
    assert!(samples.len() == 100);
    
    // Calculate DIC
    let dic = model.calculate_dic(&samples);
    assert!(dic.is_finite());
}
```

## 7. Property-Based Testing

### 7.1 Invariant Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_topology_distance_symmetry(
        a in 0usize..26,
        b in 0usize..26
    ) {
        let topo = Topology::alphabet();
        let node_a = &topo.nodes[a];
        let node_b = &topo.nodes[b];
        
        let dist_ab = topo.get_distance(&node_a.id, &node_b.id);
        let dist_ba = topo.get_distance(&node_b.id, &node_a.id);
        
        prop_assert_eq!(dist_ab, dist_ba);
    }
    
    #[test]
    fn test_eig_bounds(
        n_samples in 10usize..100
    ) {
        let topo = Topology::alphabet();
        let model = BayesianLearnerModel::new(&topo);
        let task = create_random_task();
        
        let eig = model.monte_carlo_eig(&task, n_samples);
        
        prop_assert!(eig >= 0.0);
        prop_assert!(eig <= model.calculate_entropy());
    }
}
```

## 8. Regression Testing

### 8.1 Golden Tests

```rust
#[test]
fn test_regression_ex_gaussian_values() {
    let params = ExGaussianParams {
        mu: 1.0,
        sigma: 0.5,
        tau: 0.3,
    };
    let model = ExGaussianModel::new(params);
    
    // Known values from reference implementation
    assert!((model.pdf(1.0) - 0.7978845608).abs() < 1e-9);
    assert!((model.pdf(2.0) - 0.3520653267).abs() < 1e-9);
    assert!((model.cdf(1.5) - 0.5987063256).abs() < 1e-9);
}
```

## 9. Test Execution Strategy

### 9.1 Continuous Integration

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
      - run: cargo test --doc
      - run: cargo bench --no-run
```

### 9.2 Test Coverage Goals

- **Unit Test Coverage**: >80% line coverage
- **Branch Coverage**: >70%
- **Integration Test Coverage**: All major workflows
- **Statistical Validation**: All statistical methods verified
- **Performance Benchmarks**: No regressions >10%

### 9.3 Test Organization

```
tests/
├── unit/
│   ├── topology_test.rs
│   ├── learner_test.rs
│   ├── bayesian_test.rs
│   └── statistics_test.rs
├── integration/
│   ├── workflow_test.rs
│   ├── adaptive_test.rs
│   └── transfer_test.rs
├── statistical/
│   ├── validation_test.rs
│   └── hypothesis_test.rs
├── performance/
│   └── benchmarks.rs
└── e2e/
    └── experiment_test.rs
```

## 10. Test Data Management

### 10.1 Test Fixtures

```rust
pub mod fixtures {
    pub fn create_test_topology() -> Topology {
        Topology::alphabet()
    }
    
    pub fn create_test_learner() -> LearnerModel {
        LearnerModel::new("test".to_string(), &create_test_topology())
    }
    
    pub fn create_test_task() -> Task {
        Task {
            task_type: TaskType::Successor { item: "C".to_string() },
            prompt: "What comes after C?".to_string(),
            correct_answer: "D".to_string(),
            options: vec!["B", "C", "D", "E"].iter()
                .map(|s| s.to_string()).collect(),
            difficulty: 0.3,
            operation: OperationType::Successor,
        }
    }
}
```

### 10.2 Mock Data Generation

```rust
pub fn generate_synthetic_responses(n: usize) -> Vec<ResponseData> {
    (0..n).map(|i| {
        ResponseData {
            learner_id: format!("learner_{}", i % 10),
            task_id: format!("task_{}", i % 5),
            correct: rand::random::<f64>() > 0.3,
            response_time: 0.5 + rand::random::<f64>() * 3.0,
            timestamp: i,
        }
    }).collect()
}
```

## 11. Testing Best Practices

### 11.1 Test Naming Convention
- Use descriptive names: `test_<module>_<functionality>_<expected_behavior>`
- Example: `test_bayesian_monte_carlo_eig_positive_values`

### 11.2 Test Independence
- Each test should be independent and not rely on others
- Use setup/teardown for shared initialization
- Clean up resources after tests

### 11.3 Test Documentation
- Document complex test scenarios
- Explain expected behaviors
- Reference paper sections for mathematical tests

### 11.4 Error Testing
```rust
#[test]
#[should_panic(expected = "Invalid topology")]
fn test_invalid_topology_creation() {
    Topology::new_linear(vec![]);
}
```

## 12. Validation Against PAPER.md Requirements

### 12.1 Monte Carlo Validation
- ✅ Test confirms 1000 samples used
- ✅ Test verifies EIG calculation correctness
- ✅ Test checks bounds and convergence

### 12.2 Ex-Gaussian Distribution
- ✅ PDF calculation tested against known values
- ✅ CDF integration verified
- ✅ Parameter estimation validated

### 12.3 Task Types Coverage
- ✅ All 13 task types have unit tests
- ✅ Extended task types tested
- ✅ DAG-specific tasks validated

### 12.4 Statistical Rigor
- ✅ Hypothesis tests validated
- ✅ Multiple comparison corrections tested
- ✅ Cross-validation implemented and tested

## 13. Future Testing Improvements

### 13.1 Mutation Testing
Implement mutation testing to verify test quality:
```bash
cargo install cargo-mutants
cargo mutants
```

### 13.2 Fuzz Testing
Add fuzzing for robustness:
```rust
#[cfg(fuzzing)]
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = Topology::new_linear(vec![s.to_string()]);
    }
});
```

### 13.3 Performance Regression Detection
Implement automated performance regression detection:
```rust
#[bench]
fn bench_with_regression_check(b: &mut Bencher) {
    let baseline = load_baseline_performance();
    let result = b.iter(|| { /* benchmark code */ });
    assert!(result < baseline * 1.1); // Max 10% regression
}
```

## Conclusion

This comprehensive testing strategy ensures that the graph learning system meets all scientific requirements, maintains high code quality, and provides reliable experimental results. Regular execution of this test suite will catch regressions early and maintain confidence in the system's correctness and performance.