// Services module for backend communication and business logic

pub mod api;
pub mod websocket;
pub mod sync;
pub mod storage;
pub mod timing;
pub mod adaptive_learning;
pub mod interaction_tracking;

pub use api::ApiClient;
pub use websocket::WebSocketClient;
pub use sync::SyncService;
pub use storage::StorageService;
pub use timing::PrecisionTimer;
pub use adaptive_learning::{AdaptiveLearningService, LearnerMetrics, InterventionAction};
pub use interaction_tracking::{InteractionTrackingService, XilemInteractionHandler, InteractionMetrics};