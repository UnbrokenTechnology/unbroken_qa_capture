//! Windows platform implementation stubs.
//!
//! This module provides stub implementations of the platform abstraction traits
//! for Windows. These stubs return placeholder errors or perform no-ops, allowing
//! the application to compile and run without actual Windows-specific functionality.
//!
//! # Implementation Status
//!
//! - **CaptureBridge**: Stub implementation (returns errors for all operations)
//! - **RegistryBridge**: Stub implementation (returns errors for registry operations)
//!
//! These stubs will be replaced with actual Windows implementations in future tickets.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use super::capture::{CaptureBridge, CaptureEvent, WatcherHandle};
use super::registry::RegistryBridge;
use super::error::{PlatformError, Result};

/// Windows implementation stub for `CaptureBridge`.
///
/// This stub implementation allows the application to compile and run on Windows
/// but does not provide actual screenshot capture or file watching functionality.
/// All methods return appropriate errors indicating they are not yet implemented.
pub struct WindowsCaptureBridge {
    // Placeholder for future state (e.g., active watchers, registry handles)
}

impl WindowsCaptureBridge {
    /// Creates a new Windows capture bridge stub.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WindowsCaptureBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureBridge for WindowsCaptureBridge {
    fn redirect_screenshot_output(&self, _target_folder: &Path) -> Result<PathBuf> {
        Err(PlatformError::NotImplemented {
            operation: "redirect_screenshot_output".to_string(),
            platform: "Windows (stub)".to_string(),
        })
    }

    fn restore_screenshot_output(&self, _original_path: &Path) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "restore_screenshot_output".to_string(),
            platform: "Windows (stub)".to_string(),
        })
    }

    fn trigger_screenshot(&self) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "trigger_screenshot".to_string(),
            platform: "Windows (stub)".to_string(),
        })
    }

    fn start_file_watcher(&self, _folder: &Path, _sender: Sender<CaptureEvent>) -> Result<WatcherHandle> {
        Err(PlatformError::NotImplemented {
            operation: "start_file_watcher".to_string(),
            platform: "Windows (stub)".to_string(),
        })
    }

    fn stop_file_watcher(&self, _handle: WatcherHandle) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "stop_file_watcher".to_string(),
            platform: "Windows (stub)".to_string(),
        })
    }
}

/// Windows implementation stub for `RegistryBridge`.
///
/// This stub implementation allows the application to compile and run on Windows
/// but does not provide actual registry operations. All methods return appropriate
/// errors indicating they are not yet implemented.
pub struct WindowsRegistryBridge {
    // Placeholder for future state (e.g., registry key handles, cached values)
}

impl WindowsRegistryBridge {
    /// Creates a new Windows registry bridge stub.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WindowsRegistryBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryBridge for WindowsRegistryBridge {
    fn read_screenshot_folder(&self) -> Result<PathBuf> {
        Err(PlatformError::NotImplemented {
            operation: "read_screenshot_folder".to_string(),
            platform: "Windows (stub)".to_string(),
        })
    }

    fn write_screenshot_folder(&self, _folder: &Path) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "write_screenshot_folder".to_string(),
            platform: "Windows (stub)".to_string(),
        })
    }

    fn restore_screenshot_folder(&self, _original_folder: &Path) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "restore_screenshot_folder".to_string(),
            platform: "Windows (stub)".to_string(),
        })
    }

    fn detect_and_restore_stale_redirects(&self) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "detect_and_restore_stale_redirects".to_string(),
            platform: "Windows (stub)".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn test_windows_capture_bridge_returns_not_implemented() {
        let bridge = WindowsCaptureBridge::new();
        let temp_path = PathBuf::from("C:\\temp");

        // Test redirect_screenshot_output
        let result = bridge.redirect_screenshot_output(&temp_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, platform } => {
                assert_eq!(operation, "redirect_screenshot_output");
                assert!(platform.contains("Windows"));
            }
            _ => panic!("Expected NotImplemented error"),
        }

        // Test restore_screenshot_output
        let result = bridge.restore_screenshot_output(&temp_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, .. } => {
                assert_eq!(operation, "restore_screenshot_output");
            }
            _ => panic!("Expected NotImplemented error"),
        }

        // Test trigger_screenshot
        let result = bridge.trigger_screenshot();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, .. } => {
                assert_eq!(operation, "trigger_screenshot");
            }
            _ => panic!("Expected NotImplemented error"),
        }

        // Test start_file_watcher
        let (tx, _rx) = channel();
        let result = bridge.start_file_watcher(&temp_path, tx);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, .. } => {
                assert_eq!(operation, "start_file_watcher");
            }
            _ => panic!("Expected NotImplemented error"),
        }
    }

    #[test]
    fn test_windows_registry_bridge_returns_not_implemented() {
        let bridge = WindowsRegistryBridge::new();
        let temp_path = PathBuf::from("C:\\temp");

        // Test read_screenshot_folder
        let result = bridge.read_screenshot_folder();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, platform } => {
                assert_eq!(operation, "read_screenshot_folder");
                assert!(platform.contains("Windows"));
            }
            _ => panic!("Expected NotImplemented error"),
        }

        // Test write_screenshot_folder
        let result = bridge.write_screenshot_folder(&temp_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, .. } => {
                assert_eq!(operation, "write_screenshot_folder");
            }
            _ => panic!("Expected NotImplemented error"),
        }

        // Test restore_screenshot_folder
        let result = bridge.restore_screenshot_folder(&temp_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, .. } => {
                assert_eq!(operation, "restore_screenshot_folder");
            }
            _ => panic!("Expected NotImplemented error"),
        }

        // Test detect_and_restore_stale_redirects
        let result = bridge.detect_and_restore_stale_redirects();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, .. } => {
                assert_eq!(operation, "detect_and_restore_stale_redirects");
            }
            _ => panic!("Expected NotImplemented error"),
        }
    }
}
