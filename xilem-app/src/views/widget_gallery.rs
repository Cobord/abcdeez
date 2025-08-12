use crate::components::{Component, ComponentOutput, Components, AppComponents, AppColor, AppTheme};
use crate::state::{AppState, Screen};
use crate::viz::{sparkline, progress_ring_chart};
use crate::models::Response;

/// Comprehensive hidden widget gallery showcasing all components
pub fn widget_gallery() -> ComponentOutput {
    let mut sections = vec![];
    
    // Header
    sections.push(Components::simple_label("🎨 Widget Gallery - Component Showcase".to_string()));
    sections.push(Components::simple_label("Hidden testing page for all UI components".to_string()));
    
    // 1. Basic UI Components
    sections.push(basic_components_section());
    
    // 2. Form Controls
    sections.push(form_controls_section());
    
    // 3. Navigation Components
    sections.push(navigation_components_section());
    
    // 4. Feedback & Status
    sections.push(feedback_components_section());
    
    // 5. Cards & Containers
    sections.push(cards_containers_section());
    
    // 6. Task & Learning Components
    sections.push(learning_components_section());
    
    // 7. Visualization Components
    sections.push(visualization_components_section());
    
    // 8. Demo-Aware Components
    sections.push(demo_components_section());
    
    // 9. Modal & Overlay Components
    sections.push(modal_components_section());
    
    // 10. Advanced Layouts
    sections.push(layout_components_section());
    
    // Navigation back
    sections.push(Components::simple_button("🔙 Back to Dashboard", |state: &mut AppState| {
        state.current_screen = Screen::Dashboard;
    }));
    
    Components::simple_flex_column(sections)
}

fn basic_components_section() -> ComponentOutput {
    Components::card(
        "🔤 Basic UI Components",
        Components::simple_flex_column(vec![
            Components::simple_label("Text and Labels:".to_string()),
            Components::simple_label("This is a simple label".to_string()),
            
            Components::simple_label("Buttons:".to_string()),
            Components::simple_flex_row(vec![
                Components::simple_button("Primary", |_| println!("Primary clicked")),
                Components::simple_button("Secondary", |_| println!("Secondary clicked")),
                Components::simple_button("Danger ⚠️", |_| println!("Danger clicked")),
            ]),
            
            Components::simple_label("Progress Indicators:".to_string()),
            Components::progress_bar(0.75, "75% Complete"),
            Components::progress_bar(0.33, "Loading..."),
            Components::progress_bar(1.0, "Finished! ✅"),
        ])
    )
}

fn form_controls_section() -> ComponentOutput {
    Components::card(
        "📝 Form Controls",
        Components::simple_flex_column(vec![
            Components::labeled_input(
                "Username:",
                "john_doe".to_string(),
                |_state, value| println!("Username changed: {}", value)
            ),
            
            Components::labeled_input(
                "Email:",
                "john@example.com".to_string(),
                |_state, value| println!("Email changed: {}", value)
            ),
            
            Components::checkbox(
                true,
                "Enable notifications",
                |_state, checked| println!("Notifications: {}", checked)
            ),
            
            Components::checkbox(
                false,
                "Dark mode",
                |_state, checked| println!("Dark mode: {}", checked)
            ),
            
            Components::theme_selector(
                AppTheme::Light,
                |_state, theme| println!("Theme changed: {:?}", theme)
            ),
        ])
    )
}

fn navigation_components_section() -> ComponentOutput {
    Components::card(
        "🧭 Navigation",
        Components::simple_flex_column(vec![
            Components::simple_label("Navigation Bar:".to_string()),
            Components::header_bar(
                "Application Title",
                |_state| println!("Settings clicked")
            ),
            
            Components::simple_label("Bottom Navigation:".to_string()),
            Components::bottom_nav_bar(
                Screen::Dashboard,
                |_state, screen| println!("Navigate to: {:?}", screen)
            ),
            
            Components::simple_label("Navigation Buttons:".to_string()),
            Components::simple_flex_row(vec![
                Components::nav_button("Home", Screen::Dashboard, true, |_| {}),
                Components::nav_button("Learn", Screen::Learning, false, |_| {}),
                Components::nav_button("Profile", Screen::Profile, false, |_| {}),
            ]),
        ])
    )
}

