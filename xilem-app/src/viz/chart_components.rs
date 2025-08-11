// Chart components that integrate plotters with the component system

use crate::components::{AppComponents, Component, ComponentOutput, Components};
use crate::models::Response;
use crate::state::AppState;
use crate::viz::charts;
use base64::{engine::general_purpose::STANDARD, Engine};

/// Renders a learning curve chart as a component
pub fn learning_curve_component(responses: &[Response]) -> ComponentOutput {
    let width = 800;
    let height = 400;
    
    match charts::create_learning_curve(responses, width, height) {
        Ok(image_data) => {
            // Convert RGB buffer to base64 for embedding
            let base64_image = STANDARD.encode(&image_data);
            let data_url = format!("data:image/bmp;base64,{}", base64_image);
            
            // For now, we'll display as a label with the status
            // In a real implementation, we'd create an image component
            Components::card(
                "Learning Curve",
                Components::simple_flex_column(vec![
                    Components::simple_label(format!("📊 Chart rendered ({} responses)", responses.len())),
                    Components::simple_label(format!("Size: {}x{}", width, height)),
                    // In production, this would be an actual image component
                    // Components::image(data_url)
                ])
            )
        }
        Err(e) => {
            Components::card(
                "Learning Curve",
                Components::error_message(Some(format!("Failed to render chart: {}", e)))
            )
        }
    }
}

/// Renders a response time histogram as a component
pub fn response_time_histogram_component(response_times: &[u128]) -> ComponentOutput {
    let width = 800;
    let height = 400;
    
    match charts::create_response_time_histogram(response_times, width, height) {
        Ok(image_data) => {
            let base64_image = STANDARD.encode(&image_data);
            let data_url = format!("data:image/bmp;base64,{}", base64_image);
            
            Components::card(
                "Response Time Distribution",
                Components::simple_flex_column(vec![
                    Components::simple_label(format!("📊 Histogram rendered ({} samples)", response_times.len())),
                    Components::simple_label(format!("Size: {}x{}", width, height)),
                ])
            )
        }
        Err(e) => {
            Components::card(
                "Response Time Distribution",
                Components::error_message(Some(format!("Failed to render histogram: {}", e)))
            )
        }
    }
}

/// Renders a performance heatmap as a component
pub fn performance_heatmap_component(responses: &[Response]) -> ComponentOutput {
    let width = 800;
    let height = 400;
    
    match charts::create_performance_heatmap(responses, width, height) {
        Ok(image_data) => {
            let base64_image = STANDARD.encode(&image_data);
            let data_url = format!("data:image/bmp;base64,{}", base64_image);
            
            Components::card(
                "Performance Heatmap",
                Components::simple_flex_column(vec![
                    Components::simple_label(format!("📊 Heatmap rendered ({} responses)", responses.len())),
                    Components::simple_label(format!("Size: {}x{}", width, height)),
                ])
            )
        }
        Err(e) => {
            Components::card(
                "Performance Heatmap",
                Components::error_message(Some(format!("Failed to render heatmap: {}", e)))
            )
        }
    }
}

