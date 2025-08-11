# models/user.rs - User Account and Authentication Models

## Requirements and Dataflow
- Defines user account data structures with authentication and profile information
- Supports multiple authentication providers (local, OAuth, Apple, GitHub)
- Handles user registration, login, and profile management data contracts
- Provides token response structures for JWT-based authentication

## Key Abstractions and Interfaces
- User entity with comprehensive profile and authentication data
- OAuth integration models for Apple Sign In and GitHub authentication
- Authentication request/response structures for login and registration flows
- Token management models with access and refresh token handling