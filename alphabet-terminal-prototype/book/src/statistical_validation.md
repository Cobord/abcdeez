# Statistical Validation

Statistical validation ensures that the cognitive models accurately capture learning patterns and make reliable predictions. This chapter covers the rigorous statistical methods used to validate model assumptions and performance.

## Hypothesis Testing Framework

### Model Comparison

```rust
pub struct ModelComparison {
    pub models: Vec<Box<dyn CognitiveModel>>,
    pub data: ResponseData,
}

impl ModelComparison {
    pub fn compare_models(&self) -> ComparisonResult {
        let mut results = Vec::new();
        
        for model in &self.models {
            let fit = ModelFit {
                log_likelihood: model.log_likelihood(&self.data),
                parameters: model.num_parameters(),
                n: self.data.len(),
            };
            
            results.push(ModelMetrics {
                aic: self.compute_aic(&fit),
                bic: self.compute_bic(&fit),
                dic: self.compute_dic(model, &self.data),
                waic: self.compute_waic(model, &self.data),
            });
        }
        
        ComparisonResult {
            best_model: self.select_best(&results),
            bayes_factors: self.compute_bayes_factors(&results),
            relative_weights: self.compute_akaike_weights(&results),
        }
    }
    
    fn compute_aic(&self, fit: &ModelFit) -> f64 {
        -2.0 * fit.log_likelihood + 2.0 * fit.parameters as f64
    }
    
    fn compute_bic(&self, fit: &ModelFit) -> f64 {
        -2.0 * fit.log_likelihood + fit.parameters as f64 * (fit.n as f64).ln()
    }
    
    fn compute_dic(&self, model: &dyn CognitiveModel, data: &ResponseData) -> f64 {
        // Deviance Information Criterion
        let mean_deviance = model.posterior_mean_deviance(data);
        let deviance_at_mean = model.deviance_at_posterior_mean(data);
        let p_d = mean_deviance - deviance_at_mean; // Effective parameters
        mean_deviance + p_d
    }
}
```

## Cross-Validation

### K-Fold Cross-Validation

```rust
pub struct CrossValidator {
    pub k_folds: usize,
    pub stratified: bool,
}

impl CrossValidator {
    pub fn validate(&self, model: &dyn CognitiveModel, data: &[ResponseData]) -> ValidationResult {
        let folds = self.create_folds(data);
        let mut metrics = Vec::new();
        
        for (i, test_fold) in folds.iter().enumerate() {
            // Create training set (all folds except i)
            let train_data: Vec<_> = folds.iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .flat_map(|(_, fold)| fold.clone())
                .collect();
            
            // Fit model on training data
            let fitted_model = model.fit(&train_data);
            
            // Evaluate on test fold
            let fold_metrics = FoldMetrics {
                accuracy: self.compute_accuracy(&fitted_model, test_fold),
                log_loss: self.compute_log_loss(&fitted_model, test_fold),
                calibration: self.compute_calibration(&fitted_model, test_fold),
                discrimination: self.compute_discrimination(&fitted_model, test_fold),
            };
            
            metrics.push(fold_metrics);
        }
        
        ValidationResult {
            mean_accuracy: mean(&metrics.iter().map(|m| m.accuracy).collect()),
            std_accuracy: std_dev(&metrics.iter().map(|m| m.accuracy).collect()),
            mean_log_loss: mean(&metrics.iter().map(|m| m.log_loss).collect()),
            calibration_slope: self.aggregate_calibration(&metrics),
            auc: self.aggregate_discrimination(&metrics),
        }
    }
    
    fn create_folds(&self, data: &[ResponseData]) -> Vec<Vec<ResponseData>> {
        if self.stratified {
            self.stratified_split(data)
        } else {
            self.random_split(data)
        }
    }
}
```

### Leave-One-Out Cross-Validation

