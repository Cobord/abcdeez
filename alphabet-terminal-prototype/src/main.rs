use std::env;
use std::io;
use graph_learning_core::{demo,ui::TerminalApp};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "demo" {
        demo::run_demo();
        demo::demonstrate_task_types();
        demo::demonstrate_dag_tasks();
        demo::demonstrate_statistical_analysis();
        demo::demonstrate_eig();
        demo::demonstrate_extended_tasks();
        Ok(())
    } else {
        let mut app = TerminalApp::new();
        app.run()
    }
}
