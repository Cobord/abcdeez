# main.rs - CLI Application Entry Point Abstract

## High-Level Purpose
The main binary provides a command-line interface for the abcdeez-core adaptive learning research framework, enabling users to run demonstrations and access framework functionality through a structured CLI.

## Key Data Structures and Relationships
- **Command Structure**: Hierarchical command system with primary commands and subcommands
  - Primary commands: `demo`, `version`, `help`
  - Demo subcommands: `basic`, `tasks`, `dag`, `stats`, `eig`, `extended`, `all`
- **Command Dispatch**: Pattern matching for command routing and execution
- **Help System**: Multi-level help and usage information

## Main Data Flows
- **Argument Processing**: Command-line arguments → command parsing → function dispatch
- **Demo Orchestration**: Sequential execution of demonstration functions
- **Error Handling**: Graceful error handling with user-friendly messages and exit codes

## External Dependencies
- Standard library: `std::env`, `std::process`
- Framework: `abcdeez_core::demo` module
- Error handling: `anyhow::Result` for error propagation

## State Management Patterns
- **Stateless Execution**: Pure function-based command handling without persistent state
- **Process Management**: Controlled process termination with appropriate exit codes
- **Configuration-free**: No persistent configuration or state management

## Core Algorithms and Business Logic Abstractions
- **Command Parsing**: Simple string-based command matching and routing
- **Demo Orchestration**: Sequential execution pattern for running multiple demonstrations
- **User Experience**: Comprehensive help system with usage examples and command descriptions
- **Error Recovery**: Graceful handling of invalid commands with helpful feedback

## Interface Patterns
- **CLI Convention**: Standard command-line interface patterns with version, help, and primary commands
- **Extensibility**: Modular command structure allowing easy addition of new commands
- **User Guidance**: Comprehensive error messages and help text for user assistance