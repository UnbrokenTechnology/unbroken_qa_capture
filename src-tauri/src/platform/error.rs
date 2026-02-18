//! Error types for platform-specific operations.

use std::fmt;

/// Result type for platform operations.
pub type Result<T> = std::result::Result<T, PlatformError>;

/// Errors that can occur during platform-specific operations.
#[derive(Debug, Clone)]
pub enum PlatformError {
    /// The requested operation is not implemented on this platform.
    ///
    /// This is typically returned by macOS stub implementations in v1,
    /// indicating the functionality will be added in v2.
    NotImplemented {
        operation: String,
        platform: String,
    },

    /// Registry operation failed (Windows-specific).
    RegistryError {
        key: String,
        operation: String,
        message: String,
    },

    /// File system operation failed.
    FileSystemError {
        path: String,
        operation: String,
        message: String,
    },

    /// Screenshot trigger operation failed.
    ScreenshotTriggerError {
        method: String,
        message: String,
    },

    /// Invalid path or argument provided.
    InvalidArgument {
        parameter: String,
        message: String,
    },

    /// Generic platform error for uncategorized failures.
    Other {
        message: String,
    },
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::NotImplemented { operation, platform } => {
                write!(f, "Operation '{}' not implemented on {}", operation, platform)
            }
            PlatformError::RegistryError { key, operation, message } => {
                write!(f, "Registry {} failed for key '{}': {}", operation, key, message)
            }
            PlatformError::FileSystemError { path, operation, message } => {
                write!(f, "File system {} failed for path '{}': {}", operation, path, message)
            }
            PlatformError::ScreenshotTriggerError { method, message } => {
                write!(f, "Screenshot trigger '{}' failed: {}", method, message)
            }
            PlatformError::InvalidArgument { parameter, message } => {
                write!(f, "Invalid argument '{}': {}", parameter, message)
            }
            PlatformError::Other { message } => {
                write!(f, "Platform error: {}", message)
            }
        }
    }
}

impl std::error::Error for PlatformError {}

impl From<std::io::Error> for PlatformError {
    fn from(err: std::io::Error) -> Self {
        PlatformError::FileSystemError {
            path: String::new(),
            operation: "io".to_string(),
            message: err.to_string(),
        }
    }
}
