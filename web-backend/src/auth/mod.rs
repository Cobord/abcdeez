pub mod apple_notifications;

use axum::{
    Router,
    routing::post,
};
use std::sync::Arc;

/// Configure authentication routes
pub fn auth_routes(config: Arc<apple_notifications::AppleNotificationConfig>) -> Router {
    Router::new()
        .route("/apple/notifications", post(apple_notifications::handle_apple_notification))
        .with_state(config)
}