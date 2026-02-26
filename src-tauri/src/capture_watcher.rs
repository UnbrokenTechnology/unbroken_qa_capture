//! File watcher for the `_captures/` directory.
//!
//! When a session is active the watcher monitors `{session_folder}/_captures/`
//! for new files (screenshots / recordings saved by the Snipping Tool or other
//! capture mechanisms). On detecting a new file it:
//!
//! 1. Waits briefly for the write to finish.
//! 2. Moves the file into the active bug folder (or `_unsorted/` when no bug
//!    is active).
//! 3. Creates a `Capture` DB record linking the file to the bug/session.
//! 4. Emits a `screenshot:captured` Tauri event so the frontend can refresh.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chrono::Utc;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::Connection;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::database::{BugOps, BugRepository, Capture, CaptureOps, CaptureRepository};

type SharedConn = Arc<Mutex<Connection>>;

/// Extensions we recognise as media files worth processing.
const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tif",
];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "webm", "mkv", "avi", "mov"];

/// Watches `_captures/` and routes new files to the correct bug folder.
///
/// Dropping the struct stops the watcher.
pub struct CaptureWatcher {
    _watcher: RecommendedWatcher,
}

impl CaptureWatcher {
    /// Start watching `captures_dir` for new media files.
    pub fn start(
        captures_dir: PathBuf,
        session_id: String,
        session_folder: PathBuf,
        active_bug: Arc<Mutex<Option<String>>>,
        db_conn: SharedConn,
        app_handle: AppHandle,
    ) -> Result<Self, String> {
        // Process files already sitting in _captures/ (e.g. from a crash).
        Self::process_existing_files(
            &captures_dir,
            &session_id,
            &session_folder,
            &active_bug,
            &db_conn,
            &app_handle,
        );

        // Clones for the closure (must be 'static + Send).
        let sid = session_id;
        let sf = session_folder;
        let ab = active_bug;
        let dc = db_conn;
        let ah = app_handle;

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                let Ok(event) = res else { return };
                if !matches!(event.kind, EventKind::Create(_)) {
                    return;
                }
                for path in &event.paths {
                    let path = path.clone();
                    let sid = sid.clone();
                    let sf = sf.clone();
                    let ab = Arc::clone(&ab);
                    let dc = Arc::clone(&dc);
                    let ah = ah.clone();
                    thread::spawn(move || {
                        Self::process_new_capture(&path, &sid, &sf, &ab, &dc, &ah);
                    });
                }
            },
            notify::Config::default(),
        )
        .map_err(|e| format!("Failed to create file watcher: {e}"))?;

        watcher
            .watch(&captures_dir, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch captures directory: {e}"))?;

        Ok(Self { _watcher: watcher })
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    fn process_existing_files(
        captures_dir: &Path,
        session_id: &str,
        session_folder: &Path,
        active_bug: &Arc<Mutex<Option<String>>>,
        db_conn: &SharedConn,
        app_handle: &AppHandle,
    ) {
        let Ok(entries) = std::fs::read_dir(captures_dir) else {
            return;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && Self::is_media_file(&path) {
                Self::process_new_capture(
                    &path,
                    session_id,
                    session_folder,
                    active_bug,
                    db_conn,
                    app_handle,
                );
            }
        }
    }

    /// Wait for a file to finish being written by polling for stable file size.
    /// Returns `true` if the file stabilized, `false` if timed out.
    fn wait_for_write_complete(path: &Path, max_wait: Duration) -> bool {
        let poll_interval = Duration::from_millis(100);
        let start = std::time::Instant::now();
        let mut last_size = 0u64;
        let mut stable_count = 0u32;

        loop {
            if start.elapsed() > max_wait {
                return false;
            }
            match std::fs::metadata(path) {
                Ok(m) => {
                    let size = m.len();
                    if size > 0 && size == last_size {
                        stable_count += 1;
                        if stable_count >= 3 {
                            return true; // 300ms stable
                        }
                    } else {
                        stable_count = 0;
                        last_size = size;
                    }
                }
                Err(_) => {
                    stable_count = 0;
                }
            }
            thread::sleep(poll_interval);
        }
    }

    /// Wait until no other process holds an open handle on the file.
    ///
    /// On Windows, the Snipping Tool (and similar capture tools) write all bytes
    /// in one shot — so `wait_for_write_complete` returns almost immediately —
    /// but then keep the file handle open for post-save operations (thumbnails,
    /// shell notifications). Moving the file while the handle is held causes the
    /// capture tool to spin indefinitely.
    ///
    /// This function attempts to open the file with *exclusive* access
    /// (`share_mode(0)` = no sharing). If the open succeeds, no other process
    /// holds the file and it is safe to rename/move. The handle is dropped
    /// immediately after the check.
    ///
    /// On non-Windows platforms this is a no-op (returns `true`) because POSIX
    /// file semantics allow rename even while another process has the file open.
    #[cfg(windows)]
    fn wait_for_exclusive_access(path: &Path, max_wait: Duration) -> bool {
        use std::os::windows::fs::OpenOptionsExt;

        let poll_interval = Duration::from_millis(200);
        let start = std::time::Instant::now();

        loop {
            // share_mode(0) = FILE_SHARE_NONE — fails if any other handle exists.
            let result = std::fs::OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(path);

            match result {
                Ok(_handle) => {
                    // Exclusive open succeeded — no other process holds a handle.
                    // The handle is dropped here, releasing our lock.
                    return true;
                }
                Err(_) => {
                    if start.elapsed() > max_wait {
                        return false;
                    }
                    thread::sleep(poll_interval);
                }
            }
        }
    }

    #[cfg(not(windows))]
    fn wait_for_exclusive_access(_path: &Path, _max_wait: Duration) -> bool {
        // POSIX allows rename while another process has the file open (the
        // inode stays valid), so the Snipping Tool hang cannot occur. Return
        // immediately.
        true
    }

    fn process_new_capture(
        source_path: &Path,
        session_id: &str,
        session_folder: &Path,
        active_bug: &Arc<Mutex<Option<String>>>,
        db_conn: &SharedConn,
        app_handle: &AppHandle,
    ) {
        // Poll until the writing application finishes flushing (size stable for 300ms).
        if !Self::wait_for_write_complete(source_path, Duration::from_secs(5)) {
            eprintln!(
                "CaptureWatcher: file may still be writing after 5s timeout: {:?}",
                source_path
            );
        }

        // Wait for the capturing application (e.g. Snipping Tool) to release
        // its file handle. The Snipping Tool writes all bytes at once (so size
        // stabilizes immediately) but keeps the handle open for post-save work
        // (thumbnails, shell notifications). Moving the file while the handle
        // is held causes the Snipping Tool to spin indefinitely.
        if !Self::wait_for_exclusive_access(source_path, Duration::from_secs(10)) {
            eprintln!(
                "CaptureWatcher: file handle still held after 10s timeout, proceeding anyway: {:?}",
                source_path
            );
        }

        // Validate: must exist, be a media file, have size > 0.
        if !Self::is_media_file(source_path) {
            return;
        }
        let file_size = match std::fs::metadata(source_path) {
            Ok(m) if m.len() > 0 && m.is_file() => m.len() as i64,
            _ => return,
        };

        // Snapshot the current active bug.
        let bug_id = active_bug.lock().unwrap().clone();

        // Destination: bug folder if capturing, else _unsorted/.
        let dest_dir = match bug_id {
            Some(ref bid) => Self::get_bug_folder(db_conn, bid)
                .map(PathBuf::from)
                .unwrap_or_else(|| session_folder.join("_unsorted")),
            None => session_folder.join("_unsorted"),
        };

        if let Err(e) = std::fs::create_dir_all(&dest_dir) {
            eprintln!("CaptureWatcher: cannot create dir {dest_dir:?}: {e}");
            return;
        }

        // Generate a sequential, PRD-compliant filename.
        let capture_number = crate::next_capture_number(&dest_dir);
        let (file_name, capture_type) =
            crate::make_capture_filename(source_path, capture_number);
        let dest_path = dest_dir.join(&file_name);

        // Move (rename) the file; fall back to copy+delete for cross-volume.
        if std::fs::rename(source_path, &dest_path).is_err() {
            if let Err(e) = std::fs::copy(source_path, &dest_path) {
                eprintln!("CaptureWatcher: copy failed {source_path:?} -> {dest_path:?}: {e}");
                return;
            }
            let _ = std::fs::remove_file(source_path);
        }

        // Persist a Capture record.
        let capture_id = Uuid::new_v4().to_string();
        let capture = Capture {
            id: capture_id.clone(),
            bug_id: bug_id.clone(),
            session_id: session_id.to_string(),
            file_name: file_name.clone(),
            file_path: dest_path.to_string_lossy().to_string(),
            file_type: capture_type,
            annotated_path: None,
            file_size_bytes: Some(file_size),
            is_console_capture: false,
            parsed_content: None,
            created_at: Utc::now().to_rfc3339(),
        };

        {
            let conn = db_conn.lock().unwrap();
            let repo = CaptureRepository::new(&conn);
            if let Err(e) = repo.create(&capture) {
                eprintln!("CaptureWatcher: DB insert failed: {e}");
            }
        }

        // Notify the frontend.
        let _ = app_handle.emit(
            "screenshot:captured",
            serde_json::json!({
                "filePath": dest_path.to_string_lossy(),
                "captureId": capture_id,
                "bugId": bug_id,
                "sessionId": session_id,
                "timestamp": Utc::now().timestamp_millis(),
            }),
        );
    }

    /// Look up a bug's `folder_path` from the database.
    fn get_bug_folder(db_conn: &SharedConn, bug_id: &str) -> Option<String> {
        let conn = db_conn.lock().unwrap();
        let repo = BugRepository::new(&conn);
        let bug = repo.get(bug_id).ok()??;
        Some(bug.folder_path)
    }

    /// Return `true` when the file extension looks like an image or video.
    fn is_media_file(path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        IMAGE_EXTENSIONS.contains(&ext.as_str()) || VIDEO_EXTENSIONS.contains(&ext.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_media_file() {
        assert!(CaptureWatcher::is_media_file(Path::new("screenshot.png")));
        assert!(CaptureWatcher::is_media_file(Path::new("photo.JPG")));
        assert!(CaptureWatcher::is_media_file(Path::new("clip.mp4")));
        assert!(CaptureWatcher::is_media_file(Path::new("clip.WebM")));
        assert!(!CaptureWatcher::is_media_file(Path::new("notes.txt")));
        assert!(!CaptureWatcher::is_media_file(Path::new("data.json")));
        assert!(!CaptureWatcher::is_media_file(Path::new(".hidden")));
    }

    #[test]
    fn test_wait_for_write_complete_stable_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.png");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(b"fake image data").unwrap();
        f.flush().unwrap();
        drop(f);

        // File is already stable — should return true quickly.
        assert!(CaptureWatcher::wait_for_write_complete(
            &file_path,
            Duration::from_secs(2)
        ));
    }

    #[test]
    fn test_wait_for_write_complete_missing_file_times_out() {
        let path = Path::new("/tmp/nonexistent_capture_watcher_test_file.png");
        // File doesn't exist — size never stabilizes, should time out.
        assert!(!CaptureWatcher::wait_for_write_complete(
            path,
            Duration::from_millis(400)
        ));
    }

    #[test]
    fn test_wait_for_exclusive_access_no_contention() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.png");
        std::fs::write(&file_path, b"fake image data").unwrap();

        // No other process holds the file — should return true.
        assert!(CaptureWatcher::wait_for_exclusive_access(
            &file_path,
            Duration::from_secs(1)
        ));
    }

    #[test]
    fn test_wait_for_exclusive_access_missing_file() {
        let path = Path::new("/tmp/nonexistent_exclusive_access_test.png");

        // On non-Windows, always returns true (no-op).
        // On Windows, the file doesn't exist so open fails — should time out.
        #[cfg(not(windows))]
        assert!(CaptureWatcher::wait_for_exclusive_access(
            path,
            Duration::from_millis(200)
        ));
        #[cfg(windows)]
        assert!(!CaptureWatcher::wait_for_exclusive_access(
            path,
            Duration::from_millis(400)
        ));
    }
}
