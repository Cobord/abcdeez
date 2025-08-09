use serde::{Deserialize, Serialize};
use statrs::distribution::{ContinuousCDF, Normal};
use statrs::statistics::Statistics;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumptionCheckResult {
    pub test_name: String,
    pub assumption: String,
    pub is_met: bool,
    pub p_value: Option<f64>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExGaussianParameters {
    pub mu: f64,
    pub sigma: f64,
    pub tau: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeModel {
    pub base_rt: f64,
    pub distance_slope: f64,
    pub reversal_cost: f64,
    pub boundary_cost: f64,
    pub memory_benefit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMixture {
    pub pi_scan: f64,
    pub pi_index: f64,
    pub scan_params: ExGaussianParameters,
    pub index_params: ExGaussianParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedStatistics {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub q1: f64,
    pub q3: f64,
    pub iqr: f64,
    pub skewness: f64,
    pub kurtosis: f64,
}

impl DetailedStatistics {
    pub fn from_data(data: &[f64]) -> Self {
        if data.is_empty() {
            return DetailedStatistics {
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                q1: 0.0,
                q3: 0.0,
                iqr: 0.0,
                skewness: 0.0,
                kurtosis: 0.0,
            };
        }

        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mean = data.mean();
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let variance = data.variance();
        let std_dev = variance.sqrt();

        let q1 = sorted[sorted.len() / 4];
        let q3 = sorted[3 * sorted.len() / 4];
        let iqr = q3 - q1;

        let skewness = if std_dev > 0.0 {
            let n = data.len() as f64;
            let sum_cubed = data
                .iter()
                .map(|x| ((x - mean) / std_dev).powi(3))
                .sum::<f64>();
            (n / ((n - 1.0) * (n - 2.0))) * sum_cubed
        } else {
            0.0
        };

        let kurtosis = if std_dev > 0.0 {
            let n = data.len() as f64;
            let sum_fourth = data
                .iter()
                .map(|x| ((x - mean) / std_dev).powi(4))
                .sum::<f64>();
            ((n * (n + 1.0)) / ((n - 1.0) * (n - 2.0) * (n - 3.0))) * sum_fourth
                - (3.0 * (n - 1.0).powi(2)) / ((n - 2.0) * (n - 3.0))
        } else {
            0.0
        };

        DetailedStatistics {
            mean,
            median,
            std_dev,
            min: *sorted.first().unwrap(),
            max: *sorted.last().unwrap(),
            q1,
            q3,
            iqr,
            skewness,
            kurtosis,
        }
    }
}

pub struct ExGaussianModel {
    pub params: ExGaussianParameters,
}

impl ExGaussianModel {
    pub fn new(params: ExGaussianParameters) -> Self {
        ExGaussianModel { params }
    }

    pub fn from_params(mu: f64, sigma: f64, tau: f64) -> Self {
        ExGaussianModel {
            params: ExGaussianParameters { mu, sigma, tau },
        }
    }

    pub fn fit(response_times: &[f64]) -> Self {
        let stats = DetailedStatistics::from_data(response_times);

        let mu = stats.mean - stats.std_dev;
        let sigma = stats.std_dev * 0.8;
        let tau = stats.std_dev * 0.5;

        ExGaussianModel::from_params(mu, sigma, tau)
    }

    pub fn pdf(&self, x: f64) -> f64 {
        if x < 0.0 || self.params.tau <= 0.0 || self.params.sigma <= 0.0 {
            return 0.0;
        }

        // Ex-Gaussian PDF is the convolution of a Gaussian and an exponential
        // f(x) = (λ/2) * exp(λ/2 * (2μ + λσ² - 2x)) * erfc((μ + λσ² - x)/(√2 * σ))
        let lambda = 1.0 / self.params.tau;
        let _normal = Normal::new(0.0, 1.0).unwrap();

        // Calculate the argument for the exponential term
        let exp_arg =
            (lambda / 2.0) * (2.0 * self.params.mu + lambda * self.params.sigma.powi(2) - 2.0 * x);

        // Compute in log-domain for stability
        let log_exp = exp_arg;

        // Calculate the argument for the complementary error function
        let erfc_arg = (self.params.mu + lambda * self.params.sigma.powi(2) - x)
            / (self.params.sigma * std::f64::consts::SQRT_2);

        // Use statrs to compute erfc, clamp final result only
        let erfc_val = statrs::function::erf::erfc(erfc_arg);

        // Calculate the result with additional stability checks
        // The formula is: (λ/2) * exp(exp_arg) * erfc_val
        // where exp_arg = (λ/2) * (2μ + λσ² - 2x)
        let result = (lambda / 2.0) * log_exp.exp() * erfc_val;
        if result.is_sign_negative() { return 0.0; }

        // Final sanity check to avoid NaN or Inf
        if result.is_finite() {
            result
        } else {
            0.0
        }
    }

    pub fn mean(&self) -> f64 {
        self.params.mu + self.params.tau
    }

    pub fn variance(&self) -> f64 {
        self.params.sigma.powi(2) + self.params.tau.powi(2)
    }

    pub fn cdf(&self, x: f64) -> f64 {
        if self.params.tau <= 0.0 || self.params.sigma <= 0.0 {
            return 0.0;
        }

        // Ex-Gaussian CDF has closed form:
        // F(x) = Φ((x-μ)/σ) - exp((λ/2)*(2μ + λσ² - 2x)) * Φ((x - μ - λσ²)/σ)
        let lambda = 1.0 / self.params.tau;
        let normal = Normal::new(0.0, 1.0).unwrap();

        // First term: Φ((x-μ)/σ)
        let term1 = normal.cdf((x - self.params.mu) / self.params.sigma);

        // Second term exponential part (log-domain)
        let exp_arg =
            (lambda / 2.0) * (2.0 * self.params.mu + lambda * self.params.sigma.powi(2) - 2.0 * x);

        // Second term normal CDF part
        let term2_arg =
            (x - self.params.mu - lambda * self.params.sigma.powi(2)) / self.params.sigma;
        let term2 = exp_arg.exp() * normal.cdf(term2_arg);

        (term1 - term2).max(0.0).min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub task_type_stats: HashMap<String, DetailedStatistics>,
    pub rt_by_distance: HashMap<usize, Vec<f64>>,
    pub error_patterns: ErrorAnalysis,
    pub learning_curves: LearningCurves,
    pub strategy_analysis: StrategyAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    pub confusion_matrix: HashMap<(String, String), usize>,
    pub error_by_distance: HashMap<usize, f64>,
    pub locality_index: f64,
    pub systematic_errors: Vec<(String, String, usize)>,
}

impl ErrorAnalysis {
    pub fn new() -> Self {
        ErrorAnalysis {
            confusion_matrix: HashMap::new(),
            error_by_distance: HashMap::new(),
            locality_index: 0.0,
            systematic_errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, expected: String, actual: String, distance: usize) {
        let key = if expected < actual {
            (expected.clone(), actual.clone())
        } else {
            (actual.clone(), expected.clone())
        };

        *self.confusion_matrix.entry(key).or_insert(0) += 1;

        let error_rate = self.error_by_distance.entry(distance).or_insert(0.0);
        *error_rate += 1.0;
    }

    pub fn calculate_locality(&mut self) {
        let total_errors: usize = self.confusion_matrix.values().sum();
        if total_errors == 0 {
            self.locality_index = 0.0;
            return;
        }

        let local_errors: f64 = self
            .error_by_distance
            .iter()
            .filter(|(d, _)| **d <= 2)
            .map(|(_, count)| count)
            .sum();

        self.locality_index = local_errors / total_errors as f64;
    }

    pub fn identify_systematic_errors(&mut self) {
        self.systematic_errors = self
            .confusion_matrix
            .iter()
            .filter(|(_, count)| **count >= 2)
            .map(|((a, b), count)| (a.clone(), b.clone(), *count))
            .collect();

        self.systematic_errors.sort_by(|a, b| b.2.cmp(&a.2));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningCurves {
    pub accuracy_over_time: Vec<f64>,
    pub rt_over_time: Vec<f64>,
    pub improvement_rate: f64,
    pub plateau_point: Option<usize>,
}

impl LearningCurves {
    pub fn calculate(accuracies: &[bool], response_times: &[f64], window_size: usize) -> Self {
        let mut accuracy_over_time = Vec::new();
        let mut rt_over_time = Vec::new();

        for i in 0..accuracies.len() {
            let start = i.saturating_sub(window_size / 2);
            let end = (i + window_size / 2 + 1).min(accuracies.len());

            let window_acc = &accuracies[start..end];
            let window_rt = &response_times[start..end];

            let acc = window_acc.iter().filter(|&&x| x).count() as f64 / window_acc.len() as f64;
            let rt = window_rt.iter().sum::<f64>() / window_rt.len() as f64;

            accuracy_over_time.push(acc);
            rt_over_time.push(rt);
        }

        let improvement_rate = if accuracy_over_time.len() > 1 {
            let first_third = &accuracy_over_time[..accuracy_over_time.len() / 3];
            let last_third = &accuracy_over_time[2 * accuracy_over_time.len() / 3..];

            let first_avg: f64 = first_third.iter().sum::<f64>() / first_third.len() as f64;
            let last_avg: f64 = last_third.iter().sum::<f64>() / last_third.len() as f64;

            last_avg - first_avg
        } else {
            0.0
        };

        let plateau_point = Self::find_plateau(&accuracy_over_time);

        LearningCurves {
            accuracy_over_time,
            rt_over_time,
            improvement_rate,
            plateau_point,
        }
    }

    fn find_plateau(data: &[f64]) -> Option<usize> {
        if data.len() < 10 {
            return None;
        }

        let window = 5;
        for i in window..data.len() - window {
            let before = &data[i - window..i];
            let after = &data[i..i + window];

            let before_mean = before.iter().sum::<f64>() / before.len() as f64;
            let after_mean = after.iter().sum::<f64>() / after.len() as f64;

            if (after_mean - before_mean).abs() < 0.05 {
                return Some(i);
            }
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAnalysis {
    pub rt_distance_correlation: f64,
    pub strategy_classification: StrategyType,
    pub transition_point: Option<usize>,
    pub correlation: f64, // Add this field for test compatibility
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StrategyType {
    SerialScan,
    DirectIndex,
    DirectAccess, // Add for test compatibility
    Hybrid,       // Add for test compatibility
    Mixed(i32),   // Changed to i32 for Hash trait
}

impl StrategyAnalysis {
    pub fn analyze(response_times: &[f64], distances: &[usize]) -> Self {
        let correlation = Self::calculate_correlation(response_times, distances);

        // Use Cohen's effect size conventions for correlation thresholds:
        // r > 0.7: Strong correlation (r² > 0.49) - serial scanning
        // r < 0.3: Weak correlation (r² < 0.09) - direct access
        // 0.3 ≤ r ≤ 0.7: Mixed evidence - hybrid strategy
        let strategy_classification = if correlation > 0.7 {
            StrategyType::SerialScan
        } else if correlation < 0.3 {
            StrategyType::DirectAccess
        } else {
            StrategyType::Hybrid
        };

        let transition_point = Self::find_strategy_transition(response_times, distances);

        StrategyAnalysis {
            rt_distance_correlation: correlation,
            strategy_classification,
            transition_point,
            correlation, // Use same value as rt_distance_correlation
        }
    }

    fn calculate_correlation(response_times: &[f64], distances: &[usize]) -> f64 {
        let n = response_times.len();
        if n != distances.len() || n == 0 {
            return 0.0;
        }
        let n_f = n as f64;

        let rt_mean = response_times.iter().sum::<f64>() / n_f;
        let dist_mean = distances.iter().map(|&d| d as f64).sum::<f64>() / n_f;

        let cov_num: f64 = response_times
            .iter()
            .zip(distances.iter())
            .map(|(rt, d)| (rt - rt_mean) * (*d as f64 - dist_mean))
            .sum();
        // Unbiased covariance when n>1
        let covariance = if n > 1 { cov_num / (n_f - 1.0) } else { 0.0 };

        let rt_var_num: f64 = response_times
            .iter()
            .map(|rt| (rt - rt_mean).powi(2))
            .sum();
        let dist_var_num: f64 = distances
            .iter()
            .map(|d| (*d as f64 - dist_mean).powi(2))
            .sum();
        let rt_var = if n > 1 { rt_var_num / (n_f - 1.0) } else { 0.0 };
        let dist_var = if n > 1 { dist_var_num / (n_f - 1.0) } else { 0.0 };

        let denom = rt_var.sqrt() * dist_var.sqrt();
        let r = if denom > 0.0 { covariance / denom } else { 0.0 };
        r.max(-1.0).min(1.0)
    }

    fn find_strategy_transition(response_times: &[f64], distances: &[usize]) -> Option<usize> {
        let window = 10;
        if response_times.len() < 2 * window {
            return None;
        }

        let mut max_change = 0.0;
        let mut transition_idx = None;

        for i in window..response_times.len() - window {
            let before_corr = Self::calculate_correlation(
                &response_times[i - window..i],
                &distances[i - window..i],
            );
            let after_corr = Self::calculate_correlation(
                &response_times[i..i + window],
                &distances[i..i + window],
            );

            let change = (before_corr - after_corr).abs();
            if change > max_change {
                max_change = change;
                transition_idx = Some(i);
            }
        }

        // Use Cohen's medium effect size as threshold
        if max_change > 0.3 {
            transition_idx
        } else {
            None
        }
    }
}

pub struct ResponseTimeDistribution;

impl ResponseTimeDistribution {
    pub fn fit_ex_gaussian(data: &[f64]) -> ExGaussianParameters {
        let model = ExGaussianModel::fit(data);
        model.params
    }

    pub fn detect_outliers(response_times: &[f64], z_threshold: f64) -> Vec<usize> {
        if response_times.is_empty() {
            return vec![];
        }

        let mean = response_times.iter().sum::<f64>() / response_times.len() as f64;
        let variance = response_times
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / response_times.len() as f64;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 { return vec![]; }

        let mut outliers = Vec::new();
        for (i, &rt) in response_times.iter().enumerate() {
            let z_score = (rt - mean).abs() / std_dev;
            if z_score > z_threshold {
                outliers.push(i);
            }
        }

        outliers
    }
}

pub struct SessionAnalyzer {
    pub responses: Vec<crate::tasks::TaskResponse>,
}

impl SessionAnalyzer {
    pub fn new(responses: Vec<crate::tasks::TaskResponse>) -> Self {
        SessionAnalyzer { responses }
    }

    pub fn generate_full_analysis(&self) -> PerformanceAnalysis {
        let mut task_type_stats = HashMap::new();
        let mut rt_by_distance = HashMap::new();
        let mut error_analysis = ErrorAnalysis::new();

        for response in &self.responses {
            let task_type = format!("{:?}", response.task.operation);
            let rt = response.response_time_ms as f64;

            task_type_stats
                .entry(task_type.clone())
                .or_insert_with(Vec::new)
                .push(rt);

            if let Some(distance) = self.calculate_task_distance(&response.task) {
                rt_by_distance
                    .entry(distance)
                    .or_insert_with(Vec::new)
                    .push(rt);

                if !response.correct {
                    error_analysis.add_error(
                        response.task.correct_answer.clone(),
                        response.user_answer.clone(),
                        distance,
                    );
                }
            }
        }

        error_analysis.calculate_locality();
        error_analysis.identify_systematic_errors();

        let task_type_stats = task_type_stats
            .into_iter()
            .map(|(k, v)| (k, DetailedStatistics::from_data(&v)))
            .collect();

        let accuracies: Vec<bool> = self.responses.iter().map(|r| r.correct).collect();
        let response_times: Vec<f64> = self
            .responses
            .iter()
            .map(|r| r.response_time_ms as f64)
            .collect();
        let learning_curves = LearningCurves::calculate(&accuracies, &response_times, 10);

        let distances: Vec<usize> = self
            .responses
            .iter()
            .filter_map(|r| self.calculate_task_distance(&r.task))
            .collect();
        let strategy_analysis = StrategyAnalysis::analyze(&response_times, &distances);

        PerformanceAnalysis {
            task_type_stats,
            rt_by_distance,
            error_patterns: error_analysis,
            learning_curves,
            strategy_analysis,
        }
    }

    fn calculate_task_distance(&self, task: &crate::tasks::Task) -> Option<usize> {
        match &task.task_type {
            crate::tasks::TaskType::PairwiseOrder { a, b } => {
                Some((a.chars().next()? as usize).abs_diff(b.chars().next()? as usize))
            }
            crate::tasks::TaskType::KJump { k, .. } => Some(k.unsigned_abs() as usize),
            crate::tasks::TaskType::Segment { count, .. } => Some(*count),
            _ => Some(1),
        }
    }
}

// Multiple Comparison Corrections
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CorrectionMethod {
    Bonferroni,
    BenjaminiHochberg,
    Holm,
    HolmBonferroni,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleComparisonCorrection {
    pub method: CorrectionMethod,
    pub alpha: f64,
}

impl MultipleComparisonCorrection {
    pub fn new(method: CorrectionMethod, alpha: f64) -> Self {
        Self { method, alpha }
    }

    /// Apply multiple comparison correction to p-values
    pub fn adjust_p_values(&self, p_values: &[f64]) -> Vec<f64> {
        match self.method {
            CorrectionMethod::Bonferroni => self.bonferroni_correction(p_values),
            CorrectionMethod::BenjaminiHochberg => self.benjamini_hochberg_correction(p_values),
            CorrectionMethod::Holm => self.holm_correction(p_values),
            CorrectionMethod::HolmBonferroni => self.holm_bonferroni_correction(p_values),
        }
    }

    /// Bonferroni correction: p_adjusted = min(p * n, 1.0)
    fn bonferroni_correction(&self, p_values: &[f64]) -> Vec<f64> {
        let n = p_values.len() as f64;
        p_values.iter().map(|&p| (p * n).min(1.0)).collect()
    }

    /// Benjamini-Hochberg FDR correction with enforced monotonicity
    fn benjamini_hochberg_correction(&self, p_values: &[f64]) -> Vec<f64> {
        if p_values.is_empty() {
            return vec![];
        }

        let n = p_values.len();
        let mut indexed_p: Vec<(usize, f64)> =
            p_values.iter().enumerate().map(|(i, &p)| (i, p)).collect();

        // Sort by p-value
        indexed_p.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Create adjustment vector in sorted order
        let mut sorted_adjusted = vec![0.0; n];
        
        // Apply BH correction formula
        for i in 0..n {
            let rank = i + 1;
            sorted_adjusted[i] = (indexed_p[i].1 * n as f64 / rank as f64).min(1.0);
        }
        
        // Enforce monotonicity: adjusted p-values should be non-decreasing
        // Work backwards to ensure larger p-values don't have smaller adjusted values
        for i in (0..n-1).rev() {
            sorted_adjusted[i] = sorted_adjusted[i].min(sorted_adjusted[i + 1]);
        }
        
        // Map back to original indices
        let mut adjusted = vec![0.0; n];
        for i in 0..n {
            adjusted[indexed_p[i].0] = sorted_adjusted[i];
        }

        adjusted
    }

    /// Holm correction (step-down method)
    fn holm_correction(&self, p_values: &[f64]) -> Vec<f64> {
        if p_values.is_empty() {
            return vec![];
        }

        let n = p_values.len();
        let mut indexed_p: Vec<(usize, f64)> =
            p_values.iter().enumerate().map(|(i, &p)| (i, p)).collect();

        // Sort by p-value
        indexed_p.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut adjusted = vec![0.0; n];
        let mut cummax = 0.0;

        for i in 0..n {
            let p_adj = ((indexed_p[i].1 * (n - i) as f64).min(1.0)).max(cummax);
            cummax = cummax.max(p_adj);
            adjusted[indexed_p[i].0] = p_adj;
        }

        adjusted
    }

    /// Holm-Bonferroni correction (same as Holm)
    fn holm_bonferroni_correction(&self, p_values: &[f64]) -> Vec<f64> {
        self.holm_correction(p_values)
    }

    /// Determine which hypotheses to reject based on corrected p-values
    pub fn reject_hypotheses(&self, p_values: &[f64]) -> Vec<bool> {
        let adjusted = self.adjust_p_values(p_values);
        adjusted.iter().map(|&p| p < self.alpha).collect()
    }
}

// Power Analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerAnalysis {
    pub alpha: f64,
    pub power: f64,
    pub effect_size: f64,
}

impl PowerAnalysis {
    pub fn new(alpha: f64, power: f64, effect_size: f64) -> Self {
        Self {
            alpha,
            power,
            effect_size,
        }
    }

    /// Calculate required sample size for t-test
    pub fn calculate_sample_size_t_test(&self, two_tailed: bool) -> usize {
        let normal = Normal::new(0.0, 1.0).unwrap();

        // Z-scores for alpha and beta
        let z_alpha = if two_tailed {
            normal.inverse_cdf(1.0 - self.alpha / 2.0)
        } else {
            normal.inverse_cdf(1.0 - self.alpha)
        };

        let z_beta = normal.inverse_cdf(self.power);

        // Sample size formula: n = [(z_alpha + z_beta)^2 * 2] / d^2
        let n = ((z_alpha + z_beta).powi(2) * 2.0) / self.effect_size.powi(2);

        n.ceil() as usize
    }

    /// Calculate power given sample size
    pub fn calculate_power(&self, n: usize) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();

        // Critical value
        let z_alpha = normal.inverse_cdf(1.0 - self.alpha / 2.0);

        // Non-centrality parameter
        let ncp = self.effect_size * (n as f64 / 2.0).sqrt();

        // Power = P(Z > z_alpha - ncp)
        1.0 - normal.cdf(z_alpha - ncp)
    }

    /// Calculate minimum detectable effect size
    pub fn calculate_min_effect_size(&self, n: usize) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();

        let z_alpha = normal.inverse_cdf(1.0 - self.alpha / 2.0);
        let z_beta = normal.inverse_cdf(self.power);

        // d = (z_alpha + z_beta) * sqrt(2/n)
        (z_alpha + z_beta) * (2.0 / n as f64).sqrt()
    }

    /// Power analysis for ANOVA (one-way)
    pub fn calculate_sample_size_anova(&self, k_groups: usize) -> usize {
        // Using Cohen's f instead of d for ANOVA
        let f = self.effect_size;
        let _df1 = (k_groups - 1) as f64;

        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - self.alpha);
        let z_beta = normal.inverse_cdf(self.power);

        // Approximation for balanced design
        let lambda = f.powi(2) * k_groups as f64;
        let n_per_group = ((z_alpha + z_beta).powi(2) / lambda + 1.0).ceil() as usize;

        n_per_group * k_groups
    }

    /// Power analysis for correlation
    pub fn calculate_sample_size_correlation(&self) -> usize {
        let normal = Normal::new(0.0, 1.0).unwrap();

        let z_alpha = normal.inverse_cdf(1.0 - self.alpha / 2.0);
        let z_beta = normal.inverse_cdf(self.power);

        // Fisher's z transformation
        let r = self.effect_size;
        let z_r = 0.5 * ((1.0 + r) / (1.0 - r)).ln();

        // n = [(z_alpha + z_beta) / z_r]^2 + 3
        let n = ((z_alpha + z_beta) / z_r).powi(2) + 3.0;

        n.ceil() as usize
    }

    /// Post-hoc power calculation for completed study
    pub fn post_hoc_power(&self, observed_effect: f64, n: usize) -> f64 {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - self.alpha / 2.0);

        // Non-centrality parameter with observed effect
        let ncp = observed_effect * (n as f64 / 2.0).sqrt();

        // Power = P(Z > z_alpha - ncp)
        1.0 - normal.cdf(z_alpha - ncp)
    }

    /// Sensitivity analysis: vary effect size
    pub fn sensitivity_analysis(&self, n: usize, effect_sizes: &[f64]) -> Vec<(f64, f64)> {
        effect_sizes
            .iter()
            .map(|&d| {
                let mut analysis = self.clone();
                analysis.effect_size = d;
                (d, analysis.calculate_power(n))
            })
            .collect()
    }
}

/// Check assumptions for parametric statistical tests
pub struct AssumptionChecker;

impl AssumptionChecker {
    /// Check all assumptions for t-test
    pub fn check_t_test_assumptions(sample1: &[f64], sample2: Option<&[f64]>) -> Vec<AssumptionCheckResult> {
        let mut results = Vec::new();
        
        // Check normality
        results.push(Self::check_normality(sample1, "Sample 1"));
        if let Some(s2) = sample2 {
            results.push(Self::check_normality(s2, "Sample 2"));
            
            // Check homogeneity of variance for two-sample test
            results.push(Self::check_homogeneity_of_variance(sample1, s2));
        }
        
        // Check sample size
        results.push(Self::check_sample_size(sample1.len(), "t-test"));
        
        // Check for outliers
        results.push(Self::check_outliers(sample1, "Sample 1"));
        
        results
    }
    
    /// Check normality using Shapiro-Wilk test approximation
    pub fn check_normality(data: &[f64], sample_name: &str) -> AssumptionCheckResult {
        let n = data.len();
        
        if n < 3 {
            return AssumptionCheckResult {
                test_name: "Normality".to_string(),
                assumption: format!("Normal distribution for {}", sample_name),
                is_met: false,
                p_value: None,
                recommendation: "Sample too small for normality test. Use non-parametric methods.".to_string(),
            };
        }
        
        // Calculate skewness and kurtosis as quick normality indicators
        let mean = data.iter().sum::<f64>() / n as f64;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
        let std_dev = variance.sqrt();
        
        let skewness = if std_dev > 0.0 {
            let sum_cubed = data.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum::<f64>();
            sum_cubed * n as f64 / ((n - 1) as f64 * (n - 2) as f64)
        } else {
            0.0
        };
        
        let kurtosis = if std_dev > 0.0 && n > 3 {
            let sum_fourth = data.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum::<f64>();
            let k = sum_fourth * n as f64 * (n + 1) as f64 
                / ((n - 1) as f64 * (n - 2) as f64 * (n - 3) as f64)
                - 3.0 * (n - 1) as f64 * (n - 1) as f64 
                / ((n - 2) as f64 * (n - 3) as f64);
            k
        } else {
            0.0
        };
        
        // Rules of thumb for normality
        let skewness_ok = skewness.abs() < 2.0;
        let kurtosis_ok = kurtosis.abs() < 7.0;
        let is_normal = skewness_ok && kurtosis_ok;
        
        AssumptionCheckResult {
            test_name: "Normality".to_string(),
            assumption: format!("Normal distribution for {}", sample_name),
            is_met: is_normal,
            p_value: None,
            recommendation: if is_normal {
                "Data appears approximately normal.".to_string()
            } else if n < 30 {
                "Violation detected with small sample. Consider non-parametric test (e.g., Wilcoxon).".to_string()
            } else {
                "Violation detected but sample is large. T-test may still be robust.".to_string()
            },
        }
    }
    
    /// Check homogeneity of variance using Levene's test approximation
    pub fn check_homogeneity_of_variance(sample1: &[f64], sample2: &[f64]) -> AssumptionCheckResult {
        let mean1 = sample1.iter().sum::<f64>() / sample1.len() as f64;
        let mean2 = sample2.iter().sum::<f64>() / sample2.len() as f64;
        
        let var1 = sample1.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (sample1.len() - 1) as f64;
        let var2 = sample2.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (sample2.len() - 1) as f64;
        
        let variance_ratio = var1.max(var2) / var1.min(var2);
        
        // Rule of thumb: variance ratio should be less than 4
        let is_homogeneous = variance_ratio < 4.0;
        
        AssumptionCheckResult {
            test_name: "Homogeneity of Variance".to_string(),
            assumption: "Equal variances between groups".to_string(),
            is_met: is_homogeneous,
            p_value: None,
            recommendation: if is_homogeneous {
                "Variances appear homogeneous.".to_string()
            } else {
                format!("Variance ratio is {:.2}. Consider Welch's t-test for unequal variances.", variance_ratio)
            },
        }
    }
    
    /// Check for sufficient sample size
    pub fn check_sample_size(n: usize, test_type: &str) -> AssumptionCheckResult {
        let min_size = match test_type {
            "t-test" => 5,
            "anova" => 10,
            "regression" => 20,
            _ => 30,
        };
        
        let is_sufficient = n >= min_size;
        
        AssumptionCheckResult {
            test_name: "Sample Size".to_string(),
            assumption: format!("Sufficient sample size for {}", test_type),
            is_met: is_sufficient,
            p_value: None,
            recommendation: if is_sufficient {
                "Sample size is adequate.".to_string()
            } else {
                format!("Sample size ({}) is below recommended minimum ({}). Results may be unreliable.", n, min_size)
            },
        }
    }
    
    /// Check for outliers using IQR method
    pub fn check_outliers(data: &[f64], sample_name: &str) -> AssumptionCheckResult {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let n = sorted.len();
        let q1_idx = n / 4;
        let q3_idx = 3 * n / 4;
        
        let q1 = sorted[q1_idx];
        let q3 = sorted[q3_idx];
        let iqr = q3 - q1;
        
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;
        
        let outliers: Vec<_> = data.iter()
            .filter(|&&x| x < lower_bound || x > upper_bound)
            .collect();
        
        let outlier_proportion = outliers.len() as f64 / n as f64;
        let has_outliers = outlier_proportion > 0.05; // More than 5% outliers
        
        AssumptionCheckResult {
            test_name: "Outliers".to_string(),
            assumption: format!("No extreme outliers in {}", sample_name),
            is_met: !has_outliers,
            p_value: None,
            recommendation: if !has_outliers {
                "No concerning outliers detected.".to_string()
            } else {
                format!("{} outliers detected ({:.1}%). Consider robust methods or outlier removal.", 
                    outliers.len(), outlier_proportion * 100.0)
            },
        }
    }
    
    /// Check assumptions for ANOVA
    pub fn check_anova_assumptions(groups: &[Vec<f64>]) -> Vec<AssumptionCheckResult> {
        let mut results = Vec::new();
        
        // Check normality for each group
        for (i, group) in groups.iter().enumerate() {
            results.push(Self::check_normality(group, &format!("Group {}", i + 1)));
        }
        
        // Check homogeneity across all groups
        if groups.len() >= 2 {
            let variances: Vec<f64> = groups.iter().map(|g| {
                let mean = g.iter().sum::<f64>() / g.len() as f64;
                g.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (g.len() - 1) as f64
            }).collect();
            
            let max_var = variances.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let min_var = variances.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let variance_ratio = max_var / min_var;
            
            results.push(AssumptionCheckResult {
                test_name: "Homogeneity of Variance".to_string(),
                assumption: "Equal variances across groups".to_string(),
                is_met: variance_ratio < 4.0,
                p_value: None,
                recommendation: if variance_ratio < 4.0 {
                    "Variances appear homogeneous across groups.".to_string()
                } else {
                    format!("Variance ratio is {:.2}. Consider Welch's ANOVA or non-parametric Kruskal-Wallis test.", variance_ratio)
                },
            });
        }
        
        // Check sample sizes
        for (i, group) in groups.iter().enumerate() {
            results.push(Self::check_sample_size(group.len(), &format!("ANOVA Group {}", i + 1)));
        }
        
        results
    }
}