/// Renders a radar chart for metrics as a component
pub fn metrics_radar_component(state: &AppState) -> ComponentOutput {
    // Extract metrics from state
    let mut metrics = Vec::new();
    
    // Calculate various performance metrics
    if let Some(session) = &state.session {
        // Accuracy metric (based on performance buffer)
        let accuracy = if !session.performance_buffer.is_empty() {
            let correct = session.performance_buffer.iter().filter(|m| m.correct).count();
            correct as f64 / session.performance_buffer.len() as f64
        } else {
            0.0
        };
        metrics.push(("Accuracy".to_string(), accuracy));
        
        // Speed metric (normalized from response times)
        if !state.response_times.is_empty() {
            let avg_time = state.response_times.iter().sum::<u128>() as f64 / state.response_times.len() as f64;
            // Normalize to 0-1 (faster is better, <1000ms = 1.0, >5000ms = 0.0)
            let speed_score = ((5000.0 - avg_time) / 4000.0).clamp(0.0, 1.0);
            metrics.push(("Speed".to_string(), speed_score));
        }
        
        // Consistency metric (lower variance is better)
        if state.response_times.len() > 1 {
            let mean = state.response_times.iter().sum::<u128>() as f64 / state.response_times.len() as f64;
            let variance = state.response_times.iter()
                .map(|&t| {
                    let diff = t as f64 - mean;
                    diff * diff
                })
                .sum::<f64>() / state.response_times.len() as f64;
            let std_dev = variance.sqrt();
            // Normalize (lower std dev is better, <500ms = 1.0, >2000ms = 0.0)
            let consistency = ((2000.0 - std_dev) / 1500.0).clamp(0.0, 1.0);
            metrics.push(("Consistency".to_string(), consistency));
        }
        
        // Engagement metric (based on tasks completed)
        let engagement = (session.tasks_completed as f64 / 50.0).min(1.0); // 50 tasks = full engagement
        metrics.push(("Engagement".to_string(), engagement));
        
        // Progress metric (based on tasks completed)
        let progress = (session.tasks_completed as f64 / 10.0).min(1.0); // 10 tasks = initial progress
        metrics.push(("Progress".to_string(), progress));
    }
    
    if metrics.is_empty() {
        metrics = vec![
            ("Accuracy".to_string(), 0.5),
            ("Speed".to_string(), 0.5),
            ("Consistency".to_string(), 0.5),
            ("Engagement".to_string(), 0.5),
            ("Progress".to_string(), 0.5),
        ];
    }
    
    let width = 600;
    let height = 600;
    
    match charts::create_metrics_radar_chart(&metrics, width, height) {
        Ok(image_data) => {
            let base64_image = STANDARD.encode(&image_data);
            let data_url = format!("data:image/bmp;base64,{}", base64_image);
            
            Components::card(
                "Performance Radar",
                Components::simple_flex_column(vec![
                    Components::simple_label(format!("📊 Radar chart with {} metrics", metrics.len())),
                    Components::simple_label(format!("Size: {}x{}", width, height)),
                ])
            )
        }
        Err(e) => {
            Components::card(
                "Performance Radar",
                Components::error_message(Some(format!("Failed to render radar chart: {}", e)))
            )
        }
    }
}

/// Creates a complete visualization dashboard with all charts
pub fn create_chart_dashboard(state: &AppState) -> ComponentOutput {
    // Collect response data for charts
    let responses: Vec<Response> = if !state.response_times.is_empty() {
        // Create mock responses from response times for demonstration
        state.response_times.iter().enumerate().map(|(i, &time)| {
            Response {
                id: uuid::Uuid::new_v4(),
                session_id: uuid::Uuid::new_v4(),
                sequence_number: i as i32,
                task_type: "demo".to_string(),
                task_data: serde_json::json!({}),
                user_answer: Some("A".to_string()),
                correct: time < 2000, // Responses under 2 seconds are "correct"
                response_time_ms: time,
                hint_level: Some(0),
                timestamp: chrono::Utc::now() - chrono::Duration::minutes((state.response_times.len() - i) as i64),
            }
        }).collect()
    } else {
        Vec::new()
    };
    
    let mut charts = vec![];
    
    // Add learning curve if we have responses
    if !responses.is_empty() {
        charts.push(learning_curve_component(&responses));
    }
    
    // Add response time histogram if we have times
    if !state.response_times.is_empty() {
        charts.push(response_time_histogram_component(&state.response_times));
    }
    
    // Add performance heatmap if we have responses
    if !responses.is_empty() {
        charts.push(performance_heatmap_component(&responses));
    }
    
    // Always add the radar chart
    charts.push(metrics_radar_component(state));
    
    // Add a summary card
    let summary = if !responses.is_empty() {
        let correct = responses.iter().filter(|r| r.correct).count();
        let total = responses.len();
        let accuracy = correct as f64 / total as f64 * 100.0;
        let avg_time = state.response_times.iter().sum::<u128>() as f64 / state.response_times.len() as f64;
        
        Components::card(
            "Summary Statistics",
            Components::simple_flex_column(vec![
                Components::simple_label(format!("📊 Total Responses: {}", total)),
                Components::simple_label(format!("✅ Accuracy: {:.1}%", accuracy)),
                Components::simple_label(format!("⏱️ Avg Response Time: {:.0}ms", avg_time)),
                Components::simple_label(format!("📈 Charts Generated: {}", charts.len())),
            ])
        )
    } else {
        Components::card(
            "Summary Statistics",
            Components::simple_label("No data available yet. Complete some tasks to see visualizations!".to_string())
        )
    };
    
    charts.insert(0, summary);
    
    Components::simple_flex_column(charts)
}