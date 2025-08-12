// Apple platform (iOS/macOS) logging integration
use std::ffi::CString;
use std::os::raw::c_char;
use tracing::{Event, Level, Metadata, Subscriber};
use tracing_subscriber::{layer::Context, Layer};

// FFI bindings to the native logging bridge
#[repr(C)]
#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

extern "C" {
    fn apple_log(
        level: LogLevel,
        target: *const c_char,
        message: *const c_char,
        file: *const c_char,
        line: i32,
    );
    
    fn apple_log_init(subsystem: *const c_char);
    
    fn apple_log_enabled(level: LogLevel) -> bool;
}

/// Convert tracing Level to our LogLevel
fn convert_level(level: &Level) -> LogLevel {
    match *level {
        Level::TRACE => LogLevel::Trace,
        Level::DEBUG => LogLevel::Debug,
        Level::INFO => LogLevel::Info,
        Level::WARN => LogLevel::Warn,
        Level::ERROR => LogLevel::Error,
    }
}

/// A tracing layer that forwards to Apple's unified logging system
pub struct AppleLoggingLayer;

impl<S> Layer<S> for AppleLoggingLayer
where
    S: Subscriber,
{
    fn enabled(&self, metadata: &Metadata<'_>, _ctx: Context<'_, S>) -> bool {
        let level = convert_level(metadata.level());
        unsafe { apple_log_enabled(level) }
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Extract event metadata
        let metadata = event.metadata();
        let level = convert_level(metadata.level());
        
        // Get target (module path)
        let target = CString::new(metadata.target()).unwrap_or_else(|_| CString::new("").unwrap());
        
        // Get file and line info
        let (file_cstr, line) = if let (Some(file), Some(line)) = (metadata.file(), metadata.line()) {
            let file_cstr = CString::new(file).unwrap_or_else(|_| CString::new("").unwrap());
            (Some(file_cstr), line as i32)
        } else {
            (None, 0)
        };
        
        // Format the message using a visitor
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        
        let message = CString::new(visitor.message).unwrap_or_else(|_| CString::new("").unwrap());
        
        // Log to Apple's system
        unsafe {
            apple_log(
                level,
                target.as_ptr(),
                message.as_ptr(),
                file_cstr.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
                line,
            );
        }
    }
}

/// Visitor to extract the message from an event
#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            if !self.message.is_empty() {
                self.message.push_str(", ");
            }
            self.message.push_str(&format!("{}={:?}", field.name(), value));
        }
    }
    
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            if !self.message.is_empty() {
                self.message.push_str(", ");
            }
            self.message.push_str(&format!("{}=\"{}\"", field.name(), value));
        }
    }
}

/// Initialize Apple platform logging
pub fn init_apple_logging() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;
    
    // Initialize the native logging system
    let subsystem = CString::new("com.abcdeez.app").unwrap();
    unsafe {
        apple_log_init(subsystem.as_ptr());
    }
    
    // Set up the tracing subscriber with our Apple logging layer
    let apple_layer = AppleLoggingLayer;
    // Default filter: info, but silence noisy winit iOS AboutToWait warnings and our app_state echo
    let filter = if let Ok(spec) = std::env::var("RUST_LOG") {
        EnvFilter::new(spec)
    } else {
        EnvFilter::new(
            concat!(
                "info,",
                // silence iOS app state spam
                "winit::platform_impl::ios::app_state=off,",
                // silence iOS window IME warnings
                "winit::platform_impl::ios::window=off,",
                // silence window event spam from masonry_winit when app is suspended or before window creation
                "masonry_winit::event_loop_runner=off,",
                // silence any app_state echo in our module if it appears
                "abcdeez_app::state::app_state=off"
            )
        )
    };
    
    // Also add a fmt layer for development (prints to stdout in debug builds)
    #[cfg(debug_assertions)]
    {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_file(true)
            .with_line_number(true);
        
        tracing_subscriber::registry()
            .with(filter)
            .with(apple_layer)
            .with(fmt_layer)
            .init();
    }
    
    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry()
            .with(filter)
            .with(apple_layer)
            .init();
    }
}