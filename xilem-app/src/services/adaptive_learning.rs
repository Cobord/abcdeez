// Adaptive learning service integrating abcdeez-core
use abcdeez_core::{
    Topology,
    AdaptiveScheduler, BayesianLearnerModel, LearnerModel,
    Task, TaskGenerator, TaskType, TaskType as CoreTaskType,
};
// Intervention system removed from core
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::models::{self, TaskResponse};
use crate::state::SessionState;

pub struct AdaptiveLearningService {
    scheduler: AdaptiveScheduler,
    topology: Topology,
    session_start: Instant,
    task_start: Option<Instant>,
    current_task: Option<Task>,
    learner_id: Uuid,
}

impl AdaptiveLearningService {
    pub fn new(topology: Topology, learner_id: Uuid) -> Self {
        // Create a basic learner model for this learner
        let learner_model = LearnerModel::new(learner_id.to_string(), &topology);
        
        // Create the adaptive scheduler with the model and topology
        let scheduler = AdaptiveScheduler::new(
            learner_model,
            topology.clone(),
        );
        
        Self {
            scheduler,
            topology,
            session_start: Instant::now(),
            task_start: None,
            current_task: None,
            learner_id,
        }
    }
    
    pub fn from_existing_model(
        learner_model: LearnerModel,
        topology: Topology,
    ) -> Self {
        let learner_id = Uuid::new_v4(); // Extract from model if available
        
        let scheduler = AdaptiveScheduler::new(
            learner_model,
            topology.clone(),
        );
        
        Self {
            scheduler,
            topology,
            session_start: Instant::now(),
            task_start: None,
            current_task: None,
            learner_id,
        }
    }
    
    pub fn select_next_task(&mut self) -> Result<models::Task> {
        // Use the adaptive scheduler to select the next task
        let core_task = self.scheduler.select_next_task();
        
        // Store the current task for intervention checking
        self.current_task = Some(core_task.clone());
        
        // Start timing for this task
        self.task_start = Some(Instant::now());
        
        // Convert to app model
        Ok(self.convert_task(core_task))
    }
    
    pub fn submit_response(&mut self, response: TaskResponse, task: &Task) -> Result<()> {
        // Calculate response time
        let response_time_ms = if let Some(start) = self.task_start {
            start.elapsed().as_millis() as u128
        } else {
            0
        };
        
        // Update the learner model through the scheduler
        self.scheduler.update_model(
            task,
            response.is_correct,
            response_time_ms,
        );
        
        // Custom intervention logic could be added here based on response
        
        // Clear task timer
        self.task_start = None;
        
        Ok(())
    }
    
    // Intervention system removed - custom intervention logic could be added here
    
    pub fn get_hint(&mut self, level: usize) -> String {
        // Custom hint logic based on level
        // Intervention system was removed from core
        match level {
            0 => "Take your time and think about the pattern.".to_string(),
            1 => "Consider what comes before and after in the sequence.".to_string(),
            2 => "Try to recall the complete sequence from the beginning.".to_string(),
            _ => "The answer is part of the pattern you're learning.".to_string(),
        }
    }
    
    pub fn get_learner_metrics(&self) -> LearnerMetrics {
        // Create a temporary learner model to get metrics
        let model = LearnerModel::new(self.learner_id.to_string(), &self.topology);
        
        LearnerMetrics {
            overall_proficiency: 0.5, // Default value - would need actual calculation
            operation_proficiencies: std::collections::HashMap::new(),
            bidirectionality_index: model.get_bidirectionality_index(),
            symbolic_distance_slope: model.get_symbolic_distance_slope(),
            total_responses: 0, // Would need to track responses separately
            accuracy_rate: 0.0, // Would need to calculate from response history
            mean_response_time: 0.0, // Would need to calculate from response history
        }
    }
    
    pub fn export_learner_model(&self) -> LearnerModel {
        // Return a new learner model based on current state
        LearnerModel::new(self.learner_id.to_string(), &self.topology)
    }
    
    pub fn export_bayesian_model(&self) -> BayesianLearnerModel {
        // Return a new Bayesian model
        BayesianLearnerModel::new(&self.topology)
    }
    
    fn convert_task(&self, core_task: Task) -> models::Task {
        // Convert core task type directly - all core types are now supported
        let task_type = models::TaskType::Core(core_task.task_type.clone());
        
        models::Task {
            task_type,
            prompt: core_task.prompt,
            correct_answer: core_task.correct_answer,
            options: core_task.options,
            difficulty: core_task.difficulty,
            operation: format!("{:?}", core_task.operation),
        }
    }
    
    fn get_context_sequence(&self, item: &str) -> Vec<String> {
        // Get surrounding context for the item
        vec![
            item.to_string(),
            "?".to_string(),
            self.get_next_item(item, 2).unwrap_or_else(|| "...".to_string()),
        ]
    }
    
    fn get_jump_sequence(&self, from: &str, k: usize) -> Vec<String> {
        let mut sequence = vec![from.to_string()];
        for i in 1..=3 {
            if let Some(item) = self.get_next_item(from, i * k) {
                sequence.push(if i == 1 { "?".to_string() } else { item });
            }
        }
        sequence
    }
    
    fn get_next_item(&self, from: &str, steps: usize) -> Option<String> {
        // This is a simplified version - in real implementation would use the topology
        let alphabet = vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M"];
        if let Some(idx) = alphabet.iter().position(|&x| x == from) {
            if idx + steps < alphabet.len() {
                return Some(alphabet[idx + steps].to_string());
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct LearnerMetrics {
    pub overall_proficiency: f64,
    pub operation_proficiencies: std::collections::HashMap<String, f64>,
    pub bidirectionality_index: f64,
    pub symbolic_distance_slope: f64,
    pub total_responses: usize,
    pub accuracy_rate: f64,
    pub mean_response_time: f64,
}

// Intervention actions - can be customized for the app
#[derive(Debug, Clone)]
pub enum InterventionAction {
    ShowHint(String),
    ProvideScaffolding(String),
    SuggestBreak,
    AdjustDifficulty(f64),
    ShowWorkedExample(String),
}