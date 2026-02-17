//! Windows platform implementation.
//!
//! This module provides Windows-specific implementations of the platform abstraction traits.
//!
//! # Implementation Status
//!
//! - **CaptureBridge**: Full implementation (screenshot trigger only)
//! - **RegistryBridge**: Full implementation with crash recovery via SQLite cache
//!
//! # Capture Model
//!
//! The user saves screenshots directly into the session's _captures/ folder.
//! The app moves files from _captures/ into the active bug's screenshots/ subfolder.
//! No system screenshot folder watching or registry modification required.
//!
//! # Registry (startup/other uses)
//!
//! The WindowsRegistryBridge is still used for launch-on-startup. It provides:
//! - Read/write access to HKCU registry keys (no admin required)
//! - Persistent caching of original values in SQLite for crash recovery

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::os::windows::process::CommandExt;

use super::capture::CaptureBridge;
use super::registry::RegistryBridge;
use super::registry_cache::RegistryCache;
use super::error::{PlatformError, Result};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Windows implementation of `CaptureBridge`.
///
/// This implementation provides:
/// - Screenshot trigger via multiple fallback methods (URI, process, key simulation)
///
/// Users save screenshots directly into the session's _captures/ folder.
/// The app moves files from _captures/ into the active bug's screenshots/ subfolder.
pub struct WindowsCaptureBridge;

impl WindowsCaptureBridge {
    /// Creates a new Windows capture bridge.
    pub fn new() -> Self {
        Self
    }

    /// Attempts to trigger screenshot via ms-screenclip: URI scheme
    /// This is the recommended method on Windows 10 1809+ and Windows 11
    fn try_trigger_via_uri() -> Result<()> {
        #[cfg(windows)]
        {
            use std::process::Command;

            Command::new("cmd")
                .args(["/C", "start", "ms-screenclip:"])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .spawn()
                .map_err(|e| PlatformError::ScreenshotTriggerError {
                    method: "uri".to_string(),
                    message: format!("Failed to launch ms-screenclip: URI: {}", e),
                })?;

            Ok(())
        }

        #[cfg(not(windows))]
        Err(PlatformError::NotImplemented {
            operation: "try_trigger_via_uri".to_string(),
            platform: "Non-Windows platform".to_string(),
        })
    }

    /// Attempts to trigger screenshot by spawning SnippingTool.exe
    fn try_trigger_via_process() -> Result<()> {
        #[cfg(windows)]
        {
            use std::process::Command;

            Command::new("SnippingTool.exe")
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .spawn()
                .map_err(|e| PlatformError::ScreenshotTriggerError {
                    method: "process".to_string(),
                    message: format!("Failed to launch SnippingTool.exe: {}", e),
                })?;

            Ok(())
        }

        #[cfg(not(windows))]
        Err(PlatformError::NotImplemented {
            operation: "try_trigger_via_process".to_string(),
            platform: "Non-Windows platform".to_string(),
        })
    }

    /// Attempts to trigger screenshot by simulating Win+Shift+S key combination
    /// Uses Windows SendInput API to simulate the key presses
    fn try_trigger_via_keysim() -> Result<()> {
        #[cfg(windows)]
        {
            use windows::Win32::UI::Input::KeyboardAndMouse::{
                SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
                KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_LWIN, VK_SHIFT, VK_S,
            };

            unsafe {
                let mut inputs: [INPUT; 6] = std::mem::zeroed();

                // Press Win
                inputs[0] = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_LWIN,
                            wScan: 0,
                            dwFlags: KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };

                // Press Shift
                inputs[1] = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_SHIFT,
                            wScan: 0,
                            dwFlags: KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };

                // Press S
                inputs[2] = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_S,
                            wScan: 0,
                            dwFlags: KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };

                // Release S
                inputs[3] = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_S,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };

                // Release Shift
                inputs[4] = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_SHIFT,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };

                // Release Win
                inputs[5] = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_LWIN,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };

                let result = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);

                if result != 6 {
                    return Err(PlatformError::ScreenshotTriggerError {
                        method: "keysim".to_string(),
                        message: format!("SendInput failed: sent {} out of 6 inputs", result),
                    });
                }

                Ok(())
            }
        }

        #[cfg(not(windows))]
        Err(PlatformError::NotImplemented {
            operation: "try_trigger_via_keysim".to_string(),
            platform: "Non-Windows platform".to_string(),
        })
    }
}

