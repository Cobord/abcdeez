# state/theme.rs - Theme and Styling Management

## Conceptual Overview
Centralized theme system supporting light, dark, and auto modes. Provides consistent color schemes and styling throughout the application using Xilem's color system.

## Key Data Flows
- Provides color values based on current theme mode
- Supports dynamic theme switching
- Integrates with system preferences for auto mode

## Main Responsibilities
- Theme mode management (Light/Dark/Auto)
- Consistent color palette definition
- Background, text, and accent color provision
- Status colors (success, error, warning)

## Dependencies on Other Components
- Xilem::Color for color representation

## User-Facing Functionality
- Light and dark theme support
- System theme preference integration
- Consistent visual experience across all screens
- Accessibility through appropriate color contrast