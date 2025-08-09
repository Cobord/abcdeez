use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Database model for pre-registration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PreRegistrationDb {
    pub id: String,
    pub experiment_id: String,
    pub researcher_id: String,
    pub title: String,
    pub description: String,

    pub registered_at: DateTime<Utc>,
    pub registration_hash: String,
    pub status: String,

    // JSON fields
    pub study_metadata: String,
    pub hypotheses: String,
    pub analysis_plan: String,
    pub data_collection_plan: String,
    pub exclusion_criteria: String,
    pub decision_rules: String,

    pub version: i32,
    pub parent_id: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API model for creating pre-registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePreRegistration {
    pub experiment_id: String,
    pub title: String,
    pub description: String,
    pub study_metadata: StudyMetadata,
    pub hypotheses: Hypotheses,
    pub analysis_plan: AnalysisPlan,
    pub data_collection_plan: DataCollectionPlan,
    pub exclusion_criteria: ExclusionCriteria,
    pub decision_rules: DecisionRules,
}

/// API model for updating pre-registration (only allowed in draft status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePreRegistration {
    pub title: Option<String>,
    pub description: Option<String>,
    pub study_metadata: Option<StudyMetadata>,
    pub hypotheses: Option<Hypotheses>,
    pub analysis_plan: Option<AnalysisPlan>,
    pub data_collection_plan: Option<DataCollectionPlan>,
    pub exclusion_criteria: Option<ExclusionCriteria>,
    pub decision_rules: Option<DecisionRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyMetadata {
    pub researchers: Vec<String>,
    pub institution: String,
    pub ethical_approval: Option<String>,
    pub funding_source: Option<String>,
    pub conflicts_of_interest: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypotheses {
    pub primary: Vec<Hypothesis>,
    pub secondary: Vec<Hypothesis>,
    pub directional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub description: String,
    pub operationalization: String,
    pub predicted_effect: EffectPrediction,
    pub statistical_test: String,
    pub alpha_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EffectPrediction {
    GreaterThan { value: f64 },
    LessThan { value: f64 },
    Different { from: f64 },
    Range { min: f64, max: f64 },
    EffectSize { cohens_d: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPlan {
    pub primary_analyses: Vec<PlannedAnalysis>,
    pub secondary_analyses: Vec<PlannedAnalysis>,
    pub multiple_comparison_correction: Option<String>,
    pub power_analysis: PowerAnalysisSpec,
    pub robustness_checks: Vec<RobustnessCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAnalysis {
    pub name: String,
    pub description: String,
    pub dependent_variable: String,
    pub independent_variables: Vec<String>,
    pub covariates: Vec<String>,
    pub statistical_model: String,
    pub assumptions_to_check: Vec<String>,
    pub fallback_if_assumptions_violated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerAnalysisSpec {
    pub target_power: f64,
    pub alpha_level: f64,
    pub effect_size: f64,
    pub sample_size_calculation: String,
    pub achieved_sample_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustnessCheck {
    pub name: String,
    pub description: String,
    pub alternative_specification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCollectionPlan {
    pub target_sample_size: usize,
    pub sampling_method: String,
    pub inclusion_criteria: Vec<String>,
    pub randomization_procedure: Option<String>,
    pub blinding: String, // "none", "single", "double", "triple"
    pub stopping_rule: StoppingRule,
    pub data_quality_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StoppingRule {
    FixedSampleSize {
        n: usize,
    },
    Sequential {
        max_n: usize,
        interim_analyses: Vec<usize>,
    },
    Adaptive {
        criteria: String,
    },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionCriteria {
    pub participant_level: Vec<String>,
    pub trial_level: Vec<String>,
    pub data_quality: Vec<String>,
    pub outlier_handling: String, // "none", "remove", "winsorize", "transform", "robust"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRules {
    pub success_criteria: Vec<String>,
    pub failure_criteria: Vec<String>,
    pub interpretation_guidelines: std::collections::HashMap<String, String>,
}

/// Deviation from pre-registration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PreRegistrationDeviation {
    pub id: String,
    pub preregistration_id: String,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub justification: String,
    pub impact_assessment: String,
    pub created_by: String,
}

/// Analysis validation result
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisValidation {
    pub id: String,
    pub preregistration_id: String,
    pub analysis_name: String,
    pub validation_result: String,
    pub actual_test: String,
    pub actual_variables: String, // JSON array
    pub deviation_reason: Option<String>,
    pub is_exploratory: bool,
    pub timestamp: DateTime<Utc>,
}

/// Transparency report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparencyReport {
    pub registration_id: String,
    pub registered_at: DateTime<Utc>,
    pub registration_hash: String,
    pub n_primary_hypotheses: usize,
    pub n_secondary_hypotheses: usize,
    pub n_primary_analyses: usize,
    pub n_secondary_analyses: usize,
    pub n_deviations: usize,
    pub deviation_descriptions: Vec<String>,
    pub status: String,
}

/// API response for pre-registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreRegistrationResponse {
    pub id: String,
    pub experiment_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub registered_at: Option<DateTime<Utc>>,
    pub registration_hash: Option<String>,
    pub can_edit: bool,
    pub transparency_report: Option<TransparencyReport>,
}