```rust
impl CrossValidator {
    pub fn loocv(&self, model: &dyn CognitiveModel, data: &[ResponseData]) -> f64 {
        let mut log_predictive_density = 0.0;
        
        for i in 0..data.len() {
            // Train on all except i
            let train: Vec<_> = data.iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, d)| d.clone())
                .collect();
            
            let fitted = model.fit(&train);
            let pred = fitted.predict(&data[i].task);
            
            let actual = if data[i].correct { 1.0 } else { 0.0 };
            log_predictive_density += (pred * actual + (1.0 - pred) * (1.0 - actual)).ln();
        }
        
        log_predictive_density / data.len() as f64
    }
}
```

## Goodness-of-Fit Tests

### Chi-Square Test

```rust
pub struct GoodnessOfFit {
    pub observed: Vec<f64>,
    pub expected: Vec<f64>,
}

impl GoodnessOfFit {
    pub fn chi_square_test(&self) -> TestResult {
        let statistic: f64 = self.observed.iter()
            .zip(self.expected.iter())
            .map(|(o, e)| (o - e).powi(2) / e)
            .sum();
        
        let df = self.observed.len() - 1;
        let p_value = 1.0 - chi_squared_cdf(statistic, df);
        
        TestResult {
            statistic,
            p_value,
            reject_null: p_value < 0.05,
            effect_size: self.compute_cramers_v(statistic),
        }
    }
    
    fn compute_cramers_v(&self, chi_sq: f64) -> f64 {
        let n: f64 = self.observed.iter().sum();
        let k = self.observed.len();
        (chi_sq / (n * (k - 1) as f64)).sqrt()
    }
}
```

### Kolmogorov-Smirnov Test

```rust
impl GoodnessOfFit {
    pub fn ks_test(&self, data: &[f64], theoretical: &dyn Distribution) -> TestResult {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let n = sorted.len();
        let mut max_diff = 0.0;
        
        for (i, &value) in sorted.iter().enumerate() {
            let empirical_cdf = (i + 1) as f64 / n as f64;
            let theoretical_cdf = theoretical.cdf(value);
            let diff = (empirical_cdf - theoretical_cdf).abs();
            max_diff = max_diff.max(diff);
        }
        
        let statistic = max_diff * (n as f64).sqrt();
        let p_value = self.ks_p_value(statistic);
        
        TestResult {
            statistic,
            p_value,
            reject_null: p_value < 0.05,
            effect_size: max_diff,
        }
    }
}
```

## Calibration Analysis

Model predictions should match observed frequencies:

```rust
pub struct CalibrationAnalyzer {
    pub n_bins: usize,
}

impl CalibrationAnalyzer {
    pub fn analyze(&self, predictions: &[f64], outcomes: &[bool]) -> CalibrationResult {
        // Bin predictions
        let bins = self.create_bins(predictions);
        let mut calibration_data = Vec::new();
        
        for bin in bins {
            let bin_predictions: Vec<_> = bin.indices.iter()
                .map(|&i| predictions[i])
                .collect();
            let bin_outcomes: Vec<_> = bin.indices.iter()
                .map(|&i| outcomes[i])
                .collect();
            
            let mean_predicted = mean(&bin_predictions);
            let observed_frequency = bin_outcomes.iter()
                .filter(|&&o| o)
                .count() as f64 / bin_outcomes.len() as f64;
            
            calibration_data.push((mean_predicted, observed_frequency));
        }
        
        // Fit calibration curve
        let regression = linear_regression(
            &calibration_data.iter().map(|p| p.0).collect::<Vec<_>>(),
            &calibration_data.iter().map(|p| p.1).collect::<Vec<_>>()
        );
        
        // Compute calibration metrics
        CalibrationResult {
            calibration_slope: regression.slope,
            calibration_intercept: regression.intercept,
            expected_calibration_error: self.compute_ece(&calibration_data),
            maximum_calibration_error: self.compute_mce(&calibration_data),
            brier_score: self.compute_brier_score(predictions, outcomes),
        }
    }
    
    fn compute_ece(&self, data: &[(f64, f64)]) -> f64 {
        data.iter()
            .map(|(pred, obs)| (pred - obs).abs())
            .sum::<f64>() / data.len() as f64
    }
    
    fn compute_brier_score(&self, predictions: &[f64], outcomes: &[bool]) -> f64 {
        predictions.iter()
            .zip(outcomes)
            .map(|(p, o)| {
                let y = if *o { 1.0 } else { 0.0 };
                (p - y).powi(2)
            })
            .sum::<f64>() / predictions.len() as f64
    }
}
```

