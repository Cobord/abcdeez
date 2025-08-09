
use crate::seed_management::SeedManager;
fn main() {
    let mut manager = SeedManager::new(Some(42));
    let exp_seed = manager.generate_experiment_seed("test_exp".to_string(), "Test experiment".to_string());
    let sess_seed = manager.generate_session_seed("sess1".to_string(), "test_exp".to_string(), "participant1".to_string());
    println!("Experiment seed: {}, Session seed: {}", exp_seed, sess_seed);
    
    let manifest = manager.get_reproducibility_manifest("test_exp");
    match manifest {
        Some(m) => println!("Reproducibility manifest generated with {} events", m.randomization_events.len()),
        None => println!("Failed to generate manifest"),
    }
    
    println!("Seed management integration test completed successfully!");
}

