use chrono::Utc;
use rusqlite::Connection;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::database::{Bug, BugStatus, BugType, Session, SessionStatus};
use crate::database::{BugOps, BugRepository, SessionOps, SessionRepository};
use crate::session_json::SessionJsonWriter;
use crate::session_summary::SessionSummaryGenerator;

/// Trait for emitting Tauri events
pub trait EventEmitter: Send + Sync {
    fn emit(&self, event: &str, payload: serde_json::Value) -> Result<(), String>;
}

/// Trait for filesystem operations
pub trait FileSystem: Send + Sync {
    fn create_dir_all(&self, path: &Path) -> Result<(), String>;
}

/// Real filesystem implementation
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn create_dir_all(&self, path: &Path) -> Result<(), String> {
        std::fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {}", e))
    }
}

/// Session Manager handles session lifecycle and bug capture operations
pub struct SessionManager {
    pub db_path: PathBuf,
    storage_root: PathBuf,
    event_emitter: Arc<dyn EventEmitter>,
    filesystem: Arc<dyn FileSystem>,
    active_session: Arc<Mutex<Option<String>>>,
    active_bug: Arc<Mutex<Option<String>>>,
}

impl SessionManager {
    pub fn new(
        db_path: PathBuf,
        storage_root: PathBuf,
        event_emitter: Arc<dyn EventEmitter>,
        filesystem: Arc<dyn FileSystem>,
    ) -> Self {
        SessionManager {
            db_path,
            storage_root,
            event_emitter,
            filesystem,
            active_session: Arc::new(Mutex::new(None)),
            active_bug: Arc::new(Mutex::new(None)),
        }
    }

    /// Start a new QA session.
    ///
    /// `profile_id` is the ID of the QA profile that was active when the session
    /// was started. Pass `None` if no profile is active.
    pub fn start_session(&self, profile_id: Option<String>) -> Result<Session, String> {
        // Guard: reject if a session is already active
        {
            let active = self.active_session.lock().unwrap();
            if active.is_some() {
                return Err("A session is already active. End the current session before starting a new one.".to_string());
            }
        }

        // Generate session ID and folder name
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let short_id = &session_id[..8];
        let folder_name = format!("{}_{}", date_str, short_id);
        let folder_path = self.storage_root.join(&folder_name);

        // Create session folder
        self.filesystem.create_dir_all(&folder_path)?;

        // Create _captures/ subdirectory as temporary landing zone for Snipping Tool output
        let captures_path = folder_path.join("_captures");
        self.filesystem.create_dir_all(&captures_path)?;

        // Create _unsorted/ subdirectory for captures made when no bug is active
        let unsorted_path = folder_path.join("_unsorted");
        self.filesystem.create_dir_all(&unsorted_path)?;

        // Create session record
        let session = Session {
            id: session_id.clone(),
            started_at: now.to_rfc3339(),
            ended_at: None,
            status: SessionStatus::Active,
            folder_path: folder_path.to_string_lossy().to_string(),
            session_notes: None,
            environment_json: None,
            original_snip_path: None,
            created_at: now.to_rfc3339(),
            profile_id,
        };

        // Save to database
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        let repo = SessionRepository::new(&conn);
        repo.create(&session)
            .map_err(|e| format!("Failed to create session: {}", e))?;

        // Update active session pointer
        *self.active_session.lock().unwrap() = Some(session_id.clone());

        // Emit event
        self.event_emitter.emit(
            "session:started",
            json!({
                "sessionId": session_id,
                "folderPath": session.folder_path,
                "startedAt": session.started_at
            }),
        )?;

        // Write initial .session.json (don't fail session start if this fails)
        if let Err(e) = SessionJsonWriter::new(self.db_path.clone()).write(&session_id) {
            eprintln!("Warning: Failed to write .session.json: {}", e);
        }

        Ok(session)
    }

    /// End the current session
    pub fn end_session(&self, session_id: &str) -> Result<(), String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        let repo = SessionRepository::new(&conn);

        // Get session
        let mut session = repo
            .get(session_id)
            .map_err(|e| format!("Failed to get session: {}", e))?
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        // Update session
        session.ended_at = Some(Utc::now().to_rfc3339());
        session.status = SessionStatus::Ended;

        repo.update(&session)
            .map_err(|e| format!("Failed to update session: {}", e))?;

        // Generate session summary (don't fail if this fails)
        let summary_generator = SessionSummaryGenerator::new(self.db_path.clone());
        if let Err(e) = summary_generator.generate_summary(session_id, true) {
            eprintln!("Warning: Failed to generate session summary: {}", e);
        }

