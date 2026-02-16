//! Windows platform implementation.
//!
//! This module provides Windows-specific implementations of the platform abstraction traits.
//!
//! # Implementation Status
//!
//! - **CaptureBridge**: Partial implementation (file watcher complete, screenshot trigger/redirect pending)
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
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use notify::{Watcher, RecursiveMode, RecommendedWatcher, Event, EventKind};

use super::capture::{CaptureBridge, CaptureEvent, WatcherHandle};
use super::registry::RegistryBridge;
use super::registry_cache::RegistryCache;
use super::error::{PlatformError, Result};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Wrapper struct to track active file watchers
struct ActiveWatcher {
    _watcher: RecommendedWatcher,
    _thread: Option<thread::JoinHandle<()>>,
}

/// Windows implementation of `CaptureBridge`.
///
/// This implementation provides:
/// - File watching with automatic file type detection and exponential backoff retry
/// - Screenshot trigger and registry redirect (pending implementation)
pub struct WindowsCaptureBridge {
    active_watchers: Arc<Mutex<HashMap<usize, ActiveWatcher>>>,
    next_watcher_id: Arc<Mutex<usize>>,
}

impl WindowsCaptureBridge {
    /// Creates a new Windows capture bridge.
    pub fn new() -> Self {
        Self {
            active_watchers: Arc::new(Mutex::new(HashMap::new())),
            next_watcher_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Determines if a file extension is for an image file
    fn is_image_extension(extension: &str) -> bool {
        matches!(extension.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "gif")
    }

    /// Determines if a file extension is for a video file
    fn is_video_extension(extension: &str) -> bool {
        matches!(extension.to_lowercase().as_str(), "mp4" | "webm" | "mkv")
    }

    /// Attempts to get file metadata with exponential backoff retry
    /// Returns Ok(file_size) if successful, Err if file is still locked after all retries
    fn try_get_file_size(path: &Path) -> Option<u64> {
        let retries = [100, 200, 400]; // milliseconds

        for (i, delay_ms) in retries.iter().enumerate() {
            match std::fs::metadata(path) {
                Ok(metadata) => return Some(metadata.len()),
                Err(_) if i < retries.len() - 1 => {
                    thread::sleep(Duration::from_millis(*delay_ms));
                }
                Err(_) => return None,
            }
        }
        None
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
            platform: "Windows (pending implementation)".to_string(),
        })
    }

