// Native platform implementation using xilem

use crate::state::AppState;
use super::{PlatformView, PlatformRunner};

use xilem::AnyWidgetView;

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
        
        let event_loop = xilem::EventLoop::with_user_event();
        
        let app = xilem::Xilem::new_simple(
            app_state,
            app_logic,
            xilem::WindowOptions::new("ABCDEEZ Learning")
                .with_initial_inner_size(winit::dpi::LogicalSize::new(1200.0, 800.0))
                .with_min_inner_size(winit::dpi::LogicalSize::new(320.0, 568.0))
                .with_resizable(true),
        );
        
        app.run_in(event_loop)?;
        Ok(())
    }
}

fn app_logic(state: &mut AppState) -> Box<AnyWidgetView<AppState>> {
    // Call the main app logic from app.rs and build it for native
    use crate::components::Component;
    let component = crate::app::app_logic(state);
    component.build().0
}