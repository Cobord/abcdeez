// Integration tests for the demo showcase flow
// These tests ensure the demo runs deterministically and produces expected results

use graph_learning_app::{AppData, Screen};

#[test]
fn test_demo_showcase_completes() {
    // Create app data
    let mut app_data = AppData::default();

    // Run demo showcase
    app_data.demo_showcase();

    // Verify demo created a user
    assert!(app_data.current_user.is_some(), "Demo should create a user");
    let user = app_data.current_user.as_ref().unwrap();
    assert_eq!(user.username, "DemoUser");

    // Verify demo created a learner
    assert!(
        app_data.current_learner.is_some(),
        "Demo should create a learner"
    );

    // Verify demo created session responses
    assert!(
        !app_data.session_responses.is_empty(),
        "Demo should generate responses"
    );
    assert_eq!(
        app_data.session_responses.len(),
        5,
        "Demo should generate exactly 5 responses"
    );

    // Verify metrics were updated
    assert!(
        app_data.current_metrics.total_responses > 0,
        "Metrics should be updated"
    );
    assert!(
        app_data.current_metrics.average_response_time_ms > 0.0,
        "Response times should be recorded"
    );

    // Verify session was ended
    assert!(app_data.current_session.is_some(), "Session should exist");
    let session = app_data.current_session.as_ref().unwrap();
    assert_eq!(session.status, "completed", "Session should be completed");
    assert!(session.end_time.is_some(), "Session should have end time");
    assert!(session.summary.is_some(), "Session should have summary");

    // Verify guided demo was started
    assert!(app_data.demo_active, "Guided demo should be active");
    assert_eq!(app_data.demo_step, 0, "Demo should start at step 0");

    // Verify we're on dashboard
    assert_eq!(
        app_data.current_screen,
        Screen::Dashboard,
        "Should navigate to dashboard"
    );
}

#[test]
fn test_demo_showcase_accuracy_calculation() {
    let mut app_data = AppData::default();
    app_data.demo_showcase();

    // Check that accuracy is calculated correctly
    let correct_count = app_data
        .session_responses
        .iter()
        .filter(|r| r.correct)
        .count();
    let total_count = app_data.session_responses.len();
    let expected_accuracy = if total_count > 0 {
        correct_count as f64 / total_count as f64
    } else {
        0.0
    };

    assert!(
        (app_data.current_metrics.accuracy_rate - expected_accuracy).abs() < 0.001,
        "Accuracy should be calculated correctly"
    );
}

#[test]
fn test_guided_demo_navigation() {
    let mut app_data = AppData::default();

    // Start guided demo
    app_data.demo_start();
    assert!(app_data.demo_active);
    assert_eq!(app_data.demo_step, 0);
    assert_eq!(app_data.current_screen, Screen::Dashboard);

    // Navigate through steps
    let initial_text = app_data.demo_step_text();
    assert!(initial_text.contains("Welcome to the Adaptive Learning System"));

    app_data.demo_next_step();
    assert_eq!(app_data.demo_step, 1);
    let step1_text = app_data.demo_step_text();
    assert!(step1_text.contains("Top bar"));

    // Test advancing to the end
    for _ in 0..10 {
        app_data.demo_next_step();
    }
    assert_eq!(app_data.demo_step, 6, "Demo step should be clamped at max");

    // End demo
    app_data.demo_end();
    assert!(!app_data.demo_active);
    assert_eq!(app_data.current_screen, Screen::Dashboard);
}

#[test]
fn test_session_lifecycle() {
    let mut app_data = AppData::default();

    // Create a user and learner first
    use graph_learning_app::models::User;
    app_data.current_user = Some(User {
        id: uuid::Uuid::new_v4().to_string(),
        username: "TestUser".to_string(),
        email: "test@example.com".to_string(),
        password_hash: String::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    });

    // Create learner
    app_data.create_learner();
    assert!(app_data.current_learner.is_some());

    // Start session using the sync version since tests don't process async flags
    app_data.start_session_sync();
    assert!(app_data.current_session.is_some());
    assert!(app_data.task_generator.is_some());
    assert!(app_data.adaptive_scheduler.is_some());
    assert!(app_data.intervention_system.is_some());

    // Generate and submit some tasks
    for i in 0..3 {
        app_data.generate_next_task();
        assert!(
            app_data.current_task.is_some(),
            "Task {} should be generated",
            i
        );

        // Submit answer (index 0)
        app_data.submit_answer(0);

        // Verify response was recorded
        assert_eq!(app_data.session_responses.len(), i + 1);

        // Clear feedback state to allow next submission
        app_data.show_feedback = false;
        app_data.submit_response_in_flight = false;
    }

    // End session - need to process it synchronously for tests
    // In real app flow, this would be async via end_session_in_flight flag
    if let Some(session) = &mut app_data.current_session {
        session.end_time = Some(chrono::Utc::now());
        session.status = "completed".to_string();
    }

    // Verify session state
    assert!(app_data.current_session.is_some());
    let session = app_data.current_session.as_ref().unwrap();
    assert_eq!(session.responses.len(), 3);
}

#[test]
fn test_domain_selection() {
    use graph_learning_app::models::Domain;

    let mut app_data = AppData::default();

    // Test each domain creates appropriate topology
    let domains = vec![
        Domain::Alphabet,
        Domain::DaysOfWeek,
        Domain::Music,
        Domain::Mathematics,
    ];

    for domain in domains {
        app_data.selected_domain = domain.clone();
        app_data.create_learner();

        assert!(app_data.topology.is_some());
        let topology = app_data.topology.as_ref().unwrap();

        match domain {
            Domain::Alphabet => {
                assert_eq!(topology.nodes.len(), 26, "Alphabet should have 26 nodes");
            }
            Domain::DaysOfWeek => {
                assert_eq!(topology.nodes.len(), 7, "Week should have 7 days");
            }
            Domain::Music => {
                assert_eq!(topology.nodes.len(), 7, "Music should have 7 notes");
            }
            Domain::Mathematics => {
                assert_eq!(topology.nodes.len(), 20, "Math should have 20 numbers");
            }
            _ => {}
        }
    }
}

