# views/settings.rs - Settings and Profile Management Interface

## Conceptual Overview
Comprehensive settings interface for user profile management, preferences, theme selection, and account operations. Provides organized sections for different types of settings with intuitive controls.

## Key Data Flows
- Manages user profile display and editing
- Handles theme preference changes
- Processes notification and sound settings
- Manages account operations (export, privacy, sign out)

## Main Responsibilities
- User profile section with avatar and info display
- Theme selection (Light/Dark/Auto) with live preview
- Preference management (notifications, sounds)
- Account management (data export, privacy, deletion)
- Settings organization into logical sections
- Navigation back to previous screens

## Dependencies on Other Components
- `state::AppState` - Global state for settings
- `state::ThemeMode` - Theme management
- `state::UserState` - User profile data
- Theme system for consistent styling

## User-Facing Functionality
- User profile viewing and editing
- Theme customization with immediate preview
- Notification and sound preference control
- Account data management and export
- Privacy settings access
- Account deletion and sign out options