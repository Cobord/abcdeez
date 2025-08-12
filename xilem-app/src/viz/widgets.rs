// Visualization widgets using the new component library
use crate::components::{Components, AppComponents, ComponentOutput, AppColor, SpacerSize};
use crate::components::cards::{metric_card, stat_card_with_trend, Trend};
use crate::components::layout::{dashboard_grid, page_layout};
use crate::state::AppState;

/// Learning curve chart widget
pub fn learning_curve_chart(
    data: &[(f32, f32)],
    width: u32,
    height: u32,
) -> ComponentOutput {
    let mut items = vec![
        Components::label(&format!("📈 {} data points", data.len())),
        Components::spacer(SpacerSize::Small),
    ];
    
    // Create a simple ASCII visualization
    if !data.is_empty() {
        let sparkline = create_sparkline(&data.iter().map(|(_, y)| *y).collect::<Vec<_>>());
        items.push(Components::label(&sparkline));
    }
    
    items.push(Components::label(&format!("Size: {}x{}", width, height)));
    
    Components::card(
        "Learning Curve",
        Components::simple_flex_column(items)
    )
}

/// Response time histogram chart
pub fn response_time_histogram(
    times: &[u128],
    width: u32,
    height: u32,
) -> ComponentOutput {
    let avg = if !times.is_empty() {
        times.iter().sum::<u128>() / times.len() as u128
    } else {
        0
    };
    
    let min = times.iter().min().copied().unwrap_or(0);
    let max = times.iter().max().copied().unwrap_or(0);
    
    let items = vec![
        Components::label("📊 Response Time Distribution"),
        Components::spacer(SpacerSize::Small),
        metric_card("⚡", "Average", &format!("{}ms", avg), None, AppColor::Primary),
        Components::simple_flex_row(vec![
            Components::stat_card("Min", &format!("{}ms", min), AppColor::Success),
            Components::stat_card("Max", &format!("{}ms", max), AppColor::Warning),
        ]),
        Components::label(&format!("Samples: {} | Size: {}x{}", times.len(), width, height)),
    ];
    
    Components::card(
        "Response Times",
        Components::simple_flex_column(items)
    )
}

/// Performance heatmap chart
pub fn performance_heatmap(
    sessions: usize,
    width: u32,
    height: u32,
) -> ComponentOutput {
    // Create a mock heatmap visualization
    let heatmap_rows = vec![
        "🟩🟩🟨🟩🟩🟩🟨",
        "🟩🟨🟨🟩🟩🟨🟥",
        "🟨🟩🟩🟩🟨🟨🟨",
        "🟩🟩🟩🟩🟩🟩🟩",
        "🟨🟨🟩🟩🟩🟨🟩",
    ];
    
    let mut items = vec![
        Components::label("🗓️ Weekly Performance"),
        Components::spacer(SpacerSize::Small),
    ];
    
    for row in heatmap_rows {
        items.push(Components::label(row));
    }
    
    items.push(Components::spacer(SpacerSize::Small));
    items.push(Components::label(&format!("{} sessions tracked", sessions)));
    items.push(Components::label(&format!("Size: {}x{}", width, height)));
    
    Components::card(
        "Performance Heatmap",
        Components::simple_flex_column(items)
    )
}

/// Metrics radar chart
pub fn metrics_radar_chart(
    metrics: &[(&str, f32)],
    width: u32,
    height: u32,
) -> ComponentOutput {
    let mut items = vec![
        Components::label(&format!("🎯 Multi-Dimensional Analysis ({}x{})", width, height)),
        Components::spacer(SpacerSize::Small),
    ];
    
    // Display metrics as progress bars
    for (name, value) in metrics {
        items.push(Components::progress_bar(*value as f64, name));
        items.push(Components::spacer(SpacerSize::Small));
    }
    
    Components::card(
        "Metrics Radar",
        Components::simple_flex_column(items)
    )
}

/// Progress ring chart
pub fn progress_ring_chart(
    value: f32,
    label: &str,
    size: u32,
) -> ComponentOutput {
    let percentage = (value * 100.0) as u32;
    let filled = (percentage / 10) as usize;
    let empty = 10 - filled;
    
    let ring = format!("{}{}",
        "●".repeat(filled),
        "○".repeat(empty)
    );
    
    let color = if value >= 0.8 {
        AppColor::Success
    } else if value >= 0.5 {
        AppColor::Warning
    } else {
        AppColor::Error
    };
    
    let items = vec![
        Components::label(&ring),
        Components::spacer(SpacerSize::Small),
        Components::stat_card(&format!("{}%", percentage), label, color),
        Components::label(&format!("Size: {}px", size)),
    ];
    
    Components::card(
        label,
        Components::simple_flex_column(items)
    )
}

