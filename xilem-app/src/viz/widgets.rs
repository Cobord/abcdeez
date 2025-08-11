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

/// Visualization dashboard combining multiple charts
pub fn visualization_dashboard(state: &AppState) -> ComponentOutput {
    use crate::components::{button, card, column, row, text};
    
    // Sample data for visualization
    let learning_data = vec![(0.0, 0.5), (1.0, 0.7), (2.0, 0.85), (3.0, 0.9)];
    let response_times = vec![1200, 1100, 950, 900, 850];
    let metrics = vec![
        ("Accuracy", 0.85),
        ("Speed", 0.72),
        ("Retention", 0.90),
        ("Focus", 0.68),
    ];
    
    card(
        "📊 Visualizations",
        column(vec![
            text("Performance Analytics"),
            
            row(vec![
                learning_curve_chart(&learning_data, 400, 240),
                response_time_histogram(&response_times, 400, 240),
            ]),
            
            row(vec![
                performance_heatmap(10, 400, 240),
                metrics_radar_chart(&metrics, 400, 240),
            ]),
            
            row(vec![
                progress_ring_chart(0.85, "Overall Progress", 120),
                sparkline(&[0.3, 0.5, 0.4, 0.7, 0.8, 0.75, 0.9], 200, 40),
            ]),
            
            button("Export Charts", |_state: &mut AppState| {
                // Would export visualizations
            }),
        ])
    )
}