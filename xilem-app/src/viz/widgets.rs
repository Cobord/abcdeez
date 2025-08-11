// Visualization widgets for charts and data displays
use crate::components::{Component, ComponentOutput};
use crate::state::AppState;

/// Learning curve chart widget
pub fn learning_curve_chart(
    data: &[(f32, f32)],
    width: u32,
    height: u32,
) -> ComponentOutput {
    use crate::components::{card, text};
    
    card(
        "Learning Curve",
        text(&format!("📈 Chart ({} points, {}x{})", data.len(), width, height))
    )
}

/// Response time histogram chart
pub fn response_time_histogram(
    times: &[u128],
    width: u32,
    height: u32,
) -> ComponentOutput {
    use crate::components::{card, text};
    
    let avg = if !times.is_empty() {
        times.iter().sum::<u128>() / times.len() as u128
    } else {
        0
    };
    
    card(
        "Response Times",
        text(&format!("📊 Histogram (avg: {}ms, {}x{})", avg, width, height))
    )
}

/// Performance heatmap chart
pub fn performance_heatmap(
    sessions: usize,
    width: u32,
    height: u32,
) -> ComponentOutput {
    use crate::components::{card, text};
    
    card(
        "Performance Heatmap",
        text(&format!("🗓️ Heatmap ({} sessions, {}x{})", sessions, width, height))
    )
}

/// Metrics radar chart
pub fn metrics_radar_chart(
    metrics: &[(&str, f32)],
    width: u32,
    height: u32,
) -> ComponentOutput {
    use crate::components::{card, column, text};
    
    let mut items = vec![text(&format!("🎯 Radar Chart ({}x{})", width, height))];
    for (name, value) in metrics {
        items.push(text(&format!("• {}: {:.1}%", name, value * 100.0)));
    }
    
    card(
        "Metrics Radar",
        column(items)
    )
}

/// Progress ring chart
pub fn progress_ring_chart(
    value: f32,
    label: &str,
    size: u32,
) -> ComponentOutput {
    use crate::components::{card, column, text};
    
    let percentage = (value * 100.0) as u32;
    let filled = (percentage / 10) as usize;
    let empty = 10 - filled;
    
    let ring = format!("{}{}",
        "●".repeat(filled),
        "○".repeat(empty)
    );
    
    card(
        label,
        column(vec![
            text(&ring),
            text(&format!("{}% ({}px)", percentage, size)),
        ])
    )
}

/// Sparkline mini chart
pub fn sparkline(
    values: &[f32],
    width: u32,
    height: u32,
) -> ComponentOutput {
    use crate::components::text;
    
    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max = values.iter().cloned().fold(0.0f32, f32::max);
    let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
    let range = max - min;
    
    let sparkline: String = values.iter()
        .map(|&v| {
            let normalized = if range > 0.0 {
                (v - min) / range
            } else {
                0.5
            };
            let index = ((normalized * 7.0) as usize).min(7);
            chars[index]
        })
        .collect();
    
    text(&format!("{} ({}x{})", sparkline, width, height))
}

/// Session comparison chart
pub fn session_comparison_chart(
    session1: &str,
    session2: &str,
    width: u32,
    height: u32,
) -> ComponentOutput {
    use crate::components::{card, text};
    
    card(
        "Session Comparison",
        text(&format!("📊 {} vs {} ({}x{})", session1, session2, width, height))
    )
}

/// Visualization dashboard combining multiple charts with real plotters integration
pub fn visualization_dashboard(state: &AppState) -> ComponentOutput {
    use crate::components::{button, card, column, row, text, Components};
    use crate::viz::chart_components::create_chart_dashboard;
    
    // Create the main chart dashboard with plotters charts
    let chart_dashboard = create_chart_dashboard(state);
    
    // Also include some of our custom widgets for variety
    let custom_widgets = column(vec![
        text("📊 Quick Metrics"),
        row(vec![
            progress_ring_chart(0.85, "Overall Progress", 120),
            sparkline(&[0.3, 0.5, 0.4, 0.7, 0.8, 0.75, 0.9], 200, 40),
        ]),
        button("Export Charts", |_state: &mut AppState| {
            println!("Exporting charts...");
        }),
    ]);
    
    // Combine everything
    column(vec![
        text("📊 Data Visualization Center"),
        chart_dashboard,
        card("Additional Metrics", custom_widgets),
    ])
}