## Residual Analysis

Check model assumptions through residuals:

```rust
pub struct ResidualAnalyzer {
    pub model: Box<dyn CognitiveModel>,
}

impl ResidualAnalyzer {
    pub fn analyze(&self, data: &[ResponseData]) -> ResidualDiagnostics {
        let predictions = self.model.predict_all(data);
        let residuals = self.compute_residuals(data, &predictions);
        
        ResidualDiagnostics {
            normality: self.test_normality(&residuals),
            homoscedasticity: self.test_homoscedasticity(&residuals, &predictions),
            autocorrelation: self.test_autocorrelation(&residuals),
            outliers: self.detect_outliers(&residuals),
        }
    }
    
    fn test_normality(&self, residuals: &[f64]) -> NormalityTest {
        // Shapiro-Wilk test
        let w_statistic = shapiro_wilk(residuals);
        let p_value = shapiro_wilk_p_value(w_statistic, residuals.len());
        
        NormalityTest {
            statistic: w_statistic,
            p_value,
            qq_plot: self.generate_qq_plot(residuals),
            skewness: skewness(residuals),
            kurtosis: kurtosis(residuals),
        }
    }
    
    fn test_homoscedasticity(&self, residuals: &[f64], fitted: &[f64]) -> HomoscedasticityTest {
        // Breusch-Pagan test
        let squared_residuals: Vec<_> = residuals.iter()
            .map(|r| r.powi(2))
            .collect();
        
        let regression = linear_regression(fitted, &squared_residuals);
        let n = residuals.len();
        let r_squared = regression.r_squared;
        
        let lm_statistic = n as f64 * r_squared;
        let p_value = 1.0 - chi_squared_cdf(lm_statistic, 1);
        
        HomoscedasticityTest {
            statistic: lm_statistic,
            p_value,
            reject_null: p_value < 0.05,
        }
    }
    
    fn test_autocorrelation(&self, residuals: &[f64]) -> AutocorrelationTest {
        // Durbin-Watson test
        let mut sum_sq_diff = 0.0;
        let mut sum_sq = 0.0;
        
        for i in 1..residuals.len() {
            sum_sq_diff += (residuals[i] - residuals[i-1]).powi(2);
        }
        
        for r in residuals {
            sum_sq += r.powi(2);
        }
        
        let dw_statistic = sum_sq_diff / sum_sq;
        
        AutocorrelationTest {
            durbin_watson: dw_statistic,
            significant: dw_statistic < 1.5 || dw_statistic > 2.5,
            acf: self.compute_acf(residuals),
            pacf: self.compute_pacf(residuals),
        }
    }
}
```

## Bootstrap Validation

Non-parametric validation through resampling:

