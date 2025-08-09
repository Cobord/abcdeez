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
    assert!(
        pdf_large.is_finite(),
        "PDF should be finite for large values"
    );
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

    assert!(
        (integral - 1.0).abs() < 0.01,
        "PDF should integrate to approximately 1, got {}",
        integral
    );
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
    assert!(
        (cdf_pos_inf - 1.0).abs() < 0.001,
        "CDF at +infinity should be ~1"
    );

    // CDF should be monotonically increasing
    let x_values = vec![-1.0, 0.0, 1.0, 2.0, 3.0];
    for i in 1..x_values.len() {
        let cdf_prev = model.cdf(x_values[i - 1]);
        let cdf_curr = model.cdf(x_values[i]);
        assert!(
            cdf_curr >= cdf_prev,
            "CDF should be monotonically increasing: {} < {}",
            cdf_curr,
            cdf_prev
        );
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

    assert!(
        (calculated_mean - expected_mean).abs() < 0.001,
        "Mean calculation incorrect: expected {}, got {}",
        expected_mean,
        calculated_mean
    );
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

    assert!(
        (calculated_variance - expected_variance).abs() < 0.001,
        "Variance calculation incorrect: expected {}, got {}",
        expected_variance,
        calculated_variance
    );
}

#[test]
fn test_strategy_classification_serial() {
    let response_times = vec![1000.0, 1500.0, 2000.0, 2500.0, 3000.0];
    let distances = vec![1, 2, 3, 4, 5];

    let analysis = StrategyAnalysis::analyze(&response_times, &distances);

    assert!(
        analysis.correlation > 0.8,
        "Serial strategy should have high RT-distance correlation: {}",
        analysis.correlation
    );
    assert_eq!(
        analysis.strategy_classification,
        StrategyType::SerialScan,
        "Should classify as serial scan strategy"
    );
}

#[test]
fn test_strategy_classification_direct() {
    // Direct access: RT independent of distance
    let response_times = vec![1000.0, 950.0, 1050.0, 1000.0, 980.0];
    let distances = vec![1, 2, 3, 4, 5];

    let analysis = StrategyAnalysis::analyze(&response_times, &distances);

    assert!(
        analysis.correlation.abs() < 0.3,
        "Direct access should have low RT-distance correlation: {}",
        analysis.correlation
    );
    assert_eq!(
        analysis.strategy_classification,
        StrategyType::DirectAccess,
        "Should classify as direct access strategy"
    );
}

#[test]
fn test_strategy_classification_hybrid() {
    // Hybrid: some correlation but not strong (between 0.3 and 0.7)
    // Create data with moderate correlation (~0.5) by adding noise
    let response_times = vec![
        1000.0, 1400.0, 1200.0, 1100.0, 1500.0, 1300.0, 1600.0, 1200.0,
    ];
    let distances = vec![1, 2, 2, 3, 4, 3, 5, 4];

    let analysis = StrategyAnalysis::analyze(&response_times, &distances);

    // With the noisy pattern, correlation should be moderate
    assert!(
        analysis.correlation > 0.3 && analysis.correlation < 0.7,
        "Hybrid strategy should have moderate correlation (0.3 < r < 0.7): {}",
        analysis.correlation
    );
    assert_eq!(
        analysis.strategy_classification,
        StrategyType::Hybrid,
        "Should classify as hybrid strategy"
    );
}

