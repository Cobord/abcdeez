// visualizations.rs - Beautiful data visualizations using Plotters

use plotters::prelude::*;
use plotters::style::colors::colormaps::{ColorMap, ViridisRGB};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::error::Error;

use crate::models::PerformanceMetrics;
use graph_learning_core::tasks::TaskResponse;

// Color palette for consistent, beautiful visualizations
pub struct ColorPalette {
    pub primary: RGBColor,
    pub secondary: RGBColor,
    pub success: RGBColor,
    pub error: RGBColor,
    pub warning: RGBColor,
    pub info: RGBColor,
    pub gradient_start: RGBColor,
    pub gradient_end: RGBColor,
    pub background: RGBColor,
    pub grid: RGBColor,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: RGBColor(102, 126, 234),      // Beautiful purple-blue
            secondary: RGBColor(118, 75, 162),     // Deep purple
            success: RGBColor(46, 213, 115),       // Vibrant green
            error: RGBColor(255, 71, 87),          // Soft red
            warning: RGBColor(255, 165, 2),        // Orange
            info: RGBColor(0, 123, 255),           // Bright blue
            gradient_start: RGBColor(102, 126, 234),
            gradient_end: RGBColor(255, 107, 107),
            background: RGBColor(248, 249, 250),   // Light gray
            grid: RGBColor(233, 236, 239),         // Subtle grid
        }
    }
}

/// Create a beautiful learning curve visualization
pub fn create_learning_curve(
    responses: &[TaskResponse],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height))
            .into_drawing_area();

        let palette = ColorPalette::default();
        root.fill(&palette.background)?;

        // Calculate moving average accuracy
        let window_size = 5;
        let mut accuracies = Vec::new();
        let mut timestamps = Vec::new();

        for i in 0..responses.len() {
            let start = i.saturating_sub(window_size / 2);
            let end = (i + window_size / 2 + 1).min(responses.len());
            let window = &responses[start..end];

            let accuracy = window.iter().filter(|r| r.correct).count() as f64
                / window.len() as f64 * 100.0;
            accuracies.push(accuracy);
            timestamps.push(i as f64);
        }

        // Create the chart with beautiful styling
        let mut chart = ChartBuilder::on(&root)
            .caption("Learning Progress", ("Inter", 24).into_font().color(&BLACK))
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                0f64..responses.len() as f64,
                0f64..100f64,
            )?;

        // Draw subtle grid
        chart.configure_mesh()
            .disable_mesh()
            .x_desc("Task Number")
            .y_desc("Accuracy (%)")
            .x_label_formatter(&|x| format!("{:.0}", x))
            .y_label_formatter(&|y| format!("{:.0}%", y))
            .axis_desc_style(("Inter", 12).into_font().color(&DARK_GRAY))
            .draw()?;

        // Draw custom grid lines
        for y in (0..=100).step_by(20) {
            root.draw(&PathElement::new(
                vec![(50, 50 + (height as i32 - 100) * (100 - y) / 100),
                     (width as i32 - 15, 50 + (height as i32 - 100) * (100 - y) / 100)],
                palette.grid.stroke_width(1),
            ))?;
        }

        // Draw gradient area under the curve
        let data_points: Vec<(f64, f64)> = timestamps.iter().copied()
            .zip(accuracies.iter().copied())
            .collect();

        // Create gradient fill
        for (i, window) in data_points.windows(2).enumerate() {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];

            // Calculate gradient color
            let progress = i as f32 / data_points.len() as f32;
            let r = (palette.gradient_start.0 as f32 * (1.0 - progress)
                + palette.gradient_end.0 as f32 * progress) as u8;
            let g = (palette.gradient_start.1 as f32 * (1.0 - progress)
                + palette.gradient_end.1 as f32 * progress) as u8;
            let b = (palette.gradient_start.2 as f32 * (1.0 - progress)
                + palette.gradient_end.2 as f32 * progress) as u8;

            chart.draw_series(AreaSeries::new(
                vec![(x1, 0.0), (x1, y1), (x2, y2), (x2, 0.0)],
                0.0,
                RGBColor(r, g, b).mix(0.2),
            ))?;
        }

        // Draw the main curve with glow effect
        for width in [5, 3, 2].iter() {
            let alpha = if *width == 2 { 1.0 } else { 0.2 };
            chart.draw_series(LineSeries::new(
                data_points.clone(),
                palette.primary.mix(alpha).stroke_width(*width),
            ))?;
        }

        // Draw data points with animation-like appearance
        chart.draw_series(data_points.iter().enumerate().map(|(i, (x, y))| {
            let color = if responses[i].correct {
                palette.success
            } else {
                palette.error
            };
            Circle::new((*x, *y), 3, color.filled())
        }))?;

        // Add performance zones
        let zones = [
            (80.0, 100.0, "Excellent", palette.success.mix(0.1)),
            (60.0, 80.0, "Good", palette.info.mix(0.1)),
            (40.0, 60.0, "Developing", palette.warning.mix(0.1)),
            (0.0, 40.0, "Learning", palette.error.mix(0.1)),
        ];

        for (y_min, y_max, label, color) in zones.iter() {
            chart.draw_series(std::iter::once(Rectangle::new([
                (0.0, *y_min),
                (responses.len() as f64, *y_max)
            ], color.filled())))?;

            // Add zone labels
            root.draw_text(
                label,
                &("Inter", 10).into_font().color(&DARK_GRAY),
                (width as i32 - 60, 50 + (height as i32 - 100) * (100 - ((y_min + y_max) / 2.0) as i32) / 100),
            )?;
        }

        // Add current accuracy annotation
        if let Some(last_accuracy) = accuracies.last() {
            let annotation = format!("Current: {:.1}%", last_accuracy);
            root.draw_text(
                &annotation,
                &("Inter", 14).into_font().color(&palette.primary).style(FontStyle::Bold),
                (width as i32 - 120, 30),
            )?;
        }

        root.present()?;
    }
    Ok(buffer)
}

