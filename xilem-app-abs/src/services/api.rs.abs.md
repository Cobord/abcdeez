# services/api.rs - HTTP API Client Service

## Conceptual Overview
Comprehensive HTTP client for communication with the web-backend. Handles authentication, session management, task generation, learner analytics, synchronization, and gamification features.

## Key Data Flows
- Manages authentication tokens and refresh cycles
- Sends/receives session data and task responses
- Synchronizes offline data with backend
- Retrieves gamification and leaderboard data
- Handles error responses and timeouts

## Main Responsibilities
- HTTP client configuration and management
- Authentication and token management
- Session lifecycle API calls
- Task generation and hint requests
- Response submission and feedback retrieval
- Learner profile and statistics management
- Offline data synchronization
- Gamification API integration

## Dependencies on Other Components
- `models::*` - Data models for API communication
- Reqwest for HTTP client functionality
- Serde for JSON serialization/deserialization

## User-Facing Functionality
- Seamless authentication experience
- Real-time task generation and feedback
- Learner progress synchronization
- Gamification features (levels, achievements, leaderboards)
- Offline learning with eventual consistency