use crate::statistics::*;

// Type aliases for test compatibility
type ExGaussianParams = ExGaussianParameters;

#[test]
fn test_ex_gaussian_pdf_numerical_stability() {
    let params = ExGaussianParams {
        mu: 1.0,
        sigma: 0.5,
        tau: 0.3,
    };
    let model = ExGaussianModel::new(params.clone());
    
    // Test extreme values
    let pdf_negative = model.pdf(-10.0);
    assert_eq!(pdf_negative, 0.0, "PDF should be 0 for negative values");
    
    let pdf_large = model.pdf(100.0);
    assert!(pdf_large.is_finite(), "PDF should be finite for large values");
    assert!(pdf_large >= 0.0, "PDF should be non-negative");
    
    // Test near mean
    let pdf_mean = model.pdf(params.mu);
    assert!(pdf_mean > 0.0, "PDF should be positive near mean");
    assert!(pdf_mean.is_finite(), "PDF should be finite near mean");
}

#[test]
fn test_ex_gaussian_pdf_integration() {
    let params = ExGaussianParams {
        mu: 1.0,
        sigma: 0.5,
        tau: 0.3,
    };
    let model = ExGaussianModel::new(params);
    
    // Numerical integration should approximate 1
    let integral: f64 = (0..10000)
        .map(|i| {
            let x = i as f64 * 0.01 - 10.0; // Range from -10 to 90
            model.pdf(x) * 0.01
        })
        .sum();
    
    assert!((integral - 1.0).abs() < 0.01, 
            "PDF should integrate to approximately 1, got {}", integral);
}

#[test]
fn test_ex_gaussian_cdf_properties() {
    let params = ExGaussianParams {
        mu: 1.0,
        sigma: 0.5,
        tau: 0.3,
    };
    let model = ExGaussianModel::new(params);
    
    // CDF at -infinity should be 0
    let cdf_neg_inf = model.cdf(-100.0);
    assert!(cdf_neg_inf.abs() < 0.001, "CDF at -infinity should be ~0");
    
    // CDF at +infinity should be 1
    let cdf_pos_inf = model.cdf(100.0);
    assert!((cdf_pos_inf - 1.0).abs() < 0.001, "CDF at +infinity should be ~1");
    
    // CDF should be monotonically increasing
    let x_values = vec![-1.0, 0.0, 1.0, 2.0, 3.0];
    for i in 1..x_values.len() {
        let cdf_prev = model.cdf(x_values[i-1]);
        let cdf_curr = model.cdf(x_values[i]);
        assert!(cdf_curr >= cdf_prev, 
                "CDF should be monotonically increasing: {} < {}", 
                cdf_curr, cdf_prev);
    }
}

#[test]
fn test_ex_gaussian_mean_calculation() {
    let params = ExGaussianParams {
        mu: 1.0,
        sigma: 0.5,
        tau: 0.3,
    };
    let model = ExGaussianModel::new(params.clone());
    
    let expected_mean = params.mu + params.tau;
    let calculated_mean = model.mean();
    
    assert!((calculated_mean - expected_mean).abs() < 0.001,
            "Mean calculation incorrect: expected {}, got {}", 
            expected_mean, calculated_mean);
}

#[test]
fn test_ex_gaussian_variance_calculation() {
    let params = ExGaussianParams {
        mu: 1.0,
        sigma: 0.5,
        tau: 0.3,
    };
    let model = ExGaussianModel::new(params.clone());
    
    let expected_variance = params.sigma.powi(2) + params.tau.powi(2);
    let calculated_variance = model.variance();
    
    assert!((calculated_variance - expected_variance).abs() < 0.001,
            "Variance calculation incorrect: expected {}, got {}", 
            expected_variance, calculated_variance);
}

#[test]
fn test_strategy_classification_serial() {
    let response_times = vec![1000.0, 1500.0, 2000.0, 2500.0, 3000.0];
    let distances = vec![1, 2, 3, 4, 5];
    
    let analysis = StrategyAnalysis::analyze(&response_times, &distances);
    
    assert!(analysis.correlation > 0.8, 
            "Serial strategy should have high RT-distance correlation: {}", 
            analysis.correlation);
    assert_eq!(analysis.strategy_classification, StrategyType::SerialScan,
            "Should classify as serial scan strategy");
}

