# iOS Apple Sign In Integration Guide

This document explains how to set up and use Apple Sign In authentication in the Xilem-based learning app.

## Architecture Overview

The iOS integration consists of several layers:

1. **Backend OAuth Service** - Rust backend with Apple JWT verification
2. **iOS AuthenticationServices Bridge** - Objective-C bridge to Apple's native APIs
3. **Rust FFI Layer** - C-compatible interface between iOS and Rust
4. **UI Integration** - Xilem UI components with Apple Sign In button

## Backend Setup

The backend is already configured with:
- Apple JWT verification using JWKS
- Database schema for OAuth providers
- API endpoints for Apple Sign In
- Credential validation background jobs

### Required Environment Variables

```bash
# Apple Sign In Configuration
APPLE_CLIENT_ID=your.app.bundle.identifier
APPLE_TEAM_ID=YOUR_TEAM_ID
APPLE_KEY_ID=YOUR_KEY_ID
APPLE_PRIVATE_KEY_PATH=/path/to/AuthKey_YOUR_KEY_ID.p8
APPLE_REDIRECT_URI=https://abcdeez.fg-goose.online/auth/callback

# Backend Configuration
DATABASE_URL=sqlite://./database.db
JWT_SECRET=your-super-secret-jwt-key
PORT=8080
```

## iOS Setup Instructions

### 1. Enable Apple Sign In Capability

In Xcode:
1. Select your app target
2. Go to "Signing & Capabilities"
3. Click "+ Capability"
4. Add "Sign In with Apple"

### 2. Configure Bundle Identifier

Make sure your bundle identifier matches `APPLE_CLIENT_ID` in the backend configuration.

### 3. Add Required Frameworks

The project needs these frameworks:
- `AuthenticationServices.framework` (iOS 13.0+)
- `Foundation.framework`
- `UIKit.framework`

### 4. Update Info.plist

The `Info.plist` is already configured with:
- Apple Sign In capability
- Backend API URL configuration

### 5. Build Configuration

When building for iOS, ensure:
- Target iOS 13.0+ (required for AuthenticationServices)
- Include the Objective-C bridge files in the build
- Link the required frameworks

## Usage

### From the UI

Users can tap the "🍎 Sign in with Apple" button on the welcome screen. The flow is:

1. User taps Apple Sign In button
2. iOS shows the native Apple ID authentication sheet  
3. User authenticates with Face ID/Touch ID/password
4. iOS returns identity token and user info
5. App sends token to backend for verification
6. Backend validates JWT and creates/updates user
7. App receives authenticated user and proceeds to domain selection

### From Code

```rust
// Initialize iOS auth bridge
data.init_ios_auth();

// Trigger Apple Sign In
data.apple_sign_in();

// Handle result (automatic via callback)
data.handle_oauth_login_result(result);
```

## File Structure

```
xilem-cross-platform/
├── app/src/
│   ├── api.rs              # API client with OAuth endpoints
│   ├── ios_auth.rs         # iOS authentication bridge
│   ├── models.rs           # OAuth-related data structures
│   └── screens/welcome.rs  # UI with Apple Sign In button
└── ios-src/
    ├── Info.plist             # iOS app configuration
    ├── AppleSignInBridge.h    # Objective-C bridge header
    ├── AppleSignInBridge.m    # Objective-C bridge implementation
    └── bindings.h             # C FFI declarations
```

## Security Considerations

1. **JWT Verification**: The backend verifies Apple's JWT using their JWKS endpoint
2. **State Validation**: OAuth state parameters prevent CSRF attacks
3. **Token Storage**: Access tokens are stored securely in the API client
4. **Private Email**: Handles Apple's private email relay properly
5. **Real User Detection**: Uses Apple's real user detection status

## Testing

### Local Development

1. Start the backend server:
   ```bash
   cd web-backend
   cargo run
   ```

2. Configure the iOS app to point to your local backend:
   - Update `APIBaseURL` in `Info.plist`
   - For simulator: `http://localhost:8080`
   - For device: `http://YOUR_IP_ADDRESS:8080`

3. Build and run the iOS app:
   ```bash
   cd xilem-cross-platform
   # Build for simulator or device using Xcode
   ```

### Production Setup

1. Configure Apple Developer Account:
   - Create App ID with Sign In with Apple capability
   - Generate Sign In with Apple key
   - Configure your backend domain

2. Deploy backend with production configuration
3. Update iOS app with production API URL
4. Submit to App Store

## API Endpoints

The backend provides these OAuth endpoints:

- `POST /api/auth/apple/signin` - Apple Sign In with identity token
- `GET /api/auth/oauth/:provider/authorize` - Get OAuth authorization URL  
- `POST /api/auth/oauth/callback` - Handle OAuth callback
- `POST /admin/oauth-validation` - Trigger credential validation (admin only)

## Troubleshooting

### Common Issues

1. **"Sign In with Apple unavailable"**
   - Ensure iOS 13.0+ target
   - Check capability is enabled in Xcode
   - Verify bundle identifier matches backend config

2. **JWT verification failed**
   - Check Apple key configuration in backend
   - Verify APPLE_CLIENT_ID matches app bundle ID
   - Ensure system clock is synchronized

3. **Network connection failed**
   - Check backend is running and accessible
   - Verify APIBaseURL in Info.plist
   - Check firewall/network settings

4. **User creation failed**
   - Check backend logs for database errors
   - Verify all required fields are provided
   - Check database schema is up to date

### Debug Logging

Enable debug logging by setting:
```bash
RUST_LOG=debug cargo run
```

This will show detailed OAuth flow information and JWT verification steps.

## Additional Resources

- [Apple Sign In Documentation](https://developer.apple.com/sign-in-with-apple/)
- [AuthenticationServices Framework](https://developer.apple.com/documentation/authenticationservices)
- [JWT.io](https://jwt.io) - For debugging JWTs
- [Apple Developer Portal](https://developer.apple.com) - For key management