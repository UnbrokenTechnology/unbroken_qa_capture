use rusqlite::{Connection, Result as SqlResult, params};
use crate::database::models::{Capture, CaptureType};

/// Trait defining capture operations
#[allow(dead_code)]
pub trait CaptureOps {
    fn create(&self, capture: &Capture) -> SqlResult<()>;
    fn get(&self, id: &str) -> SqlResult<Option<Capture>>;
    fn update(&self, capture: &Capture) -> SqlResult<()>;
    fn delete(&self, id: &str) -> SqlResult<()>;
    fn list_by_bug(&self, bug_id: &str) -> SqlResult<Vec<Capture>>;
    fn list_by_session(&self, session_id: &str) -> SqlResult<Vec<Capture>>;
    fn list_console_captures(&self, bug_id: &str) -> SqlResult<Vec<Capture>>;
    fn list_unsorted(&self, session_id: &str) -> SqlResult<Vec<Capture>>;
}

/// Capture repository implementation
#[allow(dead_code)]
pub struct CaptureRepository<'a> {
    conn: &'a Connection,
}

impl<'a> CaptureRepository<'a> {
    #[allow(dead_code)]
    pub fn new(conn: &'a Connection) -> Self {
        CaptureRepository { conn }
    }
}

impl<'a> CaptureOps for CaptureRepository<'a> {
    fn create(&self, capture: &Capture) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO captures (id, bug_id, session_id, file_name, file_path, file_type, annotated_path, file_size_bytes, is_console_capture, parsed_content, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                capture.id,
                capture.bug_id,
                capture.session_id,
                capture.file_name,
                capture.file_path,
                capture.file_type.as_str(),
                capture.annotated_path,
                capture.file_size_bytes,
                capture.is_console_capture,
                capture.parsed_content,
                capture.created_at,
            ],
        )?;
        Ok(())
    }

    fn get(&self, id: &str) -> SqlResult<Option<Capture>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, bug_id, session_id, file_name, file_path, file_type, annotated_path, file_size_bytes, is_console_capture, parsed_content, created_at
             FROM captures WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let type_str: String = row.get(5)?;
            Ok(Some(Capture {
                id: row.get(0)?,
                bug_id: row.get(1)?,
                session_id: row.get(2)?,
                file_name: row.get(3)?,
                file_path: row.get(4)?,
                file_type: CaptureType::from_str(&type_str).unwrap_or(CaptureType::Screenshot),
                annotated_path: row.get(6)?,
                file_size_bytes: row.get(7)?,
                is_console_capture: row.get(8)?,
                parsed_content: row.get(9)?,
                created_at: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn update(&self, capture: &Capture) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE captures SET bug_id = ?2, session_id = ?3, file_name = ?4, file_path = ?5, file_type = ?6, annotated_path = ?7, file_size_bytes = ?8, is_console_capture = ?9, parsed_content = ?10
             WHERE id = ?1",
            params![
                capture.id,
                capture.bug_id,
                capture.session_id,
                capture.file_name,
                capture.file_path,
                capture.file_type.as_str(),
                capture.annotated_path,
                capture.file_size_bytes,
                capture.is_console_capture,
                capture.parsed_content,
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM captures WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn list_by_bug(&self, bug_id: &str) -> SqlResult<Vec<Capture>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, bug_id, session_id, file_name, file_path, file_type, annotated_path, file_size_bytes, is_console_capture, parsed_content, created_at
             FROM captures WHERE bug_id = ?1 ORDER BY created_at ASC"
        )?;

        let rows = stmt.query_map(params![bug_id], |row| {
            let type_str: String = row.get(5)?;
            Ok(Capture {
                id: row.get(0)?,
                bug_id: row.get(1)?,
                session_id: row.get(2)?,
                file_name: row.get(3)?,
                file_path: row.get(4)?,
                file_type: CaptureType::from_str(&type_str).unwrap_or(CaptureType::Screenshot),
                annotated_path: row.get(6)?,
                file_size_bytes: row.get(7)?,
                is_console_capture: row.get(8)?,
                parsed_content: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;

        rows.collect()
    }

    fn list_by_session(&self, session_id: &str) -> SqlResult<Vec<Capture>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, bug_id, session_id, file_name, file_path, file_type, annotated_path, file_size_bytes, is_console_capture, parsed_content, created_at
             FROM captures WHERE session_id = ?1 ORDER BY created_at ASC"
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            let type_str: String = row.get(5)?;
            Ok(Capture {
                id: row.get(0)?,
                bug_id: row.get(1)?,
                session_id: row.get(2)?,
                file_name: row.get(3)?,
                file_path: row.get(4)?,
                file_type: CaptureType::from_str(&type_str).unwrap_or(CaptureType::Screenshot),
                annotated_path: row.get(6)?,
                file_size_bytes: row.get(7)?,
                is_console_capture: row.get(8)?,
                parsed_content: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;

        rows.collect()
    }

    fn list_console_captures(&self, bug_id: &str) -> SqlResult<Vec<Capture>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, bug_id, session_id, file_name, file_path, file_type, annotated_path, file_size_bytes, is_console_capture, parsed_content, created_at
             FROM captures WHERE bug_id = ?1 AND is_console_capture = TRUE ORDER BY created_at ASC"
        )?;

        let rows = stmt.query_map(params![bug_id], |row| {
            let type_str: String = row.get(5)?;
            Ok(Capture {
                id: row.get(0)?,
                bug_id: row.get(1)?,
                session_id: row.get(2)?,
                file_name: row.get(3)?,
                file_path: row.get(4)?,
                file_type: CaptureType::from_str(&type_str).unwrap_or(CaptureType::Screenshot),
                annotated_path: row.get(6)?,
                file_size_bytes: row.get(7)?,
                is_console_capture: row.get(8)?,
                parsed_content: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;

        rows.collect()
    }

    fn list_unsorted(&self, session_id: &str) -> SqlResult<Vec<Capture>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, bug_id, session_id, file_name, file_path, file_type, annotated_path, file_size_bytes, is_console_capture, parsed_content, created_at
             FROM captures WHERE session_id = ?1 AND bug_id IS NULL ORDER BY created_at ASC"
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            let type_str: String = row.get(5)?;
            Ok(Capture {
                id: row.get(0)?,
                bug_id: row.get(1)?,
                session_id: row.get(2)?,
                file_name: row.get(3)?,
                file_path: row.get(4)?,
                file_type: CaptureType::from_str(&type_str).unwrap_or(CaptureType::Screenshot),
                annotated_path: row.get(6)?,
                file_size_bytes: row.get(7)?,
                is_console_capture: row.get(8)?,
                parsed_content: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;

        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{Database, SessionOps, SessionRepository, BugOps, BugRepository};
    use crate::database::models::{Session, SessionStatus, Bug, BugType, BugStatus};

    fn create_test_session(db: &Database, id: &str) {
        let session = Session {
            id: id.to_string(),
            started_at: "2024-01-01T10:00:00Z".to_string(),
            ended_at: None,
            status: SessionStatus::Active,
            folder_path: "/test/sessions/session1".to_string(),
            session_notes: None,
            environment_json: None,
            original_snip_path: None,
            created_at: "2024-01-01T10:00:00Z".to_string(),
            profile_id: None,
        };
        let repo = SessionRepository::new(db.connection());
        repo.create(&session).unwrap();
    }

    fn create_test_bug(db: &Database, session_id: &str, bug_id: &str) {
        let bug = Bug {
            id: bug_id.to_string(),
            session_id: session_id.to_string(),
            bug_number: 1,
            display_id: "Bug-01".to_string(),
            bug_type: BugType::Bug,
            title: Some("Test bug".to_string()),
            notes: None,
            description: None,
            ai_description: None,
            status: BugStatus::Captured,
            meeting_id: None,
            software_version: None,
            console_parse_json: None,
            metadata_json: None,
            custom_metadata: None,
            folder_path: "/test/bugs/bug-1".to_string(),
            created_at: "2024-01-01T10:00:00Z".to_string(),
            updated_at: "2024-01-01T10:00:00Z".to_string(),
        };
        let repo = BugRepository::new(db.connection());
        repo.create(&bug).unwrap();
    }

    fn create_test_capture(session_id: &str, bug_id: &str, capture_id: &str, is_console: bool) -> Capture {
        Capture {
            id: capture_id.to_string(),
            bug_id: Some(bug_id.to_string()),
            session_id: session_id.to_string(),
            file_name: "screenshot.png".to_string(),
            file_path: "captures/screenshot.png".to_string(),
            file_type: CaptureType::Screenshot,
            annotated_path: None,
            file_size_bytes: Some(1024),
            is_console_capture: is_console,
            parsed_content: None,
            created_at: "2024-01-01T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_create_capture() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-1");
        create_test_bug(&db, "session-1", "bug-1");
        let repo = CaptureRepository::new(db.connection());
        let capture = create_test_capture("session-1", "bug-1", "capture-1", false);

        let result = repo.create(&capture);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_capture() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-2");
        create_test_bug(&db, "session-2", "bug-2");
        let repo = CaptureRepository::new(db.connection());
        let capture = create_test_capture("session-2", "bug-2", "capture-2", false);

        repo.create(&capture).unwrap();
        let retrieved = repo.get("capture-2").unwrap();

        assert!(retrieved.is_some());
        let retrieved_capture = retrieved.unwrap();
        assert_eq!(retrieved_capture.id, capture.id);
        assert_eq!(retrieved_capture.file_name, capture.file_name);
    }

    #[test]
    fn test_update_capture() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-3");
        create_test_bug(&db, "session-3", "bug-3");
        let repo = CaptureRepository::new(db.connection());
        let mut capture = create_test_capture("session-3", "bug-3", "capture-3", false);

        repo.create(&capture).unwrap();

        capture.annotated_path = Some("captures/screenshot_annotated.png".to_string());
        capture.parsed_content = Some("Console error text".to_string());
        repo.update(&capture).unwrap();

        let updated = repo.get("capture-3").unwrap().unwrap();
        assert_eq!(updated.annotated_path, Some("captures/screenshot_annotated.png".to_string()));
        assert_eq!(updated.parsed_content, Some("Console error text".to_string()));
    }

    #[test]
    fn test_delete_capture() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-4");
        create_test_bug(&db, "session-4", "bug-4");
        let repo = CaptureRepository::new(db.connection());
        let capture = create_test_capture("session-4", "bug-4", "capture-4", false);

        repo.create(&capture).unwrap();
        repo.delete("capture-4").unwrap();

        let result = repo.get("capture-4").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_by_bug() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-5");
        create_test_bug(&db, "session-5", "bug-5");
        let repo = CaptureRepository::new(db.connection());

        repo.create(&create_test_capture("session-5", "bug-5", "capture-5", false)).unwrap();
        repo.create(&create_test_capture("session-5", "bug-5", "capture-6", false)).unwrap();

        let captures = repo.list_by_bug("bug-5").unwrap();
        assert_eq!(captures.len(), 2);
    }

    #[test]
    fn test_list_by_session() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-6");
        create_test_bug(&db, "session-6", "bug-6");
        let repo = CaptureRepository::new(db.connection());

        repo.create(&create_test_capture("session-6", "bug-6", "capture-7", false)).unwrap();
        repo.create(&create_test_capture("session-6", "bug-6", "capture-8", false)).unwrap();

        let captures = repo.list_by_session("session-6").unwrap();
        assert_eq!(captures.len(), 2);
    }

    #[test]
    fn test_list_console_captures() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-7");
        create_test_bug(&db, "session-7", "bug-7");
        let repo = CaptureRepository::new(db.connection());

        repo.create(&create_test_capture("session-7", "bug-7", "capture-9", false)).unwrap();
        repo.create(&create_test_capture("session-7", "bug-7", "capture-10", true)).unwrap();
        repo.create(&create_test_capture("session-7", "bug-7", "capture-11", true)).unwrap();

        let console_captures = repo.list_console_captures("bug-7").unwrap();
        assert_eq!(console_captures.len(), 2);
        assert!(console_captures.iter().all(|c| c.is_console_capture));
    }

    #[test]
    fn test_list_unsorted() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-8");
        create_test_bug(&db, "session-8", "bug-8");
        let repo = CaptureRepository::new(db.connection());

        // Create a capture associated with a bug
        repo.create(&create_test_capture("session-8", "bug-8", "capture-12", false)).unwrap();

        // Create an unsorted capture (bug_id = None)
        let unsorted = Capture {
            id: "capture-13".to_string(),
            bug_id: None,
            session_id: "session-8".to_string(),
            file_name: "orphan.png".to_string(),
            file_path: "/test/_unsorted/orphan.png".to_string(),
            file_type: CaptureType::Screenshot,
            annotated_path: None,
            file_size_bytes: Some(512),
            is_console_capture: false,
            parsed_content: None,
            created_at: "2024-01-01T10:00:00Z".to_string(),
        };
        repo.create(&unsorted).unwrap();

        let unsorted_list = repo.list_unsorted("session-8").unwrap();
        assert_eq!(unsorted_list.len(), 1);
        assert_eq!(unsorted_list[0].id, "capture-13");
        assert!(unsorted_list[0].bug_id.is_none());
    }
}
