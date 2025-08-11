# compliance/mod.rs - Compliance Module Root Abstract

## High-Level Purpose
Module root that exposes the compliance subsystem for research ethics, regulatory adherence, and scientific integrity within the abcdeez-core framework.

## Key Data Structures and Relationships
- **Module Organization**: Simple module aggregation exposing four primary compliance domains:
  - `audit_trail`: Comprehensive audit logging and event tracking
  - `citation_manager`: Academic citation management and methodology documentation
  - `irb`: Institutional Review Board compliance and ethics documentation generation
  - `preregistration`: Study pre-registration and analysis plan validation

## Main Data Flows
- **Unified Access**: Provides centralized access point for all compliance-related functionality
- **Domain Separation**: Clean separation between different aspects of research compliance

## External Dependencies
- Standard Rust module system only

## State Management Patterns
- **Stateless Organization**: Pure module aggregation without internal state

## Core Abstractions
- **Compliance Integration**: Unified interface for diverse compliance requirements
- **Regulatory Coverage**: Comprehensive coverage of research ethics and integrity requirements