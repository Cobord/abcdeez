// Services module for backend communication and business logic

#[cfg(feature = "tokio-runtime")]
pub mod api;
#[cfg(feature = "websocket")]
pub mod websocket;
#[cfg(feature = "tokio-runtime")]
pub mod sync;
#[cfg(all(feature = "storage", feature = "tokio-runtime"))]
pub mod storage;
pub mod timing;
pub mod adaptive_learning;
pub mod interaction_tracking;

#[cfg(feature = "tokio-runtime")]
pub use api::ApiClient;
#[cfg(feature = "websocket")]
pub use websocket::WebSocketClient;
#[cfg(feature = "tokio-runtime")]
pub use sync::SyncService;
#[cfg(all(feature = "storage", feature = "tokio-runtime"))]
pub use storage::StorageService;
pub use timing::PrecisionTimer;
pub use adaptive_learning::{AdaptiveLearningService, LearnerMetrics, InterventionAction};
pub use interaction_tracking::{InteractionTrackingService, XilemInteractionHandler, InteractionMetrics};