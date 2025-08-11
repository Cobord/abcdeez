use std::{env, process};

use anyhow::Result;

use abcdeez_core::demo;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let program_name = args.first().map(String::as_str).unwrap_or("abcdeez-core");

    if args.len() < 2 {
        print_usage(program_name);
        process::exit(1);
    }

    let command = &args[1];
    let command_args = &args[2..];

    match command.as_str() {
        "demo" => run_demo_command(command_args),
        "version" | "--version" | "-v" => print_version(),
        "help" | "--help" | "-h" => print_help(program_name),
        cmd => {
            eprintln!("Unknown command '{cmd}'");
            print_usage(program_name);
            process::exit(1);
        }
    }
}

fn run_demo_command(args: &[String]) -> Result<()> {
    let demo_command = args.first().map(String::as_str).unwrap_or("all");
    
    match demo_command {
        "basic" => demo::run_demo(),
        "tasks" => demo::demonstrate_task_types(),
        "dag" => demo::demonstrate_dag_tasks(),
        "stats" => demo::demonstrate_statistical_analysis(),
        "eig" => demo::demonstrate_eig(),
        "extended" => demo::demonstrate_extended_tasks(),
        "all" => run_all_demos(),
        sub => {
            eprintln!("Unknown demo subcommand '{sub}'");
            print_demo_help();
            process::exit(1);
        }
    }
    Ok(())
}

fn run_all_demos() {
    println!("Running all demonstrations...\n");
    let demos = [
        demo::run_demo,
        demo::demonstrate_task_types,
        demo::demonstrate_dag_tasks,
        demo::demonstrate_statistical_analysis,
        demo::demonstrate_eig,
        demo::demonstrate_extended_tasks,
    ];
    
    for demo in demos {
        demo();
    }
}

fn print_demo_help() {
    let commands = [
        ("basic", "Run basic demonstration"),
        ("tasks", "Demonstrate task types"),
        ("dag", "Demonstrate DAG tasks"),
        ("stats", "Demonstrate statistical analysis"),
        ("eig", "Demonstrate Expected Information Gain"),
        ("extended", "Demonstrate extended tasks"),
        ("all", "Run all demonstrations"),
    ];
    
    eprintln!("Available demo subcommands:");
    for (cmd, desc) in commands {
        eprintln!("  {cmd:<8} - {desc}");
    }
}

fn print_version() -> Result<()> {
    println!("abcdeez-core v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {program_name} <command> [options]\n");
    eprintln!("Commands:");
    eprintln!("  demo [subcommand]  Run demonstrations");
    eprintln!("  version            Show version information");
    eprintln!("  help               Show this help message\n");
    eprintln!("Run '{program_name} help' for more information");
}

fn print_help(program_name: &str) -> Result<()> {
    println!("abcdeez-core - Adaptive Learning Research Framework\n");
    println!("Usage: {program_name} <command> [options]\n");
    
    println!("Commands:");
    println!("  demo [subcommand]  Run demonstrations of the framework capabilities");
    println!("                     Subcommands: basic, tasks, dag, stats, eig, extended, all");
    println!("  version            Show version information");
    println!("  help               Show this help message\n");
    
    println!("Examples:");
    println!("  {program_name} demo           # Run all demonstrations");
    println!("  {program_name} demo basic     # Run basic demonstration only");
    println!("  {program_name} demo stats     # Run statistical analysis demo\n");
    
    println!("For more information, visit the project documentation.");
    Ok(())
}
