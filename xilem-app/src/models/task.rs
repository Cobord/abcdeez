// Task models

use serde::{Deserialize, Serialize};
use abcdeez_core::tasks::core::TaskType as CoreTaskType;
use abcdeez_core::tasks::extended::ExtendedTaskType;
use abcdeez_core::tasks::music::MusicStructure;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_type: TaskType,
    pub prompt: String,
    pub correct_answer: String,
    pub options: Vec<String>,
    pub difficulty: f64,
    pub operation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Core(CoreTaskType),
    Extended(ExtendedTaskType),
    Music(MusicTask),
    Navigation(NavigationTask),
    Boundary(BoundaryTask),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTask {
    pub structure: MusicStructure,
    pub task_variant: String, // e.g., "interval", "scale_degree", "chord_progression"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationTask {
    pub start: String,
    pub goal: String,
    pub constraints: Vec<String>, // Simplified constraint representation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryTask {
    pub boundary_type: String, // e.g., "chunk", "category", "hierarchical"
    pub span_size: usize,
    pub boundary_positions: Vec<usize>,
}