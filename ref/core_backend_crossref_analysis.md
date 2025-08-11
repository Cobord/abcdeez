# Core-Backend Cross-Reference Analysis

## Overview
This document analyzes the relationship between the `core` and `web-backend` modules to understand their integration points and what they imply for the missing Xilem app.

## Integration Points

### 1. **Learning Models & Algorithms**
- **Core Provides**: `LearnerModel`, belief updating, EIG calculations, proficiency tracking
- **Backend Uses**: These models in `/handlers/learning.rs` for adaptive task generation
- **App Needs**: UI to display learner state, progress visualization, adaptive feedback

### 2. **Task System**
- **Core Provides**: Task types (Alphabet, Music, Math), difficulty calibration, topology management
- **Backend Uses**: Task generation endpoints, hint system, adaptive scheduling
- **App Needs**: 
  - Task presentation UI for each domain
  - Response capture with accurate timing
  - Visual/audio rendering for music tasks
  - Hint display system

### 3. **Experimental Protocol**
- **Core Provides**: Experiment design, randomization, counterbalancing
- **Backend Uses**: Protocol versioning, session management, condition assignment
- **App Needs**:
  - Consent flow UI
  - Session state management
  - Between-session persistence
  - Protocol-specific UI configurations

### 4. **Data Collection**
- **Core Provides**: Response models, metrics calculations
- **Backend Uses**: Response storage, analytics aggregation
- **App Needs**:
  - High-precision timing for response capture
  - Touch/keyboard input handling
  - Response validation before submission

### 5. **Real-time Communication**
- **Core Provides**: Belief updates, intervention triggers
- **Backend Uses**: WebSocket for live updates, adaptive interventions
- **App Needs**:
  - WebSocket client management
  - Real-time UI updates
  - Intervention display (hints, feedback)

## Data Flow Architecture

```
Xilem App <-> Web Backend <-> Core Module
    |              |              |
    v              v              v
UI Layer    API/WebSocket   Algorithm Engine
```

### Request Flow Example:
1. App requests new task
2. Backend calls core's task generator with learner state
3. Core returns task with EIG-based difficulty
4. Backend stores task, sends to app
5. App presents task, captures response
6. App sends response to backend
7. Backend updates learner model via core
8. Core returns belief updates
9. Backend broadcasts updates via WebSocket
10. App updates UI with new state

## Missing Pieces the App Must Provide

### 1. **Authentication UI**
- Login/signup flows
- OAuth integration (Apple, GitHub)
- Session management
- JWT token handling

### 2. **Learning Interface**
- Task presentation for multiple domains
- Response capture mechanisms
- Progress visualization
- Achievement displays

### 3. **Experiment Management**
- Consent forms
- Demographic surveys
- Protocol selection
- Session scheduling

### 4. **Analytics Dashboard**
- Performance graphs
- Learning curves
- Comparison views
- Export functionality

### 5. **Social Features**
- Leaderboards
- Friend management
- Challenge creation
- Federation network access

### 6. **Settings & Profile**
- User preferences
- Notification settings
- Data export/deletion
- Privacy controls

## Platform-Specific Requirements

### Web App (Progressive Web App)
- Responsive design
- Offline support via service workers
- WebSocket connection management
- Browser-based timing precision

### Mobile App (iOS/Android via Xilem)
- Native performance for timing-critical tasks
- Push notifications
- Biometric authentication
- Background sync

### Desktop App (via Xilem)
- Keyboard shortcuts for rapid response
- Multi-window support for researchers
- Local data caching
- Advanced export options

## Critical Performance Requirements

1. **Response Timing**: Sub-10ms precision for reaction time measurements
2. **Visual Latency**: <16ms frame time for smooth animations
3. **Network Resilience**: Offline queue for responses
4. **State Sync**: Optimistic updates with reconciliation

## Security Requirements

1. **Data Protection**: E2E encryption for sensitive research data
2. **Authentication**: Secure token management
3. **Compliance**: GDPR/CCPA data handling
4. **Audit Trail**: Complete action logging

## Conclusion

The app needs to be a sophisticated, multi-platform client that:
1. Provides intuitive UI for complex learning tasks
2. Handles real-time communication efficiently
3. Manages offline/online state seamlessly
4. Delivers research-grade timing precision
5. Supports multiple experimental protocols
6. Integrates social and gamification features

The core module provides the intelligence, the backend provides the infrastructure, and the app must provide the user experience that ties them together.