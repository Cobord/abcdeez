use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LearnerConfig {
    pub initial_uncertainty: f64,
    pub initial_memory_strength: f64,
    pub initial_proficiency: f64,
    pub learning_rate_base: f64,
    pub learning_rate_decay: f64,
    pub position_update_weight: f64,
    pub min_uncertainty: f64,
    pub theta_bounds: (f64, f64),
    pub memory_update_correct: f64,
    pub memory_update_incorrect: f64,
    pub memory_decay_rate: f64,
    pub edge_emphasis: f64,
}

impl LearnerConfig {
    pub fn validate(&self) -> Result<(), String> {
        macro_rules! validate_range {
            ($field:expr, $range:expr, $name:literal) => {
                if !$range.contains(&$field) {
                    return Err(format!("{} must be in {:?}", $name, $range));
                }
            };
        }
        
        macro_rules! validate_positive {
            ($field:expr, $name:literal) => {
                if $field <= 0.0 {
                    return Err(format!("{} must be positive", $name));
                }
            };
        }
        
        validate_positive!(self.initial_uncertainty, "initial_uncertainty");
        validate_range!(self.initial_memory_strength, 0.0..=1.0, "initial_memory_strength");
        validate_range!(self.learning_rate_base, 0.0..=1.0, "learning_rate_base");
        validate_range!(self.learning_rate_decay, 0.0..=1.0, "learning_rate_decay");
        validate_range!(self.position_update_weight, 0.0..=1.0, "position_update_weight");
        validate_positive!(self.min_uncertainty, "min_uncertainty");
        validate_range!(self.memory_update_correct, 0.0..=1.0, "memory_update_correct");
        validate_range!(self.memory_update_incorrect, -1.0..=0.0, "memory_update_incorrect");
        validate_range!(self.memory_decay_rate, 0.0..=1.0, "memory_decay_rate");
        validate_range!(self.edge_emphasis, 0.0..=1.0, "edge_emphasis");
        
        if self.theta_bounds.0 >= self.theta_bounds.1 {
            return Err("theta_bounds must have lower < upper".to_string());
        }
        
        Ok(())
    }

    pub fn adult() -> Self {
        Self {
            initial_uncertainty: 1.0,
            initial_memory_strength: 0.5,
            initial_proficiency: 0.0,
            learning_rate_base: 0.3,
            learning_rate_decay: 0.5,
            position_update_weight: 0.7,
            min_uncertainty: 0.1,
            theta_bounds: (-3.0, 3.0),
            memory_update_correct: 0.2,
            memory_update_incorrect: -0.1,
            memory_decay_rate: 0.05,
            edge_emphasis: 0.01,
        }
    }

    pub fn child() -> Self {
        Self {
            initial_uncertainty: 1.5,
            initial_memory_strength: 0.3,
            initial_proficiency: -0.5,
            learning_rate_base: 0.5,
            learning_rate_decay: 0.3,
            position_update_weight: 0.5,
            min_uncertainty: 0.2,
            theta_bounds: (-2.0, 2.0),
            memory_update_correct: 0.3,
            memory_update_incorrect: -0.05,
            memory_decay_rate: 0.08,
            edge_emphasis: 0.02,
        }
    }

    pub fn older_adult() -> Self {
        Self {
            initial_uncertainty: 0.8,
            initial_memory_strength: 0.4,
            initial_proficiency: 0.2,
            learning_rate_base: 0.2,
            learning_rate_decay: 0.6,
            position_update_weight: 0.8,
            min_uncertainty: 0.15,
            theta_bounds: (-2.5, 2.5),
            memory_update_correct: 0.15,
            memory_update_incorrect: -0.15,
            memory_decay_rate: 0.07,
            edge_emphasis: 0.015,
        }
    }

    pub fn learning_disability() -> Self {
        Self {
            initial_uncertainty: 2.0,
            initial_memory_strength: 0.2,
            initial_proficiency: -1.0,
            learning_rate_base: 0.4,
            learning_rate_decay: 0.2,
            position_update_weight: 0.4,
            min_uncertainty: 0.3,
            theta_bounds: (-2.0, 2.0),
            memory_update_correct: 0.4,
            memory_update_incorrect: 0.0,
            memory_decay_rate: 0.1,
            edge_emphasis: 0.03,
        }
    }

