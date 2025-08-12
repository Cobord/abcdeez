// Simple test file to verify linking
#include "RustAuthBridge.h"
#include <stdio.h>

void test_ios_auth() {
    // Test that functions are available
    bool available = ios_is_apple_signin_available();
    printf("Apple Sign In available: %s\n", available ? "YES" : "NO");
    
    // Test credential state (should return NotFound for invalid user)
    int state = ios_get_credential_state("test_user");
    printf("Credential state for test_user: %d\n", state);
}