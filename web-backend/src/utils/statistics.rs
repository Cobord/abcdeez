/// Statistical utilities for data analysis
use super::math;
use statrs::distribution::{ContinuousCDF, FisherSnedecor, StudentsT};
use rand::Rng;

/// Calculate mean of a dataset
pub fn mean(data: &[f64]) -> f64 {
    math::safe_mean(data)
}

/// Calculate median of a dataset
pub fn median(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let len = sorted.len();
    if len % 2 == 0 {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
    } else {
        sorted[len / 2]
    }
}

/// Calculate mode of a dataset (first mode if multiple exist)
pub fn mode(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    use std::collections::HashMap;
    let mut counts = HashMap::new();

    for &value in data {
        *counts.entry(value.to_bits()).or_insert(0) += 1;
    }

    counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(bits, _)| f64::from_bits(bits))
}

/// Calculate variance of a dataset
pub fn variance(data: &[f64]) -> f64 {
    math::safe_variance(data)
}

/// Calculate standard deviation of a dataset
pub fn std_dev(data: &[f64]) -> f64 {
    math::safe_std_dev(data)
}

/// Calculate skewness of a dataset
pub fn skewness(data: &[f64]) -> f64 {
    if data.len() < 3 {
        return 0.0;
    }

    let mean = math::safe_mean(data);
    let std_dev = math::safe_std_dev(data);

    if std_dev == 0.0 {
        return 0.0;
    }

    let n = data.len() as f64;
    let sum_cubed = data
        .iter()
        .map(|&x| ((x - mean) / std_dev).powi(3))
        .sum::<f64>();

    (n / ((n - 1.0) * (n - 2.0))) * sum_cubed
}

/// Calculate kurtosis of a dataset (excess kurtosis)
pub fn kurtosis(data: &[f64]) -> f64 {
    if data.len() < 4 {
        return 0.0;
    }

    let mean = math::safe_mean(data);
    let std_dev = math::safe_std_dev(data);

    if std_dev == 0.0 {
        return 0.0;
    }

    let n = data.len() as f64;
    let sum_fourth = data
        .iter()
        .map(|&x| ((x - mean) / std_dev).powi(4))
        .sum::<f64>();

    let g2 = sum_fourth / n - 3.0;

    // Fisher's correction for sample kurtosis
    ((n + 1.0) * n * g2 / ((n - 1.0) * (n - 2.0) * (n - 3.0)))
        + (6.0 * (n - 1.0) / ((n - 2.0) * (n - 3.0)))
}

/// Calculate percentile of a dataset
pub fn percentile(data: &[f64], p: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let p = p.max(0.0).min(100.0);

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let index = (p / 100.0) * (sorted.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    let weight = index - lower as f64;

    sorted[lower] * (1.0 - weight) + sorted[upper] * weight
}

/// Calculate interquartile range
pub fn iqr(data: &[f64]) -> f64 {
    percentile(data, 75.0) - percentile(data, 25.0)
}

/// Detect outliers using IQR method
pub fn outliers_iqr(data: &[f64], multiplier: f64) -> Vec<f64> {
    let q1 = percentile(data, 25.0);
    let q3 = percentile(data, 75.0);
    let iqr = q3 - q1;

    let lower_bound = q1 - multiplier * iqr;
    let upper_bound = q3 + multiplier * iqr;

    data.iter()
        .filter(|&&x| x < lower_bound || x > upper_bound)
        .copied()
        .collect()
}

/// Calculate correlation coefficient between two datasets
pub fn correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.len() < 2 {
        return 0.0;
    }

    let n = x.len() as f64;
    let mean_x = math::safe_mean(x);
    let mean_y = math::safe_mean(y);

    let cov = x
        .iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| (xi - mean_x) * (yi - mean_y))
        .sum::<f64>()
        / (n - 1.0);

    let std_x = math::safe_std_dev(x);
    let std_y = math::safe_std_dev(y);

    if std_x == 0.0 || std_y == 0.0 {
        return 0.0;
    }

    cov / (std_x * std_y)
}

/// Calculate covariance between two datasets
pub fn covariance(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }

    let n = x.len() as f64;
    let mean_x = math::safe_mean(x);
    let mean_y = math::safe_mean(y);

    x.iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| (xi - mean_x) * (yi - mean_y))
        .sum::<f64>()
        / (n - 1.0)
}

