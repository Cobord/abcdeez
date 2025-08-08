use crate::adaptive::AdaptiveScheduler;
use crate::learner::{LearnerMetrics, LearnerModel};
use crate::tasks::{TaskSession, TaskType};
use crate::topology::Topology;

pub fn run_demo() {
    println!("\n═══════════════════════════════════════════════════");
    println!("    ADAPTIVE GRAPH-CODED LEARNING SYSTEM DEMO");
    println!("═══════════════════════════════════════════════════\n");

    let topology = Topology::alphabet();
    println!("Created topology: English Alphabet (26 letters)\n");

    let learner_model = LearnerModel::new("demo_user".to_string(), &topology);
    println!("Initialized learner model for: demo_user");

    let mut scheduler = AdaptiveScheduler::new(learner_model, topology.clone());
    let mut session = TaskSession::new(topology.clone());

    println!("\n──────────────────────────────────────────────────");
    println!("Starting adaptive training session...\n");

    for i in 1..=10 {
        println!("Task {}/10", i);
        println!("──────────────────────────────────────────────────");
        
        let task = scheduler.select_next_task();
        println!("Question: {}", task.prompt);
        
        if !task.options.is_empty() {
            println!("Options:");
            for (idx, option) in task.options.iter().enumerate() {
                println!("  [{}] {}", idx + 1, option);
            }
        }
        
        let simulated_answer = if rand::random::<f64>() > 0.3 {
            task.correct_answer.clone()
        } else {
            task.options.get(1).unwrap_or(&"Wrong".to_string()).clone()
        };
        
        println!("Simulated answer: {}", simulated_answer);
        
        session.start_task(Some(task.task_type.clone()));
        let response = session.submit_answer(simulated_answer);
        
        if response.correct {
            println!("✓ CORRECT!");
        } else {
            println!("✗ INCORRECT - Correct answer: {}", response.task.correct_answer);
        }
        
        scheduler.update_model(&response.task, response.correct, response.response_time_ms);
        
        println!();
    }

    println!("\n═══════════════════════════════════════════════════");
    println!("Session Complete - Performance Metrics");
    println!("═══════════════════════════════════════════════════\n");

    let model = scheduler.get_learner_model();
    let metrics = LearnerMetrics::from_model(model);
    let session_stats = session.get_statistics();

    println!("Session Statistics:");
    println!("──────────────────────────────────────────────────");
    println!("Total tasks: {}", session_stats.total_tasks);
    println!("Accuracy: {:.1}%", session_stats.accuracy * 100.0);
    println!();

    println!("Graph-Coded Mastery Metrics:");
    println!("──────────────────────────────────────────────────");
    println!("Bidirectionality Index: {:.3}", metrics.bidirectionality_index);
    println!("  (Lower is better - measures forward/backward asymmetry)");
    println!();
    println!("Symbolic Distance Slope: {:.3}", metrics.symbolic_distance_slope);
    println!("  (Lower is better - indicates direct retrieval vs scanning)");
    println!();
    println!("Chunk Boundary Penalty: {:.3}", metrics.chunk_boundary_penalty);
    println!("  (Lower is better - measures mental segmentation)");
    println!();
    println!("Average Memory Strength: {:.1}%", metrics.avg_memory_strength * 100.0);
    println!();

    println!("Operation Proficiencies:");
    println!("──────────────────────────────────────────────────");
    for (op, prof) in metrics.operation_proficiencies {
        let bar_length = (prof * 20.0) as usize;
        let bar = "█".repeat(bar_length) + &"░".repeat(20 - bar_length);
        println!("{:25} [{}] {:.0}%", op, bar, prof * 100.0);
    }

    println!("\n═══════════════════════════════════════════════════");
    println!("Demo Complete!");
    println!("═══════════════════════════════════════════════════\n");
}

pub fn demonstrate_task_types() {
    println!("\n═══════════════════════════════════════════════════");
    println!("    TASK TYPE DEMONSTRATIONS");
    println!("═══════════════════════════════════════════════════\n");

    let topology = Topology::alphabet();
    let mut generator = crate::tasks::TaskGenerator::new(topology.clone());

    println!("1. Pairwise Order Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::PairwiseOrder {
        a: "D".to_string(),
        b: "K".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("2. Successor Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::Successor {
        item: "M".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("3. Predecessor Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::Predecessor {
        item: "P".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("4. K-Jump Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::KJump {
        start: "E".to_string(),
        k: 3,
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("5. Segment Task (Forward):");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::Segment {
        start: "L".to_string(),
        count: 4,
        reverse: false,
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("6. Segment Task (Reverse):");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::Segment {
        start: "R".to_string(),
        count: 3,
        reverse: true,
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("7. Index Position Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::Index {
        item: "T".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("8. Missing Item Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::MissingItem {
        before: "G".to_string(),
        after: "I".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("9. Shortest Distance Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::ShortestDistance {
        from: "B".to_string(),
        to: "X".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("\n═══════════════════════════════════════════════════");
    println!("Cyclic Topology Example: Days of Week");
    println!("═══════════════════════════════════════════════════\n");

    let cyclic_topology = Topology::days_of_week();
    let mut cyclic_generator = crate::tasks::TaskGenerator::new(cyclic_topology);

    println!("Successor with Wraparound:");
    println!("──────────────────────────────────────────────────");
    let task = cyclic_generator.generate_task(Some(TaskType::Successor {
        item: "Sunday".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("K-Jump with Wraparound:");
    println!("──────────────────────────────────────────────────");
    let task = cyclic_generator.generate_task(Some(TaskType::KJump {
        start: "Friday".to_string(),
        k: 3,
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);
}