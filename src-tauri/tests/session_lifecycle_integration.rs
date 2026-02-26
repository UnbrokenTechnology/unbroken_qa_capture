//! Integration tests for the full session lifecycle.
//!
//! These tests exercise the complete flow: start session → create bug captures →
//! end session → verify state. They use real file operations in temp directories
//! but mock OS-specific services (event emitter).
//!
//! Test scenarios:
//! 1. Full session lifecycle with real file I/O
//! 2. Multiple bugs with sequential auto-increment
//! 3. Session resume after end
//! 4. Crash recovery (new SessionManager re-attaches to existing DB)
//! 5. Bug capture rejected on inactive session
//! 6. Capture records linked to bugs
//! 7. Multiple sessions isolated from each other
//! 8. File detection simulation (real PNG written to bug folder)
//! 9. Settings persistence across manager instances
//! 10. Hotkey-triggered capture state transitions

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use unbroken_qa_capture_lib::database::{
    init_database, BugOps, BugRepository, CaptureOps, CaptureRepository, CaptureType,
    Capture, SessionOps, SessionRepository, SessionStatus, SettingsOps, SettingsRepository,
};
use unbroken_qa_capture_lib::session_manager::{EventEmitter, RealFileSystem, SessionManager};
use uuid::Uuid;

// ============================================================================
// Test Infrastructure
// ============================================================================

/// Mock event emitter that records all emitted events for assertion.
struct MockEventEmitter {
    events: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
}

