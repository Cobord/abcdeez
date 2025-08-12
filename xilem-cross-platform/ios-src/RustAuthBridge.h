//
//  RustAuthBridge.h
//  ABCDEEZ
//
//  C Bridge between Rust and iOS Authentication
//

#ifndef RustAuthBridge_h
#define RustAuthBridge_h

#include <stdbool.h>

// Structure matching the Rust IOSAuthResult
typedef struct {
    bool success;
    const char* user_id;
    const char* email;
    const char* given_name;
    const char* family_name;
    bool is_private_email;
    const char* error_message;
} IOSAuthResult;

// Callback types for async operations
typedef void (*AuthCompletionCallback)(IOSAuthResult*);
typedef void (*CredentialStateCallback)(int);

// Functions exported to Rust
IOSAuthResult* ios_apple_sign_in(const void* callback);  // Legacy blocking version - don't use from UI thread
void ios_apple_sign_in_async(AuthCompletionCallback callback);  // Preferred async version
void ios_free_auth_result(IOSAuthResult* result);
bool ios_is_apple_signin_available(void);
int ios_get_credential_state(const char* user_id);  // Legacy blocking version - don't use from UI thread
void ios_get_credential_state_async(const char* user_id, CredentialStateCallback callback);  // Preferred async version

#endif /* RustAuthBridge_h */