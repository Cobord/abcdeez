use xilem::{
    view::{button, flex, label, prose, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::{components::*, models::*, AppData, Screen};

// Welcome/Login Screen
pub fn welcome_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        label("Adaptive Learning System")
            .brush(Color::from_rgb8(0, 128, 255)),

        prose("An intelligent learning system that adapts to your knowledge and optimizes your learning path using graph-based cognitive models."),

        card("Login", flex((
            labeled_input(
                "Username:",
                data.username_input.clone(),
                std::sync::Arc::new(|data: &mut AppData, value: String| {
                    data.username_input = value;
                }),
            ),
            labeled_input(
                "Password:",
                data.password_input.clone(),
                std::sync::Arc::new(|data: &mut AppData, value: String| {
                    data.password_input = value;
                }),
            ),
            button("Login", |data: &mut AppData| {
                data.login();
            }),
        )).direction(Axis::Vertical)),

        card("Quick Start", flex((
            prose("Start learning immediately without creating an account")
                .alignment(TextAlignment::Middle),
            button("Start as Guest", |data: &mut AppData| {
                // Create a guest user and learner
                data.current_user = Some(User {
                    id: uuid::Uuid::new_v4().to_string(),
                    username: "Guest".to_string(),
                    email: "guest@example.com".to_string(),
                    password_hash: String::new(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                });
                data.create_learner();
                data.current_screen = Screen::DomainSelection;
            }),
            // Demo showcase for portfolio / presentation flows
            button("Demo Showcase", |data: &mut AppData| {
                // Run a short automated demo training sequence and navigate to Dashboard
                data.demo_showcase();
                data.current_screen = Screen::Dashboard;
            }),
        )).direction(Axis::Vertical)),
    ))
    .direction(Axis::Vertical)
}

// Domain Selection Screen
pub fn domain_selection_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    let domains = vec![
        Domain::Alphabet,
        Domain::DaysOfWeek,
        Domain::Music,
        Domain::Mathematics,
    ];

    let domain_cards = domains
        .into_iter()
        .map(|domain| {
            let is_selected = data.selected_domain == domain;
            let header = prose(domain.description()).alignment(TextAlignment::Start);

            // Always render a Select button; it is a no-op when the domain is already selected.
            // Use a single closure that checks the selected flag at runtime so the Button type stays the same.
            let d_cloned = domain.clone();
            let selected_flag = is_selected;
            let select_button = button("Select", move |data: &mut AppData| {
                if !selected_flag {
                    data.selected_domain = d_cloned.clone();
                    data.create_learner();
                }
            });

            card(
                domain.display_name(),
                flex((header, select_button)).direction(Axis::Vertical),
            )
        })
        .collect::<Vec<_>>();

    flex((
        label("Select Learning Domain")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        prose("Choose a domain to begin your adaptive training session")
            .alignment(TextAlignment::Middle),
        flex(domain_cards).direction(Axis::Vertical),
        card(
            "Session Settings",
            flex((
                checkbox(
                    data.use_adaptive_scheduling,
                    "Use Adaptive Scheduling (AI-powered task selection)",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.use_adaptive_scheduling = checked;
                    }),
                ),
                checkbox(
                    data.enable_hints,
                    "Enable Hints (Show help when struggling)",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.enable_hints = checked;
                    }),
                ),
            ))
            .direction(Axis::Vertical),
        ),
        button("Start Training Session", |data: &mut AppData| {
            data.start_session();
        }),
    ))
    .direction(Axis::Vertical)
}