impl MockEventEmitter {
    fn new() -> Self {
        MockEventEmitter {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn events(&self) -> Vec<(String, serde_json::Value)> {
        self.events.lock().unwrap().clone()
    }

    fn event_names(&self) -> Vec<String> {
        self.events().into_iter().map(|(name, _)| name).collect()
    }
}

impl EventEmitter for MockEventEmitter {
    fn emit(&self, event: &str, payload: serde_json::Value) -> Result<(), String> {
        self.events
            .lock()
            .unwrap()
            .push((event.to_string(), payload));
        Ok(())
    }
}

/// Test environment with real temp directory and in-file SQLite database.
struct TestEnv {
    /// Root temp directory — cleaned up when TestEnv is dropped.
    root: PathBuf,
    db_path: PathBuf,
    storage_root: PathBuf,
    /// Shared database connection — kept alive for the duration of the test.
    db_conn: Arc<Mutex<Connection>>,
}

impl TestEnv {
    fn new() -> Self {
        let root = std::env::temp_dir()
            .join("unbroken_qa_integration")
            .join(Uuid::new_v4().to_string());
        let db_path = root.join("qa.db");
        let storage_root = root.join("sessions");

        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&storage_root).unwrap();

        // Initialise the database
        let conn = Connection::open(&db_path).unwrap();
        init_database(&conn).unwrap();
        let db_conn = Arc::new(Mutex::new(conn));

        TestEnv {
            root,
            db_path,
            storage_root,
            db_conn,
        }
    }

    /// Create a SessionManager backed by the real filesystem and this test DB.
    fn session_manager(&self) -> (SessionManager, Arc<MockEventEmitter>) {
        let emitter = Arc::new(MockEventEmitter::new());
        let fs = Arc::new(RealFileSystem);
        let manager = SessionManager::new(
            Arc::clone(&self.db_conn),
            self.storage_root.clone(),
            emitter.clone() as Arc<dyn EventEmitter>,
            fs as Arc<dyn unbroken_qa_capture_lib::session_manager::FileSystem>,
        );
        (manager, emitter)
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

/// Test 1: Full session lifecycle with real filesystem.
///
/// Covers: start session → verify folder on disk → capture 3 bugs → end session
/// → verify session status in DB.
#[test]
fn test_full_session_lifecycle_real_fs() {
    let env = TestEnv::new();
    let (manager, emitter) = env.session_manager();

    // Start session
    let session = manager.start_session(None).expect("start_session failed");
    assert_eq!(session.status, SessionStatus::Active);

    // Verify session folder exists on disk
    let session_folder = Path::new(&session.folder_path);
    assert!(
        session_folder.exists(),
        "Session folder should exist: {}",
        session.folder_path
    );

    // Verify _captures/ subdirectory exists (PRD §10: temporary landing zone for Snipping Tool)
    let captures_dir = session_folder.join("_captures");
    assert!(
        captures_dir.exists(),
        "_captures/ subdirectory should be created on session start: {}",
        captures_dir.display()
    );

    // Verify _unsorted/ subdirectory exists
    let unsorted_dir = session_folder.join("_unsorted");
    assert!(
        unsorted_dir.exists(),
        "_unsorted/ subdirectory should be created on session start: {}",
        unsorted_dir.display()
    );

    let session_id = session.id.clone();

    // Capture 3 bugs
    let bug1 = manager
        .start_bug_capture(&session_id)
        .expect("start_bug_capture 1 failed");
    assert_eq!(bug1.bug_number, 1);
    assert_eq!(bug1.display_id, "BUG-001");
    assert!(Path::new(&bug1.folder_path).exists(), "bug_001 folder should exist");

    manager
        .end_bug_capture(&bug1.id)
        .expect("end_bug_capture 1 failed");

    let bug2 = manager
        .start_bug_capture(&session_id)
        .expect("start_bug_capture 2 failed");
    assert_eq!(bug2.bug_number, 2);
    assert_eq!(bug2.display_id, "BUG-002");
    manager
        .end_bug_capture(&bug2.id)
        .expect("end_bug_capture 2 failed");

    let bug3 = manager
        .start_bug_capture(&session_id)
        .expect("start_bug_capture 3 failed");
    assert_eq!(bug3.bug_number, 3);
    assert_eq!(bug3.display_id, "BUG-003");
    manager
        .end_bug_capture(&bug3.id)
        .expect("end_bug_capture 3 failed");

    // End session
    manager.end_session(&session_id).expect("end_session failed");

    // Verify session status in DB
    let conn = Connection::open(&env.db_path).unwrap();
    let repo = SessionRepository::new(&conn);
    let stored = repo.get(&session_id).unwrap().unwrap();
    assert_eq!(stored.status, SessionStatus::Ended);
    assert!(stored.ended_at.is_some());

    // Verify all events were emitted in order
    let event_names = emitter.event_names();
    assert_eq!(
        event_names,
        vec![
            "session:started",
            "bug:capture-started",
            "bug:capture-ended",
            "bug:capture-started",
            "bug:capture-ended",
            "bug:capture-started",
            "bug:capture-ended",
            "session:ended",
        ]
    );
}

/// Test 2: Bug folder naming follows the bug_NNN convention.
#[test]
fn test_bug_folder_naming_convention() {
    let env = TestEnv::new();
    let (manager, _) = env.session_manager();

    let session = manager.start_session(None).unwrap();
    let session_id = session.id.clone();

    for expected_num in 1..=5_i32 {
        let bug = manager.start_bug_capture(&session_id).unwrap();
        let folder_name = Path::new(&bug.folder_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(
            folder_name,
            format!("bug_{:03}", expected_num),
            "Bug folder name mismatch for bug {}",
            expected_num
        );
        assert!(
            Path::new(&bug.folder_path).exists(),
            "Bug folder should exist on disk"
        );
        manager.end_bug_capture(&bug.id).unwrap();
    }
}

/// Test 3: Session resume — ended session can be resumed and accepts new bugs.
#[test]
fn test_session_resume_after_end() {
    let env = TestEnv::new();
    let (manager, emitter) = env.session_manager();

    let session = manager.start_session(None).unwrap();
    let session_id = session.id.clone();

    // Capture a bug, then end session
    let bug1 = manager.start_bug_capture(&session_id).unwrap();
    manager.end_bug_capture(&bug1.id).unwrap();
    manager.end_session(&session_id).unwrap();

    // Active session should be cleared
    assert_eq!(manager.get_active_session_id(), None);

    // Resume the session
    let resumed = manager.resume_session(&session_id).unwrap();
    assert_eq!(resumed.status, SessionStatus::Active);
    assert_eq!(resumed.ended_at, None);
    assert_eq!(manager.get_active_session_id(), Some(session_id.clone()));

    // Should be able to capture another bug after resume
    let bug2 = manager.start_bug_capture(&session_id).unwrap();
    assert_eq!(bug2.bug_number, 2, "Bug numbers should continue after resume");
    manager.end_bug_capture(&bug2.id).unwrap();

    // Events: started, bug started, bug ended, ended, resumed, bug started, bug ended
    let names = emitter.event_names();
    assert!(names.contains(&"session:resumed".to_string()));
    assert_eq!(names.iter().filter(|n| *n == "bug:capture-started").count(), 2);
}

/// Test 4: Crash recovery — a new SessionManager instance can re-attach to
/// an existing database, recovering the prior session state.
#[test]
fn test_crash_recovery_new_manager_instance() {
    let env = TestEnv::new();

    // First manager instance — simulates the process before the crash
    let (manager1, _) = env.session_manager();
    let session = manager1.start_session(None).unwrap();
    let session_id = session.id.clone();

    let bug1 = manager1.start_bug_capture(&session_id).unwrap();
    manager1.end_bug_capture(&bug1.id).unwrap();

    // Simulate crash: manager1 is dropped without ending the session
    drop(manager1);

    // Second manager instance — simulates the process restarting after the crash
    let (manager2, _) = env.session_manager();

    // Active session is NOT known to manager2 (it lost in-memory state on crash)
    assert_eq!(manager2.get_active_session_id(), None);

    // But the session exists in the DB and can be resumed
    let conn = Connection::open(&env.db_path).unwrap();
    let repo = SessionRepository::new(&conn);
    let stored = repo.get(&session_id).unwrap().unwrap();
    assert_eq!(stored.status, SessionStatus::Active, "DB session should still be active");

    // Resume recovery
    let recovered = manager2.resume_session(&session_id).unwrap();
    assert_eq!(recovered.status, SessionStatus::Active);
    assert_eq!(manager2.get_active_session_id(), Some(session_id.clone()));

    // Can continue capturing bugs after recovery
    let bug2 = manager2.start_bug_capture(&session_id).unwrap();
    assert_eq!(bug2.bug_number, 2, "Bug numbers continue from where they left off");
    manager2.end_bug_capture(&bug2.id).unwrap();
    manager2.end_session(&session_id).unwrap();

    // Verify final state
    let final_session = repo.get(&session_id).unwrap().unwrap();
    assert_eq!(final_session.status, SessionStatus::Ended);

    // Verify bug count via DB
    let bug_repo = BugRepository::new(&conn);
    let bugs = bug_repo.list_by_session(&session_id).unwrap();
    assert_eq!(bugs.len(), 2);
}

/// Test 5: Bug capture is rejected when the session is not active.
#[test]
fn test_bug_capture_rejected_on_inactive_session() {
    let env = TestEnv::new();
    let (manager, _) = env.session_manager();

    let session = manager.start_session(None).unwrap();
    let session_id = session.id.clone();
    manager.end_session(&session_id).unwrap();

    let result = manager.start_bug_capture(&session_id);
    assert!(result.is_err(), "Should reject capture on ended session");
    assert!(
        result.unwrap_err().contains("not active"),
        "Error message should mention 'not active'"
    );
}

/// Test 6: Bug capture rejected for nonexistent session.
#[test]
fn test_bug_capture_rejected_on_nonexistent_session() {
    let env = TestEnv::new();
    let (manager, _) = env.session_manager();

    let result = manager.start_bug_capture("nonexistent-session-id");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

/// Test 7: Capture records can be written to the DB and linked to bugs.
#[test]
fn test_capture_records_linked_to_bugs() {
    let env = TestEnv::new();
    let (manager, _) = env.session_manager();

    let session = manager.start_session(None).unwrap();
    let session_id = session.id.clone();
    let bug = manager.start_bug_capture(&session_id).unwrap();

    // Write a fake screenshot PNG to the bug folder
    let screenshot_path = Path::new(&bug.folder_path).join("screenshot_001.png");
    let fake_png: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG header
    ];
    std::fs::write(&screenshot_path, fake_png).unwrap();

    // Insert a capture record into the DB
    let conn = Connection::open(&env.db_path).unwrap();
    let capture_repo = CaptureRepository::new(&conn);
    let capture = Capture {
        id: Uuid::new_v4().to_string(),
        bug_id: Some(bug.id.clone()),
        session_id: session_id.clone(),
        file_name: "screenshot_001.png".to_string(),
        file_path: screenshot_path.to_string_lossy().to_string(),
        file_type: CaptureType::Screenshot,
        annotated_path: None,
        file_size_bytes: Some(fake_png.len() as i64),
        is_console_capture: false,
        parsed_content: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    capture_repo.create(&capture).unwrap();

    // Verify the capture is linked to the bug
    let captures = capture_repo.list_by_bug(&bug.id).unwrap();
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0].file_name, "screenshot_001.png");
    assert_eq!(captures[0].bug_id, Some(bug.id.clone()));
    assert_eq!(captures[0].session_id, session_id);
    assert!(screenshot_path.exists(), "Screenshot file should exist on disk");

    manager.end_bug_capture(&bug.id).unwrap();
    manager.end_session(&session_id).unwrap();
}

/// Test 8: Multiple sessions are isolated — bugs and captures don't bleed across.
#[test]
fn test_multiple_sessions_isolated() {
    let env = TestEnv::new();
    let (manager, _) = env.session_manager();

    // Session A — 2 bugs
    let session_a = manager.start_session(None).unwrap();
    let id_a = session_a.id.clone();
    let a_bug1 = manager.start_bug_capture(&id_a).unwrap();
    manager.end_bug_capture(&a_bug1.id).unwrap();
    let a_bug2 = manager.start_bug_capture(&id_a).unwrap();
    manager.end_bug_capture(&a_bug2.id).unwrap();
    manager.end_session(&id_a).unwrap();

    // Session B — 1 bug
    let session_b = manager.start_session(None).unwrap();
    let id_b = session_b.id.clone();
    let b_bug1 = manager.start_bug_capture(&id_b).unwrap();
    manager.end_bug_capture(&b_bug1.id).unwrap();
    manager.end_session(&id_b).unwrap();

    // Verify isolation
    let conn = Connection::open(&env.db_path).unwrap();
    let bug_repo = BugRepository::new(&conn);

    let bugs_a = bug_repo.list_by_session(&id_a).unwrap();
    let bugs_b = bug_repo.list_by_session(&id_b).unwrap();

    assert_eq!(bugs_a.len(), 2, "Session A should have 2 bugs");
    assert_eq!(bugs_b.len(), 1, "Session B should have 1 bug");

    // Bug numbers are per-session
    assert_eq!(bugs_a[0].bug_number, 1);
    assert_eq!(bugs_a[1].bug_number, 2);
    assert_eq!(bugs_b[0].bug_number, 1, "Session B bug numbering restarts");

    // Session folders are different
    let session_repo = SessionRepository::new(&conn);
    let sa = session_repo.get(&id_a).unwrap().unwrap();
    let sb = session_repo.get(&id_b).unwrap().unwrap();
    assert_ne!(sa.folder_path, sb.folder_path);
}

/// Test 9: File-watcher simulation — write a real PNG to the bug folder and
/// verify it can be registered as a capture record.
#[test]
fn test_file_watcher_simulated_detection() {
    let env = TestEnv::new();
    let (manager, _) = env.session_manager();

    let session = manager.start_session(None).unwrap();
    let session_id = session.id.clone();
    let bug = manager.start_bug_capture(&session_id).unwrap();

    // Simulate the file-watcher detecting a new screenshot:
    // Write multiple screenshot files to the bug folder
    let bug_folder = Path::new(&bug.folder_path);
    for i in 1..=3_u32 {
        let file_name = format!("screenshot_{:03}.png", i);
        let file_path = bug_folder.join(&file_name);
        std::fs::write(&file_path, format!("fake-png-data-{}", i).as_bytes()).unwrap();
        assert!(file_path.exists(), "Screenshot {} should exist", i);

        // Register each as a capture in the DB
        let conn = Connection::open(&env.db_path).unwrap();
        let capture_repo = CaptureRepository::new(&conn);
        let capture = Capture {
            id: Uuid::new_v4().to_string(),
            bug_id: Some(bug.id.clone()),
            session_id: session_id.clone(),
            file_name: file_name.clone(),
            file_path: file_path.to_string_lossy().to_string(),
            file_type: CaptureType::Screenshot,
            annotated_path: None,
            file_size_bytes: Some(file_path.metadata().unwrap().len() as i64),
            is_console_capture: false,
            parsed_content: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        capture_repo.create(&capture).unwrap();
    }

    // Verify all 3 captures are registered
    let conn = Connection::open(&env.db_path).unwrap();
    let capture_repo = CaptureRepository::new(&conn);
    let captures = capture_repo.list_by_bug(&bug.id).unwrap();
    assert_eq!(captures.len(), 3, "All 3 screenshots should be registered");
    assert!(
        captures.iter().all(|c| c.file_type == CaptureType::Screenshot),
        "All captures should be screenshots"
    );

    manager.end_bug_capture(&bug.id).unwrap();
    manager.end_session(&session_id).unwrap();
}

/// Test 10: Settings persistence across manager instances.
#[test]
fn test_settings_persistence() {
    let env = TestEnv::new();

    // Write settings via the first manager instance
    {
        let conn = Connection::open(&env.db_path).unwrap();
        let settings_repo = SettingsRepository::new(&conn);
        settings_repo
            .set("capture_directory", "/custom/captures")
            .unwrap();
        settings_repo
            .set("hotkey_trigger", "ctrl+shift+s")
            .unwrap();
    }

    // Create a new manager instance (simulating restart) and verify settings survived
    let conn = Connection::open(&env.db_path).unwrap();
    let settings_repo = SettingsRepository::new(&conn);

    let capture_dir = settings_repo.get("capture_directory").unwrap();
    assert_eq!(
        capture_dir,
        Some("/custom/captures".to_string()),
        "capture_directory should persist"
    );

    let hotkey = settings_repo.get("hotkey_trigger").unwrap();
    assert_eq!(
        hotkey,
        Some("ctrl+shift+s".to_string()),
        "hotkey_trigger should persist"
    );

    // Nonexistent key returns None
    let missing = settings_repo.get("nonexistent_key").unwrap();
    assert_eq!(missing, None);
}

/// Test 11: Hotkey state transitions — active_bug tracks the current capture state.
#[test]
fn test_hotkey_state_transitions() {
    let env = TestEnv::new();
    let (manager, _) = env.session_manager();

    // Before any session: no active session or bug
    assert_eq!(manager.get_active_session_id(), None);
    assert_eq!(manager.get_active_bug_id(), None);

    let session = manager.start_session(None).unwrap();
    let session_id = session.id.clone();

    // After start: active session, no active bug
    assert_eq!(manager.get_active_session_id(), Some(session_id.clone()));
    assert_eq!(manager.get_active_bug_id(), None);

    // Trigger hotkey → start_bug_capture
    let bug = manager.start_bug_capture(&session_id).unwrap();
    assert_eq!(manager.get_active_bug_id(), Some(bug.id.clone()));

    // Trigger hotkey again → end_bug_capture
    manager.end_bug_capture(&bug.id).unwrap();
    assert_eq!(manager.get_active_bug_id(), None);

    // Ending the session clears active session
    manager.end_session(&session_id).unwrap();
    assert_eq!(manager.get_active_session_id(), None);
    assert_eq!(manager.get_active_bug_id(), None);
}

/// Test 12: End session also clears an in-progress bug capture.
#[test]
fn test_end_session_clears_active_bug() {
    let env = TestEnv::new();
    let (manager, _) = env.session_manager();

    let session = manager.start_session(None).unwrap();
    let session_id = session.id.clone();

    // Start a bug but don't end it (simulates sudden session end)
    let bug = manager.start_bug_capture(&session_id).unwrap();
    assert_eq!(manager.get_active_bug_id(), Some(bug.id));

    // End session clears both active session and active bug
    manager.end_session(&session_id).unwrap();
    assert_eq!(manager.get_active_session_id(), None);
    assert_eq!(manager.get_active_bug_id(), None);
}

/// Test 13: Event payload content validation.
#[test]
fn test_event_payload_content() {
    let env = TestEnv::new();
    let (manager, emitter) = env.session_manager();

    let session = manager.start_session(None).unwrap();
    let session_id = session.id.clone();

    // Validate session:started payload
    let events = emitter.events();
    assert_eq!(events.len(), 1);
    let (name, payload) = &events[0];
    assert_eq!(name, "session:started");
    assert_eq!(payload["sessionId"], session_id.as_str());
    assert!(payload["folderPath"].is_string());
    assert!(payload["startedAt"].is_string());

    // Capture a bug and validate bug:capture-started payload
    let bug = manager.start_bug_capture(&session_id).unwrap();
    let events = emitter.events();
    let (name, payload) = &events[1];
    assert_eq!(name, "bug:capture-started");
    assert_eq!(payload["bugId"], bug.id.as_str());
    assert_eq!(payload["sessionId"], session_id.as_str());
    assert_eq!(payload["bugNumber"], 1);
    assert_eq!(payload["displayId"], "BUG-001");
    assert!(payload["folderPath"].is_string());

    manager.end_bug_capture(&bug.id).unwrap();
    manager.end_session(&session_id).unwrap();

    // Validate session:ended payload
    let events = emitter.events();
    let (name, payload) = &events[3];
    assert_eq!(name, "session:ended");
    assert_eq!(payload["sessionId"], session_id.as_str());
    assert!(payload["endedAt"].is_string());
}
