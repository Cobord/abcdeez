use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::services::LearnerService;
use crate::utils::math;
use abcdeez_core::{
    AdaptiveScheduler, Task, Topology,
};

// TODO: These types need to be implemented in abcdeez_core
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HintLevel {
    // Legacy names used by websocket handlers
    Confirmation,
    Partial,
    Scaffold,
    Worked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionAction {
    ProvideHint(HintLevel),
    ProvideWorkedExample(String),
    IncreaseDifficulty,
    DecreaseDifficulty,
    SuggestBreak,
    SkipTask,
}

#[derive(Debug)]
pub struct InterventionSystem;

#[derive(Debug, Clone, PartialEq)]
pub enum StruggleLevel {
    None,
    Mild,
    Moderate,
    Severe,
}

#[derive(Clone)]
pub struct AdaptationService {
    pub learner_service: Arc<LearnerService>,
}

impl AdaptationService {
    pub fn new(learner_service: Arc<LearnerService>) -> Self {
        Self { learner_service }
    }

    pub async fn select_next_task(&self, learner_id: Uuid, topology: &Topology) -> Result<Task> {
        // Get the current learner model
        let learner_model = self.learner_service.get_learner_model(learner_id).await?;

        // Create adaptive scheduler to select optimal task using Expected Information Gain
        let mut scheduler = AdaptiveScheduler::new(learner_model, topology.clone());

        // Select the next task adaptively based on learner's current state
        let task = scheduler.select_next_task();

        Ok(task)
    }

    pub async fn calculate_eig(
        &self,
        learner_id: Uuid,
        topology: &Topology,
        task: &Task,
    ) -> Result<f64> {
        // Use sophisticated Bayesian EIG calculation instead of simple heuristics
        let mut bayesian_model = self
            .learner_service
            .get_bayesian_model(learner_id, topology)
            .await?;

        // Calculate true Expected Information Gain using Monte Carlo simulation
        let eig = bayesian_model.calculate_eig(task);

        Ok(eig)
    }

    /// Enhanced EIG calculation using Bayesian model (legacy method for compatibility)
    pub async fn calculate_eig_legacy(
        &self,
        learner_model: &abcdeez_core::LearnerModel,
        task: &Task,
    ) -> Result<f64> {
        // Calculate Expected Information Gain for this task
        // This is based on the uncertainty reduction we'd get from observing the response

        let _current_uncertainty = self.calculate_model_uncertainty(learner_model);
        let predicted_accuracy = self.predict_task_accuracy(learner_model, task);

        // EIG = Expected reduction in uncertainty
        // For correct response: uncertainty reduction weighted by P(correct)
        // For incorrect response: uncertainty reduction weighted by P(incorrect)
        let eig_correct = self.calculate_uncertainty_reduction_if_correct(learner_model, task);
        let eig_incorrect = self.calculate_uncertainty_reduction_if_incorrect(learner_model, task);

        let eig = predicted_accuracy * eig_correct + (1.0 - predicted_accuracy) * eig_incorrect;

        Ok(eig)
    }

    pub async fn should_intervene(
        &self,
        learner_id: Uuid,
        session_id: Uuid,
        elapsed_ms: u64,
        recent_errors: usize,
    ) -> Result<Option<InterventionAction>> {
        let learner_model = self.learner_service.get_learner_model(learner_id).await?;

        // Create intervention system with default topology (placeholder)
        let topology = Topology::alphabet();
        let _intervention_system = InterventionSystem::new(topology);

        // Determine struggle level based on time and recent errors
        let struggle_level = if elapsed_ms > 30000 {
            // 30 seconds
            match recent_errors {
                0..=1 => StruggleLevel::None,
                2..=3 => StruggleLevel::Mild,
                4..=6 => StruggleLevel::Moderate,
                _ => StruggleLevel::Severe,
            }
        } else if elapsed_ms > 15000 {
            // 15 seconds
            match recent_errors {
                0 => StruggleLevel::None,
                1..=2 => StruggleLevel::Mild,
                _ => StruggleLevel::Moderate,
            }
        } else {
            StruggleLevel::None
        };

        // Check if intervention is needed
        let should_intervene = match struggle_level {
            StruggleLevel::None => false,
            StruggleLevel::Mild => elapsed_ms > 20000,
            StruggleLevel::Moderate => elapsed_ms > 15000,
            StruggleLevel::Severe => elapsed_ms > 10000,
        };

        if should_intervene {
            // Determine intervention type
            let intervention = if recent_errors > 3 {
                InterventionAction::ProvideHint(HintLevel::Worked)
            } else if recent_errors > 1 {
                InterventionAction::ProvideHint(HintLevel::Partial)
            } else if elapsed_ms > 25000 {
                InterventionAction::ProvideHint(HintLevel::Partial)
            } else {
                InterventionAction::DecreaseDifficulty
            };

            // Log intervention to database
            self.log_intervention(session_id, &intervention).await?;

            Ok(Some(intervention))
        } else {
            Ok(None)
        }
    }

    pub async fn adjust_difficulty(
        &self,
        learner_id: Uuid,
        recent_responses: &[bool],
    ) -> Result<f64> {
        if recent_responses.is_empty() {
            return Ok(0.5); // Default difficulty
        }

        let correct_count = recent_responses.iter().filter(|&&correct| correct).count();
        let success_rate = math::safe_accuracy(correct_count, recent_responses.len());

        let target_success_rate = 0.75; // Target 75% success rate
        let current_difficulty = 0.5; // This would be stored/retrieved from learner state

        const TOLERANCE: f64 = 0.1;
        const ADJUSTMENT_STEP: f64 = 0.05;

        let new_difficulty = if success_rate > target_success_rate + TOLERANCE {
            math::clamp(current_difficulty + ADJUSTMENT_STEP, 0.1, 1.0)
        } else if success_rate < target_success_rate - TOLERANCE {
            math::clamp(current_difficulty - ADJUSTMENT_STEP, 0.1, 1.0)
        } else {
            current_difficulty // Keep same difficulty
        };

        Ok(new_difficulty)
    }

    pub async fn generate_hint(
        &self,
        learner_id: Uuid,
        task: &Task,
        hint_level: HintLevel,
    ) -> Result<String> {
        let learner_model = self.learner_service.get_learner_model(learner_id).await?;

        // Generate contextual hint based on learner's specific weaknesses
        let hint = match hint_level {
            HintLevel::Confirmation => self.generate_subtle_hint(task, &learner_model),
            HintLevel::Partial => self.generate_mild_hint(task, &learner_model),
            HintLevel::Scaffold => self.generate_mild_hint(task, &learner_model),
            HintLevel::Worked => self.generate_strong_hint(task, &learner_model),
        };

        Ok(hint)
    }

    // Private helper methods
    fn calculate_model_uncertainty(&self, model: &abcdeez_core::LearnerModel) -> f64 {
        // Calculate overall uncertainty in the learner model
        // This could be based on embedding uncertainties, proficiency variances, etc.
        let embedding_uncertainty: f64 = model
            .node_embeddings
            .values()
            .map(|emb| emb.uncertainty)
            .sum::<f64>()
            / model.node_embeddings.len() as f64;

        embedding_uncertainty
    }

    fn predict_task_accuracy(&self, model: &abcdeez_core::LearnerModel, task: &Task) -> f64 {
        // Predict probability of correct response for this task
        let difficulty = task.difficulty;

        // Use the operation field from task
        let operation_type = &task.operation;
        model.get_probability_correct(operation_type, difficulty)
    }

    fn calculate_uncertainty_reduction_if_correct(
        &self,
        model: &abcdeez_core::LearnerModel,
        task: &Task,
    ) -> f64 {
        // Calculate how much we'd learn if the response is correct
        // Use Monte Carlo simulation to estimate information gain
        self.monte_carlo_information_gain(model, task, true)
    }

    fn calculate_uncertainty_reduction_if_incorrect(
        &self,
        model: &abcdeez_core::LearnerModel,
        task: &Task,
    ) -> f64 {
        // Calculate how much we'd learn if the response is incorrect
        // Incorrect responses often provide more information
        self.monte_carlo_information_gain(model, task, false)
    }

    fn monte_carlo_information_gain(
        &self,
        model: &abcdeez_core::LearnerModel,
        task: &Task,
        response_correct: bool,
    ) -> f64 {
        const MC_SAMPLES: usize = 100; // Number of Monte Carlo samples

        // Current model uncertainty (entropy)
        let current_entropy = self.calculate_model_entropy(model);

        let mut expected_entropy_after = 0.0;

        // Monte Carlo simulation
        for _ in 0..MC_SAMPLES {
            // Simulate the model update with this response
            let mut simulated_model = model.clone();

            // Update model with simulated response
            self.simulate_model_update(&mut simulated_model, task, response_correct);

            // Calculate entropy after update
            let entropy_after = self.calculate_model_entropy(&simulated_model);
            expected_entropy_after += entropy_after;
        }

        expected_entropy_after /= MC_SAMPLES as f64;

        // Information gain = reduction in entropy
        let information_gain = current_entropy - expected_entropy_after;

        // Ensure non-negative (mathematical guarantee, but floating point can be tricky)
        information_gain.max(0.0)
    }

    fn calculate_model_entropy(&self, model: &abcdeez_core::LearnerModel) -> f64 {
        // Calculate Shannon entropy of the model's beliefs
        let mut total_entropy = 0.0;
        let mut node_count = 0;

        for (_, embedding) in &model.node_embeddings {
            // For each node, calculate uncertainty as entropy
            // Using uncertainty as a proxy for probability distribution entropy
            let uncertainty = embedding.uncertainty;

            // Convert uncertainty to probability-like values for entropy calculation
            let p = math::clamp(uncertainty, 1e-10, 1.0 - 1e-10);
            let q = 1.0 - p;

            // Binary entropy: -p*log2(p) - q*log2(q)
            let entropy = -p * math::safe_log2(p) - q * math::safe_log2(q);
            total_entropy += entropy;
            node_count += 1;
        }

        math::safe_divide_or(total_entropy, node_count as f64, 0.0)
    }

    fn simulate_model_update(
        &self,
        model: &mut abcdeez_core::LearnerModel,
        task: &Task,
        response_correct: bool,
    ) {
        // Simulate how the model would update given this task and response
        let operation_type = format!("{:?}", task.operation); // Convert enum to string
        let difficulty = task.difficulty;

        // Simple Bayesian update simulation
        // In practice, this would use the actual Bayesian update mechanism
        if let Some(embedding) = model.node_embeddings.get_mut(&operation_type) {
            // Update proficiency based on response
            let learning_rate = 0.1; // Could be adaptive

            if response_correct {
                // Correct response increases position (mastery), decreases uncertainty
                embedding.position += learning_rate * (1.0 - embedding.position);
                embedding.uncertainty *= 1.0 - learning_rate * 0.5;
            } else {
                // Incorrect response decreases position, may increase uncertainty
                embedding.position *= 1.0 - learning_rate * 0.5;
                embedding.uncertainty = (embedding.uncertainty + learning_rate * 0.1).min(1.0);
            }

            // Apply difficulty-based adjustments
            let difficulty_factor = (difficulty - 0.5) * 0.1;
            embedding.position = (embedding.position + difficulty_factor).max(0.0).min(1.0);
        }
    }

    fn generate_subtle_hint(
        &self,
        task: &Task,
        _model: &abcdeez_core::LearnerModel,
    ) -> String {
        match &task.task_type {
            abcdeez_core::TaskType::Successor { .. } => {
                "Think about what comes next in the sequence.".to_string()
            }
            abcdeez_core::TaskType::Predecessor { .. } => {
                "Consider what comes before in the sequence.".to_string()
            }
            abcdeez_core::TaskType::PairwiseOrder { .. } => {
                "Which one comes first in order?".to_string()
            }
            _ => "Take your time and think step by step.".to_string(),
        }
    }

    fn generate_mild_hint(
        &self,
        task: &Task,
        _model: &abcdeez_core::LearnerModel,
    ) -> String {
        match &task.task_type {
            abcdeez_core::TaskType::Successor { .. } => {
                format!(
                    "If the sequence is ...{}, what comes after {}?",
                    task.prompt.chars().take(3).collect::<String>(),
                    task.prompt.chars().last().unwrap_or('?')
                )
            }
            abcdeez_core::TaskType::Predecessor { .. } => {
                format!(
                    "If the sequence is {}..., what comes before {}?",
                    task.prompt.chars().take(3).collect::<String>(),
                    task.prompt.chars().next().unwrap_or('?')
                )
            }
            abcdeez_core::TaskType::PairwiseOrder { .. } => {
                "Try saying both options out loud to hear which comes first.".to_string()
            }
            _ => "Break the problem into smaller parts.".to_string(),
        }
    }

    fn generate_strong_hint(
        &self,
        task: &Task,
        _model: &abcdeez_core::LearnerModel,
    ) -> String {
        // Provide a very direct hint that almost gives away the answer
        if !task.options.is_empty() {
            let correct_option = &task.options[0]; // Assume first option is correct
            format!(
                "The answer starts with '{}'",
                correct_option.chars().next().unwrap_or('?')
            )
        } else {
            format!(
                "The answer is related to '{}'",
                task.prompt.chars().take(2).collect::<String>()
            )
        }
    }

    async fn log_intervention(
        &self,
        session_id: Uuid,
        intervention: &InterventionAction,
    ) -> Result<()> {
        let intervention_id = Uuid::new_v4();
        let session_id_bytes = session_id.as_bytes();
        let intervention_id_bytes = intervention_id.as_bytes();

        let (intervention_type, details) = match intervention {
            InterventionAction::ProvideHint(level) => (
                "hint",
                serde_json::json!({ "hint_level": format!("{:?}", level) }),
            ),
            InterventionAction::DecreaseDifficulty => (
                "difficulty_change",
                serde_json::json!({ "action": "reduce" }),
            ),
            InterventionAction::SuggestBreak => (
                "break_suggestion",
                serde_json::json!({ "reason": "extended_session" }),
            ),
            InterventionAction::ProvideWorkedExample(example) => {
                ("worked_example", serde_json::json!({ "example": example }))
            }
            InterventionAction::IncreaseDifficulty => (
                "difficulty_change",
                serde_json::json!({ "action": "increase" }),
            ),
            InterventionAction::SkipTask => ("skip_task", serde_json::json!({ "action": "skip" })),
        };

        sqlx::query(
            "INSERT INTO interventions (id, session_id, intervention_type, details) 
             VALUES (?, ?, ?, ?)",
        )
        .bind(&intervention_id_bytes[..])
        .bind(&session_id_bytes[..])
        .bind(intervention_type)
        .bind(details.to_string())
        .execute(self.learner_service.db.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        Ok(())
    }
}

impl InterventionSystem {
    pub fn new(_topology: Topology) -> Self {
        InterventionSystem
    }
}
