// Analytics dashboard view with comprehensive metrics and visualizations

use crate::state::{AppState, Screen};
use crate::components::{Components, AppComponents, AppColor, ComponentOutput, SpacerSize, TabItem};
use crate::components::layout::{page_layout, dashboard_grid, two_column_layout, tab_layout_with_state};
use crate::components::cards::{stat_card_with_trend, Trend, metric_card};
use crate::components::feedback::{progress_indicator};
use crate::models::Response;

pub fn analytics_view(state: &mut AppState, window_width: f64) -> ComponentOutput {
    // Build tabs for different analytics views
    let tabs = vec![
        ("Overview", overview_analytics(state, window_width)),
        ("Performance", performance_analytics(state, window_width)),
        ("Patterns", pattern_analytics(state)),
        ("Predictions", prediction_analytics(state)),
    ];
    
    // Create tabbed layout with state management
    let tabbed_content = tab_layout_with_state(state, "analytics", tabs);
    
    page_layout(
        "Analytics Dashboard",
        Some("Deep insights into your learning patterns"),
        tabbed_content
    )
}

fn overview_analytics(state: &AppState, window_width: f64) -> ComponentOutput {
    // Gather data
    let responses: Vec<Response> = Vec::new(); // TODO: Get from session history
    
    let mut items = vec![];
    
    if !responses.is_empty() {
        // Calculate key metrics
        let total_time: u128 = responses.iter().map(|r| r.response_time_ms).sum();
        let correct: usize = responses.iter().filter(|r| r.correct).count();
        let avg_time = total_time / responses.len() as u128;
        let accuracy = (correct as f64 / responses.len() as f64) * 100.0;
        
        // Key metrics cards with trends
        let metrics = vec![
            stat_card_with_trend(
                &format!("{:.1}%", accuracy),
                "Accuracy",
                if accuracy > 80.0 { Trend::Up(5) } else { Trend::Down(3) },
                if accuracy > 80.0 { AppColor::Success } else { AppColor::Warning }
            ),
            stat_card_with_trend(
                &format!("{}ms", avg_time),
                "Avg Response",
                if avg_time < 2000 { Trend::Up(10) } else { Trend::Neutral },
                AppColor::Primary
            ),
            metric_card(
                "📊",
                "Total Tasks",
                &responses.len().to_string(),
                Some("This session"),
                AppColor::Info
            ),
            metric_card(
                "🎯",
                "Success Rate",
                &format!("{}/{}", correct, responses.len()),
                None,
                AppColor::Success
            ),
        ];
        
        items.push(dashboard_grid(metrics, window_width));
        items.push(Components::spacer(SpacerSize::Large));
        
        // Performance chart
        items.push(Components::performance_chart(
            accuracy,
            responses.len(),
            correct,
            avg_time as f64
        ));
        
        // Response time distribution
        items.push(Components::spacer(SpacerSize::Medium));
        let response_times: Vec<u128> = responses.iter().map(|r| r.response_time_ms).collect();
        items.push(Components::response_time_histogram(&response_times));
        
    } else {
        items.push(Components::empty_state(
            "📈",
            "No Analytics Data Yet",
            "Complete some learning sessions to see your analytics",
            Some(Components::action_button(
                "Start Learning",
                AppColor::Primary,
                |state| state.navigate(Screen::Learning)
            ))
        ));
    }
    
    Components::simple_flex_column(items)
}

