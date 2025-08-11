# views/auth.rs - Authentication User Interface

## Conceptual Overview
Complete authentication interface supporting login and signup flows with OAuth integration. Provides form handling, validation, loading states, and seamless navigation between authentication modes.

## Key Data Flows
- Manages authentication form state (username, password, email)
- Handles authentication loading states and feedback
- Integrates with OAuth providers (Apple, GitHub)
- Updates global user state upon successful authentication
- Navigates between login/signup screens

## Main Responsibilities
- Login and signup form interfaces
- Authentication state management
- OAuth integration UI (Apple, GitHub)
- Form validation and error display
- Loading state indication
- Navigation between auth screens

## Dependencies on Other Components
- `state::AppState` - Global state for user authentication
- `state::UserState` - User profile creation
- Theme system for consistent styling

## User-Facing Functionality
- User account creation and login
- OAuth sign-in options (Apple, GitHub)
- Form validation and error feedback
- Loading indicators during authentication
- Seamless transition to main application