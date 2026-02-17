//! Platform abstraction for screenshot capture.
//!
//! The `CaptureBridge` trait defines the interface for platform-specific
//! screenshot capture operations.

use super::error::Result;

/// Platform abstraction trait for triggering screenshot capture.
///
/// This trait provides OS-specific operations for triggering the OS screenshot tool
/// programmatically. Users save screenshots directly into the session's _captures/ folder.
///
/// # Platform Implementations
///
/// - **Windows**: Multiple trigger methods (URI, process launch, key simulation).
/// - **macOS**: Uses `screencapture` CLI with output path arguments (v2)
///
/// # Thread Safety
///
/// Implementations should be `Send + Sync` to allow usage across threads.
pub trait CaptureBridge: Send + Sync {
    /// Programmatically triggers the OS screenshot tool.
    ///
    /// # Platform Behavior
    ///
    /// - **Windows**: Attempts multiple trigger methods in fallback order:
    ///   1. Launch `ms-screenclip:` URI scheme
    ///   2. Spawn `SnippingTool.exe` process
    ///   3. Simulate `Win+Shift+S` key combination via Windows API
    ///
    /// - **macOS**: Launches `screencapture -i` for interactive screenshot (v2)
    ///
    /// # Returns
    ///
    /// `Ok(())` if the screenshot tool was triggered successfully. Note that success
    /// means the tool was launched, not that the user completed the screenshot.
    ///
    /// # Errors
    ///
    /// - `PlatformError::ScreenshotTriggerError`: All trigger methods failed
    /// - `PlatformError::NotImplemented`: Platform does not support this operation (macOS v1)
    fn trigger_screenshot(&self) -> Result<()>;
}
