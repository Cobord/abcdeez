//
//  AppleLoggingBridge.m
//  ABCDEEZ
//
//  Implementation of unified logging bridge for Apple platforms
//

#import <Foundation/Foundation.h>
#import <os/log.h>
#import "AppleLoggingBridge.h"

// Global subsystem identifier
static NSString *g_subsystem = @"com.abcdeez.app";

// Log objects for different categories
static os_log_t g_log_default = NULL;
static os_log_t g_log_ui = NULL;
static os_log_t g_log_network = NULL;
static os_log_t g_log_auth = NULL;
static os_log_t g_log_learning = NULL;
static os_log_t g_log_gamification = NULL;

// Initialize log objects
static void ensure_logs_initialized(void) {
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        g_log_default = os_log_create([g_subsystem UTF8String], "default");
        g_log_ui = os_log_create([g_subsystem UTF8String], "ui");
        g_log_network = os_log_create([g_subsystem UTF8String], "network");
        g_log_auth = os_log_create([g_subsystem UTF8String], "auth");
        g_log_learning = os_log_create([g_subsystem UTF8String], "learning");
        g_log_gamification = os_log_create([g_subsystem UTF8String], "gamification");
    });
}

// Get the appropriate log object based on target
static os_log_t get_log_for_target(const char* target) {
    ensure_logs_initialized();
    
    if (!target) {
        return g_log_default;
    }
    
    NSString *targetStr = [NSString stringWithUTF8String:target];
    
    // Route to appropriate category based on module/target name
    if ([targetStr containsString:@"auth"] || [targetStr containsString:@"ios"]) {
        return g_log_auth;
    } else if ([targetStr containsString:@"views"] || [targetStr containsString:@"components"]) {
        return g_log_ui;
    } else if ([targetStr containsString:@"network"] || [targetStr containsString:@"websocket"]) {
        return g_log_network;
    } else if ([targetStr containsString:@"learning"] || [targetStr containsString:@"adaptive"]) {
        return g_log_learning;
    } else if ([targetStr containsString:@"gamification"] || [targetStr containsString:@"achievements"]) {
        return g_log_gamification;
    }
    
    return g_log_default;
}

// Convert our log level to os_log_type_t
static os_log_type_t convert_log_level(LogLevel level) {
    switch (level) {
        case LOG_LEVEL_TRACE:
        case LOG_LEVEL_DEBUG:
            return OS_LOG_TYPE_DEBUG;
        case LOG_LEVEL_INFO:
            return OS_LOG_TYPE_INFO;
        case LOG_LEVEL_WARN:
            return OS_LOG_TYPE_DEFAULT;  // Default is more visible than Info
        case LOG_LEVEL_ERROR:
            return OS_LOG_TYPE_ERROR;
        default:
            return OS_LOG_TYPE_DEFAULT;
    }
}

// Main logging function
void apple_log(
    LogLevel level,
    const char* target,
    const char* message,
    const char* file,
    int line
) {
    if (!message) return;
    
    os_log_t log = get_log_for_target(target);
    os_log_type_t log_type = convert_log_level(level);
    
    // Include file and line info for debug builds
#ifdef DEBUG
    if (file && line > 0) {
        // Extract just the filename from the path
        NSString *filePath = [NSString stringWithUTF8String:file];
        NSString *fileName = [filePath lastPathComponent];
        
        os_log_with_type(log, log_type, "[%{public}s:%d] %{public}s", 
                         [fileName UTF8String], line, message);
    } else {
        os_log_with_type(log, log_type, "%{public}s", message);
    }
#else
    // In release builds, don't include file/line info
    os_log_with_type(log, log_type, "%{public}s", message);
#endif
}

// Initialize the logging system
void apple_log_init(const char* subsystem) {
    if (subsystem) {
        g_subsystem = [NSString stringWithUTF8String:subsystem];
    }
    ensure_logs_initialized();
    
    // Log initialization
    os_log_with_type(g_log_default, OS_LOG_TYPE_INFO, 
                     "🦀 ABCDEEZ Logging initialized for subsystem: %{public}s", 
                     [g_subsystem UTF8String]);
}

// Check if a log level is enabled
bool apple_log_enabled(LogLevel level) {
    ensure_logs_initialized();
    
    // In debug builds, enable all levels
#ifdef DEBUG
    return true;
#else
    // In release builds, only enable info and above
    return level >= LOG_LEVEL_INFO;
#endif
}