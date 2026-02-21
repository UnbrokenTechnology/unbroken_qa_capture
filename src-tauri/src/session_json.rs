//! Session JSON Generation Module
//!
//! Generates .session.json files in each session folder containing complete,
//! machine-readable session metadata. This enables third-party tools and scripts
//! to process session data without accessing the SQLite database.
//!
//! Schema (per PRD Section 10):
//! - id, startedAt, endedAt, status, environment, bugs[]
//! - Each bug: displayId, type, title, description, captures, metadata

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::database::{Bug, BugOps, BugRepository, Session, SessionOps, SessionRepository};
use crate::session_summary::FileWriter;

/// The root JSON structure written to .session.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionJson {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub environment: Option<Value>,
    pub bugs: Vec<BugJson>,
}

/// Bug entry within .session.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BugJson {
    pub id: String,
    pub display_id: String,
    #[serde(rename = "type")]
    pub bug_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub captures: Vec<String>,
    pub metadata: Value,
}

/// Generates and writes .session.json for a given session
pub struct SessionJsonWriter {
    db_path: PathBuf,
    file_writer: Arc<dyn FileWriter>,
}

impl SessionJsonWriter {
    /// Create a new writer with real file I/O
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            file_writer: Arc::new(crate::session_summary::RealFileWriter),
        }
    }

    /// Create a writer with injected dependencies (for testing)
    #[allow(dead_code)]
    pub fn with_deps(db_path: PathBuf, file_writer: Arc<dyn FileWriter>) -> Self {
        Self { db_path, file_writer }
    }

    /// Write or update the .session.json file for the given session.
    ///
    /// Reads the current session and its bugs from the database, builds the JSON,
    /// and writes it to `{session_folder}/.session.json`.
    pub fn write(&self, session_id: &str) -> Result<String, String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let session_repo = SessionRepository::new(&conn);
        let bug_repo = BugRepository::new(&conn);

        let session = session_repo
            .get(session_id)
            .map_err(|e| format!("Failed to get session: {}", e))?
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        let bugs = bug_repo
            .list_by_session(session_id)
            .map_err(|e| format!("Failed to list bugs: {}", e))?;

        let json = self.build_session_json(&session, &bugs);
        let content = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("Failed to serialize session JSON: {}", e))?;

        let output_path = PathBuf::from(&session.folder_path).join(".session.json");
        self.file_writer.write_file(&output_path, &content)?;

        Ok(output_path.to_string_lossy().to_string())
    }

    /// Build the SessionJson data structure from database records
    fn build_session_json(&self, session: &Session, bugs: &[Bug]) -> SessionJson {
        // Parse environment JSON if present
        let environment = session
            .environment_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());

        let bug_jsons = bugs.iter().map(|b| self.build_bug_json(b)).collect();

        SessionJson {
            id: session.id.clone(),
            started_at: session.started_at.clone(),
            ended_at: session.ended_at.clone(),
            status: session.status.as_str().to_string(),
            environment,
            bugs: bug_jsons,
        }
    }

    /// Build a BugJson entry from a Bug record
    fn build_bug_json(&self, bug: &Bug) -> BugJson {
        // Build metadata object from available bug fields
        let mut metadata = serde_json::Map::new();

        if let Some(ref v) = bug.meeting_id {
            metadata.insert("meetingId".to_string(), Value::String(v.clone()));
        }
        if let Some(ref v) = bug.software_version {
            metadata.insert("softwareVersion".to_string(), Value::String(v.clone()));
        }
        if let Some(ref json_str) = bug.console_parse_json {
            if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
                metadata.insert("consoleParse".to_string(), parsed);
            }
        }
        if let Some(ref json_str) = bug.metadata_json {
            if let Ok(Value::Object(extra)) = serde_json::from_str::<Value>(json_str) {
                for (k, v) in extra {
                    metadata.insert(k, v);
                }
            }
        }

        // Collect capture file names from bug folder if it exists
        // We enumerate paths rather than querying the DB to keep this portable
        let captures = list_captures_in_folder(Path::new(&bug.folder_path));

        BugJson {
            id: bug.id.clone(),
            display_id: bug.display_id.clone(),
            bug_type: bug.bug_type.as_str().to_string(),
            title: bug.title.clone(),
            description: bug.description.clone().or_else(|| bug.ai_description.clone()),
            captures,
            metadata: Value::Object(metadata),
        }
    }
}

