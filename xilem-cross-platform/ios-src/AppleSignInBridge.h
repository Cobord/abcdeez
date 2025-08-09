//
//  AppleSignInBridge.h
//  Graph Learning System
//
//  iOS Bridge for Apple Sign In with AuthenticationServices
//

#import <Foundation/Foundation.h>
#import <AuthenticationServices/AuthenticationServices.h>

// C function declarations for Rust FFI
extern void handle_apple_sign_in_callback(const char* json_data, bool success);

// Apple Sign In Bridge Interface
@interface AppleSignInBridge : NSObject <ASAuthorizationControllerDelegate, ASAuthorizationControllerPresentationContextProviding>

+ (instancetype)sharedInstance;
- (void)startAppleSignIn;

@end

// C function exported to Rust
void start_apple_sign_in(void);