/// Sparkline mini chart
pub fn sparkline(
    values: &[f32],
    width: u32,
    height: u32,
) -> ComponentOutput {
    let sparkline = create_sparkline(values);
    
    // Calculate trend
    let trend = if values.len() >= 2 {
        let recent_avg = values[values.len()/2..].iter().sum::<f32>() / (values.len()/2) as f32;
        let early_avg = values[..values.len()/2].iter().sum::<f32>() / (values.len()/2) as f32;
        let diff = ((recent_avg - early_avg) / early_avg * 100.0) as i32;
        
        if diff > 5 {
            Trend::Up(diff.abs() as u32)
        } else if diff < -5 {
            Trend::Down(diff.abs() as u32)
        } else {
            Trend::Neutral
        }
    } else {
        Trend::Neutral
    };
    
    let latest = values.last().copied().unwrap_or(0.0);
    
    Components::simple_flex_column(vec![
        stat_card_with_trend(
            &format!("{:.1}", latest),
            "Latest",
            trend,
            AppColor::Primary
        ),
        Components::label(&sparkline),
        Components::label(&format!("{}x{}", width, height)),
    ])
}

/// Session comparison chart
pub fn session_comparison_chart(
    session1: &str,
    session2: &str,
    session1_data: &[f32],
    session2_data: &[f32],
    width: u32,
    height: u32,
) -> ComponentOutput {
    let spark1 = create_sparkline(session1_data);
    let spark2 = create_sparkline(session2_data);
    
    let avg1 = session1_data.iter().sum::<f32>() / session1_data.len().max(1) as f32;
    let avg2 = session2_data.iter().sum::<f32>() / session2_data.len().max(1) as f32;
    
    let items = vec![
        Components::label("📊 Session Comparison"),
        Components::spacer(SpacerSize::Medium),
        Components::simple_flex_row(vec![
            Components::stat_card(session1, &format!("{:.1}", avg1), AppColor::Primary),
            Components::label("vs"),
            Components::stat_card(session2, &format!("{:.1}", avg2), AppColor::Secondary),
        ]),
        Components::spacer(SpacerSize::Small),
        Components::label(&format!("{}: {}", session1, spark1)),
        Components::label(&format!("{}: {}", session2, spark2)),
        Components::spacer(SpacerSize::Small),
        Components::label(&format!("Size: {}x{}", width, height)),
    ];
    
    Components::card(
        "Session Comparison",
        Components::simple_flex_column(items)
    )
}

/// Helper function to create sparkline
fn create_sparkline(values: &[f32]) -> String {
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max = values.iter().cloned().fold(0.0f32, f32::max);
    let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
    let range = max - min;
    
    values.iter()
        .map(|&v| {
            let normalized = if range > 0.0 {
                (v - min) / range
            } else {
                0.5
            };
            let index = ((normalized * 7.0) as usize).min(7);
            chars[index]
        })
        .collect()
}

/// Visualization dashboard combining multiple charts
pub fn visualization_dashboard(state: &AppState, window_width: f64) -> ComponentOutput {
    use crate::viz::chart_components::create_chart_dashboard;
    
    // Create the main chart dashboard with plotters charts
    let chart_dashboard = create_chart_dashboard(state);
    
    // Create custom metric widgets
    let custom_metrics = vec![
        progress_ring_chart(0.85, "Overall Progress", 120),
        sparkline(&[0.3, 0.5, 0.4, 0.7, 0.8, 0.75, 0.9], 200, 40),
        response_time_histogram(&state.response_times, 300, 200),
    ];
    
    // Performance metrics
    let perf_metrics = vec![
        ("Speed", 0.75),
        ("Accuracy", 0.92),
        ("Consistency", 0.68),
        ("Focus", 0.84),
    ];
    
    // Session comparison
    let session_comp = session_comparison_chart(
        "Today",
        "Yesterday",
        &[0.6, 0.7, 0.75, 0.8, 0.85],
        &[0.5, 0.55, 0.6, 0.7, 0.72],
        400,
        150
    );
    
    // Build the complete dashboard
    let items = vec![
        Components::spacer(SpacerSize::Medium),
        dashboard_grid(custom_metrics, window_width),
        Components::spacer(SpacerSize::Large),
        metrics_radar_chart(&perf_metrics, 400, 400),
        Components::spacer(SpacerSize::Medium),
        session_comp,
        Components::spacer(SpacerSize::Large),
        chart_dashboard,
        Components::spacer(SpacerSize::Medium),
        Components::action_button(
            "Export Charts",
            AppColor::Primary,
            |_state| {
                println!("Exporting charts...");
            }
        ),
    ];
    
    page_layout(
        "Data Visualization Center",
        Some("Interactive charts and analytics"),
        Components::simple_flex_column(items)
    )
}