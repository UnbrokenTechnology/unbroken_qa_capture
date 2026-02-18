//! File watcher for the session's `_captures/` landing zone.
//!
//! When a new image or video file appears in `{session_folder}/_captures/`, this
//! module routes it:
//!
//! - **Active bug** → `{session_folder}/bug_NNN/` (renamed per PRD convention)
//! - **No active bug** → `{session_folder}/_unsorted/` (renamed per PRD convention)
//!
//! The file is **moved** (not copied) out of `_captures/` so that folder stays clean.
//! A `Capture` record is inserted into the database and a `capture:file-detected`
//! event is emitted to the frontend.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

use crate::database::{Capture, CaptureOps, CaptureRepository, CaptureType, Database};

/// A guard that stops the file watcher when dropped.
pub struct CaptureWatcher {
    /// The underlying watcher — kept alive for the session duration.
    _watcher: RecommendedWatcher,
}

impl CaptureWatcher {
    /// Start watching `captures_dir` for new image/video files.
    ///
    /// `session_id`, `session_folder`, and `active_bug` are threadsafe handles
    /// into the current session state so the callback can read them without
    /// holding `SESSION_MANAGER`'s lock.
    pub fn start(
        app: AppHandle,
        captures_dir: PathBuf,
        session_id: String,
        session_folder: String,
        active_bug: Arc<Mutex<Option<String>>>,
        db_path: PathBuf,
    ) -> Result<Self, String> {
        let app2 = app.clone();
        let captures_dir2 = captures_dir.clone();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            let event = match res {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("capture_watcher: notify error: {}", e);
                    return;
                }
            };