fn feedback_components_section() -> ComponentOutput {
    Components::card(
        "💬 Feedback & Status",
        Components::simple_flex_column(vec![
            Components::error_message(Some("❌ An error occurred while loading".to_string())),
            Components::error_message(None),
            
            Components::success_message(Some("✅ Operation completed successfully!".to_string())),
            Components::success_message(None),
            
            Components::toast_notification("🔔 New notification received", false),
            Components::toast_notification("✅ File uploaded successfully", true),
            
            Components::loading_spinner(Some("Loading data...")),
            Components::loading_overlay("Processing your request..."),
            
            Components::error_banner(
                "Connection lost. Please check your internet.",
                |_| println!("Error dismissed")
            ),
        ])
    )
}

fn cards_containers_section() -> ComponentOutput {
    Components::card(
        "📦 Cards & Containers",
        Components::simple_flex_column(vec![
            Components::stat_card("Total Score", "1,234", AppColor::Primary),
            Components::stat_card("Accuracy", "92.5%", AppColor::Success),
            Components::stat_card("Time Spent", "2h 15m", AppColor::Info),
            
            Components::welcome_card(
                "Alice",
                "Ready to continue your learning journey?",
                |_| println!("Start learning")
            ),
            
            Components::activity_card("Completed Alphabet Practice", "10 mins ago", false),
            Components::activity_card("🔥 New High Score!", "1 hour ago", true),
            
            Components::metric_display("Speed", "850ms", AppColor::Warning),
            Components::metric_display("Streak", "7 days", AppColor::Success),
            
            Components::empty_state(
                "📭",
                "No Messages",
                "You're all caught up!",
                Some(Components::simple_button("Refresh", |_| {}))
            ),
        ])
    )
}

fn learning_components_section() -> ComponentOutput {
    use crate::models::{Task, TaskType};
    use abcdeez_core::tasks::core::TaskType as CoreTaskType;
    
    let sample_task = Task {
        task_type: TaskType::Core(CoreTaskType::Successor { item: "B".to_string() }),
        prompt: "What comes after 'B' in the alphabet?".to_string(),
        options: vec!["A".to_string(), "C".to_string(), "D".to_string(), "E".to_string()],
        correct_answer: "C".to_string(),
        difficulty: 0.3,
        operation: "next".to_string(),
    };
    
    Components::card(
        "📚 Learning Components",
        Components::simple_flex_column(vec![
            Components::task_presenter(
                &sample_task,
                |_state, answer| println!("Answer selected: {}", answer)
            ),
            
            Components::task_options(
                vec!["Option A".to_string(), "Option B".to_string(), "Option C".to_string()],
                |_state, idx| println!("Selected option {}", idx)
            ),
            
            Components::learning_progress(5, 10, 0.8),
            Components::learning_progress(0, 10, 0.0),
            
            Components::hint_button(0, |_| println!("Hint requested")),
            Components::hint_button(1, |_| println!("More hints requested")),
            Components::hint_button(2, |_| println!("Show answer requested")),
            
            Components::alphabet_sequence(
                vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
                Some(2)
            ),
            
            Components::comparison_visual("Apple", "Banana", true),
            Components::comparison_visual("3", "7", false),
            
            Components::missing_item_visual("A", "C"),
            Components::missing_item_visual("10", "30"),
            
            Components::path_visual(
                "Start",
                "Goal",
                vec!["Step 1".to_string(), "Step 2".to_string(), "Step 3".to_string()]
            ),
        ])
    )
}

fn visualization_components_section() -> ComponentOutput {
    Components::card(
        "📊 Visualizations",
        Components::simple_flex_column(vec![
            Components::simple_label("Mini Charts:".to_string()),
            
            sparkline(&[0.2, 0.4, 0.3, 0.7, 0.8, 0.6, 0.9, 0.85], 200, 40),
            sparkline(&[1.0, 0.8, 0.6, 0.4, 0.2, 0.3, 0.5, 0.7], 200, 40),
            
            progress_ring_chart(0.75, "Completion", 100),
            progress_ring_chart(0.33, "Progress", 100),
            
            Components::response_time_histogram(&vec![1200, 980, 1100, 750, 900, 1050, 800]),
            
            Components::simple_label("Performance Metrics:".to_string()),
            Components::performance_chart(0.85, 20, 17, 950.0),
            
            Components::simple_label("Learning Analytics:".to_string()),
            Components::learning_curve_display(&vec![
                create_sample_response(true, 1000),
                create_sample_response(false, 1200),
                create_sample_response(true, 900),
                create_sample_response(true, 850),
            ]),
            
            Components::error_analysis_display(&vec![
                create_sample_response(false, 1500),
                create_sample_response(false, 1300),
                create_sample_response(true, 1000),
            ]),
            
            Components::strategy_analysis_display(vec![
                ("Memorization", 0.75),
                ("Pattern Recognition", 0.82),
                ("Speed", 0.65),
                ("Accuracy", 0.90),
            ]),
        ])
    )
}

