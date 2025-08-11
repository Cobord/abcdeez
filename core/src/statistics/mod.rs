pub mod core;
pub mod validation;
pub mod mixed_effects;
pub mod power_analysis;
pub mod prediction;

pub use core::{
    CorrectionMethod, DetailedStatistics, ExGaussianModel, ExGaussianParameters,
    MultipleComparisonCorrection, PowerAnalysis, ResponseTimeDistribution, SessionAnalyzer, 
    StrategyAnalysis, StrategyType, TestResult,
};
pub use validation::{
    AssumptionChecks, HomoscedasticityTest, NormalityTest, OutlierAnalysis, StatisticalValidator,
};
pub use mixed_effects::{
    MixedEffectsAnalyzer, MixedEffectsData, MixedEffectsModel, MixedEffectsResults,
    RandomEffectSpec,
};
pub use power_analysis::{
    EffectSizeCalculator, PowerAnalysis as AdvancedPowerAnalysis, PowerAnalyzer, 
    RealTimeEffectMonitor, RealTimeMonitor,
};
pub use prediction::{PerformancePredictor, ScheduleOptimizer};