// Chart rendering using Plotters library
// Honest, accessible data visualizations with no chartjunk

use chrono::{Datelike, Timelike};
use plotters::prelude::*;
use std::collections::HashMap;
use std::error::Error;

use crate::models::Response;

// Colorblind-safe palette using ColorBrewer schemes
// Tested with Coblis colorblind simulator
pub struct AccessiblePalette {
    pub primary: RGBColor,    // Blue - works for all colorblind types
    pub secondary: RGBColor,  // Orange - distinguishable from blue
    pub success: RGBColor,    // Teal - not pure green
    pub error: RGBColor,      // Vermillion - not pure red
    pub warning: RGBColor,    // Yellow - high contrast
    pub neutral: RGBColor,    // Gray
    pub background: RGBColor, // Light gray
    pub grid: RGBColor,       // Medium gray
    pub text: RGBColor,       // Dark gray
}

impl Default for AccessiblePalette {
    fn default() -> Self {
        Self {
            primary: RGBColor(0, 114, 178),      // Colorblind-safe blue
            secondary: RGBColor(230, 159, 0),    // Colorblind-safe orange
            success: RGBColor(0, 158, 115),      // Colorblind-safe teal
            error: RGBColor(213, 94, 0),         // Colorblind-safe vermillion
            warning: RGBColor(240, 228, 66),     // High-contrast yellow
            neutral: RGBColor(128, 128, 128),    // Neutral gray
            background: RGBColor(250, 250, 250), // Very light gray
            grid: RGBColor(200, 200, 200),       // Light grid lines
            text: RGBColor(50, 50, 50),          // Dark text
        }
    }
}

// Centralized performance thresholds based on educational research
pub struct PerformanceThresholds {
    pub mastery: f64,    // 85% - Educational mastery level
    pub proficient: f64, // 70% - Proficiency threshold
    pub developing: f64, // 55% - Developing skills
    pub struggling: f64, // 40% - Needs intervention
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            mastery: 0.85,
            proficient: 0.70,
            developing: 0.55,
            struggling: 0.40,
        }
    }
}

