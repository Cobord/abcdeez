// Responsive layout utilities

use crate::state::AppState;
use crate::components::{ComponentOutput, Components, AppComponents};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Breakpoint {
    Mobile,     // < 768px
    Tablet,     // 768px - 1024px
    Desktop,    // > 1024px
}

impl Breakpoint {
    pub fn from_width(width: f64) -> Self {
        if width < 768.0 {
            Breakpoint::Mobile
        } else if width < 1024.0 {
            Breakpoint::Tablet
        } else {
            Breakpoint::Desktop
        }
    }
}

/// Create a responsive grid that adjusts columns based on screen size
pub fn responsive_grid(
    items: Vec<ComponentOutput>,
    mobile_cols: usize,
    tablet_cols: usize,
    desktop_cols: usize,
    window_width: f64,
) -> ComponentOutput {
    let breakpoint = Breakpoint::from_width(window_width);
    let cols = match breakpoint {
        Breakpoint::Mobile => mobile_cols,
        Breakpoint::Tablet => tablet_cols,
        Breakpoint::Desktop => desktop_cols,
    };
    
    // Group items into rows based on column count
    let mut rows = vec![];
    let mut current_row = vec![];
    let items_len = items.len();
    
    for (i, item) in items.into_iter().enumerate() {
        current_row.push(item);
        if (i + 1) % cols == 0 || i == items_len - 1 {
            rows.push(Components::simple_flex_row(current_row));
            current_row = vec![];
        }
    }
    
    Components::simple_flex_column(rows)
}

/// Create a responsive container with different max widths
pub fn responsive_container(
    content: ComponentOutput,
    window_width: f64,
) -> ComponentOutput {
    let breakpoint = Breakpoint::from_width(window_width);
    let max_width = match breakpoint {
        Breakpoint::Mobile => window_width * 0.95,  // Use most of the screen
        Breakpoint::Tablet => 768.0,                // Cap at tablet width
        Breakpoint::Desktop => 1200.0,              // Cap at reasonable desktop width
    };
    
    Components::centered_container(max_width, content)
}

/// Create responsive spacing that adjusts based on screen size
pub fn responsive_spacing(window_width: f64) -> f64 {
    let breakpoint = Breakpoint::from_width(window_width);
    match breakpoint {
        Breakpoint::Mobile => 8.0,
        Breakpoint::Tablet => 16.0,
        Breakpoint::Desktop => 24.0,
    }
}

/// Create a responsive stack that switches between row and column
pub fn responsive_stack(
    items: Vec<ComponentOutput>,
    window_width: f64,
    switch_at: f64,
) -> ComponentOutput {
    if window_width >= switch_at {
        Components::simple_flex_row(items)
    } else {
        Components::simple_flex_column(items)
    }
}

/// Create responsive text size based on screen size
pub fn responsive_text_size(base_size: f64, window_width: f64) -> f64 {
    let breakpoint = Breakpoint::from_width(window_width);
    match breakpoint {
        Breakpoint::Mobile => base_size * 0.9,
        Breakpoint::Tablet => base_size,
        Breakpoint::Desktop => base_size * 1.1,
    }
}

/// Determine if sidebar should be shown
pub fn should_show_sidebar(window_width: f64) -> bool {
    window_width >= 768.0  // Show sidebar on tablet and desktop
}

/// Determine if bottom navigation should be shown
pub fn should_show_bottom_nav(window_width: f64) -> bool {
    window_width < 768.0  // Show bottom nav only on mobile
}

/// Create a responsive app shell that adapts to screen size
pub fn responsive_app_shell(
    state: &mut AppState,
    content: ComponentOutput,
    window_width: f64,
) -> ComponentOutput {
    use crate::components::navigation;
    
    let show_sidebar = should_show_sidebar(window_width);
    let show_bottom_nav = should_show_bottom_nav(window_width);
    
    if show_sidebar {
        // Desktop/tablet layout with sidebar
        let sidebar = navigation::navigation_sidebar(state);
        let sidebar_width = if window_width >= 1024.0 { 250.0 } else { 200.0 };
        let sidebar_container = Components::centered_container(sidebar_width, sidebar);
        
        // Add breadcrumb navigation at top of content
        let breadcrumb = navigation::breadcrumb_navigation(state);
        let content_with_breadcrumb = Components::simple_flex_column(vec![
            breadcrumb,
            content,
        ]);
        
        Components::simple_flex_row(vec![
            sidebar_container,
            content_with_breadcrumb,
        ])
    } else if show_bottom_nav {
        // Mobile layout with bottom navigation
        let bottom_nav = Components::bottom_nav_bar(
            state.current_screen.clone(),
            |state, screen| {
                state.navigate(screen);
            }
        );
        
        Components::app_scaffold(
            Components::empty(),
            content,
            Some(bottom_nav),
        )
    } else {
        // Fallback: just the content
        content
    }
}

/// Create responsive padding values
pub struct ResponsivePadding {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl ResponsivePadding {
    pub fn from_width(window_width: f64) -> Self {
        let breakpoint = Breakpoint::from_width(window_width);
        match breakpoint {
            Breakpoint::Mobile => Self {
                top: 8.0,
                right: 8.0,
                bottom: 8.0,
                left: 8.0,
            },
            Breakpoint::Tablet => Self {
                top: 16.0,
                right: 16.0,
                bottom: 16.0,
                left: 16.0,
            },
            Breakpoint::Desktop => Self {
                top: 24.0,
                right: 32.0,
                bottom: 24.0,
                left: 32.0,
            },
        }
    }
}