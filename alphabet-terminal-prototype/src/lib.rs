// Re-export all core modules for use as a library
pub mod ab_testing;
pub mod adaptive;
pub mod audio_recording;
pub mod audit_trail;
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
pub mod extended_tasks;
pub mod hierarchical_bayes;
pub mod hints;
pub mod interaction_tracking;
pub mod irb_compliance;
pub mod learner;
pub mod macro_learning;
pub mod mixed_effects;
pub mod multi_session;
pub mod music;
pub mod navigation;
pub mod performance_tracing;
pub mod power_analysis;
pub mod prediction;
pub mod preregistration;
pub mod protocol_version_control;
pub mod protocol_versioning;
#[cfg(feature = "cli")]
pub mod research_dashboard;
pub mod seed_management;
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
pub use ab_testing::{ABTest, ABTestFramework, ABTestResults, TestVariant};
pub use adaptive::AdaptiveScheduler;
pub use audio_recording::{
    AudioMetrics, AudioRecorder, AudioSession, ThinkAloudAnalyzer, ThinkAloudSegment,
};
pub use audit_trail::{
    Actor, ActorType, AuditConfiguration, AuditLevel, AuditTrailManager, EventType, Operation,
    Outcome, Resource,
};
pub use bayesian::{BayesianLearnerModel, ResponseData};
pub use citation_manager::{
    Author, BibliographyFormat, BibliographyStyle, CitationManager, MethodologyReport, Publication,
    Reference, ReferenceType,
};
pub use error::{Error, Result};
pub use experimental_design::{
    CounterbalancingMethod, ExperimentalDesign, ExperimentalDesigner, RandomizationType,
};
pub use export::{LearnerDataExport, PopulationAnalyzer};
pub use hints::{
    HintGenerator, HintLevel, InterventionAction, InterventionSystem, StruggleDetector,
    StruggleLevel,
};
pub use interaction_tracking::{
    InteractionMetrics, InteractionSession, InteractionTracker, KeystrokeEvent, MouseEvent,
};
pub use irb_compliance::{ConsentTemplate, IRBApplication, IRBComplianceGenerator, StudySummary};
pub use learner::{LearnerMetrics, LearnerModel, OperationType};
pub use mixed_effects::{
    MixedEffectsAnalyzer, MixedEffectsData, MixedEffectsModel, MixedEffectsResults,
    RandomEffectSpec,
};
pub use multi_session::{
    LongitudinalAnalysis, MultiSessionExperiment, MultiSessionManager, SessionPlan,
};
pub use music::{MusicStructure, MusicTaskGenerator, MusicTheory};
pub use performance_tracing::{
    AsyncPerformanceTracker, CriticalPathMonitor, PerformanceMetrics, PerformanceTracker,
};
pub use power_analysis::{
    EffectSizeCalculator, PowerAnalysis, PowerAnalyzer, RealTimeEffectMonitor, RealTimeMonitor,
};
pub use prediction::{PerformancePredictor, ScheduleOptimizer};
pub use protocol_version_control::{
    CollaboratorRole, ProtocolChange, ProtocolSnapshot, ProtocolVersion, ProtocolVersionControl,
    ProtocolVersionManager,
};
pub use protocol_versioning::{ProtocolRepository, SemanticVersion};
pub use seed_management::{
    ExperimentSeed, RandomizationEvent, ReproducibilityManifest, SeedManager, SessionSeed,
};
pub use sensor_integration::{
    MockEEGSensor, MockEyeTracker, MockGSRSensor, SensorConfig, SensorManager, SensorSession,
    SensorType,
};
pub use statistical_validation::{
    AssumptionChecks, HomoscedasticityTest, NormalityTest, OutlierAnalysis, StatisticalValidator,
};
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
