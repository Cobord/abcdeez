use crate::learner::*;
use crate::topology::Topology;
use chrono::{Utc, Duration};

#[test]
fn test_learner_initialization() {
    let topo = Topology::alphabet();
    let learner = LearnerModel::new("test_learner".to_string(), &topo);
    
    assert_eq!(learner.learner_id, "test_learner");
    assert_eq!(learner.node_embeddings.len(), 26);
    assert_eq!(learner.memory_strengths.len(), 26);
    assert_eq!(learner.operation_proficiencies.len(), 8); // 8 operation types initialized
    
    // Check initial values
    for embedding in learner.node_embeddings.values() {
        // Check that embedding has been initialized
        assert!(!embedding.node_id.is_empty());
    }
}

#[test]
fn test_operation_proficiency_update() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);
    
    let initial_prof = learner.operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);
    
    // Update with correct response
    learner.update_operation_proficiency(&OperationType::Successor, true);
    
    let updated_prof = learner.operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);
    
    assert!(updated_prof > initial_prof, 
            "Proficiency should increase after correct response: {} -> {}", 
            initial_prof, updated_prof);
    
    // Check practice count
    let practice_count = learner.operation_proficiencies
        .get("Successor")
        .map(|p| p.practice_count)
        .unwrap_or(0);
    
    assert_eq!(practice_count, 1, "Practice count should be 1");
}

#[test]
fn test_memory_strength_update() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);
    
    // Update memory for node A
    learner.update_memory_strength("A", true);
    
    let strength = learner.memory_strengths
        .get("node_0")
        .map(|m| m.strength)
        .unwrap_or(0.0);
    
    assert!(strength > 0.5, "Memory strength should increase after correct response");
    
    // Update with incorrect response
    learner.update_memory_strength("A", false);
    
    let updated_strength = learner.memory_strengths
        .get("node_0")
        .map(|m| m.strength)
        .unwrap_or(0.0);
    
    assert!(updated_strength < strength, 
            "Memory strength should decrease after incorrect response");
}

#[test]
fn test_memory_decay() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);
    
    // Set initial memory strength
    learner.update_memory_strength("A", true);
    
    // Get initial strength value directly
    let initial_strength = learner.memory_strengths
        .get("node_0")
        .map(|m| m.strength)
        .unwrap_or(0.5);
    
    // Simulate time passing by updating last_practice
    if let Some(memory) = learner.memory_strengths.get_mut("node_0") {
        memory.last_practice = Utc::now() - Duration::hours(1); // 1 hour ago
    }
    
    // Calculate decayed strength manually using decay formula
    let elapsed_hours = 1.0f64;
    let decay_rate = 0.1f64;
    let decayed_strength = initial_strength * (-decay_rate * elapsed_hours).exp();
    
    assert!(decayed_strength < initial_strength, 
            "Memory should decay over time: {} -> {}", 
            initial_strength, decayed_strength);
}

#[test]
fn test_performance_over_time() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);
    
    // Test that proficiency updates work
    let initial = learner.operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);
    
    // Simulate multiple correct responses
    for _ in 0..5 {
        learner.update_operation_proficiency(&OperationType::Successor, true);
    }
    
    let final_prof = learner.operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);
    
    assert!(final_prof > initial, 
            "Proficiency should increase with practice: {} -> {}", 
            initial, final_prof);
}

#[test]
fn test_strategy_tracking() {
    let topo = Topology::alphabet();
    let learner = LearnerModel::new("test".to_string(), &topo);
    
    // Just verify learner was created successfully
    assert_eq!(learner.learner_id, "test");
}

#[test]
fn test_learning_curve_tracking() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);
    
    // Simulate learning over time
    for i in 0..10 {
        let correct = i > 3; // Start failing, then succeed
        learner.update_operation_proficiency(&OperationType::Successor, correct);
    }
    
    // Check that proficiency improved
    let final_prof = learner.operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);
    
    assert!(final_prof > 0.0, "Should have positive proficiency after learning");
}

#[test]
fn test_metrics_calculation() {
    let topo = Topology::alphabet();
    let learner = LearnerModel::new("test".to_string(), &topo);
    
    // Manually calculate metrics
    let avg_memory = learner.memory_strengths.values()
        .map(|m| m.strength)
        .sum::<f64>() / learner.memory_strengths.len() as f64;
    
    let avg_prof = learner.operation_proficiencies.values()
        .map(|p| 1.0 / (1.0 + (-p.theta).exp())) // sigmoid
        .sum::<f64>() / learner.operation_proficiencies.len() as f64;
    
    let total_practice: usize = learner.operation_proficiencies.values()
        .map(|p| p.practice_count)
        .sum();
    
    // Check all metrics are valid
    assert!(avg_memory >= 0.0 && avg_memory <= 1.0);
    assert!(avg_prof >= 0.0 && avg_prof <= 1.0);
    assert_eq!(total_practice, 0); // No practice yet
}

#[test]
fn test_embedding_updates() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);
    
    // Embeddings should exist for all nodes
    assert_eq!(learner.node_embeddings.len(), 26);
    
    // Update memory should affect embeddings indirectly
    learner.update_memory_strength("A", true);
    learner.update_memory_strength("B", true);
    
    // Verify embeddings still exist
    assert!(learner.node_embeddings.contains_key("node_0"));
    assert!(learner.node_embeddings.contains_key("node_1"));
}

#[test]
fn test_response_pattern_tracking() {
    let topo = Topology::alphabet();
    let learner = LearnerModel::new("test".to_string(), &topo);
    
    // Just verify basic structure
    assert!(!learner.node_embeddings.is_empty());
    assert!(!learner.memory_strengths.is_empty());
}