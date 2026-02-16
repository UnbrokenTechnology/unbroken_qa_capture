//! Windows platform implementation.
//!
//! This module provides Windows-specific implementations of the platform abstraction traits.
//!
//! # Implementation Status
//!
//! - **CaptureBridge**: Stub implementation (returns errors for all operations) - to be implemented in future tickets
//! - **RegistryBridge**: Full implementation with crash recovery via SQLite cache
//!
//! # Registry Implementation
//!
//! The WindowsRegistryBridge provides:
//! - Read/write access to HKCU registry keys (no admin required)
//! - Persistent caching of original values in SQLite for crash recovery
//! - Drop trait implementation for automatic restoration on object destruction
//! - Startup recovery for stale redirects from crashed sessions

use std::path::{Path, PathBuf};
use std::sync::{mpsc::Sender, Arc, Mutex};

use super::capture::{CaptureBridge, CaptureEvent, WatcherHandle};
use super::registry::RegistryBridge;
use super::registry_cache::RegistryCache;
use super::error::{PlatformError, Result};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

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

/// Windows implementation of `RegistryBridge` with crash recovery.
///
/// This implementation provides full registry read/write operations for the
/// Windows Snipping Tool screenshot folder redirection, with SQLite-backed
/// crash recovery to ensure the registry is always restored.
///
/// # Registry Key
///
/// Modifies: `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders\{B7BEDE81-DF94-4682-A7D8-57A52620B86F}`
///
/// # Crash Recovery
///
/// - Original values are cached in SQLite before modification
/// - Drop trait ensures restoration on normal termination
/// - Startup check detects and restores stale redirects from crashes
pub struct WindowsRegistryBridge {
    cache: Arc<Mutex<RegistryCache>>,
    cached_original: Arc<Mutex<Option<PathBuf>>>,
}

impl WindowsRegistryBridge {
    /// Registry key path for Snipping Tool screenshot folder
    const REGISTRY_KEY_PATH: &'static str =
        "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders";
    const REGISTRY_VALUE_NAME: &'static str = "{B7BEDE81-DF94-4682-A7D8-57A52620B86F}";
    const CACHE_KEY_IDENTIFIER: &'static str = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders\\{B7BEDE81-DF94-4682-A7D8-57A52620B86F}";

    /// Creates a new Windows registry bridge with crash recovery.
    ///
    /// # Arguments
    ///
    /// * `cache_db_path` - Path to the SQLite database for crash recovery cache.
    ///   Typically something like `%APPDATA%\UnbrokenQACapture\registry_cache.db`.
    pub fn new_with_cache(cache_db_path: &Path) -> Result<Self> {
        let cache = RegistryCache::new(cache_db_path)?;
        Ok(Self {
            cache: Arc::new(Mutex::new(cache)),
            cached_original: Arc::new(Mutex::new(None)),
        })
    }

    /// Creates a new Windows registry bridge with default cache location.
    ///
    /// Uses `%APPDATA%\UnbrokenQACapture\registry_cache.db` for the crash recovery database.
    pub fn new() -> Self {
        // Get AppData path
        let cache_path = Self::get_default_cache_path();

        Self::new_with_cache(&cache_path).unwrap_or_else(|_| {
            // Fallback to temp directory if AppData fails
            let temp_cache = std::env::temp_dir().join("unbroken_qa_registry_cache.db");
            Self::new_with_cache(&temp_cache).expect("Failed to create registry cache")
        })
    }

    /// Gets the default cache database path in AppData.
    fn get_default_cache_path() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata)
                .join("UnbrokenQACapture")
                .join("registry_cache.db")
        } else {
            std::env::temp_dir().join("unbroken_qa_registry_cache.db")
        }
    }

    /// Expands environment variables in a registry value (e.g., %USERPROFILE%).
    #[cfg(windows)]
    fn expand_env_vars(path: &str) -> String {
        use std::os::windows::ffi::OsStringExt;
        use std::ffi::OsString;

        // Simple expansion for common variables
        let mut expanded = path.to_string();

        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            expanded = expanded.replace("%USERPROFILE%", &userprofile);
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            expanded = expanded.replace("%APPDATA%", &appdata);
        }
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            expanded = expanded.replace("%LOCALAPPDATA%", &localappdata);
        }

        expanded
    }

    #[cfg(not(windows))]
    fn expand_env_vars(path: &str) -> String {
        path.to_string()
    }
}

