pub mod adaptation_service;
pub mod analytics_service;
pub mod audit;
pub mod batch_jobs;
pub mod federation_service;
pub mod learner_service;
pub mod privacy;
pub mod privacy_accounting;
pub mod protocol_service;

// OAuth services
pub mod apple_auth_service;
pub mod github_oauth_service;
pub mod oauth_service;

pub use adaptation_service::AdaptationService;
pub use analytics_service::AnalyticsService;
pub use batch_jobs::BatchJobService;
pub use federation_service::FederationService;
pub use learner_service::LearnerService;
pub use protocol_service::ProtocolService;

// OAuth service exports
