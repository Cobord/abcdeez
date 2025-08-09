#import <Foundation/Foundation.h>
#import "AppleSignInBridge.h"

// Rust FFI exports  
void main_rs(void);
void handle_apple_sign_in_callback(const char* json_data, bool success);
