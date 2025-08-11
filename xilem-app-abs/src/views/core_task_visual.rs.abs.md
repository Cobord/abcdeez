# views/core_task_visual.rs - Core Task Visual Representations

## Conceptual Overview
Specialized visual components for core learning tasks from abcdeez_core. Provides rich, interactive visual presentations for different core task types including ordering, succession, jumps, segments, and topological operations.

## Key Data Flows
- Receives core task types and renders appropriate visuals
- Uses theme colors for consistent visual presentation
- Creates interactive visual representations of abstract learning concepts
- Supports various core task categories with distinct visual patterns

## Main Responsibilities
- Visual rendering for core task types (PairwiseOrder, Successor, KJump, etc.)
- Consistent visual design across all core task presentations
- Theme integration for visual consistency
- Interactive visual elements that enhance learning comprehension
- Complex task visualization (topological sorts, minimal/maximal elements)

## Dependencies on Other Components
- `abcdeez_core::tasks::core::TaskType` - Core task definitions
- `state::AppState` - For theme access
- Xilem UI framework for visual components

## User-Facing Functionality
- Rich visual learning experiences for core concepts
- Intuitive visual representations of abstract relationships
- Consistent visual language across different task types
- Enhanced comprehension through visual learning
- Interactive elements that support task understanding