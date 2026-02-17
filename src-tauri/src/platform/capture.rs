//! Platform abstraction for screenshot capture and file watching.
//!
//! The `CaptureBridge` trait defines the interface for platform-specific
//! screenshot capture operations and file watching.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use super::error::Result;

/// Events emitted by the file watcher when new capture files are detected.
#[derive(Debug, Clone)]
pub enum CaptureEvent {
    /// A new screenshot file was detected in the capture folder.
    ScreenshotDetected {
        /// Absolute path to the screenshot file
        file_path: PathBuf,
        /// File size in bytes
        file_size: u64,
        /// Timestamp when the file was detected (milliseconds since epoch)
        detected_at: u64,
    },

    /// A new video file was detected in the capture folder.
    VideoDetected {
        /// Absolute path to the video file
        file_path: PathBuf,
        /// File size in bytes
        file_size: u64,
        /// Timestamp when the file was detected (milliseconds since epoch)
        detected_at: u64,
    },

    /// The file watcher encountered an error.
    WatcherError {
        /// Error message
        message: String,
    },
}

/// Handle to a running file watcher.
///
/// The watcher is automatically stopped when this handle is dropped.
/// Platform implementations should use RAII patterns to ensure cleanup.
#[derive(Debug)]
pub struct WatcherHandle {
    /// Internal platform-specific watcher identifier
    #[allow(dead_code)]
    pub(crate) id: usize,
}

impl WatcherHandle {
    /// Creates a new watcher handle with the given ID.
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

/// Platform abstraction trait for screenshot capture and file watching.
///
/// This trait provides OS-specific operations for:
/// - Triggering the OS screenshot tool programmatically
/// - Watching folders for new capture files
///
/// # Platform Implementations
///
/// - **Windows**: Multiple trigger methods (URI, process launch, key simulation).
///   The app watches the system default screenshot folder (%USERPROFILE%\Pictures\Screenshots)
///   and copies new files into the session or bug folder — no registry modification required.
/// - **macOS**: Uses `screencapture` CLI with output path arguments (v2)
///
/// # Thread Safety
///
/// Implementations should be `Send + Sync` to allow usage across threads.
/// File watcher callbacks are invoked on background threads.
///
/// # Example
///
/// ```no_run
/// use unbroken_qa_capture_lib::platform::{CaptureBridge, get_capture_bridge};
/// use std::path::Path;
/// use std::sync::mpsc::channel;
///
/// let bridge = get_capture_bridge();
/// let watch_folder = Path::new("/path/to/screenshots");
///
/// // Start watching for new files
/// let (tx, rx) = channel();
/// let handle = bridge.start_file_watcher(watch_folder, tx)?;
///
/// // Trigger a screenshot
/// bridge.trigger_screenshot()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    ///
    /// # User Interaction
    ///
    /// This method triggers the OS screenshot tool but does not block. The user must
    /// interact with the screenshot UI (select region, capture, save). The file watcher
    /// will detect the resulting file when the user completes the capture.
    fn trigger_screenshot(&self) -> Result<()>;

    /// Starts watching the specified folder for new capture files.
    ///
    /// # Platform Behavior
    ///
    /// - **All platforms**: Uses the `notify` crate to monitor the folder for new files.
    ///   Detects image files (`.png`, `.jpg`, `.jpeg`, `.gif`) and video files
    ///   (`.mp4`, `.webm`, `.mkv`).
    ///
    /// # Arguments
    ///
    /// * `folder` - Absolute path to the folder to watch. Must exist.
    /// * `sender` - Channel sender for emitting `CaptureEvent` notifications.
    ///   Events are sent on a background thread.
    ///
    /// # Returns
    ///
    /// A `WatcherHandle` that can be used to stop the watcher via `stop_file_watcher()`.
    /// The watcher runs on a background thread and will continue until stopped.
    ///
    /// # Errors
    ///
    /// - `PlatformError::InvalidArgument`: Folder does not exist or is not a directory
    /// - `PlatformError::WatcherError`: Failed to start the file system watcher
    /// - `PlatformError::NotImplemented`: Platform does not support this operation (macOS v1)
    ///
    /// # Event Timing
    ///
    /// Events are emitted within 500ms of file creation. If the file is still being written
    /// (locked by another process), the implementation should retry with exponential backoff
    /// (100ms, 200ms, 400ms) up to 3 attempts.
    ///
    /// # File Type Detection
    ///
    /// - Extensions `.png`, `.jpg`, `.jpeg`, `.gif` → `CaptureEvent::ScreenshotDetected`
    /// - Extensions `.mp4`, `.webm`, `.mkv` → `CaptureEvent::VideoDetected`
    /// - Other files are ignored
    fn start_file_watcher(&self, folder: &Path, sender: Sender<CaptureEvent>) -> Result<WatcherHandle>;

    /// Stops the file watcher associated with the given handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle returned by `start_file_watcher()`.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the watcher was stopped successfully. If the watcher was already
    /// stopped or the handle is invalid, returns `Ok(())` (idempotent).
    ///
    /// # Errors
    ///
    /// - `PlatformError::WatcherError`: Failed to stop the watcher
    /// - `PlatformError::NotImplemented`: Platform does not support this operation (macOS v1)
    ///
    /// # Implementation Note
    ///
    /// The watcher background thread should be joined to ensure clean shutdown.
    fn stop_file_watcher(&self, handle: WatcherHandle) -> Result<()>;
}