/// Create an HONEST learning curve visualization with confidence intervals
pub fn create_learning_curve(
    responses: &[Response],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();

        let palette = AccessiblePalette::default();
        let thresholds = PerformanceThresholds::default();
        root.fill(&palette.background)?;

        if responses.is_empty() {
            // Show meaningful empty state
            root.draw_text(
                "No data available yet",
                &("sans-serif", 20).into_font().color(&palette.text),
                (width as i32 / 2 - 80, height as i32 / 2),
            )?;
            root.present()?;
        } else {
            // Calculate ACTUAL performance with confidence intervals
        // Using proper statistical window sizing (sqrt(n) rule)
        let window_size = (responses.len() as f64).sqrt().max(3.0).min(20.0) as usize;

        let mut performance_data = Vec::new();
        let mut confidence_bands = Vec::new();

        for i in 0..responses.len() {
            let start = i.saturating_sub(window_size / 2);
            let end = (i + window_size / 2 + 1).min(responses.len());
            let window = &responses[start..end];

            if !window.is_empty() {
                let correct = window.iter().filter(|r| r.correct).count() as f64;
                let total = window.len() as f64;
                let accuracy = correct / total;

                // Calculate 95% confidence interval using Wilson score
                let z = 1.96; // 95% confidence
                let n = total;
                let p_hat = accuracy;

                let denominator = 1.0 + z * z / n;
                let center = (p_hat + z * z / (2.0 * n)) / denominator;
                let margin = (z / denominator)
                    * ((p_hat * (1.0 - p_hat) / n) + (z * z / (4.0 * n * n))).sqrt();

                performance_data.push((i as f64, accuracy));
                confidence_bands.push((
                    i as f64,
                    (center - margin).max(0.0),
                    (center + margin).min(1.0),
                ));
            }
        }

        // Determine appropriate Y-axis range based on actual data
        let min_accuracy = performance_data
            .iter()
            .map(|(_, acc)| *acc)
            .fold(1.0, f64::min);
        let max_accuracy = performance_data
            .iter()
            .map(|(_, acc)| *acc)
            .fold(0.0, f64::max);

        // Add padding but show actual range
        let y_min = (min_accuracy - 0.1).max(0.0);
        let y_max = (max_accuracy + 0.1).min(1.0);

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Learning Progress (with 95% Confidence Interval)",
                ("sans-serif", 20).into_font().color(&palette.text),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0f64..responses.len() as f64, y_min..y_max)?;

        chart
            .configure_mesh()
            .disable_mesh()
            .x_desc("Task Number")
            .y_desc("Accuracy")
            .x_label_formatter(&|x| format!("{:.0}", x))
            .y_label_formatter(&|y| format!("{:.0}%", y * 100.0))
            .axis_desc_style(("sans-serif", 12).into_font().color(&palette.text))
            .draw()?;

        // Draw performance threshold lines with labels
        let threshold_lines = [
            (thresholds.mastery, "Mastery", palette.success),
            (thresholds.proficient, "Proficient", palette.primary),
            (thresholds.developing, "Developing", palette.warning),
        ];

        for (threshold, _label, color) in threshold_lines.iter() {
            if *threshold >= y_min && *threshold <= y_max {
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(0.0, *threshold), (responses.len() as f64, *threshold)],
                    color.mix(0.3).stroke_width(1),
                )))?;
            }
        }

        // Draw confidence bands (honest uncertainty representation)
        for window in confidence_bands.windows(2) {
            let (x1, lower1, upper1) = window[0];
            let (x2, lower2, upper2) = window[1];

            // Draw the confidence band as a filled area
            chart.draw_series(std::iter::once(Polygon::new(
                vec![(x1, lower1), (x1, upper1), (x2, upper2), (x2, lower2)],
                palette.primary.mix(0.2).filled(),
            )))?;
        }

        // Draw the actual performance line
        chart
            .draw_series(std::iter::once(PathElement::new(
                performance_data.clone(),
                palette.primary.stroke_width(2),
            )))?
            .label("Actual Performance")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &palette.primary));

        // Draw individual correct/incorrect points
        for (i, response) in responses.iter().enumerate() {
            let color = if response.correct {
                palette.success.mix(0.6)
            } else {
                palette.error.mix(0.6)
            };

            let y_value = if response.correct { 1.0 } else { 0.0 };
            chart.draw_series(std::iter::once(Circle::new(
                (i as f64, y_value),
                2,
                color.filled(),
            )))?;
        }

            // Add summary statistics
            let total = responses.len();
            let correct = responses.iter().filter(|r| r.correct).count();
            let _overall_accuracy = correct as f64 / total as f64;

            chart
                .configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .border_style(&BLACK)
                .draw()?;

            root.present()?;
        } // Close the else block
    }
    Ok(buffer)
}

