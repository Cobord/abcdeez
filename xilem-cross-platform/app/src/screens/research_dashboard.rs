use xilem::{
    view::{button, flex, label, prose, Axis, FlexExt},
    Color, TextAlignment, WidgetView,
};

use crate::{
    components::*, 
    models::*, 
    research::*,
    visualization_components::*,
    AppData, Screen,
};

pub fn research_dashboard_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    let has_controller = data.research_controller.is_some();
    let has_active_experiment = data.research_controller
        .as_ref()
        .and_then(|c| c.active_session.as_ref())
        .is_some();

    let experiment_status = if let Some(controller) = &data.research_controller {
        if let Some(session) = &controller.active_session {
            card(
                "🔬 Active Experiment",
                flex((
                    label(format!("Type: {:?}", session.experiment_type))
                        .alignment(TextAlignment::Start),
                    label(format!("Condition: {}", session.condition.name))
                        .alignment(TextAlignment::Start),
                    label(format!("Data Points: {}", session.data_points.len()))
                        .alignment(TextAlignment::Start),
                    label(format!(
                        "Duration: {}",
                        crate::visualizations::format_duration(
                            chrono::Utc::now()
                                .signed_duration_since(session.start_time)
                                .num_seconds()
                        )
                    ))
                    .alignment(TextAlignment::Start),
                    flex((
                        button("⏸ Pause", |data: &mut AppData| {
                            data.success_message = Some("Experiment paused".to_string());
                        }),
                        button("⏹ End Experiment", |data: &mut AppData| {
                            if let Some(controller) = &mut data.research_controller {
                                match controller.end_experiment() {
                                    Ok(session) => {
                                        data.success_message = Some(format!(
                                            "Experiment {} completed with {} data points",
                                            session.id,
                                            session.data_points.len()
                                        ));
                                    }
                                    Err(e) => {
                                        data.error_message = Some(e);
                                    }
                                }
                            }
                        }),
                    ))
                    .direction(Axis::Horizontal),
                ))
                .direction(Axis::Vertical),
            )
            .into_any_flex()
        } else {
            card(
                "💤 No Active Experiment",
                flex((
                    prose("Start a new experiment to begin collecting research data")
                        .alignment(TextAlignment::Middle),
                    button("🚀 Start New Experiment", |data: &mut AppData| {
                        data.show_experiment_setup = true;
                    }),
                ))
                .direction(Axis::Vertical),
            )
            .into_any_flex()
        }
    } else {
        card(
            "📊 Research Mode",
            flex((
                prose("Enable research mode to collect detailed experimental data")
                    .alignment(TextAlignment::Middle),
                button("🔬 Initialize Research Mode", |data: &mut AppData| {
                    let participant_id = data.current_user
                        .as_ref()
                        .map(|u| u.id.clone())
                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                    
                    data.research_controller = Some(ResearchController::new(participant_id));
                    data.success_message = Some("Research mode initialized".to_string());
                }),
            ))
            .direction(Axis::Vertical),
        )
        .into_any_flex()
    };

    let experiment_setup = if data.show_experiment_setup {
        Some(card(
            "🧪 Experiment Setup",
            flex((
                label("Select Experiment Type:").alignment(TextAlignment::Start),
                flex((
                    button("Learning Curve", |data: &mut AppData| {
                        data.selected_experiment_type = Some(ExperimentType::LearningCurve);
                    }),
                    button("Retention Test", |data: &mut AppData| {
                        data.selected_experiment_type = Some(ExperimentType::RetentionTest);
                    }),
                    button("Interference Study", |data: &mut AppData| {
                        data.selected_experiment_type = Some(ExperimentType::InterferenceStudy);
                    }),
                    button("Adaptive Scheduling", |data: &mut AppData| {
                        data.selected_experiment_type = Some(ExperimentType::AdaptiveScheduling);
                    }),
                ))
                .direction(Axis::Horizontal),
                label("Configure Condition:").alignment(TextAlignment::Start),
                flex((
                    button("Control Group", |data: &mut AppData| {
                        data.experiment_control_group = true;
                    }),
                    button("Experimental Group", |data: &mut AppData| {
                        data.experiment_control_group = false;
                    }),
                ))
                .direction(Axis::Horizontal),
                flex((
                    button("✅ Start", |data: &mut AppData| {
                        if let Some(controller) = &mut data.research_controller {
                            if let Some(exp_type) = data.selected_experiment_type.clone() {
                                let condition = ExperimentCondition {
                                    name: if data.experiment_control_group {
                                        "Control".to_string()
                                    } else {
                                        "Experimental".to_string()
                                    },
                                    parameters: std::collections::HashMap::new(),
                                    control_group: data.experiment_control_group,
                                };
                                
                                match controller.start_experiment(exp_type, condition) {
                                    Ok(session_id) => {
                                        data.success_message = Some(format!(
                                            "Started experiment: {}",
                                            session_id
                                        ));
                                        data.show_experiment_setup = false;
                                    }
                                    Err(e) => {
                                        data.error_message = Some(e);
                                    }
                                }
                            }
                        }
                    }),
                    button("❌ Cancel", |data: &mut AppData| {
                        data.show_experiment_setup = false;
                    }),
                ))
                .direction(Axis::Horizontal),
            ))
            .direction(Axis::Vertical),
        ))
    } else {
        None
    };

    let collected_data = if let Some(controller) = &data.research_controller {
        let total_sessions = controller.sessions.len();
        let total_data_points: usize = controller.sessions
            .iter()
            .map(|s| s.data_points.len())
            .sum();

        card(
            "📈 Collected Data",
            flex((
                flex((
                    metric_display(
                        "Sessions",
                        total_sessions.to_string(),
                        Color::from_rgb8(0, 114, 178),
                    ),
                    metric_display(
                        "Data Points",
                        total_data_points.to_string(),
                        Color::from_rgb8(230, 159, 0),
                    ),
                ))
                .direction(Axis::Horizontal),
                if !controller.sessions.is_empty() {
                    let recent_sessions: Vec<_> = controller.sessions
                        .iter()
                        .rev()
                        .take(5)
                        .map(|session| {
                            let metrics = controller.calculate_metrics(session);
                            flex((
                                label(format!("Session {}", &session.id[..8]))
                                    .alignment(TextAlignment::Start),
                                label(format!("Learning Rate: {:.2}", metrics.learning_rate))
                                    .brush(Color::from_rgb8(0, 200, 0))
                                    .alignment(TextAlignment::End),
                            ))
                            .direction(Axis::Horizontal)
                        })
                        .collect();
                    
                    Some(flex((
                        label("Recent Sessions:").alignment(TextAlignment::Start),
                        flex(recent_sessions).direction(Axis::Vertical),
                    ))
                    .direction(Axis::Vertical))
                } else {
                    None
                },
            ))
            .direction(Axis::Vertical),
        )
        .into_any_flex()
    } else {
        card(
            "📈 No Data",
            label("Initialize research mode to begin collecting data")
                .alignment(TextAlignment::Middle),
        )
        .into_any_flex()
    };

    let analysis_section = if let Some(controller) = &data.research_controller {
        if !controller.sessions.is_empty() {
            let analyzer = ResearchAnalyzer::new(controller.sessions.clone());
            let learning_curves = analyzer.analyze_learning_curves();
            let retention_score = analyzer.analyze_retention(30);
            let condition_comparison = analyzer.compare_conditions();

            card(
                "🔍 Analysis Results",
                flex((
                    label(format!("Average Retention (30min): {:.1}%", retention_score * 100.0))
                        .alignment(TextAlignment::Start),
                    if !condition_comparison.is_empty() {
                        let comparisons: Vec<_> = condition_comparison
                            .iter()
                            .map(|(condition, metrics)| {
                                flex((
                                    label(format!("Condition: {}", condition))
                                        .alignment(TextAlignment::Start),
                                    label(format!("Learning: {:.2}", metrics.learning_rate))
                                        .brush(Color::from_rgb8(0, 128, 255))
                                        .alignment(TextAlignment::End),
                                ))
                                .direction(Axis::Horizontal)
                            })
                            .collect();
                        
                        Some(flex(comparisons).direction(Axis::Vertical))
                    } else {
                        None
                    },
                ))
                .direction(Axis::Vertical),
            )
            .into_any_flex()
        } else {
            card(
                "🔍 Analysis",
                label("Complete experiments to see analysis results")
                    .alignment(TextAlignment::Middle),
            )
            .into_any_flex()
        }
    } else {
        card(
            "🔍 Analysis",
            label("No data available for analysis")
                .alignment(TextAlignment::Middle),
        )
        .into_any_flex()
    };

    let export_controls = card(
        "💾 Export Research Data",
        flex((
            prose("Export collected research data for external analysis")
                .alignment(TextAlignment::Start),
            flex((
                button("📄 Export JSON", |data: &mut AppData| {
                    if let Some(controller) = &data.research_controller {
                        match controller.export_research_data(crate::research::ExportFormat::Json) {
                            Ok(_) => {
                                data.success_message = Some("Research data exported as JSON".to_string());
                            }
                            Err(e) => {
                                data.error_message = Some(e);
                            }
                        }
                    }
                }),
                button("📊 Export CSV", |data: &mut AppData| {
                    if let Some(controller) = &data.research_controller {
                        match controller.export_research_data(crate::research::ExportFormat::Csv) {
                            Ok(_) => {
                                data.success_message = Some("Research data exported as CSV".to_string());
                            }
                            Err(e) => {
                                data.error_message = Some(e);
                            }
                        }
                    }
                }),
                button("🐍 Export Python", |data: &mut AppData| {
                    data.success_message = Some("Python export coming soon!".to_string());
                }),
            ))
            .direction(Axis::Horizontal),
        ))
        .direction(Axis::Vertical),
    );

    let privacy_controls = card(
        "🔒 Privacy Settings",
        flex((
            flex((
                label("Data Collection:").alignment(TextAlignment::Start),
                button(
                    if data.research_data_collection_enabled {
                        "✅ Enabled"
                    } else {
                        "❌ Disabled"
                    },
                    |data: &mut AppData| {
                        data.research_data_collection_enabled = !data.research_data_collection_enabled;
                        if let Some(controller) = &mut data.research_controller {
                            controller.data_collection_enabled = data.research_data_collection_enabled;
                        }
                    },
                ),
            ))
            .direction(Axis::Horizontal),
            flex((
                label("Privacy Mode:").alignment(TextAlignment::Start),
                button(
                    if data.research_privacy_mode {
                        "🔒 On"
                    } else {
                        "🔓 Off"
                    },
                    |data: &mut AppData| {
                        data.research_privacy_mode = !data.research_privacy_mode;
                        if let Some(controller) = &mut data.research_controller {
                            controller.privacy_mode = data.research_privacy_mode;
                        }
                    },
                ),
            ))
            .direction(Axis::Horizontal),
        ))
        .direction(Axis::Vertical),
    );

    let navigation = card(
        "🎮 Navigation",
        flex((
            button("📊 Dashboard", |data: &mut AppData| {
                data.current_screen = Screen::Dashboard;
            }),
            button("🎯 Training", |data: &mut AppData| {
                data.current_screen = Screen::Training;
            }),
            button("⚙️ Settings", |data: &mut AppData| {
                data.current_screen = Screen::Settings;
            }),
        ))
        .direction(Axis::Horizontal),
    );

    flex((
        label("🔬 Research Dashboard")
            .brush(Color::from_rgb8(128, 0, 255))
            .alignment(TextAlignment::Middle),
        experiment_status,
        experiment_setup,
        collected_data,
        analysis_section,
        export_controls,
        privacy_controls,
        navigation,
    ))
    .direction(Axis::Vertical)
}