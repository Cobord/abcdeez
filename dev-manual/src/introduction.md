# Developer Manual

Welcome to the Graph Learning System Developer Manual. This guide provides comprehensive technical documentation for developers working on the Graph Learning System codebase.

## What This Manual Covers

This manual is designed for software developers, engineers, and technical contributors who need to:

- **Understand the Architecture** - System design and component interactions
- **Implement OAuth Features** - Apple Sign In and GitHub OAuth integration
- **Develop Backend Services** - Rust-based API and business logic
- **Build Frontend Interfaces** - Cross-platform UI using Xilem framework
- **Write Tests** - Comprehensive testing strategies and implementation
- **Debug Issues** - Troubleshooting and performance optimization

## Technology Stack

The Graph Learning System is built with modern, performance-focused technologies:

### Backend (Rust)
- **Framework**: Axum (async web framework)
- **Database**: SQLite with SQLx (async database access)
- **Authentication**: JWT tokens with OAuth 2.0/OpenID Connect
- **Security**: Comprehensive input validation and rate limiting
- **Monitoring**: Structured logging with tracing

### Frontend (Rust + Xilem)
- **UI Framework**: Xilem (reactive UI framework)
- **Platform Support**: iOS, Android, macOS, Windows, Linux, Web
- **State Management**: Centralized app state with reactive updates
- **Networking**: Async HTTP client with offline support

### OAuth Integration
- **Apple Sign In**: JWT verification with Apple's JWKS
- **GitHub OAuth**: Authorization code flow with state validation
- **iOS Integration**: Native AuthenticationServices bridge
- **Security**: PKCE, state parameters, token validation

## Key Features Implemented

✅ **Multi-Platform OAuth Authentication**
```rust
// Apple Sign In with JWT verification
pub async fn verify_identity_token(&mut self, token: &str) -> AppResult<AppleIdToken> {
    let jwks = self.get_apple_jwks().await?;
    let public_key = self.find_matching_key(&token, &jwks)?;
    let claims = self.validate_jwt_claims(&token, &public_key)?;
    Ok(claims)
}

// GitHub OAuth with state validation
pub async fn authenticate_github(&mut self, request: OAuthAuthRequest) -> AppResult<OAuthUserProfile> {
    self.validate_state(&request.state)?;
    let access_token = self.exchange_code_for_token(&request.code).await?;
    let user_profile = self.fetch_github_profile(&access_token).await?;
    Ok(user_profile)
}
```

✅ **Cross-Platform UI with Xilem**
```rust
// App Store compliant Apple Sign In button
pub fn apple_signin_button(data: &AppData, style: AppleSignInButtonStyle) -> impl WidgetView<AppData> {
    let button_text = match data.oauth_login_in_flight {
        true => "Signing in with Apple...",
        false => "Sign in with Apple",
    };
    
    button(button_text, |data: &mut AppData| {
        data.apple_sign_in();
    }).brush(Color::from_rgb8(0, 0, 0)) // Apple's required black
}
```

✅ **Production-Ready Security**
```rust
// JWT validation with proper algorithm verification
let mut validation = Validation::new(Algorithm::RS256);
validation.set_audience(&[&self.config.apple_client_id]);
validation.set_issuer(&["https://appleid.apple.com"]);
validation.validate_exp = true;
validation.validate_nbf = true;

let token_data = decode::<AppleIdToken>(identity_token, &decoding_key, &validation)?;
```

## Development Workflow

### 1. Local Development Setup
```bash
# Clone repository
git clone https://github.com/your-org/graph-learning-system.git
cd graph-learning-system

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
cargo install sqlx-cli --no-default-features --features sqlite

# Setup database
cd web-backend
sqlx migrate run --database-url sqlite://dev.db

# Run backend
cargo run

# Run frontend (separate terminal)
cd ../xilem-cross-platform
cargo run
```

### 2. Testing Workflow
```bash
# Run all tests
cargo test

# Run OAuth-specific tests
cargo test oauth_tests

# Run with coverage
cargo tarpaulin --out Html
```

