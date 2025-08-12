// Web platform implementation using xilem_web

use xilem_web::{AnyDomView, App};
use crate::state::AppState;
use crate::components::Component;
use super::{PlatformView, PlatformRunner};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

// Type alias for type-erased web views
type WebDomView = Box<AnyDomView<AppState>>;

pub struct WebView;

impl PlatformView for WebView {
    type Output = WebDomView;
}

pub struct WebRunner;

impl PlatformRunner for WebRunner {
    fn run(mut app_state: AppState) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize with deep link if available
        app_state.init_with_deep_link();
        
        // Log the initial navigation state
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            if let Ok(location) = window.location().href() {
                web_sys::console::log_1(&format!("🔗 Initial URL: {}", location).into());
                web_sys::console::log_1(&format!("📍 Starting screen: {:?}", app_state.current_screen).into());
            }
            
            // Listen for popstate events (browser back/forward buttons)
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
                web_sys::console::log_1(&format!("🔙 Browser navigation event detected").into());
                
                if let Some(window) = web_sys::window() {
                    if let Ok(location) = window.location().href() {
                        web_sys::console::log_1(&format!("📍 New URL from browser navigation: {}", location).into());
                    }
                }
            }) as Box<dyn FnMut(_)>);
            
            window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref()).unwrap();
            closure.forget(); // Keep the closure alive
        }
        
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