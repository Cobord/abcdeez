// Native platform implementation using xilem

use crate::state::AppState;
use super::{PlatformView, PlatformRunner};

use xilem::AnyWidgetView;
#[cfg(feature = "embed-backend")]
use std::thread;
#[cfg(feature = "embed-backend")]
use std::time::Duration;

pub struct NativeView;

impl PlatformView for NativeView {
    type Output = Box<AnyWidgetView<AppState>>;
}

pub struct NativeRunner;

impl PlatformRunner for NativeRunner {
    fn run(app_state: AppState) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize tracing
        tracing_subscriber::fmt::init();
        tracing::info!("Starting ABCDEEZ Native App");

        // Optionally start embedded backend in a background thread
        #[cfg(feature = "embed-backend")]
        {
            tracing::info!("Launching embedded web-backend on 127.0.0.1:3000");
            thread::spawn(|| {
                // Dedicated Tokio runtime for the backend
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to build Tokio runtime for embedded backend");

                rt.block_on(async move {
                    if let Err(e) = web_backend::run_embedded(None).await {
                        tracing::error!(error = %e, "Embedded backend exited with error");
                    }
                });
            });
            // Small delay to let the server bind before UI starts making calls
            std::thread::sleep(Duration::from_millis(200));
        }
        
        let event_loop = xilem::EventLoop::with_user_event();
        
        let app = xilem::Xilem::new_simple(
            app_state,
            app_logic,
            xilem::WindowOptions::new("ABCDEEZ Learning")
                // Let the OS decide initial size (full screen on iOS)
                // Keep resizable and a reasonable minimum for desktop
                .with_min_inner_size(winit::dpi::LogicalSize::new(320.0, 568.0))
                .with_resizable(true),
        );
        
        app.run_in(event_loop)?;
        Ok(())
    }
}

fn app_logic(state: &mut AppState) -> Box<AnyWidgetView<AppState>> {
    // Provide conservative safe-area insets on iOS; zero elsewhere
    #[cfg(target_os = "ios")]
    {
        // Typical notch-era values; will be refined by native bridge later
        state.safe_area_insets.top = 44.0;
        state.safe_area_insets.bottom = 34.0;
        state.safe_area_insets.left = 0.0;
        state.safe_area_insets.right = 0.0;
    }
    #[cfg(target_os = "android")]
    {
        // Conservative defaults in logical units; replace with real WindowInsets via JNI
        state.safe_area_insets.top = 24.0;    // status bar
        state.safe_area_insets.bottom = 24.0; // gesture/nav bar (varies by OEM)
        state.safe_area_insets.left = 0.0;
        state.safe_area_insets.right = 0.0;
    }

    use xilem::view::*;
    use crate::components::Component;

    // Build the app's main content
    let inner = crate::app::app_logic(state).build().0;

    // Wrap with safe-area padding so content respects iOS insets
    let top_pad = state.safe_area_insets.top;
    let bottom_pad = state.safe_area_insets.bottom;

    Box::new(
        flex((
            sized_box(flex(())).height(top_pad),
            sized_box(inner).expand(),
            sized_box(flex(())).height(bottom_pad),
        ))
        .direction(Axis::Vertical)
        .must_fill_major_axis(true)
    )
}