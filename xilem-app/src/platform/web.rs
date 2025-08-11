// Web platform implementation using xilem_web

use xilem_web::{AnyDomView, DomView, App};
use xilem_web::elements::html as el;
use xilem_web::interfaces::Element;
use crate::state::AppState;
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
    use crate::state::Screen;
    
    // For now, return a simple view
    // In a full implementation, we'd translate the native views to web views
    match state.current_screen {
        Screen::Login | Screen::Signup => {
            Box::new(auth_view(state))
        },
        Screen::Dashboard => {
            Box::new(dashboard_view(state))
        },
        Screen::Learning => {
            Box::new(learning_view(state))
        },
        Screen::Settings => {
            Box::new(settings_view(state))
        },
        _ => {
            Box::new(dashboard_view(state))
        }
    }
}

// Simplified web views - these would be full implementations in production
fn auth_view(_state: &mut AppState) -> impl DomView<AppState> {
    el::div((
        el::h1("ABCDEEZ Learning"),
        el::div((
            el::input(()).attr("type", "text").attr("placeholder", "Username"),
            el::input(()).attr("type", "password").attr("placeholder", "Password"),
            el::button("Login"),
        ))
        .attr("class", "login-form"),
    ))
}

fn dashboard_view(state: &mut AppState) -> impl DomView<AppState> {
    el::div((
        el::h1("Dashboard"),
        el::p(format!("Welcome back, {}!", 
            state.user.as_ref().map(|u| u.username.as_str()).unwrap_or("Guest"))),
        el::button("Start Learning"),
        el::div((
            el::div("Sessions: 12").attr("class", "stat-card"),
            el::div("Accuracy: 85%").attr("class", "stat-card"),
            el::div("Streak: 5 days").attr("class", "stat-card"),
        ))
        .attr("class", "stats-grid"),
    ))
}

fn learning_view(_state: &mut AppState) -> impl DomView<AppState> {
    el::div((
        el::h2("Learning Session"),
        el::div("What letter comes after B?").attr("class", "question"),
        el::div((
            el::button("A").attr("class", "option"),
            el::button("C").attr("class", "option"),
            el::button("D").attr("class", "option"),
            el::button("E").attr("class", "option"),
        ))
        .attr("class", "options-grid"),
    ))
}

fn settings_view(state: &mut AppState) -> impl DomView<AppState> {
    el::div((
        el::h2("Settings"),
        el::div((
            el::h3("Profile"),
            el::p(format!("Username: {}", 
                state.user.as_ref().map(|u| u.username.as_str()).unwrap_or("Guest"))),
        )),
        el::div((
            el::h3("Preferences"),
            el::label((
                el::input(()).attr("type", "checkbox"),
                el::span("Enable notifications"),
            )),
            el::label((
                el::input(()).attr("type", "checkbox").attr("checked", "true"),
                el::span("Enable sound effects"),
            )),
        )),
        el::button("Sign Out").attr("class", "danger"),
    ))
}