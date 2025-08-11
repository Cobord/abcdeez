# /src/views - User Interface Views Dependencies

## Backend (web-backend) Requirements

### Authentication Views (`auth.rs`)
- User authentication API integration
- OAuth provider integration (Apple, GitHub)
- Form validation and error feedback
- Account creation and verification

### Dashboard Views (`dashboard.rs`)
- User statistics and progress data
- Recent activity and achievement data
- Quick action navigation endpoints
- Gamification data (levels, streaks, accuracy)

### Learning Views (`learning.rs`)
- Real-time task generation and delivery
- Response submission and immediate feedback
- Session management and progress tracking
- Hint system integration
- Performance analytics display

### Settings Views (`settings.rs`)
- User profile management
- Preference synchronization
- Account data export functionality
- Privacy settings and account deletion

## Core Library (abcdeez_core) Requirements

### Task Visual Components (`core_task_visual.rs`, `extended_task_visual.rs`)
- `tasks::core::TaskType` - Core task type definitions
- `tasks::extended::ExtendedTaskType` - Extended task types
- Task visualization algorithms and patterns
- Visual metaphor systems for abstract concepts

### Learning Interface Integration
- `core::Topology` - Learning space visualization
- Task difficulty and progress representation
- Performance metrics visualization
- Adaptive feedback systems

## Data Structure Requirements

### UI State Management
```rust
// View-specific state requirements
AppState - Global application state access
SessionState - Learning session data
UserState - Profile and gamification data
Theme - Consistent visual styling

// Task presentation data
Task - Complete task data with visual requirements
TaskType variants - Support for all task categories
ResponseMetrics - Performance visualization data
```

### Visual Presentation Models
```rust
// Task visualization requirements
CoreTaskType variants - Visual patterns for each type
ExtendedTaskType variants - Complex visual representations
Task options and prompts - Interactive presentation data

// Progress and feedback display
Performance metrics - Real-time progress indicators
Achievement data - Gamification visual elements
Statistics - Dashboard overview data
```

## Business Logic Dependencies

### Authentication Flow
- Login/signup form handling and validation
- OAuth integration with external providers
- Session management and token handling
- Error state presentation and recovery

### Learning Experience Design
- Task presentation with rich visual elements
- Response capture and immediate feedback
- Progress tracking and motivation systems
- Adaptive hint and intervention display

### Navigation and User Experience
- Screen transition management
- Loading state presentation
- Error handling and user feedback
- Accessibility considerations

### Real-time Updates
- Live session updates and synchronization
- Performance metrics display
- Connection status indicators
- Background sync status presentation

### Theme and Styling
- Consistent visual design language
- Light/dark theme support
- Accessibility compliance (contrast, sizing)
- Responsive design patterns

### Interaction Patterns
- Touch and mouse interaction handling
- Keyboard navigation support
- Form validation and user feedback
- Gesture and interaction tracking

## Integration Requirements

### Xilem Framework Integration
- Widget composition and lifecycle management
- State management and reactivity
- Event handling and user interactions
- Performance optimization for smooth UI

### Service Layer Integration
- API client integration for data fetching
- Real-time WebSocket updates
- Local storage and caching
- Background service coordination

### Analytics and Tracking
- User interaction behavior capture
- Performance monitoring and optimization
- Learning analytics data collection
- Error tracking and reporting