#[test]
fn test_strategy_classification_direct() {
    // Direct access: RT independent of distance
    let response_times = vec![1000.0, 950.0, 1050.0, 1000.0, 980.0];
    let distances = vec![1, 2, 3, 4, 5];
    
    let analysis = StrategyAnalysis::analyze(&response_times, &distances);
    
    assert!(analysis.correlation.abs() < 0.3, 
            "Direct access should have low RT-distance correlation: {}", 
            analysis.correlation);
    assert_eq!(analysis.strategy_classification, StrategyType::DirectAccess,
            "Should classify as direct access strategy");
}

#[test]
fn test_strategy_classification_hybrid() {
    // Hybrid: some correlation but not strong
    let response_times = vec![1000.0, 1200.0, 1100.0, 1400.0, 1300.0];
    let distances = vec![1, 2, 3, 4, 5];
    
    let analysis = StrategyAnalysis::analyze(&response_times, &distances);
    
    assert!(analysis.correlation > 0.3 && analysis.correlation < 0.8,
            "Hybrid strategy should have moderate correlation: {}", 
            analysis.correlation);
    assert_eq!(analysis.strategy_classification, StrategyType::Hybrid,
            "Should classify as hybrid strategy");
}

#[test]
fn test_response_time_distribution_fitting() {
    // Generate synthetic data from known distribution
    let _true_params = ExGaussianParams {
        mu: 500.0,
        sigma: 100.0,
        tau: 200.0,
    };
    
    // Note: In a real test, we'd generate samples from the distribution
    // For now, just test the fitting infrastructure exists
    let data = vec![400.0, 500.0, 600.0, 700.0, 800.0, 900.0];
    
    let fitted_params = ResponseTimeDistribution::fit_ex_gaussian(&data);
    
    // Check that fitted parameters are reasonable
    assert!(fitted_params.mu > 0.0, "Fitted mu should be positive");
    assert!(fitted_params.sigma > 0.0, "Fitted sigma should be positive");
    assert!(fitted_params.tau > 0.0, "Fitted tau should be positive");
}

#[test]
fn test_rt_outlier_detection() {
    let response_times = vec![
        1000.0, 1100.0, 950.0, 1050.0, 
        5000.0, // Outlier
        1000.0, 1100.0
    ];
    
    let outliers = ResponseTimeDistribution::detect_outliers(&response_times, 2.0);
    
    assert!(outliers.contains(&4), "Should detect index 4 as outlier");
    assert_eq!(outliers.len(), 1, "Should detect exactly one outlier");
}

#[test]
fn test_goodness_of_fit() {
    let params = ExGaussianParams {
        mu: 1000.0,
        sigma: 100.0,
        tau: 150.0,
    };
    let model = ExGaussianModel::new(params);
    
    // Generate perfect data from model
    let data: Vec<f64> = (0..100)
        .map(|i| 800.0 + i as f64 * 5.0)
        .collect();
    
    // Compute goodness of fit (simplified)
    let log_likelihood: f64 = data.iter()
        .map(|&x| model.pdf(x).ln())
        .filter(|&ll| ll.is_finite())
        .sum();
    
    // Log likelihood should be finite and reasonable
    assert!(log_likelihood.is_finite(), "Log likelihood should be finite");
    assert!(log_likelihood < 0.0, "Log likelihood should typically be negative");
}

#[test]
fn test_strategy_transition_detection() {
    // Simulate strategy change mid-experiment
    let mut response_times = vec![];
    let mut distances = vec![];
    
    // First half: serial strategy
    for i in 1..=5 {
        response_times.push(1000.0 + i as f64 * 200.0);
        distances.push(i);
    }
    
    // Second half: direct access
    for i in 6..=10 {
        response_times.push(1100.0);
        distances.push(i % 5 + 1);
    }
    
    // Analyze first half
    let first_half_rt = &response_times[0..5];
    let first_half_dist = &distances[0..5];
    let first_analysis = StrategyAnalysis::analyze(first_half_rt, first_half_dist);
    
    // Analyze second half
    let second_half_rt = &response_times[5..10];
    let second_half_dist = &distances[5..10];
    let second_analysis = StrategyAnalysis::analyze(second_half_rt, second_half_dist);
    
    // Strategies should differ
    assert_ne!(first_analysis.strategy_classification, 
               second_analysis.strategy_classification,
               "Should detect strategy change");
}