#[test]
fn test_response_time_distribution_fitting() {
    // Test with data that should produce known Ex-Gaussian parameters
    // Using a simple case where we know the approximate expected values

    // Generate data with clear Ex-Gaussian characteristics:
    // - Most values clustered around 500-600 (Gaussian component)
    // - Some values with long tail up to 1200+ (Exponential component)
    let data = vec![
        450.0, 480.0, 500.0, 510.0, 520.0, 530.0, 540.0, 550.0, // Gaussian cluster
        560.0, 570.0, 580.0, 590.0, 600.0, 610.0, 620.0, 630.0, 640.0, 680.0, 720.0,
        780.0, // Start of tail
        850.0, 920.0, 1050.0, 1200.0, // Exponential tail
    ];

    let fitted_params = ResponseTimeDistribution::fit_ex_gaussian(&data);

    // For this data, we expect:
    // - mu around 500-550 (center of Gaussian component)
    // - sigma around 50-100 (spread of Gaussian)
    // - tau around 100-200 (exponential decay rate)

    // Verify parameters are in reasonable ranges
    assert!(
        fitted_params.mu > 400.0 && fitted_params.mu < 700.0,
        "Fitted mu {} should be in range [400, 700]",
        fitted_params.mu
    );
    assert!(
        fitted_params.sigma > 10.0 && fitted_params.sigma < 200.0,
        "Fitted sigma {} should be in range [10, 200]",
        fitted_params.sigma
    );
    assert!(
        fitted_params.tau > 50.0 && fitted_params.tau < 400.0,
        "Fitted tau {} should be in range [50, 400]",
        fitted_params.tau
    );

    // Verify the model can evaluate its own PDF/CDF without errors
    let model = ExGaussianModel::new(fitted_params.clone());
    for &x in &data {
        let pdf = model.pdf(x);
        assert!(
            pdf >= 0.0 && pdf.is_finite(),
            "PDF at {} should be non-negative and finite: {}",
            x,
            pdf
        );

        let cdf = model.cdf(x);
        assert!(
            cdf >= 0.0 && cdf <= 1.0,
            "CDF at {} should be in [0,1]: {}",
            x,
            cdf
        );
    }

    // Verify mean calculation matches expected formula
    let expected_mean = fitted_params.mu + fitted_params.tau;
    let calculated_mean = model.mean();
    assert!(
        (calculated_mean - expected_mean).abs() < 0.001,
        "Mean calculation incorrect: expected {}, got {}",
        expected_mean,
        calculated_mean
    );
}

#[test]
fn test_rt_outlier_detection() {
    let response_times = vec![
        1000.0, 1100.0, 950.0, 1050.0, 5000.0, // Outlier
        1000.0, 1100.0,
    ];

    let outliers = ResponseTimeDistribution::detect_outliers(&response_times, 2.0);

    assert!(outliers.contains(&4), "Should detect index 4 as outlier");
    assert_eq!(outliers.len(), 1, "Should detect exactly one outlier");
}

#[test]
fn test_goodness_of_fit() {
    // Test that the model can recognize data from its own distribution
    let true_params = ExGaussianParams {
        mu: 1000.0,
        sigma: 100.0,
        tau: 150.0,
    };
    let model = ExGaussianModel::new(true_params.clone());

    // Generate plausible Ex-Gaussian data
    // (In production, we'd sample from the actual distribution)
    let mean = true_params.mu + true_params.tau;
    let _std = (true_params.sigma.powi(2) + true_params.tau.powi(2)).sqrt();

    // Create data that follows Ex-Gaussian shape:
    // Most points near the mode, with exponential tail
    let mut data = Vec::new();

    // Add points from the main body (Gaussian-like)
    for i in 0..30 {
        let x = true_params.mu + (i as f64 - 15.0) * 10.0;
        data.push(x);
    }

    // Add points from the tail (exponential-like)
    for i in 0..10 {
        let x = mean + (i as f64) * true_params.tau * 0.5;
        data.push(x);
    }

    // Compute log likelihood
    let log_likelihood: f64 = data
        .iter()
        .map(|&x| {
            let pdf = model.pdf(x);
            if pdf > 0.0 {
                pdf.ln()
            } else {
                -1000.0 // Penalty for zero probability
            }
        })
        .sum();

    // For data from the model's distribution, log likelihood should be reasonable
    let avg_log_likelihood = log_likelihood / data.len() as f64;

    // Average log likelihood should be better than random uniform
    // For Ex-Gaussian, typical values are around -6 to -8
    assert!(
        avg_log_likelihood > -20.0 && avg_log_likelihood < 0.0,
        "Average log likelihood {} should be in reasonable range",
        avg_log_likelihood
    );

    // Test against clearly wrong distribution
    let wrong_params = ExGaussianParams {
        mu: 0.0, // Very different parameters
        sigma: 10.0,
        tau: 5.0,
    };
    let wrong_model = ExGaussianModel::new(wrong_params);

    let wrong_log_likelihood: f64 = data
        .iter()
        .map(|&x| {
            let pdf = wrong_model.pdf(x);
            if pdf > 0.0 {
                pdf.ln()
            } else {
                -1000.0
            }
        })
        .sum();

    // Correct model should have much better likelihood than wrong model
    assert!(
        log_likelihood > wrong_log_likelihood + 100.0,
        "Correct model likelihood {} should be much better than wrong model {}",
        log_likelihood,
        wrong_log_likelihood
    );
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
    assert_ne!(
        first_analysis.strategy_classification, second_analysis.strategy_classification,
        "Should detect strategy change"
    );
}

