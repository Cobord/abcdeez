//
//  AppleLoggingBridge.h
//  ABCDEEZ
//
//  Bridge for unified logging on Apple platforms (iOS/macOS)
//

#ifndef AppleLoggingBridge_h
#define AppleLoggingBridge_h

#include <stdbool.h>

// Log levels matching Rust's tracing levels
typedef enum {
    LOG_LEVEL_TRACE = 0,
    LOG_LEVEL_DEBUG = 1,
    LOG_LEVEL_INFO = 2,
    LOG_LEVEL_WARN = 3,
    LOG_LEVEL_ERROR = 4,
} LogLevel;

// Log a message to the Apple unified logging system
void apple_log(
    LogLevel level,
    const char* target,
    const char* message,
    const char* file,
    int line
);

// Initialize the logging system with a subsystem identifier
void apple_log_init(const char* subsystem);

// Check if a log level is enabled
bool apple_log_enabled(LogLevel level);

#endif /* AppleLoggingBridge_h */