/// Create a beautiful response time distribution histogram
pub fn create_response_time_histogram(
    response_times: &[u128],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height))
            .into_drawing_area();

        let palette = ColorPalette::default();
        root.fill(&palette.background)?;

        // Calculate histogram bins
        let min_time = *response_times.iter().min().unwrap_or(&0) as f64;
        let max_time = *response_times.iter().max().unwrap_or(&10000) as f64;
        let num_bins = 20;
        let bin_width = (max_time - min_time) / num_bins as f64;

        let mut bins = vec![0; num_bins];
        for &time in response_times {
            let bin_idx = ((time as f64 - min_time) / bin_width).floor() as usize;
            if bin_idx < num_bins {
                bins[bin_idx] += 1;
            }
        }

        let max_count = *bins.iter().max().unwrap_or(&1) as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption("Response Time Distribution", ("Inter", 24).into_font().color(&BLACK))
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                min_time..max_time,
                0f64..max_count * 1.1,
            )?;

        chart.configure_mesh()
            .disable_mesh()
            .x_desc("Response Time (ms)")
            .y_desc("Frequency")
            .x_label_formatter(&|x| format!("{:.0}", x))
            .y_label_formatter(&|y| format!("{:.0}", y))
            .axis_desc_style(("Inter", 12).into_font().color(&DARK_GRAY))
            .draw()?;

        // Draw bars with gradient
        for (i, &count) in bins.iter().enumerate() {
            let x_start = min_time + i as f64 * bin_width;
            let x_end = x_start + bin_width * 0.9; // Small gap between bars

            // Calculate color based on performance
            let avg_bin_time = x_start + bin_width / 2.0;
            let color = if avg_bin_time < 1000.0 {
                palette.success
            } else if avg_bin_time < 2000.0 {
                palette.info
            } else if avg_bin_time < 3000.0 {
                palette.warning
            } else {
                palette.error
            };

            // Draw bar with gradient effect
            for j in 0..count {
                let y_start = j as f64;
                let y_end = (j + 1) as f64;
                let alpha = 0.8 - (j as f32 / count as f32) * 0.3;

                chart.draw_series(std::iter::once(Rectangle::new([
                    (x_start, y_start),
                    (x_end, y_end)
                ], color.mix(alpha).filled())))?;
            }

            // Add value label on top of bar
            if count > 0 {
                root.draw_text(
                    &format!("{}", count),
                    &("Inter", 10).into_font().color(&DARK_GRAY),
                    chart.plotting_area().get_pixel_coord((x_start + bin_width / 2.0, count as f64 + 0.5))
                        .unwrap_or((0, 0)),
                )?;
            }
        }

        // Add statistics overlay
        let mean = response_times.iter().sum::<u128>() as f64 / response_times.len() as f64;
        let median = {
            let mut sorted = response_times.to_vec();
            sorted.sort();
            sorted[sorted.len() / 2] as f64
        };

        // Draw mean line
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(mean, 0.0), (mean, max_count)],
            palette.primary.stroke_width(2),
        )))?;

        // Draw median line
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(median, 0.0), (median, max_count)],
            palette.secondary.stroke_width(2),
        )))?;

        // Add legend
        let legend_items = [
            ("Mean", palette.primary),
            ("Median", palette.secondary),
        ];

        for (i, (label, color)) in legend_items.iter().enumerate() {
            root.draw(&Rectangle::new([
                (width as i32 - 100, 20 + i as i32 * 20),
                (width as i32 - 90, 30 + i as i32 * 20)
            ], color.filled()))?;

            root.draw_text(
                label,
                &("Inter", 10).into_font(),
                (width as i32 - 85, 20 + i as i32 * 20),
            )?;
        }

        root.present()?;
    }
    Ok(buffer)
}

