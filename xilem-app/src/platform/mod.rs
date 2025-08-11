// Platform abstraction layer

#[cfg(feature = "xilem-native")]
pub mod native;

#[cfg(feature = "xilem-web")]
pub mod web;

use crate::state::AppState;

// Platform-specific view trait
pub trait PlatformView {
    type Output;
}

// Platform-specific app runner
pub trait PlatformRunner {
    fn run(app_state: AppState) -> Result<(), Box<dyn std::error::Error>>;
}

// If both features are set, prefer native. If only web is set, use web.
#[cfg(feature = "xilem-native")]
pub use native::{NativeView as CurrentPlatformView, NativeRunner as CurrentPlatformRunner};

#[cfg(all(not(feature = "xilem-native"), feature = "xilem-web"))]
pub use web::{WebView as CurrentPlatformView, WebRunner as CurrentPlatformRunner};