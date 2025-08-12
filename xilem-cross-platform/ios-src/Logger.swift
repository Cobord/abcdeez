//
//  Logger.swift
//  ABCDEEZ
//
//  Swift wrapper for unified logging
//

import Foundation
import os.log

/// Swift-friendly logger wrapper
@objc public class ABCDEEZLogger: NSObject {
    
    /// Shared instance
    @objc public static let shared = ABCDEEZLogger()
    
    private override init() {
        super.init()
    }
    
    /// Log levels matching Rust's tracing
    @objc public enum Level: Int {
        case trace = 0
        case debug = 1
        case info = 2
        case warn = 3
        case error = 4
    }
    
    /// Log a message
    @objc public func log(
        _ level: Level,
        _ message: String,
        file: String = #file,
        line: Int = #line
    ) {
        let fileName = (file as NSString).lastPathComponent
        apple_log(
            LogLevel(rawValue: UInt32(level.rawValue)),
            fileName.cString(using: .utf8),
            message.cString(using: .utf8),
            fileName.cString(using: .utf8),
            Int32(line)
        )
    }
    
    /// Convenience methods
    @objc public func trace(_ message: String, file: String = #file, line: Int = #line) {
        log(.trace, message, file: file, line: line)
    }
    
    @objc public func debug(_ message: String, file: String = #file, line: Int = #line) {
        log(.debug, message, file: file, line: line)
    }
    
    @objc public func info(_ message: String, file: String = #file, line: Int = #line) {
        log(.info, message, file: file, line: line)
    }
    
    @objc public func warn(_ message: String, file: String = #file, line: Int = #line) {
        log(.warn, message, file: file, line: line)
    }
    
    @objc public func error(_ message: String, file: String = #file, line: Int = #line) {
        log(.error, message, file: file, line: line)
    }
}

/// Global convenience functions
public func logTrace(_ message: String, file: String = #file, line: Int = #line) {
    ABCDEEZLogger.shared.trace(message, file: file, line: line)
}

public func logDebug(_ message: String, file: String = #file, line: Int = #line) {
    ABCDEEZLogger.shared.debug(message, file: file, line: line)
}

public func logInfo(_ message: String, file: String = #file, line: Int = #line) {
    ABCDEEZLogger.shared.info(message, file: file, line: line)
}

public func logWarn(_ message: String, file: String = #file, line: Int = #line) {
    ABCDEEZLogger.shared.warn(message, file: file, line: line)
}

public func logError(_ message: String, file: String = #file, line: Int = #line) {
    ABCDEEZLogger.shared.error(message, file: file, line: line)
}