# utils.rs - Utility Functions and Gamification Logic

## Requirements and Dataflow
- Provides UUID serialization utilities for consistent API responses
- Implements gamification system with XP calculation and achievement logic
- Supports flexible UUID parsing for both hyphenated and compact formats
- Calculates experience points based on performance metrics and difficulty
- Manages achievement unlock conditions and progression recommendations

## High-level Purpose and Responsibilities
- **UUID Handling**: Consistent UUID serialization and flexible parsing for API compatibility
- **Gamification Engine**: XP calculation with difficulty scaling, speed bonuses, and streak multipliers
- **Achievement System**: Rule-based achievement unlocking with threshold management
- **User Progression**: Achievement recommendation system based on current progress
- **Input Validation**: Robust parameter validation for calculation functions
- **Mathematics Module**: Exposes specialized math and statistics utility modules

## Key Abstractions and Interfaces
- UUID serialization functions for consistent string conversion
- `calculate_xp_reward()`: Core gamification function with performance-based XP calculation
- `check_achievement_unlock()`: Rule engine for achievement qualification
- `suggest_next_achievement()`: Progression guidance based on user statistics
- `UserStats` struct: Comprehensive user performance and progress tracking
- `parse_uuid_flexible()`: Robust UUID parsing supporting multiple formats

## Data Transformations and Flow
1. **UUID Serialization**: Consistent UUID-to-string conversion for API responses
2. **Performance Metrics**: Response time, accuracy, and difficulty scaling into XP rewards
3. **Achievement Evaluation**: User statistics comparison against predefined thresholds
4. **Progression Analysis**: Current progress assessment for next achievement suggestions
5. **Input Sanitization**: Parameter validation and bounds checking for calculation functions
6. **Streak Calculation**: Consecutive success tracking with bonus multipliers

## Dependencies and Interactions
- **API Layer**: UUID serialization for consistent response formatting
- **Gamification Handlers**: XP calculation and achievement unlock processing
- **User Statistics**: Integration with user performance tracking systems
- **Database Models**: UUID conversion for database storage and retrieval
- **Session Management**: Performance metrics collection during task sessions
- **Achievement System**: Rule-based qualification and unlocking logic

## Architectural Patterns
- **Utility Pattern**: Stateless functions for cross-cutting concerns
- **Validation Layer**: Input sanitization and bounds checking for robustness
- **Rule Engine**: Achievement qualification through configurable threshold checks
- **Performance Scaling**: Dynamic XP calculation based on multiple performance factors
- **Progression System**: Guided user advancement through achievement recommendations
- **Format Flexibility**: Multiple UUID parsing strategies for API compatibility