// Training Session Screen
pub fn training_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    // Session progress bar
    let progress = if data.session_responses.len() > 0 {
        let correct = data.session_responses.iter().filter(|r| r.correct).count();
        correct as f64 / data.session_responses.len() as f64
    } else {
        0.0
    };

    let progress_display = card(
        "Session Progress",
        flex((
            flex((
                metric_display(
                    "Tasks Completed:",
                    data.session_responses.len().to_string(),
                    Color::from_rgb8(0, 128, 255),
                ),
                metric_display(
                    "Accuracy:",
                    format!("{:.1}%", data.current_metrics.accuracy_rate * 100.0),
                    if data.current_metrics.accuracy_rate >= 0.7 {
                        Color::from_rgb8(0, 200, 0)
                    } else {
                        Color::from_rgb8(255, 128, 0)
                    },
                ),
                metric_display(
                    "Streak:",
                    format!(
                        "{} (Best: {})",
                        data.current_metrics.streak_count, data.current_metrics.best_streak
                    ),
                    Color::from_rgb8(255, 128, 0),
                ),
            ))
            .direction(Axis::Horizontal),
            progress_bar(progress, "Overall Progress".to_string()),
        ))
        .direction(Axis::Vertical),
    );

    // Task display (unified, homogeneous types)
    let task_display = {
        // Prompt and difficulty are always labels (same types for both branches)
        let prompt = data
            .current_task
            .as_ref()
            .map(|t| t.display_prompt.clone())
            .unwrap_or_else(|| "Generating next task...".to_string());

        let difficulty = data
            .current_task
            .as_ref()
            .map(|t| format!("Difficulty: {:.1}", t.core_task.difficulty))
            .unwrap_or_else(|| "Difficulty: ...".to_string());

        // Build option buttons vec (same concrete type) and mask when showing feedback by using empty vec
        let option_buttons = data
            .current_task
            .as_ref()
            .map(|t| {
                t.display_options
                    .iter()
                    .enumerate()
                    .map(|(index, option)| {
                        let option_text = option.clone();
                        button(option_text, move |data: &mut AppData| {
                            // only submit if not in feedback state
                            if !data.show_feedback {
                                data.submit_answer(index);
                            }
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| Vec::new());

        let masked_buttons = if data.show_feedback {
            Vec::<_>::new()
        } else {
            option_buttons
        };
        let answers = flex(masked_buttons).direction(Axis::Vertical);

        // Feedback text and color (always a Label)
        let feedback_text = if data.show_feedback {
            if data.last_response_correct {
                "Correct! Well done!".to_string()
            } else {
                // If there is no current_task (rare), show generic message
                let correct_answer = data
                    .current_task
                    .as_ref()
                    .map(|t| t.core_task.correct_answer.clone())
                    .unwrap_or_else(|| "<unknown>".to_string());
                format!("Incorrect. The correct answer was: {}", correct_answer)
            }
        } else {
            "".to_string()
        };

        let feedback_color = if data.show_feedback {
            if data.last_response_correct {
                Color::from_rgb8(0, 200, 0)
            } else {
                Color::from_rgb8(255, 0, 0)
            }
        } else {
            Color::from_rgb8(128, 128, 128)
        };

        let feedback_label = label(feedback_text)
            .brush(feedback_color)
            .alignment(TextAlignment::Middle);

        // Continue and hint buttons are always Buttons; their handlers are no-ops when not appropriate
        let continue_button = button("Continue", |data: &mut AppData| {
            if data.show_feedback {
                data.continue_to_next_task();
            }
        });

        let get_hint_button = button("Get Hint", |data: &mut AppData| {
            if !data.show_feedback && data.enable_hints {
                data.request_hint();
            }
        });

        // Hint card always present (Label inside a card). Empty when no hint.
        let hint_text = if !data.show_feedback {
            data.current_hint.clone().unwrap_or_default()
        } else {
            String::new()
        };
        let hint_card =
            card::<AppData, _>("Hint", label(hint_text).alignment(TextAlignment::Middle));

        // Combine into a single Interaction card (same concrete types regardless of state)
        let interaction = card::<AppData, _>(
            "Interaction",
            flex((
                feedback_label,
                continue_button,
                answers,
                get_hint_button,
                hint_card,
            ))
            .direction(Axis::Vertical),
        );

        // Final Current Task card: prompt, difficulty, interaction (same shape in both branches)
        card(
            "Current Task",
            flex((
                label(prompt).alignment(TextAlignment::Middle),
                label(difficulty)
                    .brush(Color::from_rgb8(128, 128, 128))
                    .alignment(TextAlignment::Middle),
                interaction,
            ))
            .direction(Axis::Vertical),
        )
    };

    // Learner metrics display (if available)
    let metrics_display = label(
        if let Some(core_metrics) = &data.current_metrics.core_metrics {
            format!(
                "Bidirectionality: {:.3} | Distance: {:.3} | Chunk Penalty: {:.3}",
                core_metrics.bidirectionality_index,
                core_metrics.symbolic_distance_slope,
                core_metrics.chunk_boundary_penalty
            )
        } else {
            "Metrics will appear after a few responses".to_string()
        },
    )
    .alignment(TextAlignment::Middle);

    flex((
        label("Training Session")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        progress_display,
        task_display,
        metrics_display,
        flex((
            button("Pause Session", |data: &mut AppData| {
                data.current_screen = Screen::Dashboard;
                data.success_message = Some("Session paused. You can resume anytime.".to_string());
            }),
            button("End Session", |data: &mut AppData| {
                data.end_session();
            }),
        ))
        .direction(Axis::Horizontal),
    ))
    .direction(Axis::Vertical)
}

// Performance Dashboard Screen
pub fn dashboard_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    // Overall performance metrics
    let overall_metrics = card(
        "Overall Performance",
        flex((
            metric_display(
                "Total Tasks:",
                data.current_metrics.total_responses.to_string(),
                Color::from_rgb8(0, 128, 255),
            ),
            metric_display(
                "Accuracy:",
                format!("{:.1}%", data.current_metrics.accuracy_rate * 100.0),
                Color::from_rgb8(0, 200, 0),
            ),
            metric_display(
                "Avg Response Time:",
                format!("{:.0}ms", data.current_metrics.average_response_time_ms),
                Color::from_rgb8(255, 128, 0),
            ),
            metric_display(
                "Best Streak:",
                data.current_metrics.best_streak.to_string(),
                Color::from_rgb8(128, 0, 255),
            ),
        ))
        .direction(Axis::Vertical),
    );

    // Learning Analytics card with unified content using option slots
    let learning_analytics = {
        let core = data.current_metrics.core_metrics.as_ref();
        let metrics_block = core.map(|m| {
            flex((
                label("Cognitive Metrics")
                    .brush(Color::from_rgb8(64, 64, 64))
                    .alignment(TextAlignment::Start),
                metric_display(
                    "Bidirectionality Index:",
                    format!("{:.3}", m.bidirectionality_index),
                    Color::from_rgb8(0, 128, 255),
                ),
                prose("Measures how well you can navigate forward and backward in the sequence")
                    .alignment(TextAlignment::Start),
                metric_display(
                    "Symbolic Distance Slope:",
                    format!("{:.3}", m.symbolic_distance_slope),
                    Color::from_rgb8(128, 0, 255),
                ),
                prose("Shows how distance affects your performance on comparison tasks")
                    .alignment(TextAlignment::Start),
                metric_display(
                    "Chunk Boundary Penalty:",
                    format!("{:.3}", m.chunk_boundary_penalty),
                    Color::from_rgb8(255, 128, 0),
                ),
                prose("Indicates difficulty crossing conceptual boundaries in the domain")
                    .alignment(TextAlignment::Start),
            ))
            .direction(Axis::Vertical)
        });

        let placeholder = if core.is_none() {
            Some(
                label("Complete more tasks to see detailed analytics")
                    .alignment(TextAlignment::Middle),
            )
        } else {
            None
        };

        card(
            "Learning Analytics",
            flex((metrics_block, placeholder)).direction(Axis::Vertical),
        )
    };

    // Operation Proficiencies unified via option slots
    let proficiencies = {
        let profs_block = data.current_learner.as_ref().map(|learner| {
            let profs = &learner.core_model.operation_proficiencies;
            let prof_displays = profs
                .values()
                .take(5)
                .map(|prof| {
                    let proficiency = (prof.theta + 2.0) / 4.0; // Normalize from [-2, 2] to [0, 1]
                    progress_bar(
                        proficiency.min(1.0).max(0.0),
                        format!("{:?} ({}x)", prof.operation, prof.practice_count),
                    )
                })
                .collect::<Vec<_>>();

            flex(prof_displays).direction(Axis::Vertical)
        });

        let no_data = if data.current_learner.is_none() {
            Some(label("No proficiency data yet").alignment(TextAlignment::Middle))
        } else {
            None
        };

        card(
            "Operation Proficiencies",
            flex((profs_block, no_data)).direction(Axis::Vertical),
        )
    };

    // Recent activity unified into one card
    let recent_display = {
        let recent_responses = data
            .session_responses
            .iter()
            .rev()
            .take(5)
            .map(|response| {
                let status = if response.correct { "✓" } else { "✗" };
                let color = if response.correct {
                    Color::from_rgb8(0, 200, 0)
                } else {
                    Color::from_rgb8(255, 0, 0)
                };

                label(format!(
                    "{} {} - {}ms",
                    status,
                    response.task.prompt.chars().take(30).collect::<String>(),
                    response.response_time_ms
                ))
                .brush(color)
                .alignment(TextAlignment::Start)
            })
            .collect::<Vec<_>>();

        let list_block = if recent_responses.is_empty() {
            None
        } else {
            Some(flex(recent_responses).direction(Axis::Vertical))
        };

        let empty_block = if list_block.is_none() {
            Some(label("No responses yet").alignment(TextAlignment::Middle))
        } else {
            None
        };

        card(
            "Recent Activity",
            flex((list_block, empty_block)).direction(Axis::Vertical),
        )
    };

    // Advanced statistics unified
    let advanced_stats = {
        let enough = data.session_responses.len() > 5;

        let hist = if enough {
            Some(response_time_histogram(
                &data
                    .session_responses
                    .iter()
                    .map(|r| r.response_time_ms as u128)
                    .collect::<Vec<_>>(),
            ))
        } else {
            None
        };

        let curve = if enough {
            Some(learning_curve_display(&data.session_responses))
        } else {
            None
        };

        let errors = if enough {
            Some(error_analysis_display(&data.session_responses))
        } else {
            None
        };

        let msg = if !enough {
            Some(
                label("Advanced statistics will appear after completing more tasks")
                    .brush(Color::from_rgb8(128, 128, 128)),
            )
        } else {
            None
        };

        flex((hist, curve, errors, msg)).direction(Axis::Horizontal)
    };

    // Strategy placeholder (keep homogeneous types)
    let strategy_placeholder = label("");

    // Controls (always render both buttons for type stability)
    // Controls (always render two buttons for type stability)
    let controls = flex((
        button("Resume Session", |data: &mut AppData| {
            data.current_screen = Screen::Training;
        }),
        button("New Session", |data: &mut AppData| {
            data.current_screen = Screen::DomainSelection;
        }),
    ))
    .direction(Axis::Horizontal);

    flex((
        label("Performance Dashboard").brush(Color::from_rgb8(0, 128, 255)),
        overall_metrics,
        learning_analytics,
        proficiencies,
        strategy_placeholder,
        advanced_stats,
        recent_display,
        controls,
    ))
    .direction(Axis::Vertical)

    /*
    UI_REPORT.md - Roadmap and Recommendations to Evolve the Xilem UI into a Robust Production App

    Overview
    - This small Xilem-based UI is a compact, cross-platform demonstration of the
      graph-coded adaptive learning system. To evolve it into a production-quality
      cognitive-science application, we need to address architecture, type-safety,
      async API integration, state management, testability, and UX polish.

    Goals
    1. Stable, well-typed UI components that avoid fragile impl-trait mismatches.
    2. Clear separation of view layer, application state, and domain logic.
    3. Reliable integration with backend model code in `alphabet-terminal-prototype/`.
    4. Deterministic demo mode and replayable session exports for reproducible research.
    5. Testing, CI, and platform-specific build pipelines.

    Immediate UI changes done in the current branch
    - Eliminated mixed-type view tuples by ensuring each flex tuple uses homogeneous view types.
      This prevents impl-trait mismatches and hard-to-debug compilation errors in Xilem.
    - Unify "Interaction" card so the Current Task card has identical concrete view shapes
      in both the live-task and loading states.
    - Added a `demo_showcase()` AppData method (in lib.rs) that performs a short, repeatable
      mock training run to populate the Dashboard for portfolio/demo purposes.
    - Introduced a Select button for domain cards that is always present but no-ops when the
      domain is already selected (keeps types stable while providing the desired UX).

    Recommended refactor roadmap (short to medium term)
    1) View/State Refactor
       - Introduce a small view-model layer: lightweight structs that prepare the UI payloads
         (strings, option flags, vecs of options) so complex closures are localized and the
         views remain simple functions returning `impl WidgetView<State>`.
       - Adopt a `router` module that maps `AppData` -> Screen enum and isolates screen composition.

    2) Componentization
       - Split big screen files into smaller modules: `screens/welcome.rs`, `screens/domain.rs`,
         `screens/training.rs`, `screens/dashboard.rs`, `screens/settings.rs`.
       - Standardize reusable widgets in `components.rs` with consistent signatures:
         e.g., `fn card<T: 'static, V: WidgetView<T> + 'static>(...) -> impl WidgetView<T> + 'static`.

    3) State and Concurrency
       - Move long-running async tasks off the sync UI path: use Xilem's `task` view helper to
         spawn background work and send messages to the main state. Avoid blocking `tokio::Runtime` on the UI thread.
       - Replace `Arc<tokio::runtime::Runtime>` + `block_on` with view-based async tasks so the UI remains responsive.

    4) Backend Integration (API contract)
       - Align UI DTOs with the backend as captured in BACKEND_DEMANDS.md:
         * Session create: accept `topology_type` strings or provide a mapping on the UI.
         * Submit responses: don't send `correct` from the client; backend computes correctness.
         * Add endpoints: list sessions per learner, list responses per session.
       - Add an integration layer `api_client.rs` with typed request/response models, retries, and auth token refresh.

    5) Demo Mode and Reproducibility
       - The `demo_showcase()` should use deterministic randomness (seeded RNG) and log the
         sequence of tasks/responses so the demo is reproducible for reproducibility audits.
       - Add an “Export replay” function that produces a JSON file with:
         * topology, learner init state (seed), sequence of tasks and responses, and final learner metrics.

    6) Testing and CI
       - Unit-test UI helpers where possible (pure functions). For view tests, add snapshot tests
         that render the view into a string representation for regression detection.
       - Add integration tests for the demo flow that call `demo_showcase()` and assert on `AppData` state.
       - Add a GitHub Actions matrix: desktop+headless, mobile cross-compile check, linting, cargo clippy.

    7) Observability and Telemetry
       - Add structured logs for user actions (task start, submit, hint request) and model updates.
       - Provide an optional aggregated telemetry mode (disabled by default) for field trials.

    8) Performance and Scalability
       - Avoid cloning heavyweight core model state in the UI. Keep immutable snapshots and explicit serialization boundaries.
       - Keep the UI reactive: only recompute heavy visualizations (histograms, learning curves) when underlying data changes; memoize expensive computations.

    Long-term architecture suggestions
    - Move core cognitive algorithms to a well-defined library (already present as `alphabet-terminal-prototype`).
      The UI should operate on a thin adapter layer that transforms library outputs into UI DTOs.
    - Separate experimental features behind feature flags: live interventions, A/B scheduler variants, export formats.
    - Implement a plugin-style dashboard for adding new visualizations (graphical charts, interactive plots) without modifying core UI.

    UX improvements to implement next (high priority)
    - Domain selection: make the Select button visibly active, show a confirmation/preview of topology (e.g., small list of nodes).
    - Task interaction: show response latency in the UI; allow explicit 'I don't know' submissions.
    - Hint system: progressive disclosure (small hint -> worked example), with inline animation to show hint provenance.
    - Dashboard: interactive drill-down from aggregate metrics to per-task logs and model snapshots.

    Conclusion
    - The repo is now in a state where the UI components are well-typed and resilient. The next steps
      are componentization, clean async handling, and API contract alignment with the backend.
    - Once the above is in place, this cute 6-file prototype can be iteratively hardened into a robust,
      well-factored production UI suitable for cognitive science experiments and demos.
    */
}

// Settings Screen
pub fn settings_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        label("Settings")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        card(
            "Learning Preferences",
            flex((
                checkbox(
                    data.use_adaptive_scheduling,
                    "Adaptive Scheduling",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.use_adaptive_scheduling = checked;
                    }),
                ),
                prose("Uses AI to select optimal tasks based on your learning state")
                    .alignment(TextAlignment::Start),
                checkbox(
                    data.enable_hints,
                    "Enable Hints",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.enable_hints = checked;
                    }),
                ),
                prose("Shows helpful hints when you're struggling with a task")
                    .alignment(TextAlignment::Start),
                label(format!(
                    "Difficulty Level: {:.0}%",
                    data.difficulty_level * 100.0
                ))
                .alignment(TextAlignment::Start),
                prose("Adjusts the baseline difficulty of tasks").alignment(TextAlignment::Start),
            ))
            .direction(Axis::Vertical),
        ),
        {
            let user_block = data.current_user.as_ref().map(|user| {
                flex((
                    metric_display(
                        "Username:",
                        user.username.clone(),
                        Color::from_rgb8(64, 64, 64),
                    ),
                    metric_display("Email:", user.email.clone(), Color::from_rgb8(64, 64, 64)),
                ))
                .direction(Axis::Vertical)
            });

            let guest_block = if data.current_user.is_none() {
                Some(label("Guest User").alignment(TextAlignment::Start))
            } else {
                None
            };

            card(
                "Account",
                flex((user_block, guest_block)).direction(Axis::Vertical),
            )
        },
        card(
            "Data Export",
            flex((
                prose("Export your learning data for analysis or backup")
                    .alignment(TextAlignment::Start),
                button("Export as JSON", |data: &mut AppData| {
                    data.export_current_data();
                    if let Some(export) = &data.export_data {
                        _ = serde_json::to_string_pretty(export).unwrap_or_default();
                        // In a real app, save to file
                        data.success_message = Some("Data exported as JSON".to_string());
                    }
                }),
            ))
            .direction(Axis::Vertical),
        ),
        flex((
            button("Back to Dashboard", |data: &mut AppData| {
                data.current_screen = Screen::Dashboard;
            }),
            button("Settings", |data: &mut AppData| {
                data.current_screen = Screen::Settings;
            }),
        ))
        .direction(Axis::Horizontal),
    ))
    .direction(Axis::Vertical)
}
