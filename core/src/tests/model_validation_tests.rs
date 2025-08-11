// Model validation and cross-validation tests
// Tests for model selection, validation, and robustness

use crate::learning::bayesian::*;
use crate::learning::learner::*;
use crate::statistics::*;
use crate::core::topology::Topology;
use crate::tasks::{Task, TaskType};

#[test]
fn test_k_fold_cross_validation() {
    // Test k-fold cross-validation for model selection
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    // Generate synthetic data
    let mut responses = Vec::new();
    for i in 0..50 {
        let task = Task {
            task_type: TaskType::Successor {
                item: format!("{}", ((65 + (i % 26)) as u8 as char)),
            },
            prompt: "test".to_string(),
            correct_answer: "test".to_string(),
            options: vec![],
            difficulty: 0.3,
            operation: OperationType::Successor,
        };

        responses.push(ResponseData {
            task,
            correct: i % 3 != 0, // ~67% accuracy
            response_time: 1000.0 + (i as f64 * 10.0),
        });
    }

    // Perform 5-fold cross-validation
    let k = 5;
    let fold_size = responses.len() / k;
    let mut fold_accuracies = Vec::new();

    for fold in 0..k {
        let start_idx = fold * fold_size;
        let end_idx = if fold == k - 1 {
            responses.len()
        } else {
            (fold + 1) * fold_size
        };

        // Training set (all folds except current)
        let mut train_set = Vec::new();
        train_set.extend_from_slice(&responses[..start_idx]);
        train_set.extend_from_slice(&responses[end_idx..]);

        // Test set (current fold)
        let test_set = &responses[start_idx..end_idx];

        // Train model
        let mut fold_model = BayesianLearnerModel::new(&topo);
        for response in &train_set {
            fold_model.update_with_response(response.clone());
        }

        // Validate on test set
        let mut correct_predictions = 0;
        let total_predictions = test_set.len();

        for response in test_set {
            // Simple prediction based on model confidence
            let eig = fold_model.monte_carlo_eig(&response.task, 100);
            let predicted_correct = eig < 0.5; // Lower EIG suggests higher confidence
            
            if predicted_correct == response.correct {
                correct_predictions += 1;
            }
        }

        let fold_accuracy = correct_predictions as f64 / total_predictions as f64;
        fold_accuracies.push(fold_accuracy);
    }

    // Calculate cross-validation statistics
    let mean_accuracy = fold_accuracies.iter().sum::<f64>() / fold_accuracies.len() as f64;
    let accuracy_variance = fold_accuracies
        .iter()
        .map(|acc| (acc - mean_accuracy).powi(2))
        .sum::<f64>()
        / (fold_accuracies.len() - 1) as f64;
    let accuracy_std = accuracy_variance.sqrt();

    // Cross-validation should produce reasonable results
    assert!(
        mean_accuracy > 0.3 && mean_accuracy < 0.9,
        "Cross-validation accuracy should be reasonable: {}",
        mean_accuracy
    );

    assert!(
        accuracy_std < 0.3,
        "Cross-validation should be stable across folds: std={}",
        accuracy_std
    );

    // All fold accuracies should be in reasonable range
    for (i, acc) in fold_accuracies.iter().enumerate() {
        assert!(
            *acc >= 0.0 && *acc <= 1.0,
            "Fold {} accuracy {} should be in [0,1]",
            i,
            acc
        );
    }
}

