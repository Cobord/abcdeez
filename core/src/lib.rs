// Core module structure
pub mod core;
pub mod learning;
pub mod tasks;
pub mod statistics;
pub mod experiments;
pub mod compliance;
pub mod protocol;
pub mod data;
pub mod ui;
pub mod demo;

// Test modules
#[cfg(test)]
mod tests;

// Re-export commonly used types at the root level for backwards compatibility
pub use core::{
    backend::*, config::*, error::{Error, Result}, topology::{Edge, Node, Topology, TopologyType}
};

pub use learning::{
    AdaptiveScheduler, BayesianLearnerModel, LearnerMetrics, LearnerModel, OperationType,
    ResponseData,
};

pub use tasks::{
    MusicStructure, MusicTaskGenerator, MusicTheory, Task, TaskGenerator, TaskSession, TaskType,
};

pub use statistics::{
    AssumptionChecks, DetailedStatistics, EffectSizeCalculator, ExGaussianModel,
    ExGaussianParameters, HomoscedasticityTest, MixedEffectsAnalyzer, MixedEffectsData,
    MixedEffectsModel, MixedEffectsResults, NormalityTest, OutlierAnalysis, PerformancePredictor,
    PowerAnalysis, PowerAnalyzer, RandomEffectSpec, RealTimeEffectMonitor, RealTimeMonitor,
    ResponseTimeDistribution, ScheduleOptimizer, SessionAnalyzer, StatisticalValidator,
    StrategyType,
};

pub use experiments::{
    ABTest, ABTestFramework, ABTestResults, CounterbalancingMethod, ExperimentalDesign,
    ExperimentalDesigner, LongitudinalAnalysis, MultiSessionExperiment, MultiSessionManager,
    RandomizationType, SessionPlan, TestVariant,
};

pub use compliance::{
    Actor, ActorType, AuditConfiguration, AuditLevel, AuditTrailManager, Author,
    BibliographyFormat, BibliographyStyle, CitationManager, ConsentTemplate, EventType,
    IRBApplication, IRBComplianceGenerator, MethodologyReport, Operation, Outcome, Publication,
    Reference, ReferenceType, Resource, StudySummary,
};

pub use protocol::{
    CollaboratorRole, ExperimentSeed, ProtocolChange, ProtocolRepository, ProtocolSnapshot,
    ProtocolVersion, ProtocolVersionControl, ProtocolVersionManager, RandomizationEvent,
    ReproducibilityManifest, SeedManager, SemanticVersion, SessionSeed,
};

pub use data::{
    AudioMetrics, AudioRecorder, AudioSession, AsyncPerformanceTracker, CriticalPathMonitor,
    InteractionMetrics, InteractionSession, InteractionTracker, KeystrokeEvent, LearnerDataExport,
    MockEEGSensor, MockEyeTracker, MockGSRSensor, MouseEvent, PerformanceMetrics,
    PerformanceTracker, PopulationAnalyzer, SensorConfig, SensorManager, SensorSession,
    SensorType, ThinkAloudAnalyzer, ThinkAloudSegment,
};

pub use ui::{
    HintGenerator, HintLevel, InterventionAction, InterventionSystem, StruggleDetector,
    StruggleLevel,
};

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        core::{Topology, TopologyType},
        learning::{AdaptiveScheduler, LearnerMetrics, LearnerModel},
        tasks::{MusicStructure, MusicTheory, Task, TaskGenerator, TaskType},
    };
}