# handlers/auth.rs - Authentication and User Management

## Requirements and Dataflow
- Manages complete user authentication lifecycle including registration, login, logout, and token refresh
- Implements secure password handling with Argon2 hashing and strength validation
- Provides OAuth integration for Apple Sign In and GitHub authentication
- Handles session management with JWT tokens and refresh token rotation
- Implements account lockout protection and failed login attempt tracking

## High-level Purpose and Responsibilities
- **User Registration**: Account creation with comprehensive input validation and security checks
- **Authentication**: Secure login with password verification and account lockout protection
- **OAuth Integration**: Apple and GitHub OAuth flows with state validation and user profile mapping
- **Token Management**: JWT token generation, refresh, and blacklisting for session control
- **Security Enforcement**: Password strength validation, rate limiting, and audit logging
- **User Profile Management**: User information retrieval and account status tracking

## Key Abstractions and Interfaces
- Registration and login endpoints with comprehensive validation
- OAuth authorization URL generation and callback handling
- JWT token generation with role-based access control (RBAC)
- Session management with token blacklisting and refresh rotation
- Password strength validation with common pattern detection
- Account lockout mechanisms with configurable retry policies

## Data Transformations and Flow
1. **Registration Flow**: Input validation → password hashing → user creation → audit logging
2. **Authentication Flow**: Credentials validation → lockout checking → token generation → session creation
3. **OAuth Flow**: Provider authentication → user profile mapping → account creation/linking → token generation
4. **Token Refresh**: Refresh token validation → new access token generation → session continuity
5. **Logout Flow**: Token blacklisting → session invalidation → audit logging
6. **Security Monitoring**: Failed attempt tracking → lockout enforcement → metrics collection

## Dependencies and Interactions
- **Database Layer**: User account storage, OAuth profile management, and session tracking
- **Cache Layer**: Token blacklisting, failed attempt counters, and lockout state management
- **OAuth Services**: Apple and GitHub authentication service integration
- **Audit System**: Comprehensive logging of authentication events and security incidents
- **Business Metrics**: User signup tracking and authentication success/failure metrics
- **Configuration System**: Security policies, OAuth credentials, and lockout parameters
- **Middleware Integration**: JWT claims extraction and role-based authorization

## Architectural Patterns
- **Secure Authentication**: Multi-layered security with hashing, validation, and monitoring
- **OAuth Provider Integration**: Standardized OAuth flow handling with state protection
- **Session Management**: JWT-based stateless authentication with refresh token support
- **Security Monitoring**: Comprehensive audit logging and metrics collection
- **Account Protection**: Rate limiting and lockout mechanisms against brute force attacks
- **User Experience**: Seamless authentication flows with proper error messaging