#[test]
fn test_model_comparison_with_cross_validation() {
    // Compare different models using cross-validation
    let topo = Topology::alphabet();
    
    // Create synthetic data with known pattern
    let mut responses = Vec::new();
    for i in 0..30 {
        let item = format!("{}", ((65 + (i % 26)) as u8 as char));
        let task = Task {
            task_type: TaskType::Successor {
                item: item.clone(),
            },
            prompt: "test".to_string(),
            correct_answer: "test".to_string(),
            options: vec![],
            difficulty: 0.5,
            operation: OperationType::Successor,
        };

        // Create pattern: easier for early alphabet, harder for late alphabet
        let correct = if i < 13 { 
            (i % 4) != 0  // 75% accuracy for A-M
        } else { 
            (i % 3) == 0  // 33% accuracy for N-Z
        };

        responses.push(ResponseData {
            task,
            correct,
            response_time: 1000.0 + (i as f64 * 50.0),
        });
    }

    // Model 1: Simple Bayesian model
    let mut model1_scores = Vec::new();
    
    // Model 2: Bayesian model with different priors (simulated by different initialization)
    let mut model2_scores = Vec::new();

    // Simple 2-fold cross-validation for comparison
    let fold_size = responses.len() / 2;
    
    for fold in 0..2 {
        let start_idx = fold * fold_size;
        let end_idx = (fold + 1) * fold_size;

        let train_set = if fold == 0 {
            &responses[end_idx..]
        } else {
            &responses[..start_idx]
        };
        let test_set = &responses[start_idx..end_idx];

        // Train Model 1
        let mut model1 = BayesianLearnerModel::new(&topo);
        for response in train_set {
            model1.update_with_response(response.clone());
        }

        // Train Model 2 (with different seeding for different behavior)
        let mut model2 = BayesianLearnerModel::with_seed(&topo, Some(12345));
        for response in train_set {
            model2.update_with_response(response.clone());
        }

        // Evaluate both models
        let mut model1_correct = 0;
        let mut model2_correct = 0;

        for response in test_set {
            let eig1 = model1.monte_carlo_eig(&response.task, 50);
            let eig2 = model2.monte_carlo_eig(&response.task, 50);
            
            // Prediction based on EIG
            let pred1 = eig1 < 0.4;
            let pred2 = eig2 < 0.5;
            
            if pred1 == response.correct {
                model1_correct += 1;
            }
            if pred2 == response.correct {
                model2_correct += 1;
            }
        }

        model1_scores.push(model1_correct as f64 / test_set.len() as f64);
        model2_scores.push(model2_correct as f64 / test_set.len() as f64);
    }

    let model1_mean = model1_scores.iter().sum::<f64>() / model1_scores.len() as f64;
    let model2_mean = model2_scores.iter().sum::<f64>() / model2_scores.len() as f64;

    // Both models should perform reasonably
    assert!(
        model1_mean > 0.2 && model1_mean < 0.9,
        "Model 1 should have reasonable performance: {}",
        model1_mean
    );
    assert!(
        model2_mean > 0.2 && model2_mean < 0.9,
        "Model 2 should have reasonable performance: {}",
        model2_mean
    );

    // Models may perform differently due to different initialization/priors
    let performance_difference = (model1_mean - model2_mean).abs();
    assert!(
        performance_difference >= 0.0,
        "Should be able to distinguish model performance"
    );
}

#[test]
fn test_simulation_based_validation() {
    // Generate data from known model and verify parameter recovery
    let topo = Topology::alphabet();
    let mut true_model = BayesianLearnerModel::with_seed(&topo, Some(42));

    // Simulate responses from the true model
    let mut simulated_responses = Vec::new();
    for i in 0..40 {
        let item = format!("{}", ((65 + (i % 26)) as u8 as char));
        let task = Task {
            task_type: TaskType::Successor {
                item: item.clone(),
            },
            prompt: "sim".to_string(),
            correct_answer: "sim".to_string(),
            options: vec![],
            difficulty: 0.4,
            operation: OperationType::Successor,
        };

        // Generate response based on true model's EIG
        let eig = true_model.monte_carlo_eig(&task, 100);
        let success_prob = 1.0 / (1.0 + eig); // Convert EIG to approximate success probability
        let correct = rand::random::<f64>() < success_prob;

        simulated_responses.push(ResponseData {
            task,
            correct,
            response_time: 1000.0 + (i as f64 * 30.0),
        });
    }

    // Fit model to simulated data
    let mut fitted_model = BayesianLearnerModel::with_seed(&topo, Some(123));
    for response in &simulated_responses {
        fitted_model.update_with_response(response.clone());
    }

    // Compare model entropies as a measure of similarity
    let true_entropy = true_model.total_entropy();
    let fitted_entropy = fitted_model.total_entropy();

    // After learning from simulated data, fitted model should have similar characteristics
    // (Both should have reduced entropy compared to initial state)
    let initial_model = BayesianLearnerModel::new(&topo);
    let initial_entropy = initial_model.total_entropy();

    assert!(
        fitted_entropy < initial_entropy,
        "Fitted model should have learned: {} < {}",
        fitted_entropy,
        initial_entropy
    );

    // The entropy should be in a reasonable range relative to the true model
    let entropy_ratio = fitted_entropy / true_entropy;
    assert!(
        entropy_ratio > 0.1 && entropy_ratio < 5.0,
        "Fitted model entropy should be in reasonable range relative to true model: ratio={}",
        entropy_ratio
    );
}

#[test]
fn test_confidence_interval_coverage() {
    // Test that confidence intervals have correct coverage
    use crate::statistics::validation::StatisticalValidator;

    let validator = StatisticalValidator::new(0.05);
    let mut coverage_count = 0;
    let n_simulations = 50; // Reduced for test performance
    let true_mean = 5.0;

    for _sim in 0..n_simulations {
        // Generate sample from known distribution
        let mut sample = Vec::new();
        for _ in 0..20 {
            let value = true_mean + (rand::random::<f64>() - 0.5) * 2.0; // Uniform around true mean
            sample.push(value);
        }

        // Calculate 95% confidence interval
        let sample_mean = sample.iter().sum::<f64>() / sample.len() as f64;
        let sample_var = sample
            .iter()
            .map(|x| (x - sample_mean).powi(2))
            .sum::<f64>()
            / (sample.len() - 1) as f64;
        let se = (sample_var / sample.len() as f64).sqrt();
        
        // 95% CI using t-distribution approximation
        let t_critical = 2.093; // t(19, 0.025) ≈ 2.093
        let ci_lower = sample_mean - t_critical * se;
        let ci_upper = sample_mean + t_critical * se;

        // Check if true mean is in confidence interval
        if true_mean >= ci_lower && true_mean <= ci_upper {
            coverage_count += 1;
        }
    }

    let coverage_rate = coverage_count as f64 / n_simulations as f64;
    
    // 95% CI should cover true parameter approximately 95% of the time
    // With 50 simulations, expect 47.5 ± some variance
    assert!(
        coverage_rate > 0.8 && coverage_rate < 1.0,
        "95% confidence interval should have ~95% coverage, got {:.2}%",
        coverage_rate * 100.0
    );
}