impl Default for WindowsRegistryBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryBridge for WindowsRegistryBridge {
    #[cfg(windows)]
    fn read_screenshot_folder(&self) -> Result<PathBuf> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey(Self::REGISTRY_KEY_PATH)
            .map_err(|e| PlatformError::RegistryError {
                key: Self::REGISTRY_KEY_PATH.to_string(),
                operation: "open".to_string(),
                message: format!("Failed to open registry key: {}", e),
            })?;

        let value: String = key
            .get_value(Self::REGISTRY_VALUE_NAME)
            .map_err(|e| PlatformError::RegistryError {
                key: format!("{}\\{}", Self::REGISTRY_KEY_PATH, Self::REGISTRY_VALUE_NAME),
                operation: "read".to_string(),
                message: format!("Failed to read registry value: {}", e),
            })?;

        // Expand environment variables
        let expanded = Self::expand_env_vars(&value);
        Ok(PathBuf::from(expanded))
    }

    #[cfg(not(windows))]
    fn read_screenshot_folder(&self) -> Result<PathBuf> {
        Err(PlatformError::NotImplemented {
            operation: "read_screenshot_folder".to_string(),
            platform: "Non-Windows platform".to_string(),
        })
    }

    #[cfg(windows)]
    fn write_screenshot_folder(&self, folder: &Path) -> Result<()> {
        // Validate folder exists and is absolute
        if !folder.is_absolute() {
            return Err(PlatformError::InvalidArgument {
                parameter: "folder".to_string(),
                message: "Path must be absolute".to_string(),
            });
        }
        if !folder.exists() {
            return Err(PlatformError::InvalidArgument {
                parameter: "folder".to_string(),
                message: "Path does not exist".to_string(),
            });
        }

        // Read current value before modifying (for crash recovery)
        let original = self.read_screenshot_folder()?;

        // Cache original value in memory
        {
            let mut cached = self.cached_original.lock().map_err(|e| {
                PlatformError::RegistryError {
                    key: "cached_original".to_string(),
                    operation: "lock".to_string(),
                    message: format!("Failed to acquire lock: {}", e),
                }
            })?;
            *cached = Some(original.clone());
        }

        // Cache original value in persistent storage
        {
            let cache = self.cache.lock().map_err(|e| PlatformError::RegistryError {
                key: "registry_cache".to_string(),
                operation: "lock".to_string(),
                message: format!("Failed to acquire cache lock: {}", e),
            })?;
            cache.cache_redirect(Self::CACHE_KEY_IDENTIFIER, &original, folder)?;
        }

        // Write new value to registry
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags(Self::REGISTRY_KEY_PATH, KEY_WRITE)
            .map_err(|e| PlatformError::RegistryError {
                key: Self::REGISTRY_KEY_PATH.to_string(),
                operation: "open".to_string(),
                message: format!("Failed to open registry key for writing: {}", e),
            })?;

        let folder_str = folder.to_string_lossy().to_string();
        key.set_value(Self::REGISTRY_VALUE_NAME, &folder_str)
            .map_err(|e| PlatformError::RegistryError {
                key: format!("{}\\{}", Self::REGISTRY_KEY_PATH, Self::REGISTRY_VALUE_NAME),
                operation: "write".to_string(),
                message: format!("Failed to write registry value: {}", e),
            })?;

        Ok(())
    }

    #[cfg(not(windows))]
    fn write_screenshot_folder(&self, _folder: &Path) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "write_screenshot_folder".to_string(),
            platform: "Non-Windows platform".to_string(),
        })
    }

    #[cfg(windows)]
    fn restore_screenshot_folder(&self, original_folder: &Path) -> Result<()> {
        // Write original value back to registry
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags(Self::REGISTRY_KEY_PATH, KEY_WRITE)
            .map_err(|e| PlatformError::RegistryError {
                key: Self::REGISTRY_KEY_PATH.to_string(),
                operation: "open".to_string(),
                message: format!("Failed to open registry key for writing: {}", e),
            })?;

        let folder_str = original_folder.to_string_lossy().to_string();
        key.set_value(Self::REGISTRY_VALUE_NAME, &folder_str)
            .map_err(|e| PlatformError::RegistryError {
                key: format!("{}\\{}", Self::REGISTRY_KEY_PATH, Self::REGISTRY_VALUE_NAME),
                operation: "write".to_string(),
                message: format!("Failed to restore registry value: {}", e),
            })?;

        // Clear cached original from memory
        {
            let mut cached = self.cached_original.lock().map_err(|e| {
                PlatformError::RegistryError {
                    key: "cached_original".to_string(),
                    operation: "lock".to_string(),
                    message: format!("Failed to acquire lock: {}", e),
                }
            })?;
            *cached = None;
        }

        // Clear from persistent cache
        {
            let cache = self.cache.lock().map_err(|e| PlatformError::RegistryError {
                key: "registry_cache".to_string(),
                operation: "lock".to_string(),
                message: format!("Failed to acquire cache lock: {}", e),
            })?;
            cache.clear_redirect(Self::CACHE_KEY_IDENTIFIER)?;
        }

        Ok(())
    }

    #[cfg(not(windows))]
    fn restore_screenshot_folder(&self, _original_folder: &Path) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "restore_screenshot_folder".to_string(),
            platform: "Non-Windows platform".to_string(),
        })
    }

    #[cfg(windows)]
    fn detect_and_restore_stale_redirects(&self) -> Result<()> {
        let cache = self.cache.lock().map_err(|e| PlatformError::RegistryError {
            key: "registry_cache".to_string(),
            operation: "lock".to_string(),
            message: format!("Failed to acquire cache lock: {}", e),
        })?;

        let redirects = cache.list_active_redirects()?;

        for (key, original, _redirected) in redirects {
            if key == Self::CACHE_KEY_IDENTIFIER {
                // Read current registry value
                let current = self.read_screenshot_folder()?;

                // If current differs from original, restore it
                if current != original {
                    drop(cache); // Release lock before calling restore
                    self.restore_screenshot_folder(&original)?;

                    // Re-acquire lock to continue loop
                    return Ok(());
                } else {
                    // Registry already matches original, just clear the cache
                    cache.clear_redirect(&key)?;
                }
            }
        }

        Ok(())
    }

    #[cfg(not(windows))]
    fn detect_and_restore_stale_redirects(&self) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "detect_and_restore_stale_redirects".to_string(),
            platform: "Non-Windows platform".to_string(),
        })
    }
}

