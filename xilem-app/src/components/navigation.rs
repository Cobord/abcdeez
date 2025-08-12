// Navigation components for the app

use crate::state::{AppState, Screen};
use crate::components::{AppColor, AppComponents, ComponentOutput, Components};
use crate::components::sidebar::{sidebar_navigation, SidebarConfig};
use crate::components::layout::{adaptive_app_layout};

/// Create a responsive navigation sidebar for desktop/tablet
/// Delegates to the new sidebar module for better organization
pub fn navigation_sidebar(state: &mut AppState) -> ComponentOutput {
    let config = SidebarConfig::default();
    sidebar_navigation(state, config)
}

/// Create a single navigation item
fn nav_item(label: &str, screen: Screen, current: &Screen, state: &mut AppState) -> ComponentOutput {
    let is_active = current == &screen;
    let screen_clone = screen.clone();
    
    Components::nav_button(
        label,
        screen,
        is_active,
        move |state| {
            state.navigate(screen_clone.clone());
        }
    )
}

/// Create a navigation section with title
fn nav_section(title: &str, items: Vec<ComponentOutput>) -> ComponentOutput {
    let title = Components::label(title);
    let mut all_items = vec![title];
    all_items.extend(items);
    Components::simple_flex_column(all_items)
}

/// Create a responsive app layout with sidebar for desktop/tablet
/// Now uses the adaptive layout system from the layout module
pub fn responsive_app_layout(
    state: &mut AppState,
    content: ComponentOutput,
    window_width: f64,
    window_height: f64,
) -> ComponentOutput {
    adaptive_app_layout(state, content, window_width, window_height)
}

/// Create a breadcrumb navigation component
pub fn breadcrumb_navigation(state: &AppState) -> ComponentOutput {
    let mut breadcrumbs = vec![];
    
    // Always start with Dashboard
    breadcrumbs.push(Components::label("Dashboard"));
    
    // Add current screen if not dashboard
    if state.current_screen != Screen::Dashboard {
        breadcrumbs.push(Components::label(" > "));
        breadcrumbs.push(Components::label(state.current_screen.title()));
    }
    
    Components::simple_flex_row(breadcrumbs)
}

/// Create a quick action bar for common actions
pub fn quick_action_bar(state: &mut AppState) -> ComponentOutput {
    let actions = vec![
        Components::action_button(
            "Start Learning",
            AppColor::Primary,
            |state| {
                state.navigate(Screen::Learning);
            }
        ),
        Components::action_button(
            "View Progress",
            AppColor::Success,
            |state| {
                state.navigate(Screen::Progress);
            }
        ),
        Components::action_button(
            "Demo Mode",
            AppColor::Info,
            |state| {
                // Start demo if controller exists
                if let Some(demo) = &mut state.demo_controller {
                    let _ = demo.start_scenario("main_tour");
                }
            }
        ),
    ];
    
    Components::simple_flex_row(actions)
}