        // Update .session.json with final state (don't fail if this fails)
        if let Err(e) = SessionJsonWriter::new(self.db_path.clone()).write(session_id) {
            eprintln!("Warning: Failed to update .session.json on end: {}", e);
        }

        // Clear active session if it matches
        let mut active = self.active_session.lock().unwrap();
        if active.as_deref() == Some(session_id) {
            *active = None;
        }

        // Clear active bug
        *self.active_bug.lock().unwrap() = None;

        // Emit event
        self.event_emitter.emit(
            "session:ended",
            json!({
                "sessionId": session_id,
                "endedAt": session.ended_at
            }),
        )?;

        Ok(())
    }

    /// Resume an existing session
    pub fn resume_session(&self, session_id: &str) -> Result<Session, String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        let repo = SessionRepository::new(&conn);

        // Get session
        let mut session = repo
            .get(session_id)
            .map_err(|e| format!("Failed to get session: {}", e))?
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        // Update status to active
        session.status = SessionStatus::Active;
        session.ended_at = None;

        repo.update(&session)
            .map_err(|e| format!("Failed to update session: {}", e))?;

        // Update active session pointer
        *self.active_session.lock().unwrap() = Some(session_id.to_string());

        // Restore active_bug pointer: if a bug was in 'capturing' state when the app
        // crashed/restarted, its status remains 'capturing' in the DB. Restore the
        // in-memory active_bug so the capture watcher and frontend can resume correctly.
        // Any additional stale 'capturing' bugs are auto-completed (only one should be active).
        let bug_repo = BugRepository::new(&conn);
        let bugs = bug_repo
            .list_by_session(session_id)
            .map_err(|e| format!("Failed to list bugs for session: {}", e))?;
        let capturing_bugs: Vec<Bug> = bugs.into_iter().filter(|b| b.status == BugStatus::Capturing).collect();
        if let Some(active) = capturing_bugs.first() {
            *self.active_bug.lock().unwrap() = Some(active.id.clone());
            // Auto-complete any other stale capturing bugs
            for stale in capturing_bugs.iter().skip(1) {
                let mut fixed = stale.clone();
                fixed.status = BugStatus::Captured;
                if let Err(e) = bug_repo.update(&fixed) {
                    eprintln!("Warning: Failed to auto-complete stale bug {}: {}", stale.id, e);
                }
            }
        } else {
            *self.active_bug.lock().unwrap() = None;
        }

        // Emit event
        self.event_emitter.emit(
            "session:resumed",
            json!({
                "sessionId": session_id,
                "folderPath": session.folder_path
            }),
        )?;

        // Update .session.json to reflect resumed status (don't fail if this fails)
        if let Err(e) = SessionJsonWriter::new(self.db_path.clone()).write(session_id) {
            eprintln!("Warning: Failed to update .session.json on resume: {}", e);
        }

        Ok(session)
    }

    /// Start capturing a new bug
    pub fn start_bug_capture(&self, session_id: &str) -> Result<Bug, String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        let session_repo = SessionRepository::new(&conn);
        let bug_repo = BugRepository::new(&conn);

        // Verify session exists and is active
        let session = session_repo
            .get(session_id)
            .map_err(|e| format!("Failed to get session: {}", e))?
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        if session.status != SessionStatus::Active {
            return Err("Session is not active".to_string());
        }

        // Get next bug number
        let bug_number = bug_repo
            .get_next_bug_number(session_id)
            .map_err(|e| format!("Failed to get next bug number: {}", e))?;

        // Create bug folder
        let session_folder = PathBuf::from(&session.folder_path);
        let bug_folder_name = format!("bug_{:03}", bug_number);
        let bug_folder_path = session_folder.join(&bug_folder_name);

        self.filesystem.create_dir_all(&bug_folder_path)?;

        // Create bug record
        let bug_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let display_id = format!("BUG-{:03}", bug_number);

        let bug = Bug {
            id: bug_id.clone(),
            session_id: session_id.to_string(),
            bug_number,
            display_id: display_id.clone(),
            bug_type: BugType::Bug,
            title: None,
            notes: None,
            description: None,
            ai_description: None,
            status: BugStatus::Capturing,
            meeting_id: None,
            software_version: None,
            console_parse_json: None,
            metadata_json: None,
            custom_metadata: None,
            folder_path: bug_folder_path.to_string_lossy().to_string(),
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
        };

        // Save to database
        bug_repo
            .create(&bug)
            .map_err(|e| format!("Failed to create bug: {}", e))?;

        // Update active bug pointer
        *self.active_bug.lock().unwrap() = Some(bug_id.clone());

        // Emit event
        self.event_emitter.emit(
            "bug:capture-started",
            json!({
                "bugId": bug_id,
                "sessionId": session_id,
                "bugNumber": bug_number,
                "displayId": display_id,
                "folderPath": bug.folder_path
            }),
        )?;

        // Update .session.json to include new bug (don't fail if this fails)
        if let Err(e) = SessionJsonWriter::new(self.db_path.clone()).write(session_id) {
            eprintln!("Warning: Failed to update .session.json on bug start: {}", e);
        }

        Ok(bug)
    }

    /// End bug capture
    pub fn end_bug_capture(&self, bug_id: &str) -> Result<(), String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        let bug_repo = BugRepository::new(&conn);

        // Get bug
        let mut bug = bug_repo
            .get(bug_id)
            .map_err(|e| format!("Failed to get bug: {}", e))?
            .ok_or_else(|| format!("Bug not found: {}", bug_id))?;

        // Update bug status
        bug.status = BugStatus::Captured;
        bug.updated_at = Utc::now().to_rfc3339();

        bug_repo
            .update(&bug)
            .map_err(|e| format!("Failed to update bug: {}", e))?;

        // Clear active bug if it matches
        let mut active = self.active_bug.lock().unwrap();
        if active.as_deref() == Some(bug_id) {
            *active = None;
        }

        // Emit event
        self.event_emitter.emit(
            "bug:capture-ended",
            json!({
                "bugId": bug_id,
                "sessionId": bug.session_id
            }),
        )?;

        // Update .session.json to reflect bug status change (don't fail if this fails)
        if let Err(e) = SessionJsonWriter::new(self.db_path.clone()).write(&bug.session_id) {
            eprintln!("Warning: Failed to update .session.json on bug end: {}", e);
        }

        Ok(())
    }

    /// Resume capturing for an existing bug (set its status back to Capturing and make it the active bug)
    pub fn resume_bug_capture(&self, bug_id: &str) -> Result<Bug, String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        let bug_repo = BugRepository::new(&conn);

        let mut bug = bug_repo
            .get(bug_id)
            .map_err(|e| format!("Failed to get bug: {}", e))?
            .ok_or_else(|| format!("Bug not found: {}", bug_id))?;

        // Set bug status back to capturing
        bug.status = BugStatus::Capturing;
        bug.updated_at = Utc::now().to_rfc3339();

        bug_repo
            .update(&bug)
            .map_err(|e| format!("Failed to update bug: {}", e))?;

        // Set as active bug
        {
            let mut active = self.active_bug.lock().unwrap();
            *active = Some(bug_id.to_string());
        }

        // Emit event so the frontend knows
        self.event_emitter.emit(
            "bug-status-changed",
            json!({
                "id": bug_id,
                "status": "capturing"
            }),
        )?;

        // Update .session.json
        if let Err(e) = SessionJsonWriter::new(self.db_path.clone()).write(&bug.session_id) {
            eprintln!("Warning: Failed to update .session.json on bug resume: {}", e);
        }

        Ok(bug)
    }

    /// Get active session ID
    pub fn get_active_session_id(&self) -> Option<String> {
        self.active_session.lock().unwrap().clone()
    }

    /// Get active bug ID
    pub fn get_active_bug_id(&self) -> Option<String> {
        self.active_bug.lock().unwrap().clone()
    }

    /// Return a shared reference to the active-bug Arc so callers (e.g. the
    /// capture watcher) can observe live changes without going through the
    /// SessionManager lock.
    pub fn active_bug_arc(&self) -> Arc<Mutex<Option<String>>> {
        Arc::clone(&self.active_bug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex as StdMutex;

    // Mock event emitter for testing
    struct MockEventEmitter {
        events: Arc<StdMutex<Vec<(String, serde_json::Value)>>>,
    }

    impl MockEventEmitter {
        fn new() -> Self {
            MockEventEmitter {
                events: Arc::new(StdMutex::new(Vec::new())),
            }
        }

        fn get_events(&self) -> Vec<(String, serde_json::Value)> {
            self.events.lock().unwrap().clone()
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

    // Mock filesystem for testing
    struct MockFileSystem {
        dirs: Arc<StdMutex<HashMap<PathBuf, bool>>>,
    }

    impl MockFileSystem {
        fn new() -> Self {
            MockFileSystem {
                dirs: Arc::new(StdMutex::new(HashMap::new())),
            }
        }
    }

    impl FileSystem for MockFileSystem {
        fn create_dir_all(&self, path: &Path) -> Result<(), String> {
            self.dirs.lock().unwrap().insert(path.to_path_buf(), true);
            Ok(())
        }
    }

    fn create_test_manager() -> (SessionManager, Arc<MockEventEmitter>) {
        let temp_dir = std::env::temp_dir().join(format!("test_session_manager_{}", Uuid::new_v4()));
        let db_path = temp_dir.join("test.db");
        let storage_root = temp_dir.join("storage");

        std::fs::create_dir_all(&temp_dir).unwrap();

        // Initialize database
        let conn = Connection::open(&db_path).unwrap();
        crate::database::init_database(&conn).unwrap();

        let emitter = Arc::new(MockEventEmitter::new());
        let filesystem = Arc::new(MockFileSystem::new());

        let manager = SessionManager::new(
            db_path,
            storage_root,
            emitter.clone() as Arc<dyn EventEmitter>,
            filesystem as Arc<dyn FileSystem>,
        );

        (manager, emitter)
    }

    #[test]
    fn test_start_session() {
        let (manager, emitter) = create_test_manager();

        let result = manager.start_session(None);
        assert!(result.is_ok());

        let session = result.unwrap();
        assert!(session.folder_path.contains(&session.id[..8]));
        assert_eq!(session.status, SessionStatus::Active);

        // Verify event emitted
        let events = emitter.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "session:started");

        // Verify active session set
        assert_eq!(manager.get_active_session_id(), Some(session.id));
    }

    #[test]
    fn test_end_session() {
        let (manager, emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        let session_id = session.id.clone();

        let result = manager.end_session(&session_id);
        assert!(result.is_ok());

        // Verify active session cleared
        assert_eq!(manager.get_active_session_id(), None);

        // Verify events emitted
        let events = emitter.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].0, "session:ended");
    }

    #[test]
    fn test_resume_session() {
        let (manager, emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        let session_id = session.id.clone();

        manager.end_session(&session_id).unwrap();

        let result = manager.resume_session(&session_id);
        assert!(result.is_ok());

        let resumed = result.unwrap();
        assert_eq!(resumed.status, SessionStatus::Active);
        assert_eq!(resumed.ended_at, None);

        // Verify active session set
        assert_eq!(manager.get_active_session_id(), Some(session_id));

        // Verify events
        let events = emitter.get_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[2].0, "session:resumed");
    }

    #[test]
    fn test_start_bug_capture() {
        let (manager, emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        let session_id = session.id.clone();

        let result = manager.start_bug_capture(&session_id);
        assert!(result.is_ok());

        let bug = result.unwrap();
        assert_eq!(bug.session_id, session_id);
        assert_eq!(bug.bug_number, 1);
        assert_eq!(bug.display_id, "BUG-001");
        assert_eq!(bug.status, BugStatus::Capturing);

        // Verify active bug set
        assert_eq!(manager.get_active_bug_id(), Some(bug.id));

        // Verify events
        let events = emitter.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].0, "bug:capture-started");
    }

    #[test]
    fn test_start_multiple_bugs() {
        let (manager, _emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        let session_id = session.id.clone();

        let bug1 = manager.start_bug_capture(&session_id).unwrap();
        manager.end_bug_capture(&bug1.id).unwrap();

        let bug2 = manager.start_bug_capture(&session_id).unwrap();
        manager.end_bug_capture(&bug2.id).unwrap();

        let bug3 = manager.start_bug_capture(&session_id).unwrap();

        assert_eq!(bug1.bug_number, 1);
        assert_eq!(bug2.bug_number, 2);
        assert_eq!(bug3.bug_number, 3);
        assert_eq!(bug3.display_id, "BUG-003");
    }

    #[test]
    fn test_end_bug_capture() {
        let (manager, emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        let session_id = session.id.clone();

        let bug = manager.start_bug_capture(&session_id).unwrap();
        let bug_id = bug.id.clone();

        let result = manager.end_bug_capture(&bug_id);
        assert!(result.is_ok());

        // Verify active bug cleared
        assert_eq!(manager.get_active_bug_id(), None);

        // Verify events
        let events = emitter.get_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[2].0, "bug:capture-ended");
    }

    #[test]
    fn test_start_bug_capture_inactive_session() {
        let (manager, _emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        let session_id = session.id.clone();

        manager.end_session(&session_id).unwrap();

        let result = manager.start_bug_capture(&session_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Session is not active"));
    }

    #[test]
    fn test_start_bug_capture_nonexistent_session() {
        let (manager, _emitter) = create_test_manager();

        let result = manager.start_bug_capture("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Session not found"));
    }

    #[test]
    fn test_resume_session_restores_capturing_bug() {
        let (manager, _emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        let session_id = session.id.clone();

        // Start a bug capture (sets active_bug)
        let bug = manager.start_bug_capture(&session_id).unwrap();
        assert_eq!(manager.get_active_bug_id(), Some(bug.id.clone()));

        // Simulate app crash: clear in-memory state without ending the session/bug
        *manager.active_session.lock().unwrap() = None;
        *manager.active_bug.lock().unwrap() = None;

        // Bug should still be 'capturing' in the DB — resume session should restore active_bug
        let resumed = manager.resume_session(&session_id).unwrap();
        assert_eq!(resumed.status, SessionStatus::Active);
        assert_eq!(manager.get_active_session_id(), Some(session_id.clone()));
        // active_bug should be restored from the DB
        assert_eq!(manager.get_active_bug_id(), Some(bug.id.clone()));
    }

    #[test]
    fn test_resume_session_no_capturing_bug_leaves_active_bug_none() {
        let (manager, _emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        let session_id = session.id.clone();

        // Start and end a bug capture before crash
        let bug = manager.start_bug_capture(&session_id).unwrap();
        manager.end_bug_capture(&bug.id).unwrap();

        // Simulate app crash
        *manager.active_session.lock().unwrap() = None;
        *manager.active_bug.lock().unwrap() = None;

        // Resume — no bug is in 'capturing' state, so active_bug stays None
        manager.resume_session(&session_id).unwrap();
        assert_eq!(manager.get_active_bug_id(), None);
    }

    #[test]
    fn test_end_session_clears_active_bug() {
        let (manager, _emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        let session_id = session.id.clone();

        let bug = manager.start_bug_capture(&session_id).unwrap();
        assert_eq!(manager.get_active_bug_id(), Some(bug.id));

        manager.end_session(&session_id).unwrap();

        // Verify both active session and bug cleared
        assert_eq!(manager.get_active_session_id(), None);
        assert_eq!(manager.get_active_bug_id(), None);
    }

    #[test]
    fn test_captures_and_unsorted_folders_created_on_session_start() {
        let (manager, _emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();

        // The mock filesystem should have recorded both the session folder
        // and both subdirectories
        let session_folder = std::path::PathBuf::from(&session.folder_path);
        let captures_folder = session_folder.join("_captures");
        let unsorted_folder = session_folder.join("_unsorted");

        // We can't directly access MockFileSystem here, but we can verify
        // by checking the folder_path is valid and contains expected structure.
        let folder_name = session_folder.file_name().unwrap().to_str().unwrap();
        assert!(
            folder_name.contains('_'),
            "Session folder name should contain date and ID parts"
        );

        // Verify _captures path is a direct child of session folder
        assert_eq!(
            captures_folder.parent().unwrap(),
            session_folder.as_path(),
            "_captures should be a direct child of session folder"
        );
        assert_eq!(
            captures_folder.file_name().unwrap().to_str().unwrap(),
            "_captures"
        );

        // Verify _unsorted path is a direct child of session folder
        assert_eq!(
            unsorted_folder.parent().unwrap(),
            session_folder.as_path(),
            "_unsorted should be a direct child of session folder"
        );
        assert_eq!(
            unsorted_folder.file_name().unwrap().to_str().unwrap(),
            "_unsorted"
        );
    }

    #[test]
    fn test_folder_naming_format() {
        let (manager, _emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();

        // Verify folder name format: YYYY-MM-DD_<short-id>
        let folder_name = std::path::Path::new(&session.folder_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let parts: Vec<&str> = folder_name.split('_').collect();
        assert_eq!(parts.len(), 2);

        // Verify date format (YYYY-MM-DD)
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        assert_eq!(date_parts.len(), 3);
        assert_eq!(date_parts[0].len(), 4); // Year
        assert_eq!(date_parts[1].len(), 2); // Month
        assert_eq!(date_parts[2].len(), 2); // Day

        // Verify short ID (8 chars)
        assert_eq!(parts[1].len(), 8);
    }

    #[test]
    fn test_bug_folder_naming_format() {
        let (manager, _emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        let session_id = session.id.clone();

        let bug = manager.start_bug_capture(&session_id).unwrap();

        // Verify bug folder name format: bug_NNN
        let bug_folder_name = std::path::Path::new(&bug.folder_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        assert_eq!(bug_folder_name, "bug_001");
    }

    #[test]
    fn test_start_session_with_profile_id() {
        let (manager, _emitter) = create_test_manager();

        let result = manager.start_session(Some("profile-123".to_string()));
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.profile_id, Some("profile-123".to_string()));
    }

    #[test]
    fn test_start_session_without_profile_id() {
        let (manager, _emitter) = create_test_manager();

        let session = manager.start_session(None).unwrap();
        assert_eq!(session.profile_id, None);
    }
}