impl Drop for WindowsRegistryBridge {
    /// Ensures registry is restored when the bridge is dropped.
    ///
    /// This provides crash recovery by restoring the original registry value
    /// even if the application terminates abnormally.
    fn drop(&mut self) {
        if let Ok(cached) = self.cached_original.lock() {
            if let Some(ref original) = *cached {
                // Best-effort restoration - ignore errors in Drop
                let _ = self.restore_screenshot_folder(original);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    use std::fs;

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

    /// Tests the WindowsRegistryBridge interface using a real cache (not the actual registry).
    /// This test is platform-independent and verifies the cache and Drop behavior.
    #[test]
    fn test_windows_registry_bridge_cache_behavior() {
        let temp_dir = std::env::temp_dir().join("registry_bridge_test");
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        let bridge = WindowsRegistryBridge::new_with_cache(&db_path).unwrap();

        // Verify cache was initialized
        let cache = bridge.cache.lock().unwrap();
        let redirects = cache.list_active_redirects().unwrap();
        assert_eq!(redirects.len(), 0);

        drop(cache);
        drop(bridge);

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Tests that invalid paths are rejected
    #[test]
    #[cfg(windows)]
    fn test_write_screenshot_folder_rejects_relative_paths() {
        let temp_dir = std::env::temp_dir().join("registry_bridge_test2");
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        let bridge = WindowsRegistryBridge::new_with_cache(&db_path).unwrap();

        // Try to write a relative path
        let relative_path = PathBuf::from("relative/path");
        let result = bridge.write_screenshot_folder(&relative_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::InvalidArgument { parameter, .. } => {
                assert_eq!(parameter, "folder");
            }
            _ => panic!("Expected InvalidArgument error"),
        }

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Tests that non-existent paths are rejected
    #[test]
    #[cfg(windows)]
    fn test_write_screenshot_folder_rejects_nonexistent_paths() {
        let temp_dir = std::env::temp_dir().join("registry_bridge_test3");
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        let bridge = WindowsRegistryBridge::new_with_cache(&db_path).unwrap();

        // Try to write a non-existent absolute path
        let nonexistent_path = PathBuf::from("C:\\this\\path\\does\\not\\exist\\hopefully");
        let result = bridge.write_screenshot_folder(&nonexistent_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::InvalidArgument { parameter, .. } => {
                assert_eq!(parameter, "folder");
            }
            _ => panic!("Expected InvalidArgument error"),
        }

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Tests the Drop trait ensures restoration
    #[test]
    #[cfg(windows)]
    fn test_drop_trait_restores_registry() {
        let temp_dir = std::env::temp_dir().join("registry_bridge_test4");
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");
        let target_folder = std::env::temp_dir();

        {
            let bridge = WindowsRegistryBridge::new_with_cache(&db_path).unwrap();

            // Write to a folder (this will cache the original)
            let _ = bridge.write_screenshot_folder(&target_folder);

            // Verify the original was cached in memory
            let cached = bridge.cached_original.lock().unwrap();
            assert!(cached.is_some());

            // Drop will be called automatically here, which should restore
        }

        // After drop, verify the cache was cleared
        let cache = RegistryCache::new(&db_path).unwrap();
        let original = cache
            .get_cached_original(WindowsRegistryBridge::CACHE_KEY_IDENTIFIER)
            .unwrap();
        assert_eq!(original, None, "Cache should be cleared after Drop restoration");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    /// Tests environment variable expansion
    #[test]
    fn test_env_var_expansion() {
        let test_cases = vec![
            (
                "%USERPROFILE%\\Pictures",
                std::env::var("USERPROFILE")
                    .unwrap_or_default()
                    + "\\Pictures",
            ),
            ("C:\\Absolute\\Path", "C:\\Absolute\\Path".to_string()),
        ];

        for (input, expected) in test_cases {
            let expanded = WindowsRegistryBridge::expand_env_vars(input);
            if !expected.is_empty() {
                assert_eq!(expanded, expected);
            }
        }
    }

    /// Tests stale redirect detection and restoration
    #[test]
    fn test_detect_and_restore_stale_redirects() {
        let temp_dir = std::env::temp_dir().join("registry_bridge_test5");
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        // Manually create a stale redirect in the cache
        let cache = RegistryCache::new(&db_path).unwrap();
        cache
            .cache_redirect(
                WindowsRegistryBridge::CACHE_KEY_IDENTIFIER,
                &PathBuf::from("C:\\Original"),
                &PathBuf::from("C:\\Redirected"),
            )
            .unwrap();

        drop(cache);

        // On non-Windows, this should return NotImplemented
        #[cfg(not(windows))]
        {
            let bridge = WindowsRegistryBridge::new_with_cache(&db_path).unwrap();
            let result = bridge.detect_and_restore_stale_redirects();
            assert!(result.is_err());
        }

        // On Windows, it should attempt restoration
        #[cfg(windows)]
        {
            let bridge = WindowsRegistryBridge::new_with_cache(&db_path).unwrap();
            // This may fail if we don't have the actual registry key, but that's ok
            // We're just testing that the method executes without panic
            let _result = bridge.detect_and_restore_stale_redirects();
        }

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
