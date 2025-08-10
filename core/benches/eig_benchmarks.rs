use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use abcdeez_core::bayesian::BayesianLearnerModel;
use abcdeez_core::learner::OperationType;
use abcdeez_core::tasks::{Task, TaskType};
use abcdeez_core::topology::Topology;

fn benchmark_eig_calculation(c: &mut Criterion) {
    let model = BayesianLearnerModel::new(&Topology::alphabet());
    let task = Task {
        task_type: TaskType::Successor {
            item: "A".to_string(),
        },
        prompt: "What comes after A?".to_string(),
        correct_answer: "B".to_string(),
        options: vec!["B".to_string(), "C".to_string()],
        difficulty: 0.5,
        operation: OperationType::Successor,
    };

    c.bench_function("EIG calculation (1000 samples)", |b| {
        b.iter(|| model.monte_carlo_eig(black_box(&task), black_box(1000)));
    });
}

fn benchmark_eig_different_sample_sizes(c: &mut Criterion) {
    let model = BayesianLearnerModel::new(&Topology::alphabet());
    let task = Task {
        task_type: TaskType::PairwiseOrder {
            a: "A".to_string(),
            b: "B".to_string(),
        },
        prompt: "Which comes first: A or B?".to_string(),
        correct_answer: "A".to_string(),
        options: vec!["A".to_string(), "B".to_string()],
        difficulty: 0.3,
        operation: OperationType::PairwiseOrder,
    };

    let mut group = c.benchmark_group("EIG_sample_sizes");
    for sample_size in [100, 500, 1000, 2000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(sample_size),
            sample_size,
            |b, &sample_size| {
                b.iter(|| model.monte_carlo_eig(black_box(&task), black_box(sample_size)));
            },
        );
    }
    group.finish();
}

fn benchmark_bayesian_update(c: &mut Criterion) {
    let mut model = BayesianLearnerModel::new(&Topology::alphabet());
    let response = abcdeez_core::bayesian::ResponseData {
        task: Task {
            task_type: TaskType::Successor {
                item: "C".to_string(),
            },
            prompt: "What comes after C?".to_string(),
            correct_answer: "D".to_string(),
            options: vec!["D".to_string(), "E".to_string()],
            difficulty: 0.4,
            operation: OperationType::Successor,
        },
        correct: true,
        response_time: 1250.0,
    };

    c.bench_function("Bayesian update", |b| {
        b.iter(|| model.update_with_response(black_box(response.clone())));
    });
}

fn benchmark_adaptive_monte_carlo(c: &mut Criterion) {
    let model = BayesianLearnerModel::new(&Topology::alphabet());
    let task = Task {
        task_type: TaskType::KJump {
            start: "E".to_string(),
            k: 3,
        },
        prompt: "Starting from E, what is 3 steps forward?".to_string(),
        correct_answer: "H".to_string(),
        options: vec![
            "F".to_string(),
            "G".to_string(),
            "H".to_string(),
            "I".to_string(),
        ],
        difficulty: 0.7,
        operation: OperationType::KJump(3),
    };

    c.bench_function("Adaptive Monte Carlo EIG", |b| {
        b.iter(|| model.adaptive_monte_carlo_eig(black_box(&task)));
    });
}

fn benchmark_topology_operations(c: &mut Criterion) {
    let topo = Topology::alphabet();

    c.bench_function("Distance calculation", |b| {
        b.iter(|| topo.get_distance(black_box("A"), black_box("Z")));
    });

    c.bench_function("Successor lookup", |b| {
        b.iter(|| topo.get_successor(black_box("M")));
    });

    c.bench_function("K-jump calculation", |b| {
        b.iter(|| topo.get_k_jump(black_box("A"), black_box(5)));
    });
}

fn benchmark_large_topology_performance(c: &mut Criterion) {
    // Create a large linear topology
    let large_items: Vec<String> = (0..1000).map(|i| format!("item_{:04}", i)).collect();
    let large_topo = Topology::new_linear(large_items);

    let model = BayesianLearnerModel::new(&large_topo);
    let task = Task {
        task_type: TaskType::PairwiseOrder {
            a: "item_0100".to_string(),
            b: "item_0800".to_string(),
        },
        prompt: "Which comes first?".to_string(),
        correct_answer: "item_0100".to_string(),
        options: vec!["item_0100".to_string(), "item_0800".to_string()],
        difficulty: 0.2,
        operation: OperationType::PairwiseOrder,
    };

    c.bench_function("EIG on large topology", |b| {
        b.iter(|| model.monte_carlo_eig(black_box(&task), black_box(100)));
    });

    c.bench_function("Distance on large topology", |b| {
        b.iter(|| large_topo.get_distance(black_box("item_0100"), black_box("item_0800")));
    });
}

fn benchmark_performance_requirements(c: &mut Criterion) {
    let model = BayesianLearnerModel::new(&Topology::alphabet());
    let task = Task {
        task_type: TaskType::Index {
            item: "M".to_string(),
        },
        prompt: "What is the position of M?".to_string(),
        correct_answer: "13".to_string(),
        options: vec!["12".to_string(), "13".to_string(), "14".to_string()],
        difficulty: 0.6,
        operation: OperationType::Index,
    };

    // Performance requirement: 10 EIG calculations should complete in under 1 second
    c.bench_function("10 EIG calculations", |b| {
        b.iter(|| {
            for _ in 0..10 {
                model.monte_carlo_eig(black_box(&task), black_box(1000));
            }
        });
    });
}

fn benchmark_concurrent_performance(c: &mut Criterion) {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let model = Arc::new(Mutex::new(BayesianLearnerModel::new(&Topology::alphabet())));
    let task = Arc::new(Task {
        task_type: TaskType::Segment {
            start: "D".to_string(),
            count: 4,
            reverse: false,
        },
        prompt: "List 4 items starting from D".to_string(),
        correct_answer: "D E F G".to_string(),
        options: vec!["D E F G".to_string(), "D C B A".to_string()],
        difficulty: 0.5,
        operation: OperationType::Segment(4, false),
    });

    c.bench_function("Concurrent EIG calculations (4 threads)", |b| {
        b.iter(|| {
            let mut handles = vec![];

            for _ in 0..4 {
                let model_clone = Arc::clone(&model);
                let task_clone = Arc::clone(&task);

                let handle = thread::spawn(move || {
                    let model_ref = model_clone.lock().unwrap();
                    model_ref.monte_carlo_eig(&task_clone, 500)
                });

                handles.push(handle);
            }

            for handle in handles {
                let _ = handle.join().unwrap();
            }
        });
    });
}

criterion_group!(
    benches,
    benchmark_eig_calculation,
    benchmark_eig_different_sample_sizes,
    benchmark_bayesian_update,
    benchmark_adaptive_monte_carlo,
    benchmark_topology_operations,
    benchmark_large_topology_performance,
    benchmark_performance_requirements,
    benchmark_concurrent_performance
);
criterion_main!(benches);
