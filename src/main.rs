mod topology;
mod learner;
mod tasks;
mod adaptive;
mod ui;
mod demo;

use std::env;
use std::io;
use ui::TerminalApp;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "demo" {
        demo::run_demo();
        demo::demonstrate_task_types();
        Ok(())
    } else {
        let mut app = TerminalApp::new();
        app.run()
    }
}