### 3. Code Quality
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Security audit
cargo audit
```

## OAuth Implementation Deep Dive

### Apple Sign In Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌────────────────┐
│   iOS App       │    │   Rust Backend   │    │   Apple JWKS   │
│                 │    │                  │    │                │
│ AuthServices    │────┼─► JWT Verify     │◄───┼─► Public Keys  │
│ Identity Token  │    │   ES256 Algo     │    │   Rotation     │
│ User Info       │    │   Audience Check │    │   Caching      │
└─────────────────┘    └──────────────────┘    └────────────────┘
        │                        │
        │                        ▼
        │               ┌──────────────────┐
        └──────────────►│   User Database  │
                        │                  │
                        │ OAuth Provider   │
                        │ Private Email    │
                        │ Session Token    │
                        └──────────────────┘
```

### Key Implementation Details

**JWT Verification Process**:
1. Extract `kid` (Key ID) from JWT header
2. Fetch Apple's JWKS from `https://appleid.apple.com/auth/keys`
3. Find matching public key using `kid`
4. Verify JWT signature using RS256 algorithm
5. Validate claims: `iss`, `aud`, `exp`, `iat`
6. Extract user information from validated claims

**iOS Native Integration**:
```objc
// Objective-C bridge to Rust
- (void)authorizationController:(ASAuthorizationController *)controller 
           didCompleteWithAuthorization:(ASAuthorization *)authorization {
    
    ASAuthorizationAppleIDCredential *credential = authorization.credential;
    NSString *identityToken = [[NSString alloc] 
        initWithData:credential.identityToken encoding:NSUTF8StringEncoding];
    
    // Call Rust function via FFI
    handle_apple_sign_in_callback([jsonString UTF8String], true);
}
```

## Performance Considerations

### Database Optimization
- Indexed OAuth provider fields
- Connection pooling for concurrent requests
- Prepared statements for JWT validation queries

### Caching Strategy
- Apple JWKS cached with TTL
- JWT validation results cached temporarily
- Session tokens with sliding expiration

### Security Measures
- Rate limiting on authentication endpoints
- Input validation and sanitization
- Secure token storage with encryption
- HTTPS-only communication

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_apple_jwt_validation() {
    let service = create_test_apple_service().await;
    let test_jwt = create_valid_test_jwt();
    
    let result = service.verify_identity_token(&test_jwt).await;
    assert!(result.is_ok());
    
    let claims = result.unwrap();
    assert_eq!(claims.aud, "com.example.testapp");
    assert_eq!(claims.iss, "https://appleid.apple.com");
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_full_oauth_flow() {
    let app = create_test_app().await;
    
    // Test Apple Sign In endpoint
    let response = app.post("/api/auth/apple/signin")
        .json(&apple_signin_request)
        .send()
        .await?;
        
    assert_eq!(response.status(), 200);
    let user: User = response.json().await?;
    assert_eq!(user.auth_provider, "apple");
}
```

## Common Development Patterns

### Error Handling
```rust
// Comprehensive error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("JWT verification failed: {0}")]
    JwtVerification(String),
    
    #[error("OAuth provider error: {0}")]
    OAuthProvider(String),
    
    #[error("User creation failed: {0}")]
    UserCreation(String),
}
```

### Async Service Pattern
```rust
pub struct OAuthService {
    config: Arc<Config>,
    http_client: reqwest::Client,
    apple_service: AppleAuthService,
    github_service: GitHubOAuthService,
}

impl OAuthService {
    pub async fn authenticate(&mut self, request: OAuthAuthRequest) -> AppResult<OAuthUserProfile> {
        match request.provider.parse()? {
            OAuthProvider::Apple => self.authenticate_apple(request).await,
            OAuthProvider::GitHub => self.authenticate_github(request).await,
        }
    }
}
```

## Next Steps

To start developing:

1. **Set up your development environment** - [Development Environment](./getting-started/dev-environment.md)
2. **Understand the architecture** - [System Overview](./architecture/overview.md)
3. **Learn the OAuth implementation** - [OAuth Architecture](./oauth/architecture.md)
4. **Write your first test** - [Testing Strategy](./testing/strategy.md)

For specific implementation details, see the relevant sections in this manual.

---

*This manual is maintained alongside the codebase. For the latest updates, check the project repository.*