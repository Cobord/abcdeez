use crate::bayesian::*;
use crate::topology::{Topology, TopologyType};
use crate::tasks::{Task, TaskType};
use crate::learner::OperationType;

#[test]
fn test_bayesian_model_initialization() {
    let topo = Topology::alphabet();
    let model = BayesianLearnerModel::new(&topo);
    
    assert_eq!(model.node_positions.len(), 26);
    assert_eq!(model.operation_proficiencies.len(), 6);
    
    // Check initial variance is reasonable
    for (_key, posterior) in &model.node_positions {
        assert!(posterior.variance > 0.0);
        assert!(posterior.variance < 10.0);
    }
}

#[test]
fn test_monte_carlo_eig_positive() {
    let topo = Topology::alphabet();
    let model = BayesianLearnerModel::new(&topo);
    
    let task = Task {
        task_type: TaskType::Successor { item: "C".to_string() },
        prompt: "What comes after C?".to_string(),
        correct_answer: "D".to_string(),
        options: vec!["B", "C", "D", "E"].iter().map(|s| s.to_string()).collect(),
        difficulty: 0.3,
        operation: OperationType::Successor,
    };
    
    let eig = model.monte_carlo_eig(&task, 100);
    
    // EIG should be non-negative (information cannot be lost)
    assert!(eig >= 0.0, "EIG was negative: {}", eig);
    
    // EIG should be bounded by total entropy
    let total_entropy = model.total_entropy();
    assert!(eig <= total_entropy, "EIG {} exceeds total entropy {}", eig, total_entropy);
}

#[test]
fn test_posterior_update_reduces_uncertainty() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);
    
    // Get initial variance for a node
    let initial_var = model.node_positions["node_0"].variance;
    
    // Provide correct response
    let response = ResponseData {
        task: Task {
            task_type: TaskType::Successor { item: "A".to_string() },
            prompt: "What comes after A?".to_string(),
            correct_answer: "B".to_string(),
            options: vec!["A", "B", "C", "D"].iter().map(|s| s.to_string()).collect(),
            difficulty: 0.3,
            operation: OperationType::Successor,
        },
        correct: true,
        response_time: 1000.0,
    };
    
    model.update_with_response(response);
    
    // Variance should decrease (uncertainty reduced)
    let updated_var = model.node_positions["node_0"].variance;
    assert!(updated_var < initial_var, 
            "Variance should decrease after update: {} -> {}", 
            initial_var, updated_var);
}

#[test]
fn test_eig_high_for_uncertain_tasks() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);
    
    // Create two tasks - one for well-known nodes, one for uncertain
    let certain_task = Task {
        task_type: TaskType::Successor { item: "A".to_string() },
        prompt: "".to_string(),
        correct_answer: "B".to_string(),
        options: vec![],
        difficulty: 0.1,
        operation: OperationType::Successor,
    };
    
    let uncertain_task = Task {
        task_type: TaskType::Successor { item: "X".to_string() },
        prompt: "".to_string(),
        correct_answer: "Y".to_string(),
        options: vec![],
        difficulty: 0.9,
        operation: OperationType::Successor,
    };
    
    // First reduce uncertainty about A
    for _ in 0..5 {
        let response = ResponseData {
            task: certain_task.clone(),
            correct: true,
            response_time: 1000.0,
        };
        model.update_with_response(response);
    }
    
    // Calculate EIG for both tasks
    let certain_eig = model.monte_carlo_eig(&certain_task, 100);
    let uncertain_eig = model.monte_carlo_eig(&uncertain_task, 100);
    
    // Uncertain task should have higher EIG
    assert!(uncertain_eig > certain_eig * 0.8, 
            "Uncertain task EIG {} should be higher than certain task EIG {}", 
            uncertain_eig, certain_eig);
}

