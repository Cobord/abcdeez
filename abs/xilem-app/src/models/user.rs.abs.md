# models/user.rs - User and Learner Data Models

## Conceptual Overview
Data models for user accounts and learner profiles. Separates user identity from learning analytics, enabling flexible user-learner relationships and comprehensive learning tracking.

## Key Data Flows
- Links user accounts to learner profiles
- Tracks learning activity and practice time
- Supports metadata for extensible user/learner properties
- Enables user authentication and profile management

## Main Responsibilities
- User account data modeling
- Learner profile and analytics tracking
- User-learner relationship management
- Practice time and activity tracking
- Metadata support for extensibility

## Dependencies on Other Components
- Serde for JSON serialization
- UUID for unique identification
- Chrono for timestamp management

## User-Facing Functionality
- User account creation and management
- Learning progress tracking
- Practice time monitoring
- Flexible user profile customization
- Learning analytics and insights