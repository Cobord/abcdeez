# services/websocket.rs - Real-time WebSocket Communication

## Conceptual Overview
WebSocket client for real-time communication with the backend during learning sessions. Provides bi-directional messaging for live task delivery, response submission, hints, interventions, and performance updates.

## Key Data Flows
- Establishes authenticated WebSocket connections
- Handles real-time task delivery and response submission
- Processes live feedback and interventions
- Manages heartbeat and connection health
- Supports session pause/resume functionality

## Main Responsibilities
- WebSocket connection management
- Authentication and session binding
- Real-time message handling (client/server)
- Connection health monitoring and reconnection
- Live task interaction support
- Performance and statistics updates

## Dependencies on Other Components
- `models::Task` - Task data structures
- Tokio for async WebSocket operations
- Serde for message serialization

## User-Facing Functionality
- Real-time learning experiences
- Immediate feedback on responses
- Live adaptive interventions
- Session progress updates
- Seamless connection management with auto-reconnect