#[cfg(test)]
mod multiple_comparison_tests {
    use crate::statistics::{CorrectionMethod, MultipleComparisonCorrection};

    #[test]
    fn test_bonferroni_correction() {
        let p_values = vec![0.01, 0.04, 0.03, 0.005];
        let corrector = MultipleComparisonCorrection::new(CorrectionMethod::Bonferroni, 0.05);
        let adjusted = corrector.adjust_p_values(&p_values);

        // Bonferroni multiplies each p-value by n
        assert_eq!(adjusted.len(), 4);
        assert!((adjusted[0] - 0.04).abs() < 1e-10); // 0.01 * 4
        assert!((adjusted[1] - 0.16).abs() < 1e-10); // 0.04 * 4
        assert!((adjusted[2] - 0.12).abs() < 1e-10); // 0.03 * 4
        assert!((adjusted[3] - 0.02).abs() < 1e-10); // 0.005 * 4

        let rejected = corrector.reject_hypotheses(&p_values);
        assert_eq!(rejected, vec![true, false, false, true]);
    }

    #[test]
    fn test_benjamini_hochberg_correction() {
        let p_values = vec![0.01, 0.04, 0.03, 0.005];
        let corrector =
            MultipleComparisonCorrection::new(CorrectionMethod::BenjaminiHochberg, 0.05);
        let adjusted = corrector.adjust_p_values(&p_values);

        // BH correction should be less conservative than Bonferroni
        assert_eq!(adjusted.len(), 4);
        for &p in &adjusted {
            assert!(p >= 0.0 && p <= 1.0);
        }

        // Verify BH property: adjusted p-values are non-decreasing when sorted by original p-values
        let mut indexed: Vec<(f64, f64)> = p_values
            .iter()
            .zip(adjusted.iter())
            .map(|(&p, &adj)| (p, adj))
            .collect();
        indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for i in 1..indexed.len() {
            assert!(
                indexed[i].1 >= indexed[i - 1].1 || (indexed[i].1 - indexed[i - 1].1).abs() < 1e-10
            );
        }
    }

    #[test]
    fn test_holm_correction() {
        let p_values = vec![0.01, 0.04, 0.03, 0.005];
        let corrector = MultipleComparisonCorrection::new(CorrectionMethod::Holm, 0.05);
        let adjusted = corrector.adjust_p_values(&p_values);

        assert_eq!(adjusted.len(), 4);
        for &p in &adjusted {
            assert!(p >= 0.0 && p <= 1.0);
        }

        // Holm should be less conservative than Bonferroni but more than BH
        let bonf_corrector = MultipleComparisonCorrection::new(CorrectionMethod::Bonferroni, 0.05);
        let bonf_adjusted = bonf_corrector.adjust_p_values(&p_values);

        // At least one Holm p-value should be <= corresponding Bonferroni
        let holm_better = adjusted
            .iter()
            .zip(bonf_adjusted.iter())
            .any(|(&h, &b)| h <= b);
        assert!(holm_better);
    }

    #[test]
    fn test_multiple_comparison_edge_cases() {
        let corrector = MultipleComparisonCorrection::new(CorrectionMethod::Bonferroni, 0.05);

        // Empty input
        let empty: Vec<f64> = vec![];
        assert_eq!(corrector.adjust_p_values(&empty), Vec::<f64>::new());

        // Single p-value
        let single = vec![0.03];
        let adjusted = corrector.adjust_p_values(&single);
        assert_eq!(adjusted, vec![0.03]); // No correction needed for single test

        // p-values at boundaries
        let boundary = vec![0.0, 0.5, 1.0];
        let adjusted = corrector.adjust_p_values(&boundary);
        assert_eq!(adjusted[0], 0.0);
        assert_eq!(adjusted[2], 1.0); // Should cap at 1.0
    }
}

#[cfg(test)]
mod power_analysis_tests {
    use crate::statistics::PowerAnalysis;