#[test]
fn test_hint_system() {
    let mut app_data = AppData::default();

    // Setup for hint testing
    use graph_learning_app::models::User;
    app_data.current_user = Some(User {
        id: uuid::Uuid::new_v4().to_string(),
        username: "HintTestUser".to_string(),
        email: "hint@test.com".to_string(),
        password_hash: String::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    });

    app_data.enable_hints = true;
    app_data.create_learner();
    app_data.start_session();
    app_data.generate_next_task();

    // Request hint
    app_data.request_hint();

    // Hints might not always be provided (depends on intervention system logic)
    // but the system should not crash
    assert!(app_data.current_task.is_some());
}

#[test]
fn test_performance_metrics_update() {
    let mut app_data = AppData::default();

    // Test metric updates
    app_data.current_metrics.update(true, 100);
    assert_eq!(app_data.current_metrics.total_responses, 1);
    assert_eq!(app_data.current_metrics.correct_responses, 1);
    assert_eq!(app_data.current_metrics.streak_count, 1);
    assert_eq!(app_data.current_metrics.accuracy_rate, 1.0);

    app_data.current_metrics.update(false, 150);
    assert_eq!(app_data.current_metrics.total_responses, 2);
    assert_eq!(app_data.current_metrics.correct_responses, 1);
    assert_eq!(app_data.current_metrics.streak_count, 0);
    assert_eq!(app_data.current_metrics.accuracy_rate, 0.5);

    app_data.current_metrics.update(true, 120);
    app_data.current_metrics.update(true, 130);
    assert_eq!(app_data.current_metrics.streak_count, 2);
    assert_eq!(app_data.current_metrics.best_streak, 2);

    // Check average response time
    let expected_avg = (100.0 + 150.0 + 120.0 + 130.0) / 4.0;
    assert!((app_data.current_metrics.average_response_time_ms - expected_avg).abs() < 0.01);
}

#[test]
fn test_export_data_creation() {
    let mut app_data = AppData::default();

    // Setup state for export
    app_data.demo_showcase();

    // Trigger export
    app_data.export_current_data();

    // Verify export data was created
    assert!(app_data.export_data.is_some());
    let export = app_data.export_data.as_ref().unwrap();

    assert!(export.learner.id == app_data.current_learner.as_ref().unwrap().id);
    assert!(!export.sessions.is_empty());
    assert_eq!(
        export.metrics.total_responses,
        app_data.current_metrics.total_responses
    );
}

#[test]
fn test_demo_showcase_deterministic() {
    // Run demo twice with same seed and verify identical results
    let seed = 42u64;

    // First run
    let mut app_data1 = AppData::default();
    app_data1.demo_showcase_with_seed(Some(seed));

    // Second run with same seed
    let mut app_data2 = AppData::default();
    app_data2.demo_showcase_with_seed(Some(seed));

    // Verify both runs produced identical results
    assert_eq!(
        app_data1.session_responses.len(),
        app_data2.session_responses.len(),
        "Same seed should produce same number of responses"
    );

    // Check that the correctness patterns are identical
    // Note: We can't guarantee identical task types since TaskGenerator doesn't use seeded RNG,
    // but the answer patterns should be deterministic based on the seed
    let pattern1: Vec<bool> = app_data1
        .session_responses
        .iter()
        .map(|r| r.correct)
        .collect();
    let pattern2: Vec<bool> = app_data2
        .session_responses
        .iter()
        .map(|r| r.correct)
        .collect();

    assert_eq!(
        pattern1, pattern2,
        "Response correctness patterns should match with same seed"
    );

    // Verify metrics are identical
    assert_eq!(
        app_data1.current_metrics.correct_responses, app_data2.current_metrics.correct_responses,
        "Correct response count should be identical"
    );
    assert_eq!(
        app_data1.current_metrics.accuracy_rate, app_data2.current_metrics.accuracy_rate,
        "Accuracy rate should be identical"
    );

    // Verify demo seeds were stored
    assert_eq!(app_data1.demo_seed, Some(seed));
    assert_eq!(app_data2.demo_seed, Some(seed));
}

#[test]
fn test_demo_showcase_different_seeds() {
    // Run demo with different seeds and verify different results
    let mut app_data1 = AppData::default();
    app_data1.demo_showcase_with_seed(Some(100));

    let mut app_data2 = AppData::default();
    app_data2.demo_showcase_with_seed(Some(200));

    // While the structure should be the same...
    assert_eq!(
        app_data1.session_responses.len(),
        app_data2.session_responses.len(),
        "Different seeds should still produce same number of responses"
    );

    // ...the correctness patterns should likely differ
    let pattern1: Vec<bool> = app_data1
        .session_responses
        .iter()
        .map(|r| r.correct)
        .collect();
    let pattern2: Vec<bool> = app_data2
        .session_responses
        .iter()
        .map(|r| r.correct)
        .collect();

    // With different seeds, we expect at least some differences
    // (there's a small chance they could be identical by coincidence, but unlikely)
    let differences = pattern1
        .iter()
        .zip(pattern2.iter())
        .filter(|(a, b)| a != b)
        .count();

    assert!(
        differences > 0,
        "Different seeds should produce different response patterns (found {} differences)",
        differences
    );
}
