//
//  RustAuthBridge.m
//  ABCDEEZ
//
//  C Bridge Implementation between Rust and iOS Authentication
//

#import <Foundation/Foundation.h>
#import <AuthenticationServices/AuthenticationServices.h>
#import "RustAuthBridge.h"
#import "AppleSignInBridge.h"

// Global storage for async callbacks
typedef void (*AuthCompletionCallback)(IOSAuthResult*);
static AuthCompletionCallback g_pendingCallback = NULL;
static dispatch_queue_t g_authCallbackQueue = NULL;

// Initialize the callback queue once with UserInitiated QoS
__attribute__((constructor))
static void initializeAuthBridge(void) {
    dispatch_queue_attr_t attr = dispatch_queue_attr_make_with_qos_class(
        DISPATCH_QUEUE_SERIAL,
        QOS_CLASS_USER_INITIATED,
        0
    );
    g_authCallbackQueue = dispatch_queue_create("com.abcdeez.auth.callback", attr);
}

// Callback handler that will be called from AppleSignInBridge
void handle_apple_sign_in_callback(const char* json_data, bool success) {
    NSLog(@"handle_apple_sign_in_callback called with success=%d, has_callback=%d", success, g_pendingCallback != NULL);
    // Process the callback on our UserInitiated QoS queue
    dispatch_async(g_authCallbackQueue, ^{
        // Allocate new result
        IOSAuthResult* authResult = (IOSAuthResult*)malloc(sizeof(IOSAuthResult));
        memset(authResult, 0, sizeof(IOSAuthResult));
        
        authResult->success = success;
    
        if (json_data) {
            NSString *jsonString = [NSString stringWithUTF8String:json_data];
            NSData *jsonData = [jsonString dataUsingEncoding:NSUTF8StringEncoding];
            NSError *error;
            NSDictionary *authData = [NSJSONSerialization JSONObjectWithData:jsonData options:0 error:&error];
            
            if (!error && authData) {
                if (success) {
                    // Extract user data from successful response
                    NSString *userId = authData[@"user_id"];
                    if (userId) {
                        authResult->user_id = strdup([userId UTF8String]);
                    }
                    
                    NSString *email = authData[@"email"];
                    if (email) {
                        authResult->email = strdup([email UTF8String]);
                    }
                    
                    NSDictionary *fullName = authData[@"full_name"];
                    if (fullName) {
                        NSString *givenName = fullName[@"given_name"];
                        if (givenName) {
                            authResult->given_name = strdup([givenName UTF8String]);
                        }
                        
                        NSString *familyName = fullName[@"family_name"];
                        if (familyName) {
                            authResult->family_name = strdup([familyName UTF8String]);
                        }
                    }
                    
                    // Check if email is private relay
                    if (email && [email containsString:@"privaterelay.appleid.com"]) {
                        authResult->is_private_email = true;
                    }
                } else {
                    // Extract error message from failed response
                    NSString *errorMsg = authData[@"error"];
                    if (errorMsg) {
                        authResult->error_message = strdup([errorMsg UTF8String]);
                    } else {
                        authResult->error_message = strdup("Unknown error");
                    }
                }
            }
        }
        
        // Call the pending callback if it exists
        if (g_pendingCallback) {
            NSLog(@"Calling pending callback with result, callback=%p", g_pendingCallback);
            AuthCompletionCallback callback = g_pendingCallback;
            g_pendingCallback = NULL;
            NSLog(@"About to invoke callback function at %p with authResult=%p", callback, authResult);
            callback(authResult);
            NSLog(@"Callback invoked successfully");
        } else {
            NSLog(@"No pending callback, freeing result");
            // No callback waiting, free the result
            ios_free_auth_result(authResult);
        }
    });
}

// Implementation of Rust FFI functions

// Async version that doesn't block
void ios_apple_sign_in_async(AuthCompletionCallback callback) {
    NSLog(@"ios_apple_sign_in_async called, setting callback=%p", callback);
    // Store the callback
    g_pendingCallback = callback;
    
    // Start Apple Sign In on main thread
    dispatch_async(dispatch_get_main_queue(), ^{
        NSLog(@"Starting Apple Sign In from main queue");
        [[AppleSignInBridge sharedInstance] startAppleSignIn];
    });
    
    // Set up a timeout handler on the callback queue
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 60 * NSEC_PER_SEC), g_authCallbackQueue, ^{
        if (g_pendingCallback) {
            // Timeout occurred - create error result
            IOSAuthResult* timeoutResult = (IOSAuthResult*)malloc(sizeof(IOSAuthResult));
            memset(timeoutResult, 0, sizeof(IOSAuthResult));
            timeoutResult->success = false;
            timeoutResult->error_message = strdup("Authentication timeout");
            
            AuthCompletionCallback callback = g_pendingCallback;
            g_pendingCallback = NULL;
            callback(timeoutResult);
        }
    });
}

