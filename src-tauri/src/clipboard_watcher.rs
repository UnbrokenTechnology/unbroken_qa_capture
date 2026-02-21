//! Clipboard watcher for screenshot capture.
//!
//! Polls the system clipboard every 500ms for new images. When a new image is
//! detected (via hash comparison), it encodes the raw RGBA data as an 8-bit PNG
//! and saves it to `{session_folder}/_captures/`. The existing [`CaptureWatcher`]
//! (file watcher) picks up the new file and handles routing to bug folders,
//! DB records, and frontend events.
//!
//! The watcher ignores whatever image is already on the clipboard when it starts,
//! so pre-existing clipboard content is not captured.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use uuid::Uuid;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Polls the system clipboard and saves new images to `_captures/`.
///
/// Dropping the struct signals the background thread to stop (within one poll
/// cycle).
pub struct ClipboardWatcher {
    stop_flag: Arc<AtomicBool>,
}

impl ClipboardWatcher {
    /// Start polling the clipboard for new images.
    ///
    /// `captures_dir` is the `{session_folder}/_captures/` directory where new
    /// PNGs will be written. The existing `CaptureWatcher` should already be
    /// monitoring this directory.
    pub fn start(captures_dir: PathBuf, app_handle: AppHandle) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop_flag);

        thread::spawn(move || {
            Self::poll_loop(captures_dir, app_handle, flag);
        });

        ClipboardWatcher { stop_flag }
    }

    fn poll_loop(captures_dir: PathBuf, app_handle: AppHandle, stop_flag: Arc<AtomicBool>) {
        // Snapshot the current clipboard so we skip any pre-existing image.
        let mut last_hash: Option<u64> = Self::current_image_hash(&app_handle);

        while !stop_flag.load(Ordering::Relaxed) {
            thread::sleep(POLL_INTERVAL);

            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            // Try to read the clipboard image.
            let image = match app_handle.clipboard().read_image() {
                Ok(img) => img,
                Err(_) => {
                    // No image on clipboard (text, empty, etc.) — reset hash so
                    // a *new* paste of the same image is still detected if the
                    // clipboard was cleared in between.
                    last_hash = None;
                    continue;
                }
            };

            let rgba = image.rgba();
            let width = image.width();
            let height = image.height();

            if rgba.is_empty() || width == 0 || height == 0 {
                last_hash = None;
                continue;
            }

            let current_hash = hash_image(rgba, width, height);

            if Some(current_hash) == last_hash {
                continue;
            }

            last_hash = Some(current_hash);

            // New image detected — encode as PNG and save to _captures/.
            match encode_png(rgba, width, height) {
                Ok(png_bytes) => {
                    let filename = format!("clipboard-{}.png", Uuid::new_v4());
                    let dest = captures_dir.join(&filename);
                    if let Err(e) = std::fs::write(&dest, &png_bytes) {
                        eprintln!("ClipboardWatcher: failed to write {dest:?}: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("ClipboardWatcher: PNG encode error: {e}");
                }
            }
        }
    }

    /// Hash the current clipboard image, or `None` if no image is present.
    fn current_image_hash(app_handle: &AppHandle) -> Option<u64> {
        let image = app_handle.clipboard().read_image().ok()?;
        let rgba = image.rgba();
        if rgba.is_empty() {
            return None;
        }
        Some(hash_image(rgba, image.width(), image.height()))
    }
}

impl Drop for ClipboardWatcher {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

// ─── Pure helpers (testable without Tauri) ──────────────────────────────

/// Hash RGBA pixel data + dimensions to detect clipboard changes.
fn hash_image(rgba: &[u8], width: u32, height: u32) -> u64 {
    let mut hasher = DefaultHasher::new();
    rgba.hash(&mut hasher);
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    hasher.finish()
}

/// Encode raw RGBA pixel data as an 8-bit PNG.
fn encode_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| format!("PNG header error: {e}"))?;
        writer
            .write_image_data(rgba)
            .map_err(|e| format!("PNG data error: {e}"))?;
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_image_deterministic() {
        let rgba = vec![255u8; 4 * 2 * 2]; // 2×2 white image
        let h1 = hash_image(&rgba, 2, 2);
        let h2 = hash_image(&rgba, 2, 2);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_image_different_data() {
        let white = vec![255u8; 4 * 2 * 2];
        let black = vec![0u8; 4 * 2 * 2];
        assert_ne!(hash_image(&white, 2, 2), hash_image(&black, 2, 2));
    }

    #[test]
    fn test_hash_image_different_dimensions() {
        let rgba = vec![255u8; 4 * 4]; // 4 pixels
        // Same bytes, different declared dimensions — should differ.
        assert_ne!(hash_image(&rgba, 2, 2), hash_image(&rgba, 4, 1));
    }

    #[test]
    fn test_encode_png_roundtrip() {
        // 2×2 red image (RGBA).
        let rgba = vec![
            255, 0, 0, 255, // red
            0, 255, 0, 255, // green
            0, 0, 255, 255, // blue
            255, 255, 0, 255, // yellow
        ];
        let png_bytes = encode_png(&rgba, 2, 2).expect("encode should succeed");

        // Verify it's a valid PNG by checking the magic bytes.
        assert!(png_bytes.len() > 8);
        assert_eq!(&png_bytes[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);

        // Decode it back and verify pixel data.
        let decoder = png::Decoder::new(std::io::Cursor::new(&png_bytes));
        let mut reader = decoder.read_info().expect("PNG decode");
        let mut decoded = vec![0u8; reader.output_buffer_size()];
        let info = reader.next_frame(&mut decoded).expect("read frame");
        let decoded = &decoded[..info.buffer_size()];

        assert_eq!(info.width, 2);
        assert_eq!(info.height, 2);
        assert_eq!(decoded, &rgba);
    }

    #[test]
    fn test_encode_png_empty_image_returns_error() {
        // 0×0 image — the png crate rejects zero-dimension images.
        let result = encode_png(&[], 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_png_large_image() {
        // 100×100 solid colour — verify it doesn't panic and produces output.
        let rgba = vec![128u8; 4 * 100 * 100];
        let png_bytes = encode_png(&rgba, 100, 100).expect("encode should succeed");
        assert!(!png_bytes.is_empty());
    }
}
