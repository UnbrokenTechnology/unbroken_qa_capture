//! Platform abstraction layer for OS-specific operations.
//!
//! This module provides trait-based abstractions for platform-specific functionality,
//! enabling cross-platform development while maintaining clean separation between
//! platform-independent and platform-specific code.
//!
//! # Platform Support
//!
//! - **Windows 11**: Full implementation (v1)
//! - **macOS**: Stubbed implementations returning `NotImplemented` errors (v2 planned)
//!
//! # Architecture
//!
//! The platform layer uses Rust traits to define contracts for platform-specific operations:
//! - `CaptureBridge`: Screenshot capture, file watching, and system integration
//! - `RegistryBridge`: Windows registry operations with crash-safe restore
//!
//! Platform-specific implementations are selected at compile time using `cfg` attributes.

mod capture;
mod registry;
pub(crate) mod registry_cache;
mod error;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

// Re-export public types
pub use capture::{CaptureBridge, CaptureEvent, WatcherHandle};
pub use registry::RegistryBridge;
pub use error::{PlatformError, Result};

/// Returns the platform-specific `CaptureBridge` implementation for the current OS.
///
/// # Platform Selection
///
/// - **Windows**: Returns `WindowsCaptureBridge` with Snipping Tool integration
/// - **macOS**: Returns `MacCaptureBridge` with stub implementations
/// - **Other**: Compile error (unsupported platform)
///
/// # Example
///
/// ```no_run
/// use unbroken_qa_capture_lib::platform::get_capture_bridge;
///
/// let bridge = get_capture_bridge();
/// // Use bridge methods...
/// ```
#[cfg(target_os = "windows")]
pub fn get_capture_bridge() -> Box<dyn CaptureBridge> {
    Box::new(windows::WindowsCaptureBridge::new())
}

#[cfg(target_os = "macos")]
pub fn get_capture_bridge() -> Box<dyn CaptureBridge> {
    Box::new(macos::MacCaptureBridge::new())
}

/// Returns the platform-specific `RegistryBridge` implementation for the current OS.
///
/// # Platform Selection
///
/// - **Windows**: Returns `WindowsRegistryBridge` for HKCU operations
/// - **macOS**: Returns `MacRegistryBridge` with stub implementations
/// - **Other**: Compile error (unsupported platform)
///
/// # Example
///
/// ```no_run
/// use unbroken_qa_capture_lib::platform::get_registry_bridge;
///
/// let bridge = get_registry_bridge();
/// // Use bridge methods...
/// ```
#[cfg(target_os = "windows")]
pub fn get_registry_bridge() -> Box<dyn RegistryBridge> {
    Box::new(windows::WindowsRegistryBridge::new())
}

#[cfg(target_os = "macos")]
pub fn get_registry_bridge() -> Box<dyn RegistryBridge> {
    Box::new(macos::MacRegistryBridge::new())
}
