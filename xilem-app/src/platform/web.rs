// Web platform implementation using xilem_web

use xilem_web::{AnyDomView, App};
use crate::state::AppState;
use crate::components::Component;
use super::{PlatformView, PlatformRunner};

// Type alias for type-erased web views
type WebDomView = Box<AnyDomView<AppState>>;

pub struct WebView;

impl PlatformView for WebView {
    type Output = WebDomView;
}

pub struct WebRunner;

impl PlatformRunner for WebRunner {
    fn run(app_state: AppState) -> Result<(), Box<dyn std::error::Error>> {
        // Get the root element from DOM
        let root = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("app")
            .expect("No element with id 'app' found");
        
        let app = App::new(root, app_state, app_logic);
        app.run();
        
        Ok(())
    }
}

fn app_logic(state: &mut AppState) -> WebDomView {
    // Use the actual app_logic from app.rs
    // When xilem-web feature is enabled, ComponentOutput is WebComponent
    use crate::components::web_components::WebComponent;
    
    let component_output = crate::app::app_logic(state);
    
    // Build the component and extract the inner DomView
    let built = component_output.build();
    
    // WebComponent is a tuple struct wrapping Box<AnyDomView<AppState>>
    // We need to extract it
    let WebComponent(dom_view) = built;
    dom_view
}