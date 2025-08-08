use serde::{Deserialize, Serialize};
use statrs::distribution::{ContinuousCDF, Normal};
use statrs::statistics::Statistics;
use std::collections::HashMap;

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
            let sum_cubed = data.iter()
                .map(|x| ((x - mean) / std_dev).powi(3))
                .sum::<f64>();
            (n / ((n - 1.0) * (n - 2.0))) * sum_cubed
        } else {
            0.0
        };

        let kurtosis = if std_dev > 0.0 {
            let n = data.len() as f64;
            let sum_fourth = data.iter()
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
    pub fn new(mu: f64, sigma: f64, tau: f64) -> Self {
        ExGaussianModel {
            params: ExGaussianParameters { mu, sigma, tau },
        }
    }

    pub fn fit(response_times: &[f64]) -> Self {
        let stats = DetailedStatistics::from_data(response_times);
        
        let mu = stats.mean - stats.std_dev;
        let sigma = stats.std_dev * 0.8;
        let tau = stats.std_dev * 0.5;

        ExGaussianModel::new(mu, sigma, tau)
    }

    pub fn pdf(&self, x: f64) -> f64 {
        if self.params.tau <= 0.0 || self.params.sigma <= 0.0 {
            return 0.0;
        }

        // Ex-Gaussian PDF is the convolution of a Gaussian and an exponential
        // f(x) = (λ/2) * exp(λ/2 * (2μ + λσ² - 2x)) * erfc((μ + λσ² - x)/(√2 * σ))
        let lambda = 1.0 / self.params.tau;
        let normal = Normal::new(0.0, 1.0).unwrap();
        
        // Calculate the argument for the exponential term
        let exp_arg = (lambda / 2.0) * (2.0 * self.params.mu + lambda * self.params.sigma.powi(2) - 2.0 * x);
        
        // Prevent numerical overflow
        if exp_arg < -20.0 {
            return 0.0;
        }
        
        // Calculate the argument for the complementary error function
        let erfc_arg = (self.params.mu + lambda * self.params.sigma.powi(2) - x) / (self.params.sigma * std::f64::consts::SQRT_2);
        
        // Use the CDF of standard normal to compute erfc
        // erfc(z) = 2 * (1 - Φ(z)) where Φ is the CDF of N(0,1)
        let erfc_val = 2.0 * (1.0 - normal.cdf(erfc_arg));
        
        (lambda / 2.0) * exp_arg.exp() * erfc_val
    }

    pub fn mean(&self) -> f64 {
        self.params.mu + self.params.tau
    }

    pub fn variance(&self) -> f64 {
        self.params.sigma.powi(2) + self.params.tau.powi(2)
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

        let local_errors: f64 = self.error_by_distance
            .iter()
            .filter(|(d, _)| **d <= 2)
            .map(|(_, count)| count)
            .sum();

        self.locality_index = local_errors / total_errors as f64;
    }

    pub fn identify_systematic_errors(&mut self) {
        self.systematic_errors = self.confusion_matrix
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StrategyType {
    SerialScan,
    DirectIndex,
    Mixed(i32), // Changed to i32 for Hash trait
}

impl StrategyAnalysis {
    pub fn analyze(response_times: &[f64], distances: &[usize]) -> Self {
        let correlation = Self::calculate_correlation(response_times, distances);
        
        let strategy_classification = if correlation > 0.7 {
            StrategyType::SerialScan
        } else if correlation < 0.3 {
            StrategyType::DirectIndex
        } else {
            StrategyType::Mixed((correlation * 100.0) as i32)
        };

        let transition_point = Self::find_strategy_transition(response_times, distances);

        StrategyAnalysis {
            rt_distance_correlation: correlation,
            strategy_classification,
            transition_point,
        }
    }

    fn calculate_correlation(response_times: &[f64], distances: &[usize]) -> f64 {
        if response_times.len() != distances.len() || response_times.is_empty() {
            return 0.0;
        }

        let rt_mean = response_times.iter().sum::<f64>() / response_times.len() as f64;
        let dist_mean = distances.iter().sum::<usize>() as f64 / distances.len() as f64;

        let covariance: f64 = response_times.iter()
            .zip(distances.iter())
            .map(|(rt, d)| (rt - rt_mean) * (*d as f64 - dist_mean))
            .sum::<f64>() / response_times.len() as f64;

        let rt_std = (response_times.iter()
            .map(|rt| (rt - rt_mean).powi(2))
            .sum::<f64>() / response_times.len() as f64)
            .sqrt();

        let dist_std = (distances.iter()
            .map(|d| (*d as f64 - dist_mean).powi(2))
            .sum::<f64>() / distances.len() as f64)
            .sqrt();

        if rt_std > 0.0 && dist_std > 0.0 {
            covariance / (rt_std * dist_std)
        } else {
            0.0
        }
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

        if max_change > 0.3 {
            transition_idx
        } else {
            None
        }
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

            task_type_stats.entry(task_type.clone())
                .or_insert_with(Vec::new)
                .push(rt);

            if let Some(distance) = self.calculate_task_distance(&response.task) {
                rt_by_distance.entry(distance)
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

        let task_type_stats = task_type_stats.into_iter()
            .map(|(k, v)| (k, DetailedStatistics::from_data(&v)))
            .collect();

        let accuracies: Vec<bool> = self.responses.iter().map(|r| r.correct).collect();
        let response_times: Vec<f64> = self.responses.iter().map(|r| r.response_time_ms as f64).collect();
        let learning_curves = LearningCurves::calculate(&accuracies, &response_times, 10);

        let distances: Vec<usize> = self.responses.iter()
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