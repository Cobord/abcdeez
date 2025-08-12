// Layout components for responsive app structure

use crate::state::{AppState, Screen};
use crate::components::{AppComponents, ComponentOutput, Components, responsive::Breakpoint};

/// Create a responsive app layout that adapts to screen size
pub fn adaptive_app_layout(
    state: &mut AppState,
    content: ComponentOutput,
    window_width: f64,
    window_height: f64,
) -> ComponentOutput {
    let breakpoint = Breakpoint::from_width(window_width);
    
    match breakpoint {
        Breakpoint::Mobile => mobile_layout(state, content),
        Breakpoint::Tablet => tablet_layout(state, content, window_width),
        Breakpoint::Desktop => desktop_layout(state, content, window_width, window_height),
    }
}

/// Desktop layout with sidebar and main content area
fn desktop_layout(
    state: &mut AppState,
    content: ComponentOutput,
    window_width: f64,
    _window_height: f64,
) -> ComponentOutput {
    use crate::components::sidebar::{sidebar_navigation, SidebarConfig};
    
    // Configure sidebar for desktop
    let sidebar_config = SidebarConfig {
        width: 280.0,
        collapsible: true,
        show_user_info: true,
        show_search: true,
    };
    
    let sidebar_width = sidebar_config.width;
    let sidebar = sidebar_navigation(state, sidebar_config);
    
    // Calculate content area width
    let content_width = window_width - sidebar_width - 40.0; // 40px for margins
    
    // Wrap content with proper spacing
    let content_area = Components::centered_container(
        content_width.max(800.0),
        content
    );
    
    // Use split pane for resizable layout
    Components::split_pane(sidebar, content_area, 0.25)
}

/// Tablet layout with collapsible sidebar
fn tablet_layout(
    state: &mut AppState,
    content: ComponentOutput,
    window_width: f64,
) -> ComponentOutput {
    use crate::components::sidebar::{compact_sidebar, sidebar_navigation, SidebarConfig};
    
    // Use compact sidebar for tablets in portrait, full for landscape
    let is_landscape = window_width > 900.0;
    
    let sidebar = if is_landscape {
        let config = SidebarConfig {
            width: 240.0,
            collapsible: true,
            show_user_info: true,
            show_search: false,
        };
        sidebar_navigation(state, config)
    } else {
        compact_sidebar(state)
    };
    
    // Stack horizontally with appropriate spacing
    Components::simple_flex_row(vec![sidebar, content])
}

/// Mobile layout with bottom navigation
fn mobile_layout(
    state: &mut AppState,
    content: ComponentOutput,
) -> ComponentOutput {
    // Simple header for mobile
    let header = mobile_header(state);
    
    // Bottom navigation bar
    let bottom_nav = Components::bottom_nav_bar(
        state.current_screen.clone(),
        |state, screen| {
            state.navigate(screen);
        }
    );
    
    // Use app scaffold for mobile
    Components::app_scaffold(header, content, Some(bottom_nav))
}

/// Mobile header with menu button and title
fn mobile_header(state: &AppState) -> ComponentOutput {
    let title = state.current_screen.title();
    let items = vec![
        Components::label("☰"),  // Menu icon
        Components::label(title),
        Components::spacer(crate::components::SpacerSize::Small),
    ];
    Components::simple_flex_row(items)
}

/// Create a dashboard grid layout
pub fn dashboard_grid(
    items: Vec<ComponentOutput>,
    window_width: f64,
) -> ComponentOutput {
    let breakpoint = Breakpoint::from_width(window_width);
    
    let cols = match breakpoint {
        Breakpoint::Mobile => 1,
        Breakpoint::Tablet => 2,
        Breakpoint::Desktop => 3,
    };
    
    Components::grid_layout(items, cols)
}

