# ABCDEEZ Logging Integration

This directory contains the native logging bridge that integrates Rust's `tracing` crate with Apple's unified logging system (os_log).

## Features

- ✅ All Rust `tracing` logs appear in Console.app and Xcode console
- ✅ Structured logging with categories (ui, network, auth, learning, gamification)
- ✅ Different log levels (trace, debug, info, warn, error)
- ✅ File and line number information in debug builds
- ✅ Works on iOS, iPadOS, and macOS
- ✅ Swift wrapper for native iOS/macOS code

## How It Works

```
Rust tracing → AppleLoggingLayer → FFI → AppleLoggingBridge → os_log → Console.app
```

## Viewing Logs

### Method 1: Console.app (macOS)

1. Open Console.app
2. Click "Start Streaming"
3. Filter by subsystem: `com.abcdeez.app`
4. You can further filter by category:
   - `default` - General logs
   - `ui` - UI and view logs
   - `network` - Network and WebSocket logs
   - `auth` - Authentication logs
   - `learning` - Learning system logs
   - `gamification` - Gamification and achievements

### Method 2: Xcode Console

When running in Xcode, logs automatically appear in the debug console with proper formatting.

### Method 3: Command Line

Use the provided script to view logs in real-time:

```bash
# View all logs
./view_logs.sh

# View specific category
./view_logs.sh auth
./view_logs.sh learning
./view_logs.sh gamification
```

### Method 4: Using `log` Command

```bash
# Stream all ABCDEEZ logs
log stream --predicate 'subsystem == "com.abcdeez.app"' --level debug

# Stream specific category
log stream --predicate 'subsystem == "com.abcdeez.app" AND category == "auth"' --level debug

# Show historical logs
log show --predicate 'subsystem == "com.abcdeez.app"' --last 1h
```

## Log Levels

- **TRACE** (0) - Very detailed debugging information
- **DEBUG** (1) - Debugging information
- **INFO** (2) - Informational messages
- **WARN** (3) - Warning messages
- **ERROR** (4) - Error messages

In debug builds, all levels are enabled. In release builds, only INFO and above are enabled.

## Using from Rust

The logging is automatically initialized when the app starts. Just use the `tracing` macros:

```rust
use tracing::{trace, debug, info, warn, error};

// Simple messages
info!("App started");
debug!("Processing task");
warn!("Slow response");
error!("Connection failed");

// Structured logging
info!(
    user_id = %user.id,
    task_count = tasks.len(),
    "Session started"
);

// With spans
let span = tracing::span!(tracing::Level::INFO, "authentication");
let _enter = span.enter();
info!("Starting Apple Sign In");
```

## Using from Swift

```swift
// Using the wrapper
let logger = ABCDEEZLogger.shared
logger.info("Swift code logging")
logger.debug("Debug message")
logger.error("Error occurred")

// Using global functions
logInfo("Quick info message")
logDebug("Debug from Swift")
```

## Performance

- Logging is very efficient on Apple platforms
- Messages are only formatted if the log level is enabled
- In release builds, debug/trace logs are compiled out
- Logs are automatically compressed and rotated by the system

## Privacy

By default, all log messages are marked as public. If you need to log sensitive data:
- Consider using hashed values instead of actual data
- Use categories to separate sensitive operations
- Remember that logs can be extracted from devices

## Troubleshooting

### Logs Not Appearing

1. Check that the subsystem is correct: `com.abcdeez.app`
2. Ensure log level is appropriate (debug logs won't show in release)
3. Try clearing Console.app filters
4. Check that the bridging header includes `AppleLoggingBridge.h`

### Too Many Logs

1. Filter by category in Console.app
2. Adjust log levels in code
3. Use the predicate filters in `log` command
4. Disable trace level in release builds

### Crash or Performance Issues

1. Check for logging in tight loops
2. Avoid logging large data structures
3. Use appropriate log levels
4. Consider sampling for high-frequency events