/// Ex-Gaussian distribution parameters for response time modeling
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExGaussianParams {
    pub mu: f64,    // Mean of normal component
    pub sigma: f64, // Standard deviation of normal component
    pub tau: f64,   // Mean/scale of exponential component (λ = 1/τ)
}

impl ExGaussianParams {
    pub fn new(mu: f64, sigma: f64, tau: f64) -> Self {
        Self {
            mu: mu.max(0.0),
            sigma: sigma.max(f64::EPSILON),
            tau: tau.max(f64::EPSILON),
        }
    }
}

/// Fit Ex-Gaussian distribution to response times using method of moments
pub fn fit_ex_gaussian(response_times: &[f64]) -> Result<ExGaussianParams, &'static str> {
    if response_times.len() < 3 {
        return Err("Need at least 3 data points for Ex-Gaussian fitting");
    }

    let mean = math::safe_mean(response_times);
    let variance = math::safe_variance(response_times);
    let skew = skewness(response_times);

    if variance <= 0.0 || skew <= 0.0 {
        return Err("Invalid distribution parameters");
    }

    // Method of moments (tau has same units as time)
    // A practical approximation: tau ≈ (skew/2)^(1/3) * s
    let s = variance.sqrt();
    let tau = (skew / 2.0).powf(1.0 / 3.0) * s;
    let mut sigma_squared = variance - tau.powi(2);
    if sigma_squared <= f64::EPSILON {
        sigma_squared = f64::EPSILON;
    }

    if sigma_squared <= 0.0 {
        return Err("Invalid sigma calculation");
    }

    let sigma = sigma_squared.sqrt();
    let mu = mean - tau;

    Ok(ExGaussianParams::new(mu, sigma, tau))
}

/// Calculate Ex-Gaussian probability density function
pub fn ex_gaussian_pdf(x: f64, params: &ExGaussianParams) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }

    let lambda = 1.0 / params.tau;
    let z = (x - params.mu) / params.sigma - params.sigma * lambda;
    let erfcz = math::erfc(z / std::f64::consts::SQRT_2);

    (lambda / 2.0)
        * (-(lambda * (x - params.mu)) + (lambda.powi(2) * params.sigma.powi(2)) / 2.0).exp()
        * erfcz
}

/// Two-sample t-test for comparing means
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TTestResult {
    pub t_statistic: f64,
    pub degrees_of_freedom: f64,
    pub p_value: f64,
    pub significant: bool,
}

pub fn t_test_two_sample(
    sample1: &[f64],
    sample2: &[f64],
    alpha: f64,
) -> Result<TTestResult, &'static str> {
    if sample1.len() < 2 || sample2.len() < 2 {
        return Err("Need at least 2 samples in each group");
    }

    let mean1 = math::safe_mean(sample1);
    let mean2 = math::safe_mean(sample2);
    let var1 = math::safe_variance(sample1);
    let var2 = math::safe_variance(sample2);
    let n1 = sample1.len() as f64;
    let n2 = sample2.len() as f64;

    // Welch's t-test (unequal variances)
    let se = (var1 / n1 + var2 / n2).sqrt();
    if se == 0.0 {
        return Err("Standard error is zero");
    }

    let t_statistic = (mean1 - mean2) / se;

    // Welch-Satterthwaite degrees of freedom
    let df_num = (var1 / n1 + var2 / n2).powi(2);
    let df_denom = (var1 / n1).powi(2) / (n1 - 1.0) + (var2 / n2).powi(2) / (n2 - 1.0);
    let degrees_of_freedom = df_num / df_denom;

    // Approximate p-value using t-distribution CDF
    let p_value = 2.0 * (1.0 - t_distribution_cdf(t_statistic.abs(), degrees_of_freedom));
    let significant = p_value < alpha;

    Ok(TTestResult {
        t_statistic,
        degrees_of_freedom,
        p_value,
        significant,
    })
}

/// One-way ANOVA F-test
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnovaResult {
    pub f_statistic: f64,
    pub df_between: f64,
    pub df_within: f64,
    pub p_value: f64,
    pub significant: bool,
}

