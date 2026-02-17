//! macOS platform implementation stubs.
//!
//! This module provides stub implementations of the platform abstraction traits
//! for macOS. All methods return `NotImplemented` errors, as macOS support is
//! planned for v2 of the application.
//!
//! # Implementation Status
//!
//! - **CaptureBridge**: Stub implementation (returns `NotImplemented` for all operations)
//! - **RegistryBridge**: Stub implementation (returns `NotImplemented` for all operations)
//!
//! # Future Implementation (v2)
//!
//! When macOS support is added, this module will be replaced with actual implementations:
//! - `CaptureBridge` will use `screencapture -i` CLI for interactive screenshots
//! - `RegistryBridge` will be a no-op (macOS does not have a Windows-style registry)

use std::path::{Path, PathBuf};

use super::capture::CaptureBridge;
use super::registry::RegistryBridge;
use super::error::{PlatformError, Result};

/// macOS stub implementation for `CaptureBridge`.
///
/// This stub implementation returns `NotImplemented` errors for all operations.
/// It allows the application to compile on macOS but does not provide actual
/// screenshot capture or file watching functionality.
pub struct MacCaptureBridge {
    // Placeholder for future state
}

impl MacCaptureBridge {
    /// Creates a new macOS capture bridge stub.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MacCaptureBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureBridge for MacCaptureBridge {
    fn trigger_screenshot(&self) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "trigger_screenshot".to_string(),
            platform: "macOS".to_string(),
        })
    }
}

/// macOS stub implementation for `RegistryBridge`.
///
/// This stub implementation returns `NotImplemented` errors for all operations.
/// macOS does not have a Windows-style registry, so this trait is not applicable
/// to the platform. The stub exists for code consistency and to allow compilation.
pub struct MacRegistryBridge {
    // Placeholder for future state (none needed - macOS has no registry)
}

impl MacRegistryBridge {
    /// Creates a new macOS registry bridge stub.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MacRegistryBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryBridge for MacRegistryBridge {
    fn read_screenshot_folder(&self) -> Result<PathBuf> {
        Err(PlatformError::NotImplemented {
            operation: "read_screenshot_folder".to_string(),
            platform: "macOS".to_string(),
        })
    }

    fn write_screenshot_folder(&self, _folder: &Path) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "write_screenshot_folder".to_string(),
            platform: "macOS".to_string(),
        })
    }

    fn restore_screenshot_folder(&self, _original_folder: &Path) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "restore_screenshot_folder".to_string(),
            platform: "macOS".to_string(),
        })
    }

    fn detect_and_restore_stale_redirects(&self) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "detect_and_restore_stale_redirects".to_string(),
            platform: "macOS".to_string(),
        })
    }
}

/// macOS platform stub implementation
#[allow(dead_code)]
pub struct MacPlatform;

impl super::Platform for MacPlatform {
    fn enable_startup(&self) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "enable_startup".to_string(),
            platform: "macOS".to_string(),
        })
    }

    fn disable_startup(&self) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "disable_startup".to_string(),
            platform: "macOS".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_capture_bridge_returns_not_implemented() {
        let bridge = MacCaptureBridge::new();

        // Test trigger_screenshot
        let result = bridge.trigger_screenshot();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, platform } => {
                assert_eq!(operation, "trigger_screenshot");
                assert_eq!(platform, "macOS");
            }
            _ => panic!("Expected NotImplemented error"),
        }
    }

    #[test]
    fn test_macos_registry_bridge_returns_not_implemented() {
        let bridge = MacRegistryBridge::new();
        let temp_path = PathBuf::from("/tmp/test");

        // Test read_screenshot_folder
        let result = bridge.read_screenshot_folder();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, platform } => {
                assert_eq!(operation, "read_screenshot_folder");
                assert_eq!(platform, "macOS");
            }
            _ => panic!("Expected NotImplemented error"),
        }

        // Test write_screenshot_folder
        let result = bridge.write_screenshot_folder(&temp_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, platform } => {
                assert_eq!(operation, "write_screenshot_folder");
                assert_eq!(platform, "macOS");
            }
            _ => panic!("Expected NotImplemented error"),
        }

        // Test restore_screenshot_folder
        let result = bridge.restore_screenshot_folder(&temp_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, platform } => {
                assert_eq!(operation, "restore_screenshot_folder");
                assert_eq!(platform, "macOS");
            }
            _ => panic!("Expected NotImplemented error"),
        }

        // Test detect_and_restore_stale_redirects
        let result = bridge.detect_and_restore_stale_redirects();
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::NotImplemented { operation, platform } => {
                assert_eq!(operation, "detect_and_restore_stale_redirects");
                assert_eq!(platform, "macOS");
            }
            _ => panic!("Expected NotImplemented error"),
        }
    }

    #[test]
    fn test_macos_bridges_default_constructors() {
        let _capture_bridge = MacCaptureBridge::default();
        let _registry_bridge = MacRegistryBridge::default();
        // Just verify they can be constructed
    }
}