fn performance_analytics(state: &AppState, window_width: f64) -> ComponentOutput {
    let mut items = vec![];
    
    // Mock performance data
    let performance_over_time = vec![
        ("Mon", 72.5),
        ("Tue", 78.3),
        ("Wed", 81.2),
        ("Thu", 79.8),
        ("Fri", 85.6),
        ("Sat", 88.1),
        ("Sun", 87.3),
    ];
    
    items.push(Components::label("Weekly Performance Trend"));
    items.push(Components::spacer(SpacerSize::Small));
    
    // Create bar chart visualization
    for (day, score) in &performance_over_time {
        let bar_width = (score / 100.0 * 20.0) as usize;
        let bar = "█".repeat(bar_width);
        let label = format!("{}: {} {:.1}%", day, bar, score);
        items.push(Components::label(&label));
    }
    
    items.push(Components::spacer(SpacerSize::Large));
    
    // Performance by domain
    items.push(Components::label("Performance by Domain"));
    items.push(Components::spacer(SpacerSize::Small));
    
    let domains = vec![
        metric_card("🔤", "Alphabet", "92%", Some("245 tasks"), AppColor::Success),
        metric_card("🔢", "Numbers", "87%", Some("189 tasks"), AppColor::Success),
        metric_card("🎨", "Colors", "78%", Some("156 tasks"), AppColor::Warning),
        metric_card("📅", "Days", "95%", Some("203 tasks"), AppColor::Success),
    ];
    
    items.push(dashboard_grid(domains, window_width));
    
    // Difficulty progression
    items.push(Components::spacer(SpacerSize::Large));
    items.push(Components::label("Difficulty Progression"));
    items.push(Components::spacer(SpacerSize::Small));
    
    let difficulties = vec![
        Components::progress_bar(1.0, "Easy (Mastered)"),
        Components::progress_bar(0.85, "Medium (Proficient)"),
        Components::progress_bar(0.45, "Hard (Learning)"),
        Components::progress_bar(0.15, "Expert (Challenging)"),
    ];
    
    for diff in difficulties {
        items.push(diff);
        items.push(Components::spacer(SpacerSize::Small));
    }
    
    Components::simple_flex_column(items)
}

fn pattern_analytics(state: &AppState) -> ComponentOutput {
    let mut items = vec![];
    
    items.push(Components::label("Learning Patterns Detected"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Time of day analysis
    items.push(Components::card(
        "Best Performance Times",
        Components::simple_flex_column(vec![
            Components::label("🌅 Morning: 85% accuracy"),
            Components::label("☀️ Afternoon: 78% accuracy"),
            Components::label("🌙 Evening: 92% accuracy"),
            Components::label("Your peak time: Evening"),
        ])
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Common mistake patterns
    items.push(Components::card(
        "Common Mistake Patterns",
        Components::simple_flex_column(vec![
            Components::label("1. Rushing on easy questions (-15% accuracy)"),
            Components::label("2. Overthinking medium difficulty (+8s response time)"),
            Components::label("3. Pattern confusion: B→D vs B→C (23% error rate)"),
            Components::label("4. Fatigue after 15 minutes (accuracy drops 10%)"),
        ])
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Strategy effectiveness
    let strategies = vec![
        ("Visual Recognition", 0.92),
        ("Pattern Matching", 0.85),
        ("Memory Recall", 0.78),
        ("Sequential Logic", 0.88),
        ("Spatial Reasoning", 0.72),
    ];
    
    items.push(Components::label("Strategy Effectiveness"));
    items.push(Components::spacer(SpacerSize::Small));
    
    for (strategy, effectiveness) in strategies {
        items.push(Components::progress_bar(effectiveness, strategy));
        items.push(Components::spacer(SpacerSize::Small));
    }
    
    Components::simple_flex_column(items)
}

fn prediction_analytics(state: &AppState) -> ComponentOutput {
    let mut items = vec![];
    
    items.push(Components::label("AI-Powered Predictions"));
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Predicted performance
    items.push(Components::card(
        "Next Session Prediction",
        Components::simple_flex_column(vec![
            Components::label("Expected Accuracy: 88-92%"),
            Components::label("Estimated Duration: 12-15 minutes"),
            Components::label("Recommended Difficulty: Medium-Hard"),
            Components::label("Suggested Focus: Pattern Recognition"),
        ])
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Learning velocity
    items.push(Components::card(
        "Learning Velocity",
        Components::simple_flex_column(vec![
            Components::progress_bar(0.75, "Current Speed: 75% of optimal"),
            Components::spacer(SpacerSize::Small),
            Components::label("Time to next level: ~3 more sessions"),
            Components::label("Projected mastery: 8 days"),
        ])
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Recommendations
    items.push(Components::card(
        "Personalized Recommendations",
        Components::simple_flex_column(vec![
            Components::label("✅ Increase session frequency to daily"),
            Components::label("✅ Focus on 'predecessor' task types"),
            Components::label("✅ Take breaks every 10 minutes"),
            Components::label("✅ Review mistakes before next session"),
        ])
    ));
    
    items.push(Components::spacer(SpacerSize::Medium));
    
    // Action button
    items.push(Components::action_button(
        "Start Optimized Session",
        AppColor::Primary,
        |state| {
            // Would start a session with AI-optimized parameters
            state.navigate(Screen::Learning);
        }
    ));
    
    Components::simple_flex_column(items)
}