/// List capture filenames inside a bug folder, sorted alphabetically.
/// Returns empty vec if folder doesn't exist (e.g. in tests with mock FS).
fn list_captures_in_folder(folder: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };

    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| {
            // Skip hidden files like .bug-notes.txt
            let name = e.file_name();
            let s = name.to_string_lossy();
            !s.starts_with('.')
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{init_database, BugStatus, BugType, SessionStatus};
    use crate::session_summary::FileWriter;
    use rusqlite::Connection;
    use std::collections::HashMap;
    use std::sync::Mutex as StdMutex;

    struct MockFileWriter {
        files: Arc<StdMutex<HashMap<PathBuf, String>>>,
    }

    impl MockFileWriter {
        fn new() -> Self {
            MockFileWriter {
                files: Arc::new(StdMutex::new(HashMap::new())),
            }
        }

        fn get_file(&self, path: &Path) -> Option<String> {
            self.files.lock().unwrap().get(path).cloned()
        }
    }

    impl FileWriter for MockFileWriter {
        fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), content.to_string());
            Ok(())
        }
    }

    fn setup_db() -> (PathBuf, Connection) {
        let temp_dir =
            std::env::temp_dir().join(format!("test_session_json_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        init_database(&conn).unwrap();
        (db_path, conn)
    }

    fn insert_session(conn: &Connection, id: &str, ended_at: Option<&str>) -> Session {
        use crate::database::SessionOps;
        let session = Session {
            id: id.to_string(),
            started_at: "2024-01-15T10:00:00Z".to_string(),
            ended_at: ended_at.map(|s| s.to_string()),
            status: if ended_at.is_some() {
                SessionStatus::Ended
            } else {
                SessionStatus::Active
            },
            folder_path: "/tmp/test-session".to_string(),
            session_notes: None,
            environment_json: Some(r#"{"os":"Windows 11","display_resolution":"1920x1080"}"#.to_string()),
            original_snip_path: None,
            created_at: "2024-01-15T10:00:00Z".to_string(),
            profile_id: None,
        };
        SessionRepository::new(conn).create(&session).unwrap();
        session
    }

    fn insert_bug(conn: &Connection, session_id: &str, number: i32) -> Bug {
        use crate::database::BugOps;
        let bug = Bug {
            id: format!("bug-{}", number),
            session_id: session_id.to_string(),
            bug_number: number,
            display_id: format!("BUG-{:03}", number),
            bug_type: BugType::Bug,
            title: Some(format!("Bug {}", number)),
            notes: None,
            description: Some(format!("Description of bug {}", number)),
            ai_description: None,
            status: BugStatus::Captured,
            meeting_id: Some("meet-123".to_string()),
            software_version: Some("1.0.0".to_string()),
            console_parse_json: None,
            metadata_json: None,
            custom_metadata: None,
            folder_path: format!("/tmp/test-session/bug_{:03}", number),
            created_at: "2024-01-15T10:15:00Z".to_string(),
            updated_at: "2024-01-15T10:15:00Z".to_string(),
        };
        BugRepository::new(conn).create(&bug).unwrap();
        bug
    }

    #[test]
    fn test_write_creates_session_json_file() {
        let (db_path, conn) = setup_db();
        let session = insert_session(&conn, "sess-1", None);
        let _ = insert_bug(&conn, &session.id, 1);

        let writer_mock = Arc::new(MockFileWriter::new());
        let writer = SessionJsonWriter::with_deps(db_path, writer_mock.clone());

        let result = writer.write(&session.id);
        assert!(result.is_ok(), "write() should succeed");

        let expected_path = PathBuf::from(&session.folder_path).join(".session.json");
        let content = writer_mock.get_file(&expected_path);
        assert!(content.is_some(), ".session.json should be written");
    }

    #[test]
    fn test_session_json_schema() {
        let (db_path, conn) = setup_db();
        let session = insert_session(&conn, "sess-2", Some("2024-01-15T12:00:00Z"));
        let _ = insert_bug(&conn, &session.id, 1);
        let _ = insert_bug(&conn, &session.id, 2);

        let writer_mock = Arc::new(MockFileWriter::new());
        let writer = SessionJsonWriter::with_deps(db_path, writer_mock.clone());

        writer.write(&session.id).unwrap();

        let expected_path = PathBuf::from(&session.folder_path).join(".session.json");
        let raw = writer_mock.get_file(&expected_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();

        assert_eq!(parsed["id"], "sess-2");
        assert_eq!(parsed["startedAt"], "2024-01-15T10:00:00Z");
        assert_eq!(parsed["endedAt"], "2024-01-15T12:00:00Z");
        assert_eq!(parsed["status"], "ended");

        // environment should be parsed from JSON string
        assert!(parsed["environment"].is_object());
        assert_eq!(parsed["environment"]["os"], "Windows 11");

        // bugs array
        let bugs = parsed["bugs"].as_array().unwrap();
        assert_eq!(bugs.len(), 2);

        let b0 = &bugs[0];
        assert_eq!(b0["displayId"], "BUG-001");
        assert_eq!(b0["type"], "bug");
        assert_eq!(b0["title"], "Bug 1");
        assert_eq!(b0["description"], "Description of bug 1");
        assert!(b0["captures"].is_array());
        assert!(b0["metadata"].is_object());
        assert_eq!(b0["metadata"]["meetingId"], "meet-123");
        assert_eq!(b0["metadata"]["softwareVersion"], "1.0.0");
    }

    #[test]
    fn test_active_session_has_no_ended_at() {
        let (db_path, conn) = setup_db();
        let session = insert_session(&conn, "sess-3", None);

        let writer_mock = Arc::new(MockFileWriter::new());
        let writer = SessionJsonWriter::with_deps(db_path, writer_mock.clone());

        writer.write(&session.id).unwrap();

        let expected_path = PathBuf::from(&session.folder_path).join(".session.json");
        let raw = writer_mock.get_file(&expected_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();

        assert_eq!(parsed["status"], "active");
        assert!(parsed["endedAt"].is_null());
        assert_eq!(parsed["bugs"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_nonexistent_session_returns_error() {
        let (db_path, _conn) = setup_db();
        let writer_mock = Arc::new(MockFileWriter::new());
        let writer = SessionJsonWriter::with_deps(db_path, writer_mock.clone());

        let result = writer.write("does-not-exist");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Session not found"));
    }

    #[test]
    fn test_bug_uses_ai_description_fallback() {
        let (db_path, conn) = setup_db();
        let session = insert_session(&conn, "sess-4", None);

        // Insert bug with no description but with ai_description
        use crate::database::BugOps;
        let bug = Bug {
            id: "bug-ai".to_string(),
            session_id: session.id.clone(),
            bug_number: 1,
            display_id: "BUG-001".to_string(),
            bug_type: BugType::Bug,
            title: Some("AI bug".to_string()),
            notes: None,
            description: None,
            ai_description: Some("AI-generated description".to_string()),
            status: BugStatus::Captured,
            meeting_id: None,
            software_version: None,
            console_parse_json: None,
            metadata_json: None,
            custom_metadata: None,
            folder_path: "/tmp/test-session/bug_001".to_string(),
            created_at: "2024-01-15T10:15:00Z".to_string(),
            updated_at: "2024-01-15T10:15:00Z".to_string(),
        };
        BugRepository::new(&conn).create(&bug).unwrap();

        let writer_mock = Arc::new(MockFileWriter::new());
        let writer = SessionJsonWriter::with_deps(db_path, writer_mock.clone());

        writer.write(&session.id).unwrap();

        let expected_path = PathBuf::from(&session.folder_path).join(".session.json");
        let raw = writer_mock.get_file(&expected_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();

        assert_eq!(parsed["bugs"][0]["description"], "AI-generated description");
    }
}
