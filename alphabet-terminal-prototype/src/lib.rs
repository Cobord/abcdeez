// Re-export all core modules for use as a library
pub mod adaptive;
pub mod backend;
pub mod bayesian;
pub mod boundaries;
pub mod demo;
pub mod error;
pub mod experiments;
pub mod export;
pub mod extended_tasks;
pub mod hierarchical_bayes;
pub mod hints;
pub mod learner;
pub mod macro_learning;
pub mod music;
pub mod navigation;
pub mod prediction;
pub mod statistical_validation;
pub mod statistics;
pub mod strategy_mixture;
pub mod tasks;
pub mod topology;
pub mod transfer_learning;
#[cfg(feature = "cli")]
pub mod tui;
#[cfg(feature = "cli")]
pub mod ui;

// Test modules
#[cfg(test)]
mod tests;

// Re-export commonly used types at the root level
pub use adaptive::AdaptiveScheduler;
pub use bayesian::{BayesianLearnerModel, ResponseData};
pub use error::{Error, Result};
pub use export::{LearnerDataExport, PopulationAnalyzer};
pub use hints::{
    HintGenerator, HintLevel, InterventionAction, InterventionSystem, StruggleDetector,
    StruggleLevel,
};
pub use learner::{LearnerMetrics, LearnerModel, OperationType};
pub use music::{MusicStructure, MusicTaskGenerator, MusicTheory};
pub use prediction::{PerformancePredictor, ScheduleOptimizer};
pub use statistics::{
    DetailedStatistics, ExGaussianModel, ExGaussianParameters, ResponseTimeDistribution,
    SessionAnalyzer, StrategyType,
};
pub use tasks::{Task, TaskGenerator, TaskSession, TaskType};
pub use topology::{Edge, Node, Topology, TopologyType};

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        AdaptiveScheduler, LearnerMetrics, LearnerModel, MusicStructure, MusicTheory, Task,
        TaskGenerator, TaskType, Topology, TopologyType,
    };
}