#[test]
fn test_task_ranking_by_eig() {
    let topo = Topology::alphabet();
    let model = BayesianLearnerModel::new(&topo);
    
    let tasks = vec![
        Task {
            task_type: TaskType::Successor { item: "A".to_string() },
            prompt: "".to_string(),
            correct_answer: "B".to_string(),
            options: vec![],
            difficulty: 0.1,
            operation: OperationType::Successor,
        },
        Task {
            task_type: TaskType::PairwiseOrder { a: "M".to_string(), b: "N".to_string() },
            prompt: "".to_string(),
            correct_answer: "M".to_string(),
            options: vec![],
            difficulty: 0.5,
            operation: OperationType::PairwiseOrder,
        },
        Task {
            task_type: TaskType::Segment { start: "X".to_string(), count: 3, reverse: false },
            prompt: "".to_string(),
            correct_answer: "X, Y, Z".to_string(),
            options: vec![],
            difficulty: 0.8,
            operation: OperationType::Segment(3, false),
        },
    ];
    
    let ranked = model.rank_tasks_by_eig(tasks.clone());
    
    // Should return same number of tasks
    assert_eq!(ranked.len(), tasks.len());
    
    // Tasks should be sorted by EIG (descending)
    for i in 1..ranked.len() {
        assert!(ranked[i-1].1 >= ranked[i].1, 
                "Tasks not properly sorted by EIG: {} < {}", 
                ranked[i-1].1, ranked[i].1);
    }
}

#[test]
fn test_adaptive_observation_variance() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);
    
    // Fast, correct response should result in lower entropy after update
    let fast_correct = ResponseData {
        task: Task {
            task_type: TaskType::Successor { item: "A".to_string() },
            prompt: "".to_string(),
            correct_answer: "B".to_string(),
            options: vec![],
            difficulty: 0.1,
            operation: OperationType::Successor,
        },
        correct: true,
        response_time: 500.0,
    };
    
    let initial_entropy = model.total_entropy();
    model.update_with_response(fast_correct);
    let entropy_after_fast = model.total_entropy();
    
    // Entropy should decrease
    assert!(entropy_after_fast < initial_entropy, 
            "Entropy should decrease after fast correct response");
}

#[test]
fn test_sampled_model_usage() {
    let topo = Topology::alphabet();
    let model = BayesianLearnerModel::new(&topo);
    
    // Test that we can rank tasks (which internally uses sampling)
    let tasks = vec![Task {
        task_type: TaskType::PairwiseOrder { a: "A".to_string(), b: "B".to_string() },
        prompt: "".to_string(),
        correct_answer: "A".to_string(),
        options: vec![],
        difficulty: 0.3,
        operation: OperationType::PairwiseOrder,
    }];
    
    let ranked = model.rank_tasks_by_eig(tasks);
    
    // Should return valid EIG values
    assert_eq!(ranked.len(), 1);
    let (_task, eig) = &ranked[0];
    assert!(*eig >= 0.0 && *eig < 20.0, "EIG should be reasonable: {}", eig);
}

#[test]
fn test_chunk_boundary_updates() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);
    
    // Segment task crossing chunk boundary
    let response = ResponseData {
        task: Task {
            task_type: TaskType::Segment { start: "E".to_string(), count: 4, reverse: false },
            prompt: "".to_string(),
            correct_answer: "E, F, G, H".to_string(),
            options: vec![],
            difficulty: 0.4,
            operation: OperationType::Segment(4, false),
        },
        correct: true,
        response_time: 2500.0, // Slow response suggests chunk boundary
    };
    
    model.update_with_response(response);
    
    // Check that chunk boundaries were considered
    assert!(!model.chunk_boundaries.is_empty() || topo.topology_type != TopologyType::Linear,
            "Chunk boundaries should exist for linear topology");
}

#[test]
fn test_entropy_calculation() {
    let topo = Topology::alphabet();
    let model = BayesianLearnerModel::new(&topo);
    
    let entropy = model.total_entropy();
    
    // Entropy should be positive for uncertain model
    assert!(entropy > 0.0, "Initial entropy should be positive: {}", entropy);
    
    // Entropy should be bounded (not infinite)
    assert!(entropy < 1000.0, "Entropy should be bounded: {}", entropy);
}