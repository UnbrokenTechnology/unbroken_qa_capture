//! Platform abstraction for Windows registry operations.
//!
//! The `RegistryBridge` trait defines the interface for Windows registry
//! read/write operations with crash-safe restoration. On non-Windows platforms,
//! implementations return `NotImplemented` errors.

use std::path::{Path, PathBuf};

use super::error::Result;

/// Platform abstraction trait for Windows registry operations.
///
/// This trait provides Windows-specific registry operations for redirecting
/// the Snipping Tool screenshot output folder. It includes crash-safe restoration
/// mechanisms to ensure the registry is always returned to its original state.
///
/// # Platform Implementations
///
/// - **Windows**: Full implementation using Windows Registry API (HKCU access, no admin required)
/// - **macOS**: Stub implementation returning `NotImplemented` errors (v2)
///
/// # Registry Key
///
/// The Windows implementation modifies:
/// ```text
/// HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders\{B7BEDE81-DF94-4682-A7D8-57A52620B86F}
/// ```
///
/// This is the user-level Snipping Tool screenshot folder key. No administrator
/// privileges are required for HKCU operations.
///
/// # Crash Safety
///
/// The registry must **always** be restored to its original value, even if the
/// application crashes. Implementations should:
///
/// 1. Cache the original value in persistent storage (SQLite) before modification
/// 2. Implement a `Drop` guard that restores the registry on object destruction
/// 3. Check for stale redirects on app startup and restore them
///
/// # Thread Safety
///
/// Implementations should be `Send + Sync` to allow usage across threads.
///
/// # Example
///
/// ```no_run
/// use unbroken_qa_capture_lib::platform::{RegistryBridge, get_registry_bridge};
/// use std::path::Path;
///
/// let bridge = get_registry_bridge();
/// let target = Path::new("C:\\Users\\Tester\\QA\\Session1");
///
/// // Read current value
/// let original = bridge.read_screenshot_folder()?;
///
/// // Redirect to session folder
/// bridge.write_screenshot_folder(target)?;
///
/// // ... session active ...
///
/// // Restore original value
/// bridge.restore_screenshot_folder(&original)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait RegistryBridge: Send + Sync {
    /// Reads the current Snipping Tool screenshot output folder from the registry.
    ///
    /// # Platform Behavior
    ///
    /// - **Windows**: Reads the value from
    ///   `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders\{B7BEDE81-DF94-4682-A7D8-57A52620B86F}`
    ///
    /// - **macOS**: Returns `NotImplemented` error (registry does not exist)
    ///
    /// # Returns
    ///
    /// The absolute path to the current screenshot folder. On Windows, this is typically
    /// something like `%USERPROFILE%\Pictures\Screenshots` or an absolute path if already
    /// redirected.
    ///
    /// # Errors
    ///
    /// - `PlatformError::RegistryError`: Registry key does not exist or cannot be read
    /// - `PlatformError::NotImplemented`: Platform does not support registry operations (macOS)
    ///
    /// # Implementation Note
    ///
    /// The registry value may contain environment variables (e.g., `%USERPROFILE%`).
    /// Implementations should expand these to absolute paths before returning.
    fn read_screenshot_folder(&self) -> Result<PathBuf>;

    /// Writes a new screenshot output folder path to the registry.
    ///
    /// # Platform Behavior
    ///
    /// - **Windows**: Writes the absolute path to
    ///   `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders\{B7BEDE81-DF94-4682-A7D8-57A52620B86F}`
    ///
    /// - **macOS**: Returns `NotImplemented` error
    ///
    /// # Arguments
    ///
    /// * `folder` - Absolute path to the new screenshot folder. Must be a valid,
    ///   existing directory with write permissions.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the registry was updated successfully. The change takes effect
    /// immediately for new Snipping Tool invocations.
    ///
    /// # Errors
    ///
    /// - `PlatformError::InvalidArgument`: Path is not absolute, does not exist, or is not writable
    /// - `PlatformError::RegistryError`: Registry write operation failed (permissions, key not found)
    /// - `PlatformError::NotImplemented`: Platform does not support registry operations (macOS)
    ///
    /// # Crash Safety
    ///
    /// Before calling this method, implementations should cache the original registry
    /// value in persistent storage (SQLite) to enable restoration if the app crashes
    /// before `restore_screenshot_folder()` is called.
    fn write_screenshot_folder(&self, folder: &Path) -> Result<()>;

    /// Restores the screenshot output folder to a previous value.
    ///
    /// # Platform Behavior
    ///
    /// - **Windows**: Writes the provided path back to the registry key. This is
    ///   the inverse of `write_screenshot_folder()`.
    ///
    /// - **macOS**: Returns `Ok(())` (no-op)
    ///
    /// # Arguments
    ///
    /// * `original_folder` - The path to restore, typically obtained from
    ///   `read_screenshot_folder()` before modification.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the registry was restored successfully.
    ///
    /// # Errors
    ///
    /// - `PlatformError::RegistryError`: Registry write operation failed
    /// - `PlatformError::NotImplemented`: Platform does not support registry operations (macOS)
    ///
    /// # Implementation Note
    ///
    /// This method should be called:
    /// 1. On session end (normal shutdown)
    /// 2. In a `Drop` guard (crash recovery)
    /// 3. On app startup if stale redirects are detected
    ///
    /// It should be idempotent - safe to call multiple times with the same value.
    fn restore_screenshot_folder(&self, original_folder: &Path) -> Result<()>;

    /// Detects and restores stale screenshot folder redirects from a previous session.
    ///
    /// This method is called on app startup to detect if the previous session crashed
    /// before restoring the registry. It checks persistent storage (SQLite) for cached
    /// original values and restores them if the current registry value differs.
    ///
    /// # Platform Behavior
    ///
    /// - **Windows**: Checks SQLite for cached redirects, compares with current registry
    ///   value, and restores if mismatched. Clears the cache after successful restoration.
    ///
    /// - **macOS**: Returns `Ok(())` (no-op)
    ///
    /// # Returns
    ///
    /// `Ok(())` if no stale redirects were found, or if they were restored successfully.
    ///
    /// # Errors
    ///
    /// - `PlatformError::RegistryError`: Failed to read or write registry
    /// - `PlatformError::FileSystemError`: Failed to read persistent storage
    /// - `PlatformError::NotImplemented`: Platform does not support registry operations (macOS)
    ///
    /// # Implementation Note
    ///
    /// This should be called during app initialization, before any sessions are started.
    /// It ensures that registry state is clean even if the previous app instance was
    /// killed (e.g., via Task Manager) before restoration.
    fn detect_and_restore_stale_redirects(&self) -> Result<()>;
}