    #[test]
    fn test_sample_size_calculation() {
        // Standard parameters: alpha=0.05, power=0.80, medium effect (d=0.5)
        let analysis = PowerAnalysis::new(0.05, 0.80, 0.5);
        let n = analysis.calculate_sample_size_t_test(true);

        // Should be around 64 per group (128 total) for medium effect
        assert!(
            n >= 60 && n <= 70,
            "Sample size should be ~64 for medium effect, got {}",
            n
        );

        // Large effect size should require smaller sample
        let large_effect = PowerAnalysis::new(0.05, 0.80, 0.8);
        let n_large = large_effect.calculate_sample_size_t_test(true);
        assert!(n_large < n, "Larger effect should require smaller sample");

        // Small effect size should require larger sample
        let small_effect = PowerAnalysis::new(0.05, 0.80, 0.2);
        let n_small = small_effect.calculate_sample_size_t_test(true);
        assert!(n_small > n, "Smaller effect should require larger sample");
    }

    #[test]
    fn test_power_calculation() {
        let analysis = PowerAnalysis::new(0.05, 0.80, 0.5);

        // With adequate sample size, should achieve target power
        let power = analysis.calculate_power(64);
        assert!(
            (power - 0.80).abs() < 0.05,
            "Power should be close to 0.80, got {}",
            power
        );

        // Smaller sample should have less power
        let low_power = analysis.calculate_power(20);
        assert!(low_power < 0.50, "Small sample should have low power");

        // Larger sample should have more power
        let high_power = analysis.calculate_power(200);
        assert!(high_power > 0.95, "Large sample should have high power");
    }

    #[test]
    fn test_minimum_detectable_effect() {
        let analysis = PowerAnalysis::new(0.05, 0.80, 0.5);

        // With n=64, should detect d≈0.5
        let mde = analysis.calculate_min_effect_size(64);
        assert!(
            (mde - 0.5).abs() < 0.1,
            "MDE should be close to 0.5, got {}",
            mde
        );

        // Larger sample can detect smaller effects
        let mde_large = analysis.calculate_min_effect_size(200);
        assert!(
            mde_large < mde,
            "Larger sample should detect smaller effects"
        );
    }

    #[test]
    fn test_anova_sample_size() {
        let analysis = PowerAnalysis::new(0.05, 0.80, 0.25); // f=0.25 is medium effect

        // 3 groups
        let n_3groups = analysis.calculate_sample_size_anova(3);
        assert!(
            n_3groups > 50 && n_3groups < 200,
            "ANOVA sample size reasonable"
        );

        // More groups require larger total sample
        let n_5groups = analysis.calculate_sample_size_anova(5);
        assert!(
            n_5groups > n_3groups,
            "More groups need larger total sample"
        );
    }

    #[test]
    fn test_correlation_sample_size() {
        // Medium correlation r=0.3
        let analysis = PowerAnalysis::new(0.05, 0.80, 0.3);
        let n = analysis.calculate_sample_size_correlation();

        // Should be around 84 for r=0.3
        assert!(
            n >= 80 && n <= 90,
            "Sample size for r=0.3 should be ~84, got {}",
            n
        );

        // Strong correlation needs smaller sample
        let strong = PowerAnalysis::new(0.05, 0.80, 0.5);
        let n_strong = strong.calculate_sample_size_correlation();
        assert!(n_strong < n, "Stronger correlation needs smaller sample");
    }

    #[test]
    fn test_post_hoc_power() {
        let analysis = PowerAnalysis::new(0.05, 0.80, 0.5);

        // If observed effect matches expected, post-hoc power ≈ planned power
        let post_hoc = analysis.post_hoc_power(0.5, 64);
        assert!(
            (post_hoc - 0.80).abs() < 0.05,
            "Post-hoc power should match planned"
        );

        // If observed effect is smaller, post-hoc power is lower
        let post_hoc_small = analysis.post_hoc_power(0.3, 64);
        assert!(
            post_hoc_small < 0.60,
            "Smaller observed effect means lower power"
        );
    }

    #[test]
    fn test_sensitivity_analysis() {
        let analysis = PowerAnalysis::new(0.05, 0.80, 0.5);
        let effect_sizes = vec![0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];

        let sensitivity = analysis.sensitivity_analysis(64, &effect_sizes);

        // Power should increase with effect size
        for i in 1..sensitivity.len() {
            assert!(
                sensitivity[i].1 >= sensitivity[i - 1].1,
                "Power should increase with effect size"
            );
        }

        // Check specific values
        let power_at_05 = sensitivity
            .iter()
            .find(|(d, _)| (*d - 0.5).abs() < 0.01)
            .map(|(_, p)| *p)
            .unwrap();
        assert!(
            (power_at_05 - 0.80).abs() < 0.05,
            "Power at d=0.5 should be ~0.80"
        );
    }
}