            // We only care about file creation/modification events (not renames/removals)
            if !matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(notify::event::ModifyKind::Data(_))
            ) {
                return;
            }

            for path in event.paths {
                // Only act on files directly inside _captures/ (not subdirectory events)
                if path.parent() != Some(captures_dir2.as_path()) {
                    continue;
                }

                // Skip if this is not a regular file or if it doesn't look like
                // an image/video (ignore temp files, .part files, etc.)
                if !is_capture_file(&path) {
                    continue;
                }

                // Wait briefly to ensure the file write is complete
                wait_for_file_ready(&path);

                // Read the active bug ID (if any) — this is an Arc<Mutex> so we
                // don't need to go through SESSION_MANAGER at all.
                let maybe_bug_id = active_bug.lock().unwrap().clone();

                match maybe_bug_id {
                    Some(bug_id) => {
                        // Route to the active bug's folder
                        let bug_folder = get_bug_folder(&session_folder, &bug_id, &db_path);
                        match bug_folder {
                            Some(folder) => {
                                route_to_bug(
                                    &app2,
                                    &path,
                                    &session_id,
                                    &bug_id,
                                    &folder,
                                    &db_path,
                                );
                            }
                            None => {
                                // Bug folder not found — fall back to _unsorted/
                                eprintln!(
                                    "capture_watcher: could not find folder for bug {}; routing to _unsorted",
                                    bug_id
                                );
                                route_to_unsorted(
                                    &app2,
                                    &path,
                                    &session_folder,
                                    &session_id,
                                    &db_path,
                                );
                            }
                        }
                    }
                    None => {
                        route_to_unsorted(
                            &app2,
                            &path,
                            &session_folder,
                            &session_id,
                            &db_path,
                        );
                    }
                }
            }
        })
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        watcher
            .watch(&captures_dir, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch _captures/ directory: {}", e))?;

        Ok(CaptureWatcher { _watcher: watcher })
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Returns `true` if `path` looks like a screenshot or video file we should route.
fn is_capture_file(path: &Path) -> bool {
    // Must be a file, not a directory
    if !path.is_file() {
        return false;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "mp4" | "webm" | "mkv"
    )
}

/// Poll until the file size stops growing (max ~2 s) to avoid reading a
/// partially-written file.  A simple heuristic sufficient for local writes.
fn wait_for_file_ready(path: &Path) {
    let mut prev_size = 0u64;
    for _ in 0..10 {
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if size > 0 && size == prev_size {
            break;
        }
        prev_size = size;
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

/// Look up the bug's folder path from the database.
fn get_bug_folder(session_folder: &str, bug_id: &str, db_path: &Path) -> Option<String> {
    use crate::database::{BugOps, BugRepository};
    let db = Database::open(db_path).ok()?;
    let repo = BugRepository::new(db.connection());
    let bug = repo.get(bug_id).ok()??;
    // If the bug folder path is non-empty, use it directly
    if !bug.folder_path.is_empty() {
        Some(bug.folder_path)
    } else {
        // Derive from bug_number
        let folder_name = format!("bug_{:03}", bug.bug_number);
        Some(
            Path::new(session_folder)
                .join(folder_name)
                .to_string_lossy()
                .to_string(),
        )
    }
}

/// Move `src` to `dest`.  Tries `rename` first; falls back to copy+delete.
fn move_file(src: &Path, dest: &Path) -> std::io::Result<()> {
    if std::fs::rename(src, dest).is_ok() {
        return Ok(());
    }
    std::fs::copy(src, dest)?;
    std::fs::remove_file(src)?;
    Ok(())
}

/// Route a file from `_captures/` to `_unsorted/`, create a DB record, and
/// emit events to the frontend.
fn route_to_unsorted(
    app: &AppHandle,
    src: &Path,
    session_folder: &str,
    session_id: &str,
    db_path: &Path,
) {
    let unsorted_dir = Path::new(session_folder).join("_unsorted");

    if let Err(e) = std::fs::create_dir_all(&unsorted_dir) {
        eprintln!(
            "capture_watcher: failed to create _unsorted/ directory: {}",
            e
        );
        return;
    }

    let file_size = std::fs::metadata(src).map(|m| m.len()).unwrap_or(0);
    let capture_num = crate::next_capture_number(&unsorted_dir);
    let (file_name, capture_type) = crate::make_capture_filename(src, capture_num);
    let dest = unsorted_dir.join(&file_name);

    if let Err(e) = move_file(src, &dest) {
        eprintln!("capture_watcher: failed to move file to _unsorted/: {}", e);
        return;
    }

    record_and_emit(app, RecordCtx {
        dest: &dest,
        session_id,
        bug_id: None,
        file_name: &file_name,
        capture_type,
        file_size,
        db_path,
    });
}

/// Route a file from `_captures/` to `bug_folder/`, create a DB record, and
/// emit events to the frontend.
fn route_to_bug(
    app: &AppHandle,
    src: &Path,
    session_id: &str,
    bug_id: &str,
    bug_folder: &str,
    db_path: &Path,
) {
    let bug_dir = Path::new(bug_folder);

    if let Err(e) = std::fs::create_dir_all(bug_dir) {
        eprintln!(
            "capture_watcher: failed to create bug directory: {}",
            e
        );
        return;
    }

    let file_size = std::fs::metadata(src).map(|m| m.len()).unwrap_or(0);
    let capture_num = crate::next_capture_number(bug_dir);
    let (file_name, capture_type) = crate::make_capture_filename(src, capture_num);
    let dest = bug_dir.join(&file_name);

    if let Err(e) = move_file(src, &dest) {
        eprintln!("capture_watcher: failed to move file to bug folder: {}", e);
        return;
    }

    record_and_emit(app, RecordCtx {
        dest: &dest,
        session_id,
        bug_id: Some(bug_id),
        file_name: &file_name,
        capture_type,
        file_size,
        db_path,
    });
}

/// Captures the information needed to record a file in the database and
/// notify the frontend.  Used to avoid a too-many-arguments function signature.
struct RecordCtx<'a> {
    dest: &'a Path,
    session_id: &'a str,
    bug_id: Option<&'a str>,
    file_name: &'a str,
    capture_type: CaptureType,
    file_size: u64,
    db_path: &'a Path,
}

/// Insert a `Capture` row into the database and emit `capture:file-detected`
/// and `screenshot:captured` events to the frontend.
fn record_and_emit(app: &AppHandle, ctx: RecordCtx<'_>) {
    use chrono::Utc;
    use uuid::Uuid;

    let db = match Database::open(ctx.db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("capture_watcher: failed to open database: {}", e);
            return;
        }
    };

    let repo = CaptureRepository::new(db.connection());
    let capture_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let dest_str = ctx.dest.to_string_lossy().to_string();
    let type_str = ctx.capture_type.as_str().to_string();

    let capture = Capture {
        id: capture_id.clone(),
        bug_id: ctx.bug_id.map(|s| s.to_string()),
        session_id: ctx.session_id.to_string(),
        file_name: ctx.file_name.to_string(),
        file_path: dest_str.clone(),
        file_type: ctx.capture_type,
        annotated_path: None,
        file_size_bytes: Some(ctx.file_size as i64),
        is_console_capture: false,
        parsed_content: None,
        created_at: now,
    };

    if let Err(e) = repo.create(&capture) {
        eprintln!("capture_watcher: failed to create capture record: {}", e);
        return;
    }

    // capture:file-detected — picked up by the capture list in the UI
    let _ = app.emit(
        "capture:file-detected",
        serde_json::json!({
            "filePath": dest_str,
            "captureId": capture_id,
            "sessionId": ctx.session_id,
            "bugId": ctx.bug_id,
            "type": type_str,
        }),
    );

    // screenshot:captured — picked up by the annotation flow
    let _ = app.emit(
        "screenshot:captured",
        serde_json::json!({
            "filePath": dest_str,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }),
    );
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_capture_file ───────────────────────────────────────────────────────

    #[test]
    fn test_is_capture_file_accepts_image_extensions() {
        let dir = std::env::temp_dir().join(format!("cw_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        for ext in &["png", "jpg", "jpeg", "gif", "bmp", "webp"] {
            let file = dir.join(format!("screenshot.{}", ext));
            std::fs::write(&file, b"data").unwrap();
            assert!(is_capture_file(&file), "expected {} to be accepted", ext);
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_is_capture_file_accepts_video_extensions() {
        let dir = std::env::temp_dir().join(format!("cw_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        for ext in &["mp4", "webm", "mkv"] {
            let file = dir.join(format!("recording.{}", ext));
            std::fs::write(&file, b"data").unwrap();
            assert!(is_capture_file(&file), "expected {} to be accepted", ext);
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_is_capture_file_rejects_unsupported_extensions() {
        let dir = std::env::temp_dir().join(format!("cw_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        for ext in &["txt", "md", "tmp", "part", "pdf", "exe"] {
            let file = dir.join(format!("file.{}", ext));
            std::fs::write(&file, b"data").unwrap();
            assert!(!is_capture_file(&file), "expected {} to be rejected", ext);
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_is_capture_file_rejects_directories() {
        let dir = std::env::temp_dir().join(format!("cw_test_{}", uuid::Uuid::new_v4()));
        // The path itself is a directory — should be rejected even if it ends in .png
        let png_dir = dir.join("screenshot.png");
        std::fs::create_dir_all(&png_dir).unwrap();

        assert!(!is_capture_file(&png_dir));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_is_capture_file_rejects_nonexistent_path() {
        let path = std::path::Path::new("/nonexistent/path/screenshot.png");
        assert!(!is_capture_file(path));
    }

    #[test]
    fn test_is_capture_file_case_insensitive() {
        let dir = std::env::temp_dir().join(format!("cw_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        // Extensions are lowercased before matching
        let upper = dir.join("SCREENSHOT.PNG");
        std::fs::write(&upper, b"data").unwrap();
        assert!(is_capture_file(&upper));

        std::fs::remove_dir_all(&dir).ok();
    }

    // ── move_file ─────────────────────────────────────────────────────────────

    #[test]
    fn test_move_file_basic() {
        let dir = std::env::temp_dir().join(format!("cw_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let src = dir.join("src.png");
        let dest = dir.join("dest.png");
        std::fs::write(&src, b"hello").unwrap();

        move_file(&src, &dest).unwrap();

        assert!(!src.exists(), "source should be gone after move");
        assert!(dest.exists(), "destination should exist after move");
        assert_eq!(std::fs::read(&dest).unwrap(), b"hello");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_move_file_preserves_content() {
        let dir = std::env::temp_dir().join(format!("cw_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let content = b"PNG\x89\x50\x4E\x47 fake image bytes";
        let src = dir.join("capture.png");
        let dest = dir.join("capture-001.png");
        std::fs::write(&src, content).unwrap();

        move_file(&src, &dest).unwrap();

        assert_eq!(std::fs::read(&dest).unwrap(), content);

        std::fs::remove_dir_all(&dir).ok();
    }
}