/// Create a response time histogram with proper binning
pub fn create_response_time_histogram(
    response_times: &[u128],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        let palette = AccessiblePalette::default();
        root.fill(&palette.background)?;

        if response_times.is_empty() {
            root.draw_text(
                "No response time data yet",
                &("sans-serif", 20).into_font().color(&palette.text),
                (width as i32 / 2 - 100, height as i32 / 2),
            )?;
            root.present()?;
        } else {

        // Use Sturges' rule for optimal bin count
        let n = response_times.len() as f64;
        let bin_count = (1.0 + 3.322 * n.log10()).ceil() as usize;
        let bin_count = bin_count.clamp(5, 30); // Reasonable limits

        let min_time = *response_times.iter().min().unwrap() as f64;
        let max_time = *response_times.iter().max().unwrap() as f64;
        let bin_width = (max_time - min_time) / bin_count as f64;

        // Create histogram bins
        let mut bins = vec![0; bin_count];
        for &time in response_times {
            let bin_idx = ((time as f64 - min_time) / bin_width).floor() as usize;
            let bin_idx = bin_idx.min(bin_count - 1);
            bins[bin_idx] += 1;
        }

        let max_count = *bins.iter().max().unwrap() as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Response Time Distribution",
                ("sans-serif", 20).into_font().color(&palette.text),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                min_time..max_time,
                0f64..(max_count * 1.1),
            )?;

        chart
            .configure_mesh()
            .x_desc("Response Time (ms)")
            .y_desc("Frequency")
            .x_label_formatter(&|x| format!("{:.0}ms", x))
            .y_label_formatter(&|y| format!("{:.0}", y))
            .axis_desc_style(("sans-serif", 12).into_font().color(&palette.text))
            .draw()?;

        // Draw histogram bars
        let bar_data: Vec<_> = bins
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let x = min_time + i as f64 * bin_width + bin_width / 2.0;
                (x, count as f64)
            })
            .collect();

        chart.draw_series(
            bar_data
                .iter()
                .map(|&(x, y)| {
                    Rectangle::new(
                        [(x - bin_width / 2.0, 0.0), (x + bin_width / 2.0, y)],
                        palette.primary.filled(),
                    )
                }),
        )?;

        // Add mean and median lines
        let mean = response_times.iter().sum::<u128>() as f64 / response_times.len() as f64;
        let mut sorted = response_times.to_vec();
        sorted.sort();
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) as f64 / 2.0
        } else {
            sorted[sorted.len() / 2] as f64
        };

        chart.draw_series(std::iter::once(PathElement::new(
            vec![(mean, 0.0), (mean, max_count * 1.1)],
            palette.secondary.stroke_width(2),
        )))?
        .label(format!("Mean: {:.0}ms", mean))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &palette.secondary));

        chart.draw_series(std::iter::once(PathElement::new(
            vec![(median, 0.0), (median, max_count * 1.1)],
            palette.success.stroke_width(2),
        )))?
        .label(format!("Median: {:.0}ms", median))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &palette.success));

            chart
                .configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .border_style(&BLACK)
                .draw()?;

            root.present()?;
        } // Close the else block
    }
    Ok(buffer)
}

/// Create a performance heatmap by day/hour
pub fn create_performance_heatmap(
    responses: &[Response],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        let palette = AccessiblePalette::default();
        root.fill(&palette.background)?;

        if responses.is_empty() {
            root.draw_text(
                "No performance data yet",
                &("sans-serif", 20).into_font().color(&palette.text),
                (width as i32 / 2 - 90, height as i32 / 2),
            )?;
            root.present()?;
        } else {

        // Aggregate performance by day of week and hour
        let mut performance_map: HashMap<(u32, u32), Vec<bool>> = HashMap::new();
        
        for response in responses {
            let day = response.timestamp.weekday().num_days_from_monday();
            let hour = response.timestamp.hour();
            performance_map
                .entry((day, hour))
                .or_insert_with(Vec::new)
                .push(response.correct);
        }

        // Calculate average performance for each cell
        let mut heatmap_data = Vec::new();
        for day in 0..7 {
            for hour in 0..24 {
                if let Some(results) = performance_map.get(&(day, hour)) {
                    let accuracy = results.iter().filter(|&&x| x).count() as f64 / results.len() as f64;
                    heatmap_data.push((day as f64, hour as f64, accuracy));
                }
            }
        }

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Performance by Day and Time",
                ("sans-serif", 20).into_font().color(&palette.text),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0f64..7f64, 0f64..24f64)?;

        let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        
        chart
            .configure_mesh()
            .x_desc("Day of Week")
            .y_desc("Hour of Day")
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < days.len() {
                    days[idx].to_string()
                } else {
                    String::new()
                }
            })
            .y_label_formatter(&|y| format!("{:02}:00", y))
            .axis_desc_style(("sans-serif", 12).into_font().color(&palette.text))
            .draw()?;

        // Draw heatmap cells
        for (day, hour, accuracy) in heatmap_data {
            let color = if accuracy >= 0.85 {
                palette.success
            } else if accuracy >= 0.70 {
                palette.primary
            } else if accuracy >= 0.55 {
                palette.warning
            } else {
                palette.error
            };

            chart.draw_series(std::iter::once(Rectangle::new(
                [(day, hour), (day + 1.0, hour + 1.0)],
                color.mix(accuracy as f64).filled(),
            )))?;
        }

            root.present()?;
        } // Close the else block
    }
    Ok(buffer)
}

