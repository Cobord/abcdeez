# views/dashboard.rs - Main Dashboard Interface

## Conceptual Overview
Primary landing screen after authentication featuring welcome messages, learning statistics, quick actions, and recent activity. Serves as the central hub for navigation to all app features.

## Key Data Flows
- Displays user progress and learning statistics
- Provides quick navigation to major app functions
- Shows recent learning activity and achievements
- Integrates with bottom navigation system

## Main Responsibilities
- Welcome and overview interface
- Learning statistics display (sessions, accuracy, streaks)
- Quick action navigation (practice, progress, leaderboard)
- Recent activity timeline
- Bottom navigation integration
- Header with settings access

## Dependencies on Other Components
- `state::AppState` - Global state for user and navigation
- Theme system for consistent styling
- Navigation integration with all major screens

## User-Facing Functionality
- Personalized welcome experience
- Quick access to learning sessions
- Progress overview and motivation
- Easy navigation to all app features
- Recent activity and achievement highlights