fn demo_components_section() -> ComponentOutput {
    Components::card(
        "🎯 Demo-Aware Components",
        Components::simple_flex_column(vec![
            Components::simple_label("Components with demo highlights:".to_string()),
            
            Components::demo_button(
                "demo_btn_1",
                "Click Me!",
                true,
                Some("This button is highlighted".to_string())
            ),
            
            Components::demo_button(
                "demo_btn_2",
                "Normal Button",
                false,
                None
            ),
            
            Components::demo_card(
                "demo_card_1",
                "Highlighted Card",
                Components::simple_label("This card has a tooltip".to_string()),
                true,
                Some("Important information here!".to_string())
            ),
            
            Components::highlighted(
                "highlight_1",
                Components::simple_label("✨ Highlighted content".to_string()),
                true,
                Some("Pay attention to this!".to_string())
            ),
            
            Components::highlighted(
                "highlight_2",
                Components::simple_label("Normal content".to_string()),
                false,
                None
            ),
        ])
    )
}

fn modal_components_section() -> ComponentOutput {
    Components::card(
        "🗨️ Modals & Overlays",
        Components::simple_flex_column(vec![
            Components::confirm_modal(
                "Confirm Action",
                "Are you sure you want to proceed with this action?",
                |_| println!("Confirmed!"),
                |_| println!("Cancelled!")
            ),
            
            Components::simple_label("Settings Section:".to_string()),
            Components::settings_section(
                "Preferences",
                vec![
                    Components::setting_row(
                        "Notifications",
                        Components::checkbox(true, "Enabled", |_, _| {})
                    ),
                    Components::setting_row(
                        "Theme",
                        Components::theme_selector(AppTheme::Auto, |_, _| {})
                    ),
                ]
            ),
        ])
    )
}

fn layout_components_section() -> ComponentOutput {
    Components::card(
        "📐 Advanced Layouts",
        Components::simple_flex_column(vec![
            Components::simple_label("Centered Container:".to_string()),
            Components::centered_container(
                700.0,
                Components::card(
                    "Centered",
                    Components::simple_label("This is centered with max width".to_string())
                )
            ),
            
            Components::simple_label("Stats Grid:".to_string()),
            Components::stats_grid(vec![
                Components::stat_card("Stat 1", "100", AppColor::Primary),
                Components::stat_card("Stat 2", "200", AppColor::Success),
                Components::stat_card("Stat 3", "300", AppColor::Warning),
                Components::stat_card("Stat 4", "400", AppColor::Info),
            ]),
            
            Components::simple_label("App Scaffold:".to_string()),
            Components::app_scaffold(
                Components::header_bar("App Title", |_| {}),
                Components::simple_label("Main content area".to_string()),
                Some(Components::simple_label("Footer content".to_string()))
            ),
            
            Components::simple_label("Domain Cards:".to_string()),
            Components::domain_card("Mathematics", "Practice arithmetic and algebra", false),
            Components::domain_card("Language", "Learn vocabulary and grammar", true),
            
            Components::simple_label("Session Info:".to_string()),
            Components::session_info("sess_123", "Active", "15:32", "Alphabet"),
            
            Components::simple_label("Task Card:".to_string()),
            Components::task_card_display(
                "What is 2 + 2?",
                Some("Think about adding two items to two more items")
            ),
            
            Components::simple_label("Answer Options:".to_string()),
            Components::answer_options_display(
                vec!["3".to_string(), "4".to_string(), "5".to_string(), "6".to_string()],
                |_state, idx| println!("Selected answer index: {}", idx)
            ),
        ])
    )
}

// Helper function to create sample responses
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