```rust
pub struct BootstrapValidator {
    pub n_bootstrap: usize,
    pub sample_size: Option<usize>,
}

impl BootstrapValidator {
    pub fn validate(&self, model: &dyn CognitiveModel, data: &[ResponseData]) -> BootstrapResult {
        let mut metrics = Vec::new();
        
        for _ in 0..self.n_bootstrap {
            let sample = self.bootstrap_sample(data);
            let fitted = model.fit(&sample);
            
            // Out-of-bag evaluation
            let oob_indices = self.get_oob_indices(data.len(), &sample);
            let oob_data: Vec<_> = oob_indices.iter()
                .map(|&i| data[i].clone())
                .collect();
            
            if !oob_data.is_empty() {
                metrics.push(BootstrapMetric {
                    accuracy: self.compute_accuracy(&fitted, &oob_data),
                    parameters: fitted.get_parameters(),
                });
            }
        }
        
        // Compute confidence intervals
        let accuracies: Vec<_> = metrics.iter().map(|m| m.accuracy).collect();
        let param_estimates = self.aggregate_parameters(&metrics);
        
        BootstrapResult {
            mean_accuracy: mean(&accuracies),
            ci_95: (percentile(&accuracies, 0.025), percentile(&accuracies, 0.975)),
            parameter_ci: self.compute_parameter_ci(&param_estimates),
            bias: self.compute_bootstrap_bias(model, data, &metrics),
        }
    }
    
    fn bootstrap_sample(&self, data: &[ResponseData]) -> Vec<ResponseData> {
        let n = self.sample_size.unwrap_or(data.len());
        let mut rng = thread_rng();
        
        (0..n)
            .map(|_| data[rng.gen_range(0..data.len())].clone())
            .collect()
    }
}
```

## Permutation Tests

Test specific hypotheses without distributional assumptions:

```rust
pub struct PermutationTest {
    pub n_permutations: usize,
}

impl PermutationTest {
    pub fn test_difference(&self, group1: &[f64], group2: &[f64]) -> TestResult {
        let observed_diff = mean(group1) - mean(group2);
        let mut null_distribution = Vec::new();
        
        let combined: Vec<_> = group1.iter().chain(group2.iter()).cloned().collect();
        
        for _ in 0..self.n_permutations {
            let shuffled = self.shuffle(&combined);
            let perm_group1 = &shuffled[..group1.len()];
            let perm_group2 = &shuffled[group1.len()..];
            
            let perm_diff = mean(perm_group1) - mean(perm_group2);
            null_distribution.push(perm_diff);
        }
        
        let p_value = null_distribution.iter()
            .filter(|&&d| d.abs() >= observed_diff.abs())
            .count() as f64 / self.n_permutations as f64;
        
        TestResult {
            statistic: observed_diff,
            p_value,
            reject_null: p_value < 0.05,
            effect_size: observed_diff / pooled_std(group1, group2),
        }
    }
}
```

## Power Analysis

Determine sample size requirements:

```rust
pub struct PowerAnalyzer {
    pub alpha: f64,
    pub power: f64,
}

impl PowerAnalyzer {
    pub fn compute_sample_size(&self, effect_size: f64) -> usize {
        // For two-sample t-test
        let z_alpha = normal_ppf(1.0 - self.alpha / 2.0);
        let z_beta = normal_ppf(self.power);
        
        let n = 2.0 * ((z_alpha + z_beta) / effect_size).powi(2);
        n.ceil() as usize
    }
    
    pub fn compute_power(&self, n: usize, effect_size: f64) -> f64 {
        let z_alpha = normal_ppf(1.0 - self.alpha / 2.0);
        let z = effect_size * (n as f64 / 2.0).sqrt() - z_alpha;
        normal_cdf(z)
    }
    
    pub fn sensitivity_analysis(&self, n_range: Range<usize>, effect_range: Range<f64>) -> PowerCurve {
        let mut results = Vec::new();
        
        for n in n_range {
            for effect in effect_range.step_by(0.1) {
                let power = self.compute_power(n, effect);
                results.push((n, effect, power));
            }
        }
        
        PowerCurve { results }
    }
}
```

## Summary

Statistical validation ensures:
- **Model selection**: AIC, BIC, DIC, WAIC comparisons
- **Predictive accuracy**: Cross-validation, bootstrap validation
- **Goodness-of-fit**: Chi-square, KS tests
- **Calibration**: Predictions match observed frequencies
- **Residual diagnostics**: Check model assumptions
- **Power analysis**: Adequate sample sizes

These rigorous validation methods ensure the cognitive models are both statistically sound and practically useful.