/// Create a beautiful performance heatmap
pub fn create_performance_heatmap(
    responses: &[TaskResponse],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height))
            .into_drawing_area();

        let palette = ColorPalette::default();
        root.fill(&palette.background)?;

        // Group responses by hour of day and day of week
        let mut heatmap_data: Vec<Vec<f64>> = vec![vec![0.0; 24]; 7];
        let mut counts: Vec<Vec<u32>> = vec![vec![0; 24]; 7];

        for response in responses {
            let datetime = response.timestamp;
            let hour = datetime.hour() as usize;
            let day = datetime.weekday().num_days_from_monday() as usize;

            if response.correct {
                heatmap_data[day][hour] += 1.0;
            }
            counts[day][hour] += 1;
        }

        // Calculate percentages
        for day in 0..7 {
            for hour in 0..24 {
                if counts[day][hour] > 0 {
                    heatmap_data[day][hour] = heatmap_data[day][hour] / counts[day][hour] as f64 * 100.0;
                } else {
                    heatmap_data[day][hour] = -1.0; // Mark as no data
                }
            }
        }

        let mut chart = ChartBuilder::on(&root)
            .caption("Performance Heatmap by Time", ("Inter", 24).into_font().color(&BLACK))
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0f64..24f64,
                0f64..7f64,
            )?;

        chart.configure_mesh()
            .disable_mesh()
            .x_desc("Hour of Day")
            .y_desc("Day of Week")
            .x_label_formatter(&|x| format!("{:02}:00", x))
            .y_label_formatter(&|y| {
                let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
                days[*y as usize].to_string()
            })
            .axis_desc_style(("Inter", 12).into_font().color(&DARK_GRAY))
            .draw()?;

        // Draw heatmap cells
        for day in 0..7 {
            for hour in 0..24 {
                let value = heatmap_data[day][hour];

                if value >= 0.0 {
                    // Calculate color based on performance
                    let color = if value >= 80.0 {
                        palette.success
                    } else if value >= 60.0 {
                        palette.info
                    } else if value >= 40.0 {
                        palette.warning
                    } else {
                        palette.error
                    };

                    let intensity = value / 100.0;

                    chart.draw_series(std::iter::once(Rectangle::new([
                        (hour as f64, day as f64),
                        (hour as f64 + 0.95, day as f64 + 0.95)
                    ], color.mix(intensity * 0.8).filled())))?;

                    // Add text label for high activity cells
                    if counts[day][hour] > 5 {
                        root.draw_text(
                            &format!("{:.0}%", value),
                            &("Inter", 8).into_font().color(&WHITE),
                            chart.plotting_area().get_pixel_coord((hour as f64 + 0.5, day as f64 + 0.5))
                                .unwrap_or((0, 0)),
                        )?;
                    }
                }
            }
        }

        // Add color scale legend
        let scale_steps = 5;
        for i in 0..scale_steps {
            let percentage = i as f64 / (scale_steps - 1) as f64 * 100.0;
            let color = if percentage >= 80.0 {
                palette.success
            } else if percentage >= 60.0 {
                palette.info
            } else if percentage >= 40.0 {
                palette.warning
            } else {
                palette.error
            };

            root.draw(&Rectangle::new([
                (width as i32 - 60, height as i32 - 100 - i as i32 * 20),
                (width as i32 - 40, height as i32 - 80 - i as i32 * 20)
            ], color.mix(percentage / 100.0 * 0.8).filled()))?;

            root.draw_text(
                &format!("{:.0}%", percentage),
                &("Inter", 8).into_font(),
                (width as i32 - 35, height as i32 - 95 - i as i32 * 20),
            )?;
        }

        root.present()?;
    }
    Ok(buffer)
}