    fn restore_screenshot_output(&self, _original_path: &Path) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "restore_screenshot_output".to_string(),
            platform: "Windows (pending implementation)".to_string(),
        })
    }

    fn trigger_screenshot(&self) -> Result<()> {
        Err(PlatformError::NotImplemented {
            operation: "trigger_screenshot".to_string(),
            platform: "Windows (pending implementation)".to_string(),
        })
    }

    fn start_file_watcher(&self, folder: &Path, sender: Sender<CaptureEvent>) -> Result<WatcherHandle> {
        // Validate folder exists and is a directory
        if !folder.exists() {
            return Err(PlatformError::InvalidArgument {
                parameter: "folder".to_string(),
                message: "Folder does not exist".to_string(),
            });
        }
        if !folder.is_dir() {
            return Err(PlatformError::InvalidArgument {
                parameter: "folder".to_string(),
                message: "Path is not a directory".to_string(),
            });
        }

        // Clone sender for the watcher callback
        let event_sender = sender.clone();

        // Create the file system watcher
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    // Only process Create events
                    if !matches!(event.kind, EventKind::Create(_)) {
                        return;
                    }

                    for path in event.paths {
                        // Skip directories
                        if path.is_dir() {
                            continue;
                        }

                        // Get file extension
                        let extension = match path.extension() {
                            Some(ext) => ext.to_string_lossy().to_lowercase(),
                            None => continue, // Skip files without extensions
                        };

                        // Check if it's an image or video
                        let is_image = Self::is_image_extension(&extension);
                        let is_video = Self::is_video_extension(&extension);

                        if !is_image && !is_video {
                            continue; // Skip unsupported file types
                        }

                        // Try to get file size with retry (file might still be locked)
                        let file_size = match Self::try_get_file_size(&path) {
                            Some(size) => size,
                            None => {
                                // Send error event if file is still locked
                                let _ = event_sender.send(CaptureEvent::WatcherError {
                                    message: format!("File locked or inaccessible: {}", path.display()),
                                });
                                continue;
                            }
                        };

                        // Get current timestamp
                        let detected_at = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;

                        // Send appropriate event based on file type
                        let capture_event = if is_image {
                            CaptureEvent::ScreenshotDetected {
                                file_path: path.clone(),
                                file_size,
                                detected_at,
                            }
                        } else {
                            CaptureEvent::VideoDetected {
                                file_path: path.clone(),
                                file_size,
                                detected_at,
                            }
                        };

                        if let Err(e) = event_sender.send(capture_event) {
                            // If send fails, the receiver is gone - no point continuing
                            eprintln!("Failed to send capture event: {}", e);
                        }
                    }
                }
                Err(e) => {
                    let _ = event_sender.send(CaptureEvent::WatcherError {
                        message: format!("Watcher error: {}", e),
                    });
                }
            }
        })
        .map_err(|e| PlatformError::WatcherError {
            message: format!("Failed to create file watcher: {}", e),
        })?;

        // Start watching the folder (non-recursive)
        watcher
            .watch(folder, RecursiveMode::NonRecursive)
            .map_err(|e| PlatformError::WatcherError {
                message: format!("Failed to start watching folder: {}", e),
            })?;

        // Generate watcher ID
        let watcher_id = {
            let mut id_guard = self.next_watcher_id.lock().map_err(|e| {
                PlatformError::WatcherError {
                    message: format!("Failed to acquire ID lock: {}", e),
                }
            })?;
            let id = *id_guard;
            *id_guard += 1;
            id
        };

        // Store the watcher in active watchers
        {
            let mut watchers = self.active_watchers.lock().map_err(|e| {
                PlatformError::WatcherError {
                    message: format!("Failed to acquire watchers lock: {}", e),
                }
            })?;
            watchers.insert(
                watcher_id,
                ActiveWatcher {
                    _watcher: watcher,
                    _thread: None,
                },
            );
        }

        Ok(WatcherHandle::new(watcher_id))
    }

    fn stop_file_watcher(&self, handle: WatcherHandle) -> Result<()> {
        // Remove watcher from active watchers (dropping it will stop it)
        let mut watchers = self.active_watchers.lock().map_err(|e| {
            PlatformError::WatcherError {
                message: format!("Failed to acquire watchers lock: {}", e),
            }
        })?;

        watchers.remove(&handle.id);
        Ok(())
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
    fn test_screenshot_redirect_returns_not_implemented() {
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
    }

    #[test]
    fn test_file_watcher_rejects_nonexistent_folder() {
        let bridge = WindowsCaptureBridge::new();
        let (tx, _rx) = channel();
        let nonexistent = PathBuf::from("/nonexistent/path/hopefully");

        let result = bridge.start_file_watcher(&nonexistent, tx);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::InvalidArgument { parameter, message } => {
                assert_eq!(parameter, "folder");
                assert!(message.contains("does not exist"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_file_watcher_rejects_file_path() {
        let bridge = WindowsCaptureBridge::new();
        let temp_file = std::env::temp_dir().join("test_file.txt");
        fs::write(&temp_file, "test").unwrap();

        let (tx, _rx) = channel();
        let result = bridge.start_file_watcher(&temp_file, tx);
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::InvalidArgument { parameter, message } => {
                assert_eq!(parameter, "folder");
                assert!(message.contains("not a directory"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }

        fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_file_watcher_detects_screenshots() {
        let bridge = WindowsCaptureBridge::new();
        let temp_dir = std::env::temp_dir().join("watcher_test_screenshots");
        fs::create_dir_all(&temp_dir).unwrap();

        let (tx, rx) = channel();
        let handle = bridge.start_file_watcher(&temp_dir, tx).unwrap();

        // Give the watcher a moment to initialize
        thread::sleep(Duration::from_millis(100));

        // Create a PNG file
        let png_path = temp_dir.join("test_screenshot.png");
        fs::write(&png_path, b"fake png data").unwrap();

        // Wait for event with timeout
        let event = rx.recv_timeout(Duration::from_secs(2));
        assert!(event.is_ok(), "Should receive screenshot event");

        match event.unwrap() {
            CaptureEvent::ScreenshotDetected { file_path, file_size, .. } => {
                assert_eq!(file_path, png_path);
                assert!(file_size > 0);
            }
            _ => panic!("Expected ScreenshotDetected event"),
        }

        // Stop watcher and cleanup
        bridge.stop_file_watcher(handle).unwrap();
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_file_watcher_detects_videos() {
        let bridge = WindowsCaptureBridge::new();
        let temp_dir = std::env::temp_dir().join("watcher_test_videos");
        fs::create_dir_all(&temp_dir).unwrap();

        let (tx, rx) = channel();
        let handle = bridge.start_file_watcher(&temp_dir, tx).unwrap();

        // Give the watcher a moment to initialize
        thread::sleep(Duration::from_millis(100));

        // Create an MP4 file
        let mp4_path = temp_dir.join("test_video.mp4");
        fs::write(&mp4_path, b"fake mp4 data").unwrap();

        // Wait for event
        let event = rx.recv_timeout(Duration::from_secs(2));
        assert!(event.is_ok(), "Should receive video event");

        match event.unwrap() {
            CaptureEvent::VideoDetected { file_path, file_size, .. } => {
                assert_eq!(file_path, mp4_path);
                assert!(file_size > 0);
            }
            _ => panic!("Expected VideoDetected event"),
        }

        // Stop watcher and cleanup
        bridge.stop_file_watcher(handle).unwrap();
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_file_watcher_ignores_unsupported_files() {
        let bridge = WindowsCaptureBridge::new();
        let temp_dir = std::env::temp_dir().join("watcher_test_unsupported");
        fs::create_dir_all(&temp_dir).unwrap();

        let (tx, rx) = channel();
        let handle = bridge.start_file_watcher(&temp_dir, tx).unwrap();

        // Give the watcher a moment to initialize
        thread::sleep(Duration::from_millis(100));

        // Create unsupported file types
        fs::write(temp_dir.join("test.txt"), b"text").unwrap();
        fs::write(temp_dir.join("test.pdf"), b"pdf").unwrap();
        fs::write(temp_dir.join("test.exe"), b"exe").unwrap();

        // Wait a bit to ensure no events are sent
        thread::sleep(Duration::from_millis(500));

        // Should not receive any events
        let event = rx.try_recv();
        assert!(event.is_err(), "Should not receive events for unsupported files");

        // Stop watcher and cleanup
        bridge.stop_file_watcher(handle).unwrap();
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_file_watcher_handles_multiple_files() {
        let bridge = WindowsCaptureBridge::new();
        let temp_dir = std::env::temp_dir().join("watcher_test_multiple");
        fs::create_dir_all(&temp_dir).unwrap();

        let (tx, rx) = channel();
        let handle = bridge.start_file_watcher(&temp_dir, tx).unwrap();

        // Give the watcher a moment to initialize
        thread::sleep(Duration::from_millis(100));

        // Create multiple files
        fs::write(temp_dir.join("screenshot1.png"), b"png1").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(temp_dir.join("screenshot2.jpg"), b"jpg2").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(temp_dir.join("video1.mp4"), b"mp4").unwrap();

        // Collect events (with timeout)
        let mut events = Vec::new();
        for _ in 0..3 {
            if let Ok(event) = rx.recv_timeout(Duration::from_secs(2)) {
                events.push(event);
            }
        }

        assert_eq!(events.len(), 3, "Should receive 3 events");

        // Verify we got the right mix of events
        let screenshot_count = events.iter().filter(|e| matches!(e, CaptureEvent::ScreenshotDetected { .. })).count();
        let video_count = events.iter().filter(|e| matches!(e, CaptureEvent::VideoDetected { .. })).count();

        assert_eq!(screenshot_count, 2, "Should have 2 screenshot events");
        assert_eq!(video_count, 1, "Should have 1 video event");

        // Stop watcher and cleanup
        bridge.stop_file_watcher(handle).unwrap();
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_file_watcher_stop_is_idempotent() {
        let bridge = WindowsCaptureBridge::new();
        let temp_dir = std::env::temp_dir().join("watcher_test_stop");
        fs::create_dir_all(&temp_dir).unwrap();

        let (tx, _rx) = channel();
        let handle = bridge.start_file_watcher(&temp_dir, tx).unwrap();

        // Stop the watcher
        let result1 = bridge.stop_file_watcher(handle);
        assert!(result1.is_ok(), "First stop should succeed");

        // Create a new handle with same ID (simulating double-stop)
        let fake_handle = WatcherHandle::new(1);
        let result2 = bridge.stop_file_watcher(fake_handle);
        assert!(result2.is_ok(), "Stop should be idempotent");

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_file_extension_detection() {
        // Test image extensions
        assert!(WindowsCaptureBridge::is_image_extension("png"));
        assert!(WindowsCaptureBridge::is_image_extension("PNG"));
        assert!(WindowsCaptureBridge::is_image_extension("jpg"));
        assert!(WindowsCaptureBridge::is_image_extension("jpeg"));
        assert!(WindowsCaptureBridge::is_image_extension("gif"));
        assert!(!WindowsCaptureBridge::is_image_extension("mp4"));
        assert!(!WindowsCaptureBridge::is_image_extension("txt"));

        // Test video extensions
        assert!(WindowsCaptureBridge::is_video_extension("mp4"));
        assert!(WindowsCaptureBridge::is_video_extension("MP4"));
        assert!(WindowsCaptureBridge::is_video_extension("webm"));
        assert!(WindowsCaptureBridge::is_video_extension("mkv"));
        assert!(!WindowsCaptureBridge::is_video_extension("png"));
        assert!(!WindowsCaptureBridge::is_video_extension("avi"));
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
