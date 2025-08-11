# lib.rs - Library Module Exports

## Conceptual Overview
Library module that exposes the main modules of the xilem-app for external use. Serves as the public API surface for the application components.

## Key Data Flows
- Exposes module hierarchies as public interfaces
- Enables modular access to different subsystems

## Main Responsibilities
- Module organization and public API definition
- Encapsulation of internal implementation details
- Providing clean interfaces for external consumption

## Dependencies on Other Components
- All major application modules (app, components, models, services, state, utils, views)

## User-Facing Functionality
- Enables library-style usage of the application components
- Supports testing and modular development