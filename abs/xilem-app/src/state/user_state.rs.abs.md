# state/user_state.rs - User Profile and Gamification State

## Conceptual Overview
Manages user profile information and gamification elements including levels, experience points, achievements, and learning streaks.

## Key Data Flows
- Links to learner_id for learning analytics
- Tracks gamification progress (XP, levels, streaks)
- Manages achievement collections
- Stores user profile information

## Main Responsibilities
- User identity and profile management
- Gamification system state (levels, XP, achievements)
- Learning streak tracking
- User creation timestamp tracking

## Dependencies on Other Components
- UUID system for unique identification
- Chrono for timestamp management

## User-Facing Functionality
- User profile display and management
- Gamification elements (levels, achievements, streaks)
- Learning progress visualization
- Motivation through gamified learning experience