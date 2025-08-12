// Exhaustive widget gallery showcasing ALL components in the new component library

use crate::components::{Component, ComponentOutput, Components, AppComponents, AppColor, AppTheme, SpacerSize, Orientation, TabItem};
use crate::components::layout::{page_layout, dashboard_grid, tab_layout, two_column_layout, content_container, with_fab};
use crate::components::cards::{stat_card_with_trend, Trend, metric_card, achievement_card, leaderboard_card, challenge_card, ChallengeDifficulty, session_summary_card, feature_card, domain_card_enhanced, DomainStats};
use crate::components::forms::{form_field, FieldType, toggle_switch, slider, radio_group, dropdown, date_picker, time_picker, multi_select, file_upload, color_picker, form_with_validation, FormField};
use crate::components::feedback::{toast, ToastType, progress_indicator, skeleton_loader, alert, AlertType, AlertAction, confirmation_dialog, snackbar, status_badge, StatusType, loading_overlay_with_progress, help_tooltip, inline_error, success_indicator, step_indicator};
use crate::state::{AppState, Screen};
use crate::models::Response;
use crate::demo::NotificationType;

/// Exhaustive widget gallery showcasing ALL components
pub fn widget_gallery(window_width: f64) -> ComponentOutput {
    let tabs = vec![
        ("Core", core_components_tab(window_width)),
        ("Layout", layout_components_tab(window_width)),
        ("Cards", cards_components_tab()),
        ("Forms", forms_components_tab()),
        ("Feedback", feedback_components_tab()),
        ("Navigation", navigation_components_tab()),
        ("Learning", learning_components_tab()),
        ("Visualization", visualization_components_tab()),
        ("Advanced", advanced_components_tab()),
    ];
    
    let tabbed_content = tab_layout(tabs, 0);
    
    page_layout(
        "🎨 Component Gallery",
        Some("Exhaustive showcase of all UI components"),
        tabbed_content
    )
}

fn core_components_tab(window_width: f64) -> ComponentOutput {
    let mut items = vec![];
    
    // Spacers
    items.push(Components::label("Spacers (visual representation):"));
    items.push(Components::spacer(SpacerSize::Small));
    items.push(Components::label("↑ Small spacer (8px)"));
    items.push(Components::spacer(SpacerSize::Medium));
    items.push(Components::label("↑ Medium spacer (16px)"));
    items.push(Components::spacer(SpacerSize::Large));
    items.push(Components::label("↑ Large spacer (24px)"));
    items.push(Components::spacer(SpacerSize::XLarge));
    items.push(Components::label("↑ XLarge spacer (32px)"));
    
    // Dividers
    items.push(Components::label("Dividers:"));
    items.push(Components::divider(Orientation::Horizontal));
    items.push(Components::label("↑ Horizontal divider"));
    
    // Basic components
    items.push(Components::label("Basic Components:"));
    items.push(Components::simple_label("Simple label text".to_string()));
    items.push(Components::label("Regular label"));
    
    // Buttons
    items.push(Components::spacer(SpacerSize::Large));
    items.push(Components::label("Buttons:"));
    
    let button_row = vec![
        Components::simple_button("Simple", |state: &mut AppState| {
            tracing::info!("Simple button clicked");
            state.add_notification("Simple button clicked!", NotificationType::Info);
        }),
        Components::action_button("Primary", AppColor::Primary, |state: &mut AppState| {
            tracing::info!("Primary action triggered");
            state.add_notification("Primary action executed", NotificationType::Success);
        }),
        Components::action_button("Success", AppColor::Success, |state: &mut AppState| {
            tracing::info!("Success action triggered");
            state.add_notification("Success! Operation completed", NotificationType::Success);
        }),
        Components::action_button("Warning", AppColor::Warning, |state: &mut AppState| {
            tracing::info!("Warning action triggered");
            state.add_notification("Warning: Check your settings", NotificationType::Warning);
        }),
        Components::action_button("Error", AppColor::Error, |state: &mut AppState| {
            tracing::info!("Error action triggered");
            state.add_error("Error: This is a demo error".to_string(), false);
        }),
    ];
    items.push(Components::simple_flex_row(button_row));
    
    // Progress bars
    items.push(Components::spacer(SpacerSize::Large));
    items.push(Components::label("Progress Bars:"));
    items.push(Components::progress_bar(0.0, "Empty"));
    items.push(Components::progress_bar(0.33, "33% Complete"));
    items.push(Components::progress_bar(0.67, "67% Complete"));
    items.push(Components::progress_bar(1.0, "Complete!"));
    
    // Empty states
    items.push(Components::spacer(SpacerSize::Large));
    items.push(Components::label("Empty States:"));
    items.push(Components::empty_state(
        "📭",
        "No Data",
        "Start adding items to see them here",
        Some(Components::action_button("Add Item", AppColor::Primary, |state: &mut AppState| {
            tracing::info!("Add item clicked from empty state");
            state.add_notification("Opening add item dialog...", NotificationType::Info);
            // In a real app, this would open a dialog or navigate to an add item screen
        }))
    ));
    
    // Loading states
    items.push(Components::spacer(SpacerSize::Large));
    items.push(Components::label("Loading States:"));
    items.push(Components::loading_spinner(Some("Loading content...")));
    items.push(Components::loading_overlay("Processing request..."));
    
    Components::simple_flex_column(items)
}