/// Create a spider/radar chart for multi-dimensional metrics
pub fn create_metrics_radar_chart(
    metrics: &PerformanceMetrics,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height))
            .into_drawing_area();

        let palette = ColorPalette::default();
        root.fill(&palette.background)?;

        // Define metrics for radar chart
        let dimensions = vec![
            ("Accuracy", metrics.accuracy_rate * 100.0),
            ("Speed", (5000.0 - metrics.average_response_time_ms.min(5000.0)) / 50.0),
            ("Consistency", 100.0 - (metrics.recent_accuracy - metrics.accuracy_rate).abs() * 100.0),
            ("Improvement", metrics.improvement_rate * 100.0),
            ("Streak", (metrics.best_streak as f64).min(20.0) * 5.0),
        ];

        let center_x = width as i32 / 2;
        let center_y = height as i32 / 2;
        let radius = (width.min(height) as i32 / 3) as f64;

        // Draw title
        root.draw_text(
            "Performance Metrics",
            &("Inter", 24).into_font().color(&BLACK),
            (center_x - 80, 20),
        )?;

        // Draw radar grid
        for level in 1..=5 {
            let r = radius * level as f64 / 5.0;
            let mut points = Vec::new();

            for i in 0..dimensions.len() {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / dimensions.len() as f64 - std::f64::consts::PI / 2.0;
                let x = center_x + (r * angle.cos()) as i32;
                let y = center_y + (r * angle.sin()) as i32;
                points.push((x, y));
            }

            // Close the polygon
            points.push(points[0]);

            root.draw(&PathElement::new(
                points,
                palette.grid.stroke_width(1),
            ))?;
        }

        // Draw axes
        for (i, (label, _)) in dimensions.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / dimensions.len() as f64 - std::f64::consts::PI / 2.0;
            let x_end = center_x + (radius * angle.cos()) as i32;
            let y_end = center_y + (radius * angle.sin()) as i32;

            root.draw(&PathElement::new(
                vec![(center_x, center_y), (x_end, y_end)],
                palette.grid.stroke_width(2),
            ))?;

            // Draw labels
            let label_offset = 1.15;
            let x_label = center_x + (radius * label_offset * angle.cos()) as i32;
            let y_label = center_y + (radius * label_offset * angle.sin()) as i32;

            root.draw_text(
                label,
                &("Inter", 12).into_font().color(&DARK_GRAY),
                (x_label - 30, y_label - 5),
            )?;
        }

        // Draw data polygon
        let mut data_points = Vec::new();
        for (i, (_, value)) in dimensions.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / dimensions.len() as f64 - std::f64::consts::PI / 2.0;
            let r = radius * value / 100.0;
            let x = center_x + (r * angle.cos()) as i32;
            let y = center_y + (r * angle.sin()) as i32;
            data_points.push((x, y));
        }

        // Close the polygon
        data_points.push(data_points[0]);

        // Draw filled area with gradient
        root.draw(&Polygon::new(
            data_points.clone(),
            palette.primary.mix(0.3).filled(),
        ))?;

        // Draw outline
        root.draw(&PathElement::new(
            data_points.clone(),
            palette.primary.stroke_width(3),
        ))?;

        // Draw data points
        for point in data_points.iter().take(dimensions.len()) {
            root.draw(&Circle::new(
                *point,
                5,
                palette.primary.filled(),
            ))?;
            root.draw(&Circle::new(
                *point,
                3,
                WHITE.filled(),
            ))?;
        }

        // Add value annotations
        for (i, (name, value)) in dimensions.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / dimensions.len() as f64 - std::f64::consts::PI / 2.0;
            let r = radius * value / 100.0 * 0.7;
            let x = center_x + (r * angle.cos()) as i32;
            let y = center_y + (r * angle.sin()) as i32;

            root.draw_text(
                &format!("{:.0}", value),
                &("Inter", 10).into_font().color(&WHITE).style(FontStyle::Bold),
                (x - 10, y - 5),
            )?;
        }

        root.present()?;
    }
    Ok(buffer)
}