pub fn one_way_anova(groups: &[Vec<f64>], alpha: f64) -> Result<AnovaResult, &'static str> {
    if groups.len() < 2 {
        return Err("Need at least 2 groups for ANOVA");
    }

    let mut all_values = Vec::new();
    let mut group_means = Vec::new();
    let mut group_sizes = Vec::new();

    for group in groups {
        if group.len() < 2 {
            return Err("Each group needs at least 2 observations");
        }

        all_values.extend(group);
        group_means.push(math::safe_mean(group));
        group_sizes.push(group.len());
    }

    let grand_mean = math::safe_mean(&all_values);
    let total_n = all_values.len() as f64;
    let k = groups.len() as f64;

    // Sum of squares between groups
    let mut ss_between = 0.0;
    for (i, group_mean) in group_means.iter().enumerate() {
        let n_i = group_sizes[i] as f64;
        ss_between += n_i * (group_mean - grand_mean).powi(2);
    }

    // Sum of squares within groups
    let mut ss_within = 0.0;
    for (i, group) in groups.iter().enumerate() {
        let group_mean = group_means[i];
        for &value in group {
            ss_within += (value - group_mean).powi(2);
        }
    }

    let df_between = k - 1.0;
    let df_within = total_n - k;

    if df_within == 0.0 || ss_within == 0.0 {
        return Err("Invalid degrees of freedom or sum of squares");
    }

    let ms_between = ss_between / df_between;
    let ms_within = ss_within / df_within;
    let f_statistic = ms_between / ms_within;

    // Approximate p-value using F-distribution
    let p_value = 1.0 - f_distribution_cdf(f_statistic, df_between, df_within);
    let significant = p_value < alpha;

    Ok(AnovaResult {
        f_statistic,
        df_between,
        df_within,
        p_value,
        significant,
    })
}

/// Approximate t-distribution CDF
fn t_distribution_cdf(t: f64, df: f64) -> f64 {
    if df <= 0.0 {
        return 0.5;
    }
    let dist = StudentsT::new(0.0, 1.0, df.max(1.0)).unwrap();
    dist.cdf(t)
}

/// Approximate F-distribution CDF
fn f_distribution_cdf(f: f64, df1: f64, df2: f64) -> f64 {
    if f <= 0.0 || df1 <= 0.0 || df2 <= 0.0 {
        return 0.0;
    }
    let dist = FisherSnedecor::new(df1, df2).unwrap();
    dist.cdf(f)
}

/// Standard normal CDF approximation
fn standard_normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + math::error_function(x / std::f64::consts::SQRT_2))
}

/// Detect outliers using multiple methods
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutlierAnalysis {
    pub iqr_outliers: Vec<f64>,
    pub z_score_outliers: Vec<f64>,
    pub modified_z_outliers: Vec<f64>,
}

pub fn comprehensive_outlier_detection(data: &[f64]) -> OutlierAnalysis {
    let iqr_outliers = outliers_iqr(data, 1.5);
    let z_score_outliers = outliers_z_score(data, 3.0);
    let modified_z_outliers = outliers_modified_z_score(data, 3.5);

    OutlierAnalysis {
        iqr_outliers,
        z_score_outliers,
        modified_z_outliers,
    }
}

/// Detect outliers using Z-score method
pub fn outliers_z_score(data: &[f64], threshold: f64) -> Vec<f64> {
    let mean = math::safe_mean(data);
    let std_dev = math::safe_std_dev(data);

    if std_dev == 0.0 {
        return Vec::new();
    }

    data.iter()
        .filter(|&&x| ((x - mean) / std_dev).abs() > threshold)
        .copied()
        .collect()
}

/// Detect outliers using Modified Z-score method (using median)
pub fn outliers_modified_z_score(data: &[f64], threshold: f64) -> Vec<f64> {
    let median_val = median(data);
    let deviations: Vec<f64> = data.iter().map(|&x| (x - median_val).abs()).collect();
    let mad = median(&deviations); // Median Absolute Deviation

    if mad == 0.0 {
        return Vec::new();
    }

    data.iter()
        .filter(|&&x| 0.6745 * (x - median_val).abs() / mad > threshold)
        .copied()
        .collect()
}

