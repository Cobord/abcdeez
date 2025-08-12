# iOS Native Authentication Bridge

This directory contains the native iOS implementation for Apple Sign In integration with the ABCDEEZ Rust application.

## Files

- `AppleSignInBridge.h/m` - Main Apple Sign In implementation using AuthenticationServices framework
- `RustAuthBridge.h/m` - C FFI bridge between Rust and Objective-C
- `ABCDEEZ-Bridging-Header.h` - Swift/Objective-C bridging header
- `Info.plist` - iOS app configuration with Apple Sign In capability

## Xcode Project Setup

To integrate these files with your Xcode project:

### 1. Add Files to Xcode Project

1. Open your Xcode project
2. Right-click on your project in the navigator
3. Select "Add Files to [Your Project]"
4. Add all `.h` and `.m` files from this directory
5. Ensure "Copy items if needed" is unchecked (reference files in place)
6. Add to your app target

### 2. Configure Build Settings

In your target's Build Settings:

1. **Objective-C Bridging Header**
   - Set to: `$(PROJECT_DIR)/xilem-cross-platform/ios-src/ABCDEEZ-Bridging-Header.h`

2. **Other Linker Flags**
   - Add: `-ObjC`

3. **Enable Modules**
   - Set "Enable Modules" to YES

### 3. Add Required Frameworks

In your target's "Frameworks, Libraries, and Embedded Content":

1. Add `AuthenticationServices.framework`
2. Add `Foundation.framework`
3. Add `UIKit.framework`

### 4. Configure Capabilities

In your target's "Signing & Capabilities":

1. Click "+ Capability"
2. Add "Sign in with Apple"

### 5. Update Bundle Identifier

Ensure your bundle identifier matches what's registered in your Apple Developer account with Sign in with Apple capability.

## Testing

### Simulator Testing
- Apple Sign In works in the iOS Simulator
- You'll need to sign in with a real Apple ID

### Device Testing
- Requires a development provisioning profile with Sign in with Apple capability
- Device must be running iOS 13.0 or later

## Rust Integration

The Rust code in `xilem-app/src/auth/ios.rs` will automatically use these native functions when compiled for iOS target.

### Building for iOS

```bash
# Build the Rust library for iOS
cargo build --target aarch64-apple-ios --release

# Or for simulator
cargo build --target aarch64-apple-ios-sim --release
```

## Troubleshooting

### Undefined Symbols Error
If you get undefined symbol errors for `ios_apple_sign_in`, etc.:
1. Ensure `RustAuthBridge.m` is added to your target
2. Check that the file is included in "Compile Sources" build phase

### Apple Sign In Not Working
1. Check that your provisioning profile includes Sign in with Apple capability
2. Verify bundle identifier matches your app configuration
3. Ensure you're testing on iOS 13.0+

### Authentication Fails Silently
Check the Xcode console for error messages from the native code. The bridge logs errors before passing them to Rust.