fn layout_components_tab(window_width: f64) -> ComponentOutput {
    let mut items = vec![];
    
    items.push(Components::label("Layout Components:"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Grid layout
    items.push(Components::label("Grid Layout (responsive):"));
    let grid_items = vec![
        Components::card("Item 1", Components::label("Grid item 1")),
        Components::card("Item 2", Components::label("Grid item 2")),
        Components::card("Item 3", Components::label("Grid item 3")),
        Components::card("Item 4", Components::label("Grid item 4")),
    ];
    items.push(dashboard_grid(grid_items, window_width));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Two column layout
    items.push(Components::label("Two Column Layout:"));
    let left = Components::card("Left Column", Components::label("Left side content"));
    let right = Components::card("Right Column", Components::label("Right side content"));
    items.push(two_column_layout(left, right, window_width));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Split pane
    items.push(Components::label("Split Pane (40/60):"));
    let left_pane = Components::card("Left Pane", Components::label("40% width"));
    let right_pane = Components::card("Right Pane", Components::label("60% width"));
    items.push(Components::split_pane(left_pane, right_pane, 0.4));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Stack layout
    items.push(Components::label("Stack Layout:"));
    let stack_items = vec![
        Components::card("First", Components::label("Stacked item 1")),
        Components::card("Second", Components::label("Stacked item 2")),
        Components::card("Third", Components::label("Stacked item 3")),
    ];
    items.push(Components::stack(stack_items, 12.0));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Centered container
    items.push(Components::label("Centered Container (max-width: 600px):"));
    items.push(Components::centered_container(
        600.0,
        Components::card("Centered", Components::label("This content has a maximum width"))
    ));
    
    // Scrollable container
    items.push(Components::spacer(SpacerSize::Large));
    items.push(Components::label("Scrollable Container:"));
    let many_items = (0..10).map(|i| Components::label(&format!("Item {}", i))).collect();
    items.push(Components::scrollable(Components::simple_flex_column(many_items)));
    
    // FAB layout
    items.push(Components::spacer(SpacerSize::Large));
    items.push(Components::label("Floating Action Button:"));
    let content_with_fab = with_fab(
        Components::card("Content", Components::label("Main content with FAB")),
        "➕ Add",
        |_| {}
    );
    items.push(content_with_fab);
    
    Components::simple_flex_column(items)
}

fn cards_components_tab() -> ComponentOutput {
    let mut items = vec![];
    
    // Stat cards with trends
    items.push(Components::label("Stat Cards with Trends:"));
    items.push(Components::spacer(SpacerSize::Small));
    
    let stat_cards = vec![
        stat_card_with_trend("95%", "Accuracy", Trend::Up(5), AppColor::Success),
        stat_card_with_trend("1,234", "Points", Trend::Neutral, AppColor::Primary),
        stat_card_with_trend("45ms", "Speed", Trend::Down(10), AppColor::Warning),
    ];
    items.push(Components::simple_flex_row(stat_cards));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Metric cards
    items.push(Components::label("Metric Cards:"));
    let metric_cards = vec![
        metric_card("🏆", "Level", "12", Some("Expert"), AppColor::Primary),
        metric_card("🔥", "Streak", "7", Some("days"), AppColor::Error),
        metric_card("⭐", "XP", "2,450", None, AppColor::Warning),
    ];
    items.push(Components::simple_flex_row(metric_cards));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Achievement cards
    items.push(Components::label("Achievement Cards:"));
    items.push(achievement_card("🎯", "Sharpshooter", "10 perfect sessions", true, None));
    items.push(achievement_card("📚", "Bookworm", "Read 100 items", false, Some(0.65)));
    items.push(achievement_card("⚡", "Speed Demon", "Complete tasks in < 1s", false, Some(0.3)));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Leaderboard cards
    items.push(Components::label("Leaderboard Cards:"));
    items.push(leaderboard_card(1, "Alice", 5420, Some("👑"), false));
    items.push(leaderboard_card(2, "You", 4850, Some("🎯"), true));
    items.push(leaderboard_card(3, "Bob", 4200, Some("🎮"), false));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Challenge cards
    items.push(Components::label("Challenge Cards:"));
    items.push(challenge_card(
        "Daily Quest",
        "Complete 20 tasks today",
        ChallengeDifficulty::Easy,
        100,
        Some("5 hours"),
        Some(0.75),
        |_| {}
    ));
    items.push(challenge_card(
        "Perfect Week",
        "Maintain 100% accuracy for 7 days",
        ChallengeDifficulty::Expert,
        1000,
        None,
        Some(0.43),
        |_| {}
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Session summary
    items.push(Components::label("Session Summary:"));
    items.push(session_summary_card(15, 42, 0.88, 420));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Feature cards
    items.push(Components::label("Feature Cards:"));
    items.push(feature_card(
        "🚀",
        "Quick Start",
        "Jump right into learning",
        "Start Now",
        |_| {}
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Domain cards
    items.push(Components::label("Domain Cards:"));
    let domain_stats = DomainStats {
        sessions: 45,
        best_streak: 12,
        mastery: 0.78,
    };
    items.push(domain_card_enhanced(
        "🔤",
        "Alphabet",
        "Master the ABCs",
        "Easy",
        true,
        Some(domain_stats),
        |_| {}
    ));
    
    Components::simple_flex_column(items)
}

fn forms_components_tab() -> ComponentOutput {
    let mut items = vec![];
    
    // Form fields
    items.push(Components::label("Form Fields:"));
    items.push(form_field("Username", "john_doe".to_string(), Some("Enter username"), FieldType::Text, |_, _| {}));
    items.push(form_field("Email", "".to_string(), Some("user@example.com"), FieldType::Email, |_, _| {}));
    items.push(form_field("Password", "".to_string(), None, FieldType::Password, |_, _| {}));
    items.push(form_field("Age", "25".to_string(), None, FieldType::Number, |_, _| {}));
    items.push(form_field("Bio", "".to_string(), Some("Tell us about yourself"), FieldType::TextArea, |_, _| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Toggle switches
    items.push(Components::label("Toggle Switches:"));
    items.push(toggle_switch("Enable Notifications", true, |_, _| {}));
    items.push(toggle_switch("Dark Mode", false, |_, _| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Sliders
    items.push(Components::label("Sliders:"));
    items.push(slider("Volume", 0.7, 0.0, 1.0, 0.1, |_, _| {}));
    items.push(slider("Difficulty", 3.0, 1.0, 5.0, 1.0, |_, _| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Radio groups
    items.push(Components::label("Radio Groups:"));
    let options = vec![
        ("small".to_string(), "Small".to_string()),
        ("medium".to_string(), "Medium".to_string()),
        ("large".to_string(), "Large".to_string()),
    ];
    items.push(radio_group("Size", options, &"medium".to_string(), |_, _| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Dropdowns
    items.push(Components::label("Dropdowns:"));
    let countries = vec![
        ("us".to_string(), "United States".to_string()),
        ("uk".to_string(), "United Kingdom".to_string()),
        ("ca".to_string(), "Canada".to_string()),
    ];
    items.push(dropdown("Country", countries, Some(&"us".to_string()), Some("Select country"), |_, _| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Date and time pickers
    items.push(Components::label("Date & Time Pickers:"));
    items.push(date_picker("Birth Date", Some("1990-01-01".to_string()), |_, _| {}));
    items.push(time_picker("Appointment Time", Some("14:30".to_string()), |_, _| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Multi-select
    items.push(Components::label("Multi-Select:"));
    let interests = vec![
        ("coding".to_string(), "Coding".to_string()),
        ("music".to_string(), "Music".to_string()),
        ("sports".to_string(), "Sports".to_string()),
    ];
    items.push(multi_select("Interests", interests, vec!["coding".to_string()], |_, _| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // File upload
    items.push(Components::label("File Upload:"));
    items.push(file_upload("Avatar", vec![".jpg", ".png"], |_, _| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Color picker
    items.push(Components::label("Color Picker:"));
    items.push(color_picker("Theme Color", "#3B82F6".to_string(), |_, _| {}));
    
    Components::simple_flex_column(items)
}

fn feedback_components_tab() -> ComponentOutput {
    let mut items = vec![];
    
    // Toasts
    items.push(Components::label("Toast Notifications:"));
    items.push(toast("Success! Your changes have been saved.", ToastType::Success, Some(3000), Some(|state: &mut AppState| {
        tracing::info!("Success toast dismissed");
        state.add_notification("Success toast was dismissed", NotificationType::Info);
    })));
    items.push(toast("Error: Unable to connect to server", ToastType::Error, Some(5000), Some(|state: &mut AppState| {
        tracing::info!("Error toast dismissed, retrying connection");
        state.add_error("Retrying connection...".to_string(), false);
    })));
    items.push(toast("Warning: Low battery", ToastType::Warning, None, Some(|state: &mut AppState| {
        tracing::info!("Warning acknowledged");
        state.add_notification("Battery warning acknowledged", NotificationType::Warning);
    })));
    items.push(toast("Info: New update available", ToastType::Info, None, Some(|state: &mut AppState| {
        tracing::info!("Navigating to settings for update");
        state.navigate(Screen::Settings);
    })));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Progress indicators
    items.push(Components::label("Progress Indicators:"));
    items.push(progress_indicator("Loading data...", None, false));
    items.push(progress_indicator("Uploading file...", Some(0.65), true));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Skeleton loaders
    items.push(Components::label("Skeleton Loaders:"));
    items.push(skeleton_loader(3, false));
    items.push(skeleton_loader(2, true));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Alerts
    items.push(Components::label("Alerts:"));
    let alert_actions = vec![
        AlertAction {
            label: "Retry".to_string(),
            color: AppColor::Primary,
            on_click: Box::new(|_| {}),
        },
        AlertAction {
            label: "Dismiss".to_string(),
            color: AppColor::Secondary,
            on_click: Box::new(|_| {}),
        },
    ];
    items.push(alert("Success", "Operation completed successfully!", AlertType::Success, vec![]));
    items.push(alert("Error", "Something went wrong", AlertType::Error, alert_actions));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Confirmation dialog
    items.push(Components::label("Confirmation Dialog:"));
    items.push(confirmation_dialog(
        "Delete Item",
        "Are you sure you want to delete this item? This action cannot be undone.",
        "Delete",
        "Cancel",
        true,
        |_| {},
        |_| {}
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Snackbars
    items.push(Components::label("Snackbars:"));
    items.push(snackbar("File uploaded successfully", None, Some(|state: &mut AppState| {
        tracing::info!("Snackbar dismissed");
        state.add_notification("Upload confirmed", NotificationType::Success);
    })));
    items.push(snackbar("Network connection lost", Some("Retry"), Some(|state: &mut AppState| {
        tracing::info!("Retrying network connection");
        state.add_notification("Retrying connection...", NotificationType::Info);
        // In a real app, this would trigger a reconnection attempt
    })));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Status badges
    items.push(Components::label("Status Badges:"));
    let badges = vec![
        status_badge("Active", StatusType::Active),
        status_badge("Inactive", StatusType::Inactive),
        status_badge("Pending", StatusType::Pending),
        status_badge("Completed", StatusType::Completed),
    ];
    items.push(Components::simple_flex_row(badges));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Loading overlay with progress
    items.push(Components::label("Loading Overlay:"));
    items.push(loading_overlay_with_progress(
        "Processing",
        "Please wait while we process your request...",
        Some(0.45),
        true,
        Some(|state: &mut AppState| {
            tracing::info!("Loading cancelled by user");
            state.add_notification("Process cancelled", NotificationType::Warning);
        })
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Help tooltips
    items.push(Components::label("Help Tooltips:"));
    items.push(help_tooltip("Password must be 8+ characters", "Password requirements"));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Inline errors
    items.push(Components::label("Inline Errors:"));
    items.push(inline_error("Username already taken"));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Success indicators
    items.push(Components::label("Success Indicators:"));
    items.push(success_indicator("Registration complete!", true));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Step indicators
    items.push(Components::label("Step Indicators:"));
    items.push(step_indicator(2, 5, Some(vec!["Start", "Details", "Review", "Payment", "Done"])));
    
    Components::simple_flex_column(items)
}

fn navigation_components_tab() -> ComponentOutput {
    let mut items = vec![];
    
    // Header bar
    items.push(Components::label("Header Bar:"));
    items.push(Components::header_bar("Application Title", |_| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Navigation buttons
    items.push(Components::label("Navigation Buttons:"));
    let nav_buttons = vec![
        Components::nav_button("Home", Screen::Dashboard, true, |_| {}),
        Components::nav_button("Learning", Screen::Learning, false, |_| {}),
        Components::nav_button("Progress", Screen::Progress, false, |_| {}),
        Components::nav_button("Settings", Screen::Settings, false, |_| {}),
    ];
    items.push(Components::simple_flex_row(nav_buttons));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Tab bar
    items.push(Components::label("Tab Bar:"));
    let tabs = vec![
        TabItem { label: "Overview".to_string(), icon: Some("📊".to_string()), badge_count: None },
        TabItem { label: "Details".to_string(), icon: Some("📝".to_string()), badge_count: Some(3) },
        TabItem { label: "History".to_string(), icon: Some("📜".to_string()), badge_count: None },
    ];
    items.push(Components::tab_bar(tabs, 0, |_, _| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Bottom navigation
    items.push(Components::label("Bottom Navigation Bar:"));
    items.push(Components::bottom_nav_bar(Screen::Dashboard, |_, _| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Sidebar navigation (simplified for gallery)
    items.push(Components::label("Sidebar Navigation:"));
    items.push(Components::sidebar_nav(Screen::Dashboard, |_, _| {}));
    
    Components::simple_flex_column(items)
}

fn learning_components_tab() -> ComponentOutput {
    let mut items = vec![];
    
    // Learning progress
    items.push(Components::label("Learning Progress:"));
    items.push(Components::learning_progress(3, 10, 0.75));
    items.push(Components::learning_progress(10, 10, 1.0));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Hint buttons
    items.push(Components::label("Hint Buttons:"));
    let hint_buttons = vec![
        Components::hint_button(0, |_| {}),
        Components::hint_button(1, |_| {}),
        Components::hint_button(2, |_| {}),
    ];
    items.push(Components::simple_flex_row(hint_buttons));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Task visuals
    items.push(Components::label("Task Visuals:"));
    items.push(Components::alphabet_sequence(
        vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
        Some(2)
    ));
    items.push(Components::comparison_visual("Apple", "Banana", true));
    items.push(Components::missing_item_visual("1", "3"));
    items.push(Components::path_visual(
        "Start",
        "End",
        vec!["Step 1".to_string(), "Step 2".to_string()]
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Task options
    items.push(Components::label("Task Options:"));
    items.push(Components::task_options(
        vec!["Option A".to_string(), "Option B".to_string(), "Option C".to_string(), "Option D".to_string()],
        |_, _| {}
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Task card display
    items.push(Components::label("Task Card:"));
    items.push(Components::task_card_display(
        "What comes after B in the alphabet?",
        Some("Think about the order of letters")
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Answer options display
    items.push(Components::label("Answer Options:"));
    items.push(Components::answer_options_display(
        vec!["A".to_string(), "C".to_string(), "D".to_string(), "E".to_string()],
        |_, _| {}
    ));
    
    Components::simple_flex_column(items)
}

fn visualization_components_tab() -> ComponentOutput {
    let mut items = vec![];
    
    // Charts
    items.push(Components::label("Charts & Visualizations:"));
    
    items.push(Components::response_time_histogram(&vec![850, 920, 1100, 780, 950, 1200, 890, 1050]));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    items.push(Components::performance_chart(0.92, 50, 46, 875.5));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Mock response data
    let responses = vec![
        create_sample_response(true, 900),
        create_sample_response(true, 850),
        create_sample_response(false, 1200),
        create_sample_response(true, 780),
    ];
    
    items.push(Components::learning_curve_display(&responses));
    items.push(Components::error_analysis_display(&responses));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Strategy analysis
    items.push(Components::strategy_analysis_display(vec![
        ("Pattern Recognition", 0.85),
        ("Memory Recall", 0.72),
        ("Speed", 0.90),
        ("Accuracy", 0.88),
    ]));
    
    Components::simple_flex_column(items)
}

fn advanced_components_tab() -> ComponentOutput {
    let mut items = vec![];
    
    // Settings sections
    items.push(Components::label("Settings Section:"));
    items.push(Components::settings_section(
        "Preferences",
        vec![
            Components::setting_row("Theme", Components::theme_selector(AppTheme::Auto, |_, _| {})),
            Components::setting_row("Language", Components::label("English")),
            Components::checkbox(true, "Enable animations", |_, _| {}),
        ]
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // App scaffold
    items.push(Components::label("App Scaffold:"));
    items.push(Components::app_scaffold(
        Components::header_bar("App Header", |_| {}),
        Components::card("Content", Components::label("Main application content")),
        Some(Components::label("Footer content"))
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Domain cards
    items.push(Components::label("Domain Cards:"));
    items.push(Components::domain_card("Mathematics", "Numbers and calculations", false));
    items.push(Components::domain_card("Language", "Words and grammar", true));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Session info
    items.push(Components::label("Session Info:"));
    items.push(Components::session_info("session_123", "Active", "15:42", "Alphabet"));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Confirmation modal
    items.push(Components::label("Confirmation Modal:"));
    items.push(Components::confirm_modal(
        "Confirm Action",
        "Are you sure you want to proceed?",
        |_| {},
        |_| {}
    ));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Error messages
    items.push(Components::label("Error Messages:"));
    items.push(Components::error_message(Some("An error occurred".to_string())));
    items.push(Components::error_banner("Connection lost", |_| {}));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Success messages
    items.push(Components::label("Success Messages:"));
    items.push(Components::success_message(Some("Operation completed!".to_string())));
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Toast notifications
    items.push(Components::label("Toast Notifications:"));
    items.push(Components::toast_notification("New message received", false));
    items.push(Components::toast_notification("File uploaded", true));
    
    Components::simple_flex_column(items)
}

// Helper function
fn create_sample_response(correct: bool, time_ms: u128) -> Response {
    Response {
        id: uuid::Uuid::new_v4(),
        session_id: uuid::Uuid::new_v4(),
        sequence_number: 1,
        task_type: "sample".to_string(),
        task_data: serde_json::json!({}),
        user_answer: Some("A".to_string()),
        correct,
        response_time_ms: time_ms,
        hint_level: Some(0),
        timestamp: chrono::Utc::now(),
    }
}