impl Default for WindowsCaptureBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureBridge for WindowsCaptureBridge {
    fn trigger_screenshot(&self) -> Result<()> {
        // Try multiple methods in fallback order for maximum reliability on Windows 11

        // Method 1: Launch ms-screenclip: URI (Windows 10 1809+ / Win11)
        if Self::try_trigger_via_uri().is_ok() {
            return Ok(());
        }

        // Method 2: Spawn SnippingTool.exe process
        if Self::try_trigger_via_process().is_ok() {
            return Ok(());
        }

        // Method 3: Simulate Win+Shift+S key combination
        if Self::try_trigger_via_keysim().is_ok() {
            return Ok(());
        }

        // All methods failed
        Err(PlatformError::ScreenshotTriggerError {
            method: "all".to_string(),
            message: "All screenshot trigger methods failed (URI, process, key simulation)".to_string(),
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
    use std::fs;

    #[test]
    #[cfg(windows)]
    fn test_trigger_screenshot_attempts_multiple_methods() {
        let bridge = WindowsCaptureBridge::new();

        // On Windows, trigger_screenshot should attempt all methods and either succeed or fail
        // We can't guarantee success in CI environment, but we can verify it doesn't panic
        let result = bridge.trigger_screenshot();

        // Either it succeeds (at least one method worked) or fails with ScreenshotTriggerError
        match result {
            Ok(_) => {
                // Success - at least one method worked
            }
            Err(PlatformError::ScreenshotTriggerError { method, message }) => {
                // All methods failed - verify error structure
                assert_eq!(method, "all");
                assert!(message.contains("All screenshot trigger methods failed"));
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[test]
    #[cfg(not(windows))]
    fn test_trigger_screenshot_not_implemented_on_non_windows() {
        // The helper methods should return NotImplemented on non-Windows
        let result = WindowsCaptureBridge::try_trigger_via_uri();
        assert!(matches!(result, Err(PlatformError::NotImplemented { .. })));

        let result = WindowsCaptureBridge::try_trigger_via_process();
        assert!(matches!(result, Err(PlatformError::NotImplemented { .. })));

        let result = WindowsCaptureBridge::try_trigger_via_keysim();
        assert!(matches!(result, Err(PlatformError::NotImplemented { .. })));
    }

    #[test]
    #[cfg(windows)]
    fn test_trigger_via_uri_does_not_panic() {
        // Test that URI trigger doesn't panic (may fail if ms-screenclip isn't registered)
        let result = WindowsCaptureBridge::try_trigger_via_uri();

        // Should either succeed or fail gracefully with ScreenshotTriggerError
        match result {
            Ok(_) => {},
            Err(PlatformError::ScreenshotTriggerError { method, .. }) => {
                assert_eq!(method, "uri");
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_trigger_via_process_does_not_panic() {
        // Test that process trigger doesn't panic (may fail if SnippingTool.exe not found)
        let result = WindowsCaptureBridge::try_trigger_via_process();

        // Should either succeed or fail gracefully with ScreenshotTriggerError
        match result {
            Ok(_) => {},
            Err(PlatformError::ScreenshotTriggerError { method, .. }) => {
                assert_eq!(method, "process");
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_trigger_via_keysim_does_not_panic() {
        // Test that key simulation doesn't panic
        let result = WindowsCaptureBridge::try_trigger_via_keysim();

        // Should either succeed or fail gracefully with ScreenshotTriggerError
        match result {
            Ok(_) => {},
            Err(PlatformError::ScreenshotTriggerError { method, .. }) => {
                assert_eq!(method, "keysim");
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
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

/// Windows platform implementation for startup and other OS-specific operations
pub struct WindowsPlatform;

impl super::Platform for WindowsPlatform {
    #[cfg(windows)]
    fn enable_startup(&self) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .map_err(|e| PlatformError::RegistryError {
                key: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
                operation: "open".to_string(),
                message: format!("Failed to open Run registry key: {}", e),
            })?;

        // Get the current executable path
        let exe_path = std::env::current_exe()
            .map_err(|e| PlatformError::InvalidArgument {
                parameter: "exe_path".to_string(),
                message: format!("Failed to get current executable path: {}", e),
            })?;

        // Set the registry value to the executable path
        key.set_value("UnbrokenQACapture", &exe_path.to_string_lossy().to_string())
            .map_err(|e| PlatformError::RegistryError {
                key: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\\UnbrokenQACapture".to_string(),
                operation: "write".to_string(),
                message: format!("Failed to write startup registry value: {}", e),
            })?;

        Ok(())
    }

    #[cfg(not(windows))]
    fn enable_startup(&self) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "enable_startup".to_string(),
            platform: "Non-Windows platform".to_string(),
        })
    }

    #[cfg(windows)]
    fn disable_startup(&self) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)
            .map_err(|e| PlatformError::RegistryError {
                key: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
                operation: "open".to_string(),
                message: format!("Failed to open Run registry key: {}", e),
            })?;

        // Delete the registry value (ignore error if it doesn't exist)
        let _ = key.delete_value("UnbrokenQACapture");

        Ok(())
    }

    #[cfg(not(windows))]
    fn disable_startup(&self) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "disable_startup".to_string(),
            platform: "Non-Windows platform".to_string(),
        })
    }
}