/// Create an animated progress ring
pub fn create_progress_ring(
    percentage: f64,
    width: u32,
    height: u32,
    label: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height))
            .into_drawing_area();

        let palette = ColorPalette::default();
        root.fill(&WHITE)?;

        let center_x = width as i32 / 2;
        let center_y = height as i32 / 2;
        let outer_radius = (width.min(height) as i32 / 2 - 20) as f64;
        let inner_radius = outer_radius * 0.7;

        // Draw background ring
        for angle_deg in 0..360 {
            let angle = angle_deg as f64 * std::f64::consts::PI / 180.0;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            root.draw(&PathElement::new(
                vec![
                    (center_x + (inner_radius * cos_a) as i32, center_y + (inner_radius * sin_a) as i32),
                    (center_x + (outer_radius * cos_a) as i32, center_y + (outer_radius * sin_a) as i32),
                ],
                palette.grid.stroke_width(2),
            ))?;
        }

        // Draw progress arc
        let progress_angle = percentage / 100.0 * 360.0;
        for angle_deg in 0..(progress_angle as i32) {
            let angle = (angle_deg as f64 - 90.0) * std::f64::consts::PI / 180.0;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            // Calculate gradient color
            let progress = angle_deg as f32 / 360.0;
            let r = (palette.gradient_start.0 as f32 * (1.0 - progress)
                + palette.gradient_end.0 as f32 * progress) as u8;
            let g = (palette.gradient_start.1 as f32 * (1.0 - progress)
                + palette.gradient_end.1 as f32 * progress) as u8;
            let b = (palette.gradient_start.2 as f32 * (1.0 - progress)
                + palette.gradient_end.2 as f32 * progress) as u8;

            root.draw(&PathElement::new(
                vec![
                    (center_x + (inner_radius * cos_a) as i32, center_y + (inner_radius * sin_a) as i32),
                    (center_x + (outer_radius * cos_a) as i32, center_y + (outer_radius * sin_a) as i32),
                ],
                RGBColor(r, g, b).stroke_width(3),
            ))?;
        }

        // Draw center text
        root.draw_text(
            &format!("{:.0}%", percentage),
            &("Inter", 32).into_font().color(&BLACK).style(FontStyle::Bold),
            (center_x - 35, center_y - 15),
        )?;

        root.draw_text(
            label,
            &("Inter", 14).into_font().color(&DARK_GRAY),
            (center_x - label.len() as i32 * 4, center_y + 15),
        )?;

        root.present()?;
    }
    Ok(buffer)
}

/// Create a beautiful scatter plot with trend line
pub fn create_scatter_plot(
    data: &[(f64, f64)],
    width: u32,
    height: u32,
    x_label: &str,
    y_label: &str,
    title: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height))
            .into_drawing_area();

        let palette = ColorPalette::default();
        root.fill(&palette.background)?;

        let x_min = data.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
        let x_max = data.iter().map(|(x, _)| *x).fold(f64::NEG_INFINITY, f64::max);
        let y_min = data.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
        let y_max = data.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max);

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("Inter", 24).into_font().color(&BLACK))
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(
                x_min * 0.9..x_max * 1.1,
                y_min * 0.9..y_max * 1.1,
            )?;

        chart.configure_mesh()
            .disable_mesh()
            .x_desc(x_label)
            .y_desc(y_label)
            .axis_desc_style(("Inter", 12).into_font().color(&DARK_GRAY))
            .draw()?;

        // Calculate trend line using simple linear regression
        let n = data.len() as f64;
        let sum_x: f64 = data.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = data.iter().map(|(_, y)| y).sum();
        let sum_xx: f64 = data.iter().map(|(x, _)| x * x).sum();
        let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Draw trend line
        let trend_points: Vec<(f64, f64)> = vec![
            (x_min, slope * x_min + intercept),
            (x_max, slope * x_max + intercept),
        ];
        
        chart.draw_series(LineSeries::new(trend_points, &palette.primary.stroke_width(2)))?;

        // Draw scatter points
        chart.draw_series(
            data.iter().map(|&(x, y)| Circle::new((x, y), 3, palette.primary.filled())),
        )?;

        root.present()?;
    }
    Ok(buffer)
}
