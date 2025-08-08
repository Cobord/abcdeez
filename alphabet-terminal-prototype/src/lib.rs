// Re-export all core modules for use as a library
pub mod topology;
pub mod learner;
pub mod tasks;
pub mod adaptive;
pub mod statistics;
pub mod bayesian;
pub mod extended_tasks;
pub mod music;
pub mod export;
pub mod navigation;
pub mod boundaries;
pub mod prediction;
pub mod hints;
pub mod macro_learning;
pub mod transfer_learning;
pub mod statistical_validation;
pub mod strategy_mixture;
pub mod hierarchical_bayes;
pub mod experiments;
pub mod demo;
#[cfg(feature = "cli")]
pub mod ui;

// Re-export commonly used types at the root level
pub use topology::{Topology, TopologyType, Node, Edge};
pub use learner::{LearnerModel, LearnerMetrics, OperationType};
pub use tasks::{Task, TaskType, TaskGenerator, TaskSession};
pub use adaptive::AdaptiveScheduler;
pub use bayesian::BayesianLearnerModel;
pub use statistics::{ExGaussianModel, SessionAnalyzer, StrategyType};
pub use export::{LearnerDataExport, PopulationAnalyzer};
pub use music::{MusicTheory, MusicStructure, MusicTaskGenerator};
pub use prediction::{PerformancePredictor, ScheduleOptimizer};
pub use hints::{StruggleDetector, HintGenerator, InterventionSystem};

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        Topology, TopologyType, 
        LearnerModel, LearnerMetrics,
        Task, TaskType, TaskGenerator,
        AdaptiveScheduler,
        MusicTheory, MusicStructure,
    };
}