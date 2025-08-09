# OAuth Authentication Overview

The Graph Learning System implements secure OAuth 2.0 and OpenID Connect authentication with support for multiple identity providers. This section provides a comprehensive overview of the OAuth implementation for administrators.

## Supported Identity Providers

### Apple Sign In (Required for iOS App Store)

- **Protocol**: OpenID Connect 1.0
- **Token Type**: JWT (JSON Web Tokens)
- **Verification**: Apple's JWKS (JSON Web Key Set)
- **Use Case**: iOS app authentication, web authentication
- **Required**: Yes (mandatory for iOS App Store submission)

### GitHub OAuth (Optional)

- **Protocol**: OAuth 2.0 Authorization Code Flow
- **Token Type**: Bearer tokens
- **Verification**: GitHub API validation
- **Use Case**: Web authentication, developer accounts
- **Required**: No (but recommended for developer users)

## Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌────────────────┐
│   iOS/Web App   │    │   Backend API    │    │   Identity     │
│                 │    │                  │    │   Providers    │
├─────────────────┤    ├──────────────────┤    ├────────────────┤
│                 │    │                  │    │                │
│ Apple Sign In   │◄──►│ JWT Verification │◄──►│ Apple JWKS     │
│ GitHub OAuth    │    │ OAuth Service    │    │ GitHub API     │
│                 │    │                  │    │                │
└─────────────────┘    └──────────────────┘    └────────────────┘
                              │
                              ▼
                       ┌──────────────┐
                       │   Database   │
                       │              │
                       │ User Accounts│
                       │ OAuth Links  │
                       │ Sessions     │
                       └──────────────┘
```

## Security Model

### Token Verification

**Apple Sign In**:
- Identity tokens are JWT signed with ES256
- Tokens verified using Apple's public JWKS
- Audience validation ensures tokens are for your app
- Issuer validation confirms tokens from Apple
- Expiration time validation prevents replay attacks

**GitHub OAuth**:
- Authorization code flow with state parameter
- State parameter prevents CSRF attacks
- Tokens exchanged securely on backend
- User profile verified via GitHub API

### Data Protection

- **Private Email Handling**: Apple's private email relay supported
- **Minimal Scope**: Only request necessary permissions (name, email)
- **Secure Storage**: OAuth credentials encrypted in database
- **Session Management**: Secure JWT sessions with proper expiration

## Database Schema

The OAuth implementation extends the user model with provider-specific fields:

```sql
-- User table with OAuth support
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL,
    
    -- OAuth provider fields
    apple_user_id TEXT UNIQUE,
    github_user_id TEXT UNIQUE,
    oauth_provider_id TEXT,
    auth_provider TEXT DEFAULT 'local' CHECK (auth_provider IN ('local', 'apple', 'github')),
    is_private_email BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- OAuth credential validation tracking
CREATE TABLE oauth_credential_checks (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    provider TEXT NOT NULL CHECK (provider IN ('apple', 'github')),
    provider_user_id TEXT NOT NULL,
    last_check_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    credential_state TEXT CHECK (credential_state IN ('authorized', 'revoked', 'not_found', 'unknown')),
    error_details TEXT
);
```

## API Endpoints

### Apple Sign In

```http
POST /api/auth/apple/signin
Content-Type: application/json

{
    "identity_token": "eyJ...",
    "authorization_code": "c123...",
    "user_info": {
        "name": {
            "first_name": "John",
            "last_name": "Doe"
        },
        "email": "john@privaterelay.appleid.com"
    }
}
```

### GitHub OAuth

```http
GET /api/auth/oauth/github/authorize?state=secure_random_state

POST /api/auth/oauth/callback
Content-Type: application/json

{
    "provider": "github",
    "code": "authorization_code_from_github",
    "state": "secure_random_state"
}
```

## Configuration Requirements

### Environment Variables

```bash
# Apple Sign In
APPLE_CLIENT_ID=your.app.bundle.identifier
APPLE_TEAM_ID=YOUR_TEAM_ID
APPLE_KEY_ID=YOUR_KEY_ID
APPLE_PRIVATE_KEY_PATH=/path/to/AuthKey.p8
APPLE_REDIRECT_URI=https://abcdeez.fg-goose.online/auth/callback

# GitHub OAuth
GITHUB_CLIENT_ID=your_github_client_id
GITHUB_CLIENT_SECRET=your_github_client_secret
GITHUB_REDIRECT_URI=https://abcdeez.fg-goose.online/auth/github/callback

# Security
JWT_SECRET=your-super-secure-jwt-secret-minimum-32-characters
```

### File Requirements

- Apple private key file (`.p8` format)
- SSL certificate for HTTPS
- Proper file permissions (600 for private keys)

## User Flow Examples

### Apple Sign In Flow

1. User taps "Sign in with Apple" in iOS app
2. iOS shows native Apple ID authentication sheet
3. User authenticates with Face ID/Touch ID/password
4. iOS returns identity token and user info
5. App sends token to backend for verification
6. Backend validates JWT using Apple's JWKS
7. User account created/updated in database
8. Session token returned to app

### GitHub OAuth Flow

1. User clicks "Sign in with GitHub" on web
2. Browser redirects to GitHub authorization URL
3. User authorizes the application
4. GitHub redirects back with authorization code
5. Backend exchanges code for access token
6. User profile fetched from GitHub API
7. User account created/updated in database
8. Session token returned to browser

## Monitoring and Alerts

### Key Metrics

- OAuth authentication success/failure rates
- Token verification performance
- Apple JWKS fetch latency
- GitHub API response times
- Private email relay usage

### Health Checks

```bash
# Check OAuth provider configuration
curl https://abcdeez.fg-goose.online/health

# Expected response
{
    "status": "healthy",
    "oauth_providers": {
        "apple": "configured",
        "github": "configured"
    }
}
```

### Log Monitoring

Look for these log patterns:
- `oauth.apple.jwt_verification_failed`
- `oauth.github.token_exchange_failed`
- `oauth.user_creation_failed`
- `oauth.credential_validation_failed`

## Next Steps

- [Apple Sign In Setup](./apple-setup.md) - Detailed Apple configuration
- [GitHub OAuth Setup](./github-setup.md) - GitHub OAuth configuration
- [Security Best Practices](./security.md) - Security recommendations