// Helper structures for blocking callbacks
typedef struct {
    IOSAuthResult* result;
    dispatch_semaphore_t semaphore;
} BlockingAuthContext;

typedef struct {
    int result;
    dispatch_semaphore_t semaphore;
} BlockingCredentialContext;

// Global storage for blocking contexts (not thread-safe for multiple concurrent calls)
static BlockingAuthContext* g_blockingAuthContext = NULL;
static BlockingCredentialContext* g_blockingCredentialContext = NULL;

// C function callback for blocking auth
static void blocking_auth_callback(IOSAuthResult* authResult) {
    if (g_blockingAuthContext) {
        g_blockingAuthContext->result = authResult;
        dispatch_semaphore_signal(g_blockingAuthContext->semaphore);
    }
}

// C function callback for blocking credential state
static void blocking_credential_callback(int state) {
    if (g_blockingCredentialContext) {
        g_blockingCredentialContext->result = state;
        dispatch_semaphore_signal(g_blockingCredentialContext->semaphore);
    }
}

// Legacy blocking version - now uses async internally with a semaphore
// WARNING: This should only be called from background threads, never from main/UI thread
IOSAuthResult* ios_apple_sign_in(const void* unused_callback) {
    BlockingAuthContext context = {
        .result = NULL,
        .semaphore = dispatch_semaphore_create(0)
    };
    
    // Store context for the callback
    g_blockingAuthContext = &context;
    
    // Use the async version with a C function callback
    ios_apple_sign_in_async(blocking_auth_callback);
    
    // Wait for completion
    dispatch_semaphore_wait(context.semaphore, DISPATCH_TIME_FOREVER);
    
    g_blockingAuthContext = NULL;
    return context.result;
}

void ios_free_auth_result(IOSAuthResult* result) {
    if (result) {
        if (result->user_id) free((void*)result->user_id);
        if (result->email) free((void*)result->email);
        if (result->given_name) free((void*)result->given_name);
        if (result->family_name) free((void*)result->family_name);
        if (result->error_message) free((void*)result->error_message);
        free(result);
    }
}

bool ios_is_apple_signin_available(void) {
    if (@available(iOS 13.0, *)) {
        return true;
    }
    return false;
}

// Async version for credential state check
void ios_get_credential_state_async(const char* user_id, void (*callback)(int)) {
    if (!user_id) {
        dispatch_async(g_authCallbackQueue, ^{
            callback(2); // NotFound
        });
        return;
    }
    
    if (@available(iOS 13.0, *)) {
        NSString *userIdString = [NSString stringWithUTF8String:user_id];
        ASAuthorizationAppleIDProvider *appleIDProvider = [[ASAuthorizationAppleIDProvider alloc] init];
        
        [appleIDProvider getCredentialStateForUserID:userIdString completion:^(ASAuthorizationAppleIDProviderCredentialState state, NSError * _Nullable error) {
            int credentialState;
            switch (state) {
                case ASAuthorizationAppleIDProviderCredentialRevoked:
                    credentialState = 0; // Revoked
                    break;
                case ASAuthorizationAppleIDProviderCredentialAuthorized:
                    credentialState = 1; // Authorized
                    break;
                case ASAuthorizationAppleIDProviderCredentialNotFound:
                    credentialState = 2; // NotFound
                    break;
                case ASAuthorizationAppleIDProviderCredentialTransferred:
                    credentialState = 3; // Transferred
                    break;
                default:
                    credentialState = -1; // Unknown
                    break;
            }
            
            // Call back on the auth callback queue
            dispatch_async(g_authCallbackQueue, ^{
                callback(credentialState);
            });
        }];
    } else {
        dispatch_async(g_authCallbackQueue, ^{
            callback(2); // NotFound
        });
    }
}

// Legacy blocking version - WARNING: Don't call from main thread
int ios_get_credential_state(const char* user_id) {
    BlockingCredentialContext context = {
        .result = 2, // Default to NotFound
        .semaphore = dispatch_semaphore_create(0)
    };
    
    // Store context for the callback
    g_blockingCredentialContext = &context;
    
    ios_get_credential_state_async(user_id, blocking_credential_callback);
    
    // Wait for completion with timeout
    dispatch_semaphore_wait(context.semaphore, dispatch_time(DISPATCH_TIME_NOW, 5 * NSEC_PER_SEC));
    
    g_blockingCredentialContext = NULL;
    return context.result;
}