/// Create a two-column layout for forms and content
pub fn two_column_layout(
    left: ComponentOutput,
    right: ComponentOutput,
    window_width: f64,
) -> ComponentOutput {
    let breakpoint = Breakpoint::from_width(window_width);
    
    match breakpoint {
        Breakpoint::Mobile => {
            // Stack vertically on mobile
            Components::simple_flex_column(vec![left, right])
        }
        _ => {
            // Side by side on tablet/desktop
            let ratio = if window_width > 1200.0 { 0.4 } else { 0.5 };
            Components::split_pane(left, right, ratio)
        }
    }
}

/// Create a centered content layout with max width
pub fn content_container(
    content: ComponentOutput,
    window_width: f64,
) -> ComponentOutput {
    let breakpoint = Breakpoint::from_width(window_width);
    
    let max_width = match breakpoint {
        Breakpoint::Mobile => window_width - 32.0,  // 16px padding each side
        Breakpoint::Tablet => 720.0,
        Breakpoint::Desktop => 1080.0,
    };
    
    Components::centered_container(max_width, content)
}

/// Create a page layout with title and content
pub fn page_layout(
    title: &str,
    subtitle: Option<&str>,
    content: ComponentOutput,
) -> ComponentOutput {
    let mut header_items = vec![
        Components::label(title),
    ];
    
    if let Some(subtitle) = subtitle {
        header_items.push(Components::label(subtitle));
    }
    
    header_items.push(Components::divider(crate::components::Orientation::Horizontal));
    header_items.push(content);
    
    Components::simple_flex_column(header_items)
}

/// Create a floating action button layout
pub fn with_fab<F>(
    content: ComponentOutput,
    fab_text: &str,
    on_click: F,
) -> ComponentOutput 
where
    F: Fn(&mut AppState) + Send + Sync + 'static,
{
    // Stack content with floating button
    let fab = Components::action_button(
        fab_text,
        crate::components::AppColor::Primary,
        on_click
    );
    
    Components::stack(vec![content, fab], 0.0)
}

/// Create a tab layout with state management
pub fn tab_layout_with_state(
    state: &mut AppState,
    screen_key: &str,
    tabs: Vec<(&str, ComponentOutput)>,
) -> ComponentOutput {
    use crate::components::TabItem;
    
    // Get current tab index from state
    let current_index = state.get_tab_state(screen_key);
    
    // Build tab items
    let tab_items: Vec<TabItem> = tabs.iter()
        .map(|(label, _)| TabItem {
            label: label.to_string(),
            icon: None,
            badge_count: None,
        })
        .collect();
    
    // Get current content or empty
    let content = if current_index < tabs.len() {
        tabs.into_iter().nth(current_index).map(|(_, c)| c).unwrap_or(Components::empty())
    } else {
        Components::empty()
    };
    
    // Create tab bar with state management
    let screen_key_clone = screen_key.to_string();
    let tab_bar = Components::tab_bar(
        tab_items,
        current_index,
        move |state, index| {
            state.set_tab_state(screen_key_clone.clone(), index);
        }
    );
    
    Components::simple_flex_column(vec![tab_bar, content])
}

/// Create a tab layout (legacy - for compatibility)
pub fn tab_layout(
    tabs: Vec<(&str, ComponentOutput)>,
    current_index: usize,
) -> ComponentOutput {
    use crate::components::TabItem;
    
    // Build tab items
    let tab_items: Vec<TabItem> = tabs.iter()
        .map(|(label, _)| TabItem {
            label: label.to_string(),
            icon: None,
            badge_count: None,
        })
        .collect();
    
    // Get current content or empty
    let content = if current_index < tabs.len() {
        tabs.into_iter().nth(current_index).map(|(_, c)| c).unwrap_or(Components::empty())
    } else {
        Components::empty()
    };
    
    // Create tab bar (simplified - would need state management in real app)
    let tab_bar = Components::tab_bar(
        tab_items,
        current_index,
        |_state, _index| {
            // Tab switching would be handled here
        }
    );
    
    Components::simple_flex_column(vec![tab_bar, content])
}