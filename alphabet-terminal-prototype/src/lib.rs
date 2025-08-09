// Re-export all core modules for use as a library
pub mod ab_testing;
pub mod adaptive;
pub mod audio_recording;
pub mod backend;
pub mod bayesian;
pub mod boundaries;
pub mod citation_manager;
pub mod config;
pub mod demo;
pub mod error;
pub mod experimental_design;
pub mod experiments;
pub mod export;
pub mod irb_compliance;
pub mod protocol_version_control;
pub mod seed_management;
pub mod audit_trail;
pub mod extended_tasks;
pub mod hierarchical_bayes;
pub mod performance_tracing;
pub mod hints;
pub mod interaction_tracking;
pub mod learner;
pub mod macro_learning;
pub mod mixed_effects;
pub mod multi_session;
pub mod music;
pub mod navigation;
pub mod power_analysis;
pub mod prediction;
pub mod preregistration;
pub mod protocol_versioning;
#[cfg(feature = "cli")]
pub mod research_dashboard;
pub mod sensor_integration;
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
pub use ab_testing::{ABTestFramework, ABTest, TestVariant, ABTestResults};
pub use adaptive::AdaptiveScheduler;
pub use audio_recording::{AudioRecorder, AudioSession, ThinkAloudAnalyzer, ThinkAloudSegment, AudioMetrics};
pub use bayesian::{BayesianLearnerModel, ResponseData};
pub use citation_manager::{CitationManager, Reference, MethodologyReport, BibliographyStyle, BibliographyFormat};
pub use error::{Error, Result};
pub use experimental_design::{ExperimentalDesign, ExperimentalDesigner, CounterbalancingMethod, RandomizationType};
pub use export::{LearnerDataExport, PopulationAnalyzer};
pub use hints::{
    HintGenerator, HintLevel, InterventionAction, InterventionSystem, StruggleDetector,
    StruggleLevel,
};
pub use interaction_tracking::{InteractionTracker, InteractionSession, KeystrokeEvent, MouseEvent, InteractionMetrics};
pub use irb_compliance::{IRBComplianceGenerator, ConsentTemplate, StudySummary, IRBApplication};
pub use protocol_version_control::{ProtocolVersionManager, ProtocolVersionControl, ProtocolSnapshot, ProtocolVersion, ProtocolChange, CollaboratorRole};
pub use seed_management::{SeedManager, ExperimentSeed, SessionSeed, RandomizationEvent, ReproducibilityManifest};
pub use audit_trail::{AuditTrailManager, AuditConfiguration, AuditLevel, EventType, Actor, ActorType, Resource, Operation, Outcome};
pub use performance_tracing::{PerformanceMetrics, PerformanceTracker, AsyncPerformanceTracker, CriticalPathMonitor};
pub use power_analysis::{PowerAnalyzer, EffectSizeCalculator, RealTimeMonitor, PowerAnalysis, RealTimeEffectMonitor};
pub use learner::{LearnerMetrics, LearnerModel, OperationType};
pub use mixed_effects::{MixedEffectsAnalyzer, MixedEffectsModel, MixedEffectsResults, MixedEffectsData, RandomEffectSpec};
pub use multi_session::{MultiSessionManager, MultiSessionExperiment, SessionPlan, LongitudinalAnalysis};
pub use music::{MusicStructure, MusicTaskGenerator, MusicTheory};
pub use prediction::{PerformancePredictor, ScheduleOptimizer};
pub use protocol_versioning::{ProtocolRepository, SemanticVersion};
pub use sensor_integration::{SensorManager, SensorSession, SensorConfig, SensorType, MockEEGSensor, MockGSRSensor, MockEyeTracker};
pub use statistical_validation::{StatisticalValidator, AssumptionChecks, NormalityTest, OutlierAnalysis, HomoscedasticityTest};
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
