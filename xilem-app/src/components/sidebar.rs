// Sidebar navigation component for desktop/tablet layouts

use crate::state::{AppState, Screen};
use crate::components::{AppColor, AppComponents, ComponentOutput, Components, TabItem};

#[derive(Debug, Clone)]
pub struct SidebarConfig {
    pub width: f64,
    pub collapsible: bool,
    pub show_user_info: bool,
    pub show_search: bool,
}

impl Default for SidebarConfig {
    fn default() -> Self {
        Self {
            width: 260.0,
            collapsible: true,
            show_user_info: true,
            show_search: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavGroup {
    pub title: String,
    pub items: Vec<NavItem>,
    pub collapsed: bool,
}

#[derive(Debug, Clone)]
pub struct NavItem {
    pub label: String,
    pub screen: Screen,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub sub_items: Vec<NavItem>,
}

impl NavItem {
    pub fn new(label: impl Into<String>, screen: Screen) -> Self {
        Self {
            label: label.into(),
            screen,
            icon: None,
            badge: None,
            sub_items: Vec::new(),
        }
    }
    
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    
    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
    
    pub fn with_sub_items(mut self, items: Vec<NavItem>) -> Self {
        self.sub_items = items;
        self
    }
}

/// Build the main navigation structure
pub fn build_navigation_structure(state: &AppState) -> Vec<NavGroup> {
    let mut groups = vec![];
    
    // Main navigation group
    groups.push(NavGroup {
        title: "Main".to_string(),
        items: vec![
            NavItem::new("Dashboard", Screen::Dashboard)
                .with_icon("📊"),
            NavItem::new("Learning", Screen::Learning)
                .with_icon("📚"),
            NavItem::new("Training", Screen::Training)
                .with_icon("🎯"),
        ],
        collapsed: false,
    });
    
    // Analytics group
    groups.push(NavGroup {
        title: "Analytics".to_string(),
        items: vec![
            NavItem::new("Progress", Screen::Progress)
                .with_icon("📈"),
            NavItem::new("Analytics", Screen::Analytics)
                .with_icon("📉"),
            NavItem::new("Visualizations", Screen::Visualizations)
                .with_icon("📊"),
        ],
        collapsed: false,
    });
    
    // Social group - with badges for notifications
    let mut social_items = vec![
        NavItem::new("Leaderboard", Screen::Leaderboard)
            .with_icon("🏆"),
        NavItem::new("Challenges", Screen::Challenges)
            .with_icon("⚔️"),
    ];
    
    // Add badge for friend requests if any
    if let Some(profile) = state.get_current_user_profile() {
        if profile.pending_friend_requests > 0 {
            social_items.push(
                NavItem::new("Friends", Screen::Friends)
                    .with_icon("👥")
                    .with_badge(profile.pending_friend_requests.to_string())
            );
        } else {
            social_items.push(
                NavItem::new("Friends", Screen::Friends)
                    .with_icon("👥")
            );
        }
    } else {
        social_items.push(
            NavItem::new("Friends", Screen::Friends)
                .with_icon("👥")
        );
    }
    
    groups.push(NavGroup {
        title: "Social".to_string(),
        items: social_items,
        collapsed: false,
    });
    
    // User group
    groups.push(NavGroup {
        title: "Account".to_string(),
        items: vec![
            NavItem::new("Profile", Screen::Profile)
                .with_icon("👤"),
            NavItem::new("Settings", Screen::Settings)
                .with_icon("⚙️"),
        ],
        collapsed: false,
    });
    
    // Developer tools (always show for now - can add debug check later)
    groups.push(NavGroup {
        title: "Developer".to_string(),
        items: vec![
            NavItem::new("Widget Gallery", Screen::WidgetGallery)
                .with_icon("🎨"),
            NavItem::new("Demo Mode", Screen::Dashboard)  // Will trigger demo
                .with_icon("🎭"),
        ],
        collapsed: false,
    });
    
    groups
}

/// Create a sidebar navigation component
pub fn sidebar_navigation(
    state: &mut AppState,
    config: SidebarConfig,
) -> ComponentOutput {
    let current_screen = state.current_screen.clone();
    let nav_groups = build_navigation_structure(state);
    
    let mut sidebar_items = vec![];
    
    // User info section at top
    if config.show_user_info {
        sidebar_items.push(user_info_section(state));
        sidebar_items.push(Components::divider(crate::components::Orientation::Horizontal));
    }
    
    // Search bar
    if config.show_search {
        sidebar_items.push(search_bar(state));
        sidebar_items.push(Components::spacer(crate::components::SpacerSize::Small));
    }
    
    // Navigation groups
    for group in nav_groups {
        sidebar_items.push(nav_group_component(group, &current_screen, state));
    }
    
    // Quick actions at bottom
    sidebar_items.push(Components::spacer(crate::components::SpacerSize::Large));
    sidebar_items.push(quick_actions_section(state));
    
    // Wrap in scrollable container
    let sidebar_content = Components::simple_flex_column(sidebar_items);
    let scrollable_sidebar = Components::scrollable(sidebar_content);
    
    // Apply width constraint
    Components::centered_container(config.width, scrollable_sidebar)
}

/// Create user info section for sidebar
fn user_info_section(state: &AppState) -> ComponentOutput {
    let username = state.user.as_ref()
        .map(|u| u.username.as_str())
        .unwrap_or("Guest");
    
    let profile = state.get_current_user_profile();
    let (level, streak) = if let Some(profile) = profile {
        (
            format!("Level {}", profile.level),
            profile.streak.display_status()
        )
    } else {
        ("Level 1".to_string(), "No streak".to_string())
    };
    
    let items = vec![
        Components::label(username),
        Components::label(&format!("{} • {}", level, streak)),
    ];
    
    Components::simple_flex_column(items)
}

/// Create search bar for sidebar
fn search_bar(state: &mut AppState) -> ComponentOutput {
    Components::labeled_input(
        "Search...",
        state.search_query.clone(),
        |state, query| {
            state.search_query = query;
            // In a real app, this would filter navigation items
        }
    )
}

/// Create a navigation group component
fn nav_group_component(
    group: NavGroup,
    current_screen: &Screen,
    state: &mut AppState,
) -> ComponentOutput {
    let mut items = vec![];
    
    // Group title
    items.push(Components::label(&group.title));
    
    // Navigation items
    for nav_item in group.items {
        items.push(nav_item_component(nav_item, current_screen, state, 0));
    }
    
    Components::simple_flex_column(items)
}

/// Create a single navigation item
fn nav_item_component(
    item: NavItem,
    current_screen: &Screen,
    state: &mut AppState,
    indent_level: usize,
) -> ComponentOutput {
    let is_active = &item.screen == current_screen;
    let screen = item.screen.clone();
    
    // Build label with icon and badge
    let mut label_parts = vec![];
    if let Some(icon) = &item.icon {
        label_parts.push(icon.clone());
    }
    label_parts.push(item.label.clone());
    if let Some(badge) = &item.badge {
        label_parts.push(format!("({})", badge));
    }
    let label = label_parts.join(" ");
    
    // Create the nav button
    let button = Components::nav_button(
        &label,
        item.screen,
        is_active,
        move |state| {
            state.navigate(screen.clone());
        }
    );
    
    // Handle sub-items if present
    if !item.sub_items.is_empty() {
        let mut sub_item_views = vec![button];
        for sub_item in item.sub_items {
            sub_item_views.push(nav_item_component(
                sub_item,
                current_screen,
                state,
                indent_level + 1
            ));
        }
        Components::simple_flex_column(sub_item_views)
    } else {
        button
    }
}

/// Create quick actions section
fn quick_actions_section(state: &mut AppState) -> ComponentOutput {
    let actions = vec![
        Components::action_button(
            "Quick Start",
            AppColor::Primary,
            |state| {
                state.navigate(Screen::Learning);
            }
        ),
        Components::action_button(
            "Daily Goal",
            AppColor::Success,
            |state| {
                state.navigate(Screen::Progress);
            }
        ),
    ];
    
    Components::simple_flex_row(actions)
}

/// Create a compact sidebar for mobile/collapsed state
pub fn compact_sidebar(
    state: &mut AppState,
) -> ComponentOutput {
    let current_screen = state.current_screen.clone();
    
    // Just show icons for main screens
    let nav_items = vec![
        ("📊", Screen::Dashboard),
        ("📚", Screen::Learning),
        ("📈", Screen::Progress),
        ("👤", Screen::Profile),
        ("⚙️", Screen::Settings),
    ];
    
    let buttons: Vec<ComponentOutput> = nav_items.into_iter().map(|(icon, screen)| {
        let is_active = screen == current_screen;
        let screen_clone = screen.clone();
        Components::nav_button(
            icon,
            screen,
            is_active,
            move |state| {
                state.navigate(screen_clone.clone());
            }
        )
    }).collect();
    
    Components::simple_flex_column(buttons)
}