    pub fn expert() -> Self {
        Self {
            initial_uncertainty: 0.5,
            initial_memory_strength: 0.7,
            initial_proficiency: 1.0,
            learning_rate_base: 0.4,
            learning_rate_decay: 0.7,
            position_update_weight: 0.6,
            min_uncertainty: 0.05,
            theta_bounds: (-4.0, 4.0),
            memory_update_correct: 0.1,
            memory_update_incorrect: -0.2,
            memory_decay_rate: 0.02,
            edge_emphasis: 0.005,
        }
    }
}

impl Default for LearnerConfig {
    fn default() -> Self {
        Self::adult()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AdaptiveSchedulingConfig {
    pub initial_epsilon: f64,
    pub epsilon_decay: f64,
    pub min_epsilon: f64,
    pub target_success_rate: f64,
    pub success_tolerance: f64,
    pub scoring_weights: ScoringWeights,
    pub use_eig: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ScoringWeights {
    pub difficulty: f64,
    pub uncertainty: f64,
    pub practice_need: f64,
    pub weak_link: f64,
}

impl AdaptiveSchedulingConfig {
    pub fn validate(&self) -> Result<(), String> {
        const UNIT_RANGE: std::ops::RangeInclusive<f64> = 0.0..=1.0;
        
        macro_rules! validate_unit {
            ($field:expr, $name:literal) => {
                if !UNIT_RANGE.contains(&$field) {
                    return Err(format!("{} must be in [0, 1]", $name));
                }
            };
        }
        
        validate_unit!(self.initial_epsilon, "initial_epsilon");
        validate_unit!(self.epsilon_decay, "epsilon_decay");
        validate_unit!(self.min_epsilon, "min_epsilon");
        validate_unit!(self.target_success_rate, "target_success_rate");
        validate_unit!(self.success_tolerance, "success_tolerance");
        
        if self.min_epsilon > self.initial_epsilon {
            return Err("min_epsilon must be <= initial_epsilon".to_string());
        }
        
        let weights = &self.scoring_weights;
        let weight_values = [weights.difficulty, weights.uncertainty, weights.practice_need, weights.weak_link];
        
        if weight_values.iter().any(|&w| w < 0.0) {
            return Err("All scoring weights must be non-negative".to_string());
        }
        
        let weight_sum: f64 = weight_values.iter().sum();
        if (weight_sum - 1.0).abs() > 0.001 {
            return Err(format!("Scoring weights must sum to 1.0, got {weight_sum}"));
        }
        
        Ok(())
    }

    pub fn standard() -> Self {
        Self {
            initial_epsilon: 0.15,
            epsilon_decay: 0.995,
            min_epsilon: 0.01,
            target_success_rate: 0.75,
            success_tolerance: 0.15,
            scoring_weights: ScoringWeights {
                difficulty: 0.3,
                uncertainty: 0.3,
                practice_need: 0.2,
                weak_link: 0.2,
            },
            use_eig: false,
        }
    }

    pub fn exploratory() -> Self {
        Self {
            initial_epsilon: 0.3,
            epsilon_decay: 0.999,
            min_epsilon: 0.05,
            target_success_rate: 0.65,
            success_tolerance: 0.25,
            scoring_weights: ScoringWeights {
                difficulty: 0.2,
                uncertainty: 0.5,
                practice_need: 0.2,
                weak_link: 0.1,
            },
            use_eig: true,
        }
    }

    pub fn mastery() -> Self {
        Self {
            initial_epsilon: 0.05,
            epsilon_decay: 0.99,
            min_epsilon: 0.001,
            target_success_rate: 0.85,
            success_tolerance: 0.1,
            scoring_weights: ScoringWeights {
                difficulty: 0.4,
                uncertainty: 0.1,
                practice_need: 0.3,
                weak_link: 0.2,
            },
            use_eig: false,
        }
    }
}

impl Default for AdaptiveSchedulingConfig {
    fn default() -> Self {
        Self::standard()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HintInterventionConfig {
    pub struggle_rt_threshold_ms: u64,
    pub error_streak_threshold: usize,
    pub hint_delay_ms: u64,
    pub adaptive_rt_multiplier: f64,
    pub too_easy_error_rate: f64,
    pub too_easy_rt_ms: u64,
    pub too_hard_error_rate: f64,
    pub mild_errors: usize,
    pub mild_rt_multiplier: f64,
    pub moderate_errors: usize,
    pub moderate_rt_multiplier: f64,
    pub severe_errors: usize,
    pub severe_rt_multiplier: f64,
}

impl HintInterventionConfig {
    pub fn standard() -> Self {
        Self {
            struggle_rt_threshold_ms: 5000,
            error_streak_threshold: 3,
            hint_delay_ms: 3000,
            adaptive_rt_multiplier: 2.0,
            too_easy_error_rate: 0.1,
            too_easy_rt_ms: 2000,
            too_hard_error_rate: 0.5,
            mild_errors: 1,
            mild_rt_multiplier: 1.5,
            moderate_errors: 3,
            moderate_rt_multiplier: 2.0,
            severe_errors: 5,
            severe_rt_multiplier: 3.0,
        }
    }

    pub fn child() -> Self {
        Self {
            struggle_rt_threshold_ms: 3000,
            error_streak_threshold: 2,
            hint_delay_ms: 2000,
            adaptive_rt_multiplier: 1.5,
            too_easy_error_rate: 0.05,
            too_easy_rt_ms: 1500,
            too_hard_error_rate: 0.4,
            mild_errors: 1,
            mild_rt_multiplier: 1.3,
            moderate_errors: 2,
            moderate_rt_multiplier: 1.5,
            severe_errors: 3,
            severe_rt_multiplier: 2.0,
        }
    }

    pub fn supportive() -> Self {
        Self {
            struggle_rt_threshold_ms: 8000,
            error_streak_threshold: 1,
            hint_delay_ms: 1000,
            adaptive_rt_multiplier: 3.0,
            too_easy_error_rate: 0.0,
            too_easy_rt_ms: 10000,
            too_hard_error_rate: 0.3,
            mild_errors: 0,
            mild_rt_multiplier: 1.2,
            moderate_errors: 1,
            moderate_rt_multiplier: 1.5,
            severe_errors: 2,
            severe_rt_multiplier: 2.0,
        }
    }
}

impl Default for HintInterventionConfig {
    fn default() -> Self {
        Self::standard()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DomainConfig {
    pub task_difficulties: TaskDifficultyConfig,
    pub chunk_boundaries: Vec<ChunkBoundaryConfig>,
    pub baseline_rt_ms: ResponseTimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TaskDifficultyConfig {
    pub successor: f64,
    pub predecessor: f64,
    pub k_jump_base: f64,
    pub k_jump_increment: f64,
    pub segment_base: f64,
    pub segment_length_factor: f64,
    pub segment_reverse_penalty: f64,
    pub distance_difficulty_scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ChunkBoundaryConfig {
    pub name: String,
    pub positions: Vec<usize>,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ResponseTimeConfig {
    pub simple_task: u64,
    pub moderate_task: u64,
    pub complex_task: u64,
}

impl DomainConfig {
    pub fn alphabet() -> Self {
        Self {
            task_difficulties: TaskDifficultyConfig {
                successor: 0.3,
                predecessor: 0.4,
                k_jump_base: 0.3,
                k_jump_increment: 0.1,
                segment_base: 0.3,
                segment_length_factor: 0.1,
                segment_reverse_penalty: 0.2,
                distance_difficulty_scale: 0.1,
            },
            chunk_boundaries: vec![
                ChunkBoundaryConfig {
                    name: "Major".to_string(),
                    positions: vec![6, 13, 19],
                    strength: 0.8,
                },
                ChunkBoundaryConfig {
                    name: "Minor".to_string(),
                    positions: vec![3, 9, 16, 22],
                    strength: 0.4,
                },
                ChunkBoundaryConfig {
                    name: "Vowels".to_string(),
                    positions: vec![0, 4, 8, 14, 20],
                    strength: 0.3,
                },
            ],
            baseline_rt_ms: ResponseTimeConfig {
                simple_task: 1500,
                moderate_task: 3000,
                complex_task: 5000,
            },
        }
    }

    pub fn music() -> Self {
        Self {
            task_difficulties: TaskDifficultyConfig {
                successor: 0.25,
                predecessor: 0.35,
                k_jump_base: 0.4,
                k_jump_increment: 0.15,
                segment_base: 0.4,
                segment_length_factor: 0.15,
                segment_reverse_penalty: 0.25,
                distance_difficulty_scale: 0.12,
            },
            chunk_boundaries: vec![
                ChunkBoundaryConfig {
                    name: "Octave".to_string(),
                    positions: vec![7],
                    strength: 0.9,
                },
                ChunkBoundaryConfig {
                    name: "Tetrachord".to_string(),
                    positions: vec![3],
                    strength: 0.5,
                },
            ],
            baseline_rt_ms: ResponseTimeConfig {
                simple_task: 2000,
                moderate_task: 4000,
                complex_task: 6000,
            },
        }
    }

    pub fn mathematics() -> Self {
        Self {
            task_difficulties: TaskDifficultyConfig {
                successor: 0.2,
                predecessor: 0.25,
                k_jump_base: 0.25,
                k_jump_increment: 0.05,
                segment_base: 0.2,
                segment_length_factor: 0.05,
                segment_reverse_penalty: 0.15,
                distance_difficulty_scale: 0.05,
            },
            chunk_boundaries: vec![
                ChunkBoundaryConfig {
                    name: "Decade".to_string(),
                    positions: vec![9, 19, 29],
                    strength: 0.7,
                },
                ChunkBoundaryConfig {
                    name: "Five".to_string(),
                    positions: vec![4, 14, 24],
                    strength: 0.4,
                },
            ],
            baseline_rt_ms: ResponseTimeConfig {
                simple_task: 1000,
                moderate_task: 2000,
                complex_task: 3500,
            },
        }
    }
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self::alphabet()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemConfig {
    pub learner: LearnerConfig,
    pub adaptive: AdaptiveSchedulingConfig,
    pub hints: HintInterventionConfig,
    pub domain: DomainConfig,
}

impl SystemConfig {
    /// Create a preset configuration
    pub fn preset(population: PopulationType, domain: DomainType, goal: LearningGoal) -> Self {
        let learner = match population {
            PopulationType::Adult => LearnerConfig::adult(),
            PopulationType::Child => LearnerConfig::child(),
            PopulationType::OlderAdult => LearnerConfig::older_adult(),
            PopulationType::LearningDisability => LearnerConfig::learning_disability(),
            PopulationType::Expert => LearnerConfig::expert(),
        };

        let adaptive = match goal {
            LearningGoal::Exploration => AdaptiveSchedulingConfig::exploratory(),
            LearningGoal::Mastery => AdaptiveSchedulingConfig::mastery(),
            LearningGoal::Standard => AdaptiveSchedulingConfig::standard(),
        };

        let hints = match population {
            PopulationType::Child => HintInterventionConfig::child(),
            PopulationType::LearningDisability => HintInterventionConfig::supportive(),
            _ => HintInterventionConfig::standard(),
        };

        let domain = match domain {
            DomainType::Alphabet => DomainConfig::alphabet(),
            DomainType::Music => DomainConfig::music(),
            DomainType::Mathematics => DomainConfig::mathematics(),
        };

        Self {
            learner,
            adaptive,
            hints,
            domain,
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self::preset(PopulationType::Adult, DomainType::Alphabet, LearningGoal::Standard)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PopulationType {
    Adult,
    Child,
    OlderAdult,
    LearningDisability,
    Expert,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DomainType {
    Alphabet,
    Music,
    Mathematics,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LearningGoal {
    Exploration,
    Mastery,
    Standard,
}
