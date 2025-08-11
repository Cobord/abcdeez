pub mod audio_recording;
pub mod sensor_integration;
pub mod interaction_tracking;
pub mod performance_tracing;
pub mod export;

pub use audio_recording::{
    AudioMetrics, AudioRecorder, AudioSession, ThinkAloudAnalyzer, ThinkAloudSegment,
};
pub use sensor_integration::{
    MockEEGSensor, MockEyeTracker, MockGSRSensor, SensorConfig, SensorManager, SensorSession,
    SensorType,
};
pub use interaction_tracking::{
    InteractionMetrics, InteractionSession, InteractionTracker, KeystrokeEvent, MouseEvent,
};
pub use performance_tracing::{
    AsyncPerformanceTracker, CriticalPathMonitor, PerformanceMetrics, PerformanceTracker,
};
pub use export::{LearnerDataExport, PopulationAnalyzer};