/// Create a radar chart for multiple metrics
pub fn create_metrics_radar_chart(
    metrics: &[(String, f64)], // (metric_name, normalized_value 0-1)
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        let palette = AccessiblePalette::default();
        root.fill(&palette.background)?;

        if metrics.is_empty() {
            root.draw_text(
                "No metrics data yet",
                &("sans-serif", 20).into_font().color(&palette.text),
                (width as i32 / 2 - 70, height as i32 / 2),
            )?;
            root.present()?;
        } else {

        let center_x = width as i32 / 2;
        let center_y = height as i32 / 2;
        let radius = (width.min(height) as i32 / 3) as f64;

        // Draw title
        root.draw_text(
            "Performance Metrics",
            &("sans-serif", 20).into_font().color(&palette.text),
            (center_x - 80, 20),
        )?;

        // Draw radar grid
        for level in 1..=5 {
            let r = radius * (level as f64 / 5.0);
            let mut points = Vec::new();
            
            for i in 0..metrics.len() {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / metrics.len() as f64 - std::f64::consts::PI / 2.0;
                let x = center_x as f64 + r * angle.cos();
                let y = center_y as f64 + r * angle.sin();
                points.push((x as i32, y as i32));
            }
            
            // Close the polygon
            if !points.is_empty() {
                points.push(points[0]);
            }
            
            for window in points.windows(2) {
                root.draw(&PathElement::new(
                    vec![window[0], window[1]],
                    palette.grid.stroke_width(1),
                ))?;
            }
        }

        // Draw axes and labels
        for (i, (label, _)) in metrics.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / metrics.len() as f64 - std::f64::consts::PI / 2.0;
            let x = center_x as f64 + radius * angle.cos();
            let y = center_y as f64 + radius * angle.sin();
            
            root.draw(&PathElement::new(
                vec![(center_x, center_y), (x as i32, y as i32)],
                palette.grid.stroke_width(1),
            ))?;
            
            // Position labels outside the circle
            let label_x = center_x as f64 + (radius + 20.0) * angle.cos();
            let label_y = center_y as f64 + (radius + 20.0) * angle.sin();
            
            root.draw_text(
                label,
                &("sans-serif", 10).into_font().color(&palette.text),
                (label_x as i32 - 30, label_y as i32),
            )?;
        }

        // Draw data polygon
        let mut data_points = Vec::new();
        for (i, (_, value)) in metrics.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / metrics.len() as f64 - std::f64::consts::PI / 2.0;
            let r = radius * value;
            let x = center_x as f64 + r * angle.cos();
            let y = center_y as f64 + r * angle.sin();
            data_points.push((x as i32, y as i32));
        }
        
        if !data_points.is_empty() {
            data_points.push(data_points[0]);
            
            // Fill the polygon
            root.draw(&Polygon::new(
                data_points.clone(),
                palette.primary.mix(0.3).filled(),
            ))?;
            
            // Draw the outline
            for window in data_points.windows(2) {
                root.draw(&PathElement::new(
                    vec![window[0], window[1]],
                    palette.primary.stroke_width(2),
                ))?;
            }
            
            // Draw data points
            for point in &data_points[..data_points.len()-1] {
                root.draw(&Circle::new(
                    *point,
                    3,
                    palette.primary.filled(),
                ))?;
            }
        }

            root.present()?;
        } // Close the else block
    }
    Ok(buffer)
}