/// Bootstrap confidence interval for any statistic
pub fn bootstrap_confidence_interval<F>(
    data: &[f64],
    statistic: F,
    confidence: f64,
    n_bootstrap: usize,
) -> (f64, f64)
where
    F: Fn(&[f64]) -> f64 + Copy,
{
    let mut rng = rand::thread_rng();
    let mut bootstrap_stats = Vec::with_capacity(n_bootstrap);

    for _ in 0..n_bootstrap {
        // Resample with replacement
        let bootstrap_sample: Vec<f64> = (0..data.len())
            .map(|_| data[rng.gen_range(0..data.len())])
            .collect();

        bootstrap_stats.push(statistic(&bootstrap_sample));
    }

    bootstrap_stats.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let alpha = 1.0 - confidence;
    let lower_idx = ((alpha / 2.0) * n_bootstrap as f64) as usize;
    let upper_idx = ((1.0 - alpha / 2.0) * n_bootstrap as f64) as usize;

    let lower_idx = lower_idx.min(bootstrap_stats.len() - 1);
    let upper_idx = upper_idx.min(bootstrap_stats.len() - 1);

    (bootstrap_stats[lower_idx], bootstrap_stats[upper_idx])
}

/// Response time analysis using Ex-Gaussian modeling
pub fn analyze_response_times(times: &[f64]) -> Result<ResponseTimeAnalysis, &'static str> {
    if times.is_empty() {
        return Err("No response times provided");
    }

    let ex_gaussian_params = fit_ex_gaussian(times)?;
    let outliers = comprehensive_outlier_detection(times);

    // Bootstrap confidence intervals for mean
    let (mean_ci_lower, mean_ci_upper) = bootstrap_confidence_interval(times, mean, 0.95, 1000);

    Ok(ResponseTimeAnalysis {
        params: ex_gaussian_params,
        outliers,
        mean_ci: (mean_ci_lower, mean_ci_upper),
        n_samples: times.len(),
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseTimeAnalysis {
    pub params: ExGaussianParams,
    pub outliers: OutlierAnalysis,
    pub mean_ci: (f64, f64),
    pub n_samples: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median() {
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3.0);
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), 2.5);
        assert_eq!(median(&[]), 0.0);
    }

    #[test]
    fn test_percentile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 50.0), 3.0);
        assert_eq!(percentile(&data, 0.0), 1.0);
        assert_eq!(percentile(&data, 100.0), 5.0);
    }

    #[test]
    fn test_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        assert!((correlation(&x, &y) - 1.0).abs() < 0.0001);

        let z = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        assert!((correlation(&x, &z) + 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_ex_gaussian_fitting() {
        // Generate some sample response times
        let times = vec![
            500.0, 600.0, 550.0, 700.0, 800.0, 650.0, 750.0, 900.0, 1000.0, 1200.0,
        ];
        let result = fit_ex_gaussian(&times);
        assert!(result.is_ok());

        let params = result.unwrap();
        assert!(params.mu > 0.0);
        assert!(params.sigma > 0.0);
        assert!(params.tau > 0.0);
    }

    #[test]
    fn test_t_test() {
        let group1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let group2 = vec![6.0, 7.0, 8.0, 9.0, 10.0];

        let result = t_test_two_sample(&group1, &group2, 0.05);
        assert!(result.is_ok());

        let test_result = result.unwrap();
        assert!(test_result.significant); // These groups should be significantly different
    }

    #[test]
    fn test_anova() {
        let groups = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let result = one_way_anova(&groups, 0.05);
        assert!(result.is_ok());

        let anova_result = result.unwrap();
        assert!(anova_result.f_statistic > 0.0);
    }

    #[test]
    fn test_outlier_detection() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0]; // 100.0 should be an outlier
        let analysis = comprehensive_outlier_detection(&data);

        assert!(!analysis.iqr_outliers.is_empty());
        assert!(!analysis.z_score_outliers.is_empty());
    }

    #[test]
    fn test_bootstrap_ci() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (lower, upper) = bootstrap_confidence_interval(&data, mean, 0.95, 100);

        assert!(lower <= upper);
        assert!(lower <= mean(&data));
        assert!(upper >= mean(&data));
    }
}