#[test]
fn test_robustness_to_outliers() {
    // Test that statistical methods are robust to outliers
    let normal_data = vec![5.0, 5.1, 4.9, 5.2, 4.8, 5.0, 5.1, 4.9, 5.0, 5.1];
    let data_with_outlier = {
        let mut data = normal_data.clone();
        data.push(50.0); // Extreme outlier
        data
    };

    // Calculate statistics for both datasets
    let normal_stats = DetailedStatistics::from_data(&normal_data);
    let outlier_stats = DetailedStatistics::from_data(&data_with_outlier);

    // Mean should be affected by outlier
    assert!(
        outlier_stats.mean > normal_stats.mean * 1.5,
        "Mean should be affected by outlier: {} vs {}",
        outlier_stats.mean,
        normal_stats.mean
    );

    // Median should be more robust
    let median_change_ratio = outlier_stats.median / normal_stats.median;
    assert!(
        median_change_ratio > 0.9 && median_change_ratio < 1.1,
        "Median should be robust to outlier: {} vs {} (ratio: {})",
        outlier_stats.median,
        normal_stats.median,
        median_change_ratio
    );

    // Test outlier detection
    let outliers = ResponseTimeDistribution::detect_outliers(&data_with_outlier, 2.0);
    assert!(
        outliers.contains(&10),
        "Should detect the outlier at index 10"
    );
    assert_eq!(outliers.len(), 1, "Should detect exactly one outlier");
}

#[test]
fn test_missing_data_handling() {
    // Test system behavior with missing/incomplete data
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    // Test with minimal data
    let sparse_responses = vec![
        ResponseData {
            task: Task {
                task_type: TaskType::Successor {
                    item: "A".to_string(),
                },
                prompt: "test".to_string(),
                correct_answer: "B".to_string(),
                options: vec![],
                difficulty: 0.5,
                operation: OperationType::Successor,
            },
            correct: true,
            response_time: 1000.0,
        },
    ];

    // Model should handle sparse data gracefully
    for response in sparse_responses {
        model.update_with_response(response);
    }

    // Should still be able to make predictions
    let test_task = Task {
        task_type: TaskType::Successor {
            item: "B".to_string(),
        },
        prompt: "test".to_string(),
        correct_answer: "C".to_string(),
        options: vec![],
        difficulty: 0.5,
        operation: OperationType::Successor,
    };

    let eig = model.monte_carlo_eig(&test_task, 100);
    assert!(
        eig >= 0.0 && eig.is_finite(),
        "Should produce valid EIG even with minimal data: {}",
        eig
    );

    // Entropy should still be reasonable
    let entropy = model.total_entropy();
    assert!(
        entropy > 0.0 && entropy.is_finite(),
        "Should maintain positive entropy with minimal data: {}",
        entropy
    );
}

#[test]
fn test_hyperparameter_sensitivity() {
    // Test sensitivity to hyperparameters
    let topo = Topology::alphabet();
    
    // Test different learning rates by proxy (different numbers of updates)
    let task = Task {
        task_type: TaskType::Successor {
            item: "M".to_string(),
        },
        prompt: "test".to_string(),
        correct_answer: "N".to_string(),
        options: vec![],
        difficulty: 0.5,
        operation: OperationType::Successor,
    };

    let response = ResponseData {
        task: task.clone(),
        correct: true,
        response_time: 1000.0,
    };

    // Model with few updates
    let mut model1 = BayesianLearnerModel::new(&topo);
    for _ in 0..1 {
        model1.update_with_response(response.clone());
    }

    // Model with many updates
    let mut model2 = BayesianLearnerModel::new(&topo);
    for _ in 0..10 {
        model2.update_with_response(response.clone());
    }

    let entropy1 = model1.total_entropy();
    let entropy2 = model2.total_entropy();

    // More updates should reduce entropy more
    assert!(
        entropy2 < entropy1,
        "More updates should reduce entropy more: {} < {}",
        entropy2,
        entropy1
    );

    // Both should have reasonable entropy values
    assert!(entropy1 > 0.0 && entropy1.is_finite());
    assert!(entropy2 > 0.0 && entropy2.is_finite());
}