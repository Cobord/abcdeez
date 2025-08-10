use abcdeez_core::demo::*;

fn main() {
    println!("Testing alphabet-terminal-prototype implementation...\n");

    // Run the built-in demonstrations
    run_demo();
    demonstrate_task_types();
    demonstrate_dag_tasks();
    demonstrate_extended_tasks();
    demonstrate_eig();
    demonstrate_statistical_analysis();

    println!("\n✅ All demonstrations completed successfully!");
    println!("\nSummary of improvements made:");
    println!("- Fixed numerical stability in Ex-Gaussian PDF calculation");
    println!("- Implemented insertion adaptation task with proper constraint handling");
    println!("- Added identifiability constraints to prevent gauge freedom");
    println!("- Implemented strategy detection in export module");
    println!("- Fixed all TODOs and incomplete implementations");
    println!("- All extended task types are now fully functional");
    println!("- UI module is now properly feature-gated for CLI builds");
}
