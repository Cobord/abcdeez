# lib.rs - Library Root Module Abstract

## High-Level Purpose
The library root module serves as the public API gateway for the abcdeez-core adaptive learning research framework. It defines the organizational structure and provides unified access to all functional domains of the system.

## Key Data Structures and Relationships
- **Module Organization**: Hierarchical module structure exposing eight primary domains:
  - `compliance`: Research compliance and regulatory requirements
  - `core`: Fundamental system architecture and error handling
  - `data`: Data collection, processing, and export capabilities
  - `demo`: Demonstration and example implementations
  - `experiments`: Experimental design and execution framework
  - `learning`: Adaptive learning algorithms and strategies
  - `protocol`: Communication protocols and version management
  - `statistics`: Statistical analysis and validation tools
  - `tasks`: Task definition and execution framework

## Main Data Flows
- **Public API Exposure**: Re-exports core error types (`Error`, `Result`) for unified error handling across the framework
- **Version Management**: Exposes package version information at compile time
- **Prelude Pattern**: Provides a convenience module for common imports

## External Dependencies
- Standard Rust module system
- Cargo build system for version information (`env!` macro)
- Conditional compilation for test modules

## State Management Patterns
- **Stateless Module Organization**: Pure module aggregation without internal state
- **Error Propagation**: Centralized error type system through re-exports

## Core Abstractions
- **Domain Separation**: Clear separation of concerns across functional domains
- **API Consistency**: Unified error handling and result types across all modules
- **Build-time Configuration**: Compile-time version embedding for runtime access