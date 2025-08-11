# config.rs - Environment Configuration Management

## Requirements and Dataflow
- Loads application configuration from environment variables with fallback defaults
- Validates production-specific security requirements and constraints
- Manages OAuth provider credentials and TLS certificate configuration
- Provides differential privacy parameters and security policy settings
- Ensures configuration safety across different deployment environments

## High-level Purpose and Responsibilities
- **Environment Parsing**: Converts environment variables to strongly-typed configuration structures
- **Production Validation**: Enforces security policies and prevents weak configurations in production
- **OAuth Management**: Handles Apple and GitHub OAuth provider configuration with validation
- **TLS Configuration**: Manages SSL/TLS settings including Let's Encrypt integration
- **Security Policies**: Defines JWT secrets, CORS policies, rate limiting, and privacy parameters
- **Development Defaults**: Provides sensible defaults for local development environments

## Key Abstractions and Interfaces
- `Config` struct: Comprehensive configuration container with all application settings
- `Environment` enum: Deployment environment classification (Development, Staging, Production)
- `from_env()`: Main configuration loading function with environment variable parsing
- `validate_production_safety()`: Production-specific security validation
- `require_in_production()`: Helper for mandatory production configuration values

## Data Transformations and Flow
1. **Environment Variables → Typed Config**: Parses and validates environment variables
2. **Default Value Application**: Applies environment-appropriate defaults where values are missing
3. **OAuth Credential Validation**: Verifies OAuth provider configuration completeness
4. **TLS Setting Validation**: Ensures proper certificate configuration for HTTPS
5. **Security Policy Enforcement**: Validates JWT secrets, CORS origins, and privacy parameters
6. **Production Safety Checks**: Prevents deployment with insecure configuration values

## Dependencies and Interactions
- **Environment System**: Reads from process environment variables and .env files
- **OAuth Providers**: Validates Apple and GitHub OAuth application credentials
- **TLS Infrastructure**: Coordinates with certificate management and HTTPS setup
- **Database Layer**: Provides database connection URL and pooling configuration
- **Cache System**: Configures Redis/cache connection parameters
- **Security Middleware**: Supplies JWT secrets, CORS policies, and rate limiting settings
- **Privacy Systems**: Provides differential privacy epsilon/delta parameters

## Architectural Patterns
- **Environment-Driven Configuration**: Deployment-specific behavior through environment variables
- **Validation Layers**: Multi-stage validation with development vs production requirements
- **Fail-Fast Strategy**: Early configuration validation prevents runtime failures
- **Security-First Design**: Production safety validation prevents insecure deployments
- **Provider Abstraction**: Unified OAuth provider configuration pattern
- **Privacy by Design**: Built-in differential privacy parameter configuration