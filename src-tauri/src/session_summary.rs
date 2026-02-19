//! Session Summary Generation Module
//!
//! Generates session-summary.md files containing:
//! - Session metadata (date, duration, bug count)
//! - List of all bugs with titles/IDs
//! - Optionally: AI-generated high-level summary from bug descriptions (using Claude CLI)

use chrono::DateTime;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::claude_cli::{ClaudeInvoker, ClaudeRequest, PromptTask, RealClaudeInvoker, load_credentials};
use crate::database::{Bug, BugOps, BugRepository, Session, SessionOps, SessionRepository};

/// Trait for file system operations (enables testing)
pub trait FileWriter: Send + Sync {
    fn write_file(&self, path: &Path, content: &str) -> Result<(), String>;
}

/// Real file writer implementation
pub struct RealFileWriter;

impl FileWriter for RealFileWriter {
    fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write file {}: {}", path.display(), e))
    }
}

/// Session summary generator
pub struct SessionSummaryGenerator {
    db_path: PathBuf,
    file_writer: Arc<dyn FileWriter>,
    claude_invoker: Option<Arc<dyn ClaudeInvoker>>,
}

impl SessionSummaryGenerator {
    /// Create a new generator with real file writer.
    /// Attempts to load Claude Code OAuth credentials for AI summaries.
    /// If credentials are not available, claude_invoker is set to None and AI summaries
    /// are silently skipped.
    pub fn new(db_path: PathBuf) -> Self {
        let claude_invoker = load_credentials()
            .ok()
            .map(|creds| Arc::new(RealClaudeInvoker::new(creds)) as Arc<dyn ClaudeInvoker>);
        Self {
            db_path,
            file_writer: Arc::new(RealFileWriter),
            claude_invoker,
        }
    }

    /// Create a new generator with custom dependencies (for testing)
    #[allow(dead_code)]
    pub fn with_deps(
        db_path: PathBuf,
        file_writer: Arc<dyn FileWriter>,
        claude_invoker: Option<Arc<dyn ClaudeInvoker>>,
    ) -> Self {
        Self {
            db_path,
            file_writer,
            claude_invoker,
        }
    }

    /// Generate session summary markdown
    pub fn generate_summary(
        &self,
        session_id: &str,
        include_ai_summary: bool,
    ) -> Result<String, String> {
        // Get session and bugs from database
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

        // Generate summary content
        let summary_path = PathBuf::from(&session.folder_path).join("session-summary.md");
        let content = self.build_summary_content(&session, &bugs, include_ai_summary)?;

        // Write to file
        self.file_writer.write_file(&summary_path, &content)?;

        Ok(summary_path.to_string_lossy().to_string())
    }

    /// Build summary markdown content
    fn build_summary_content(
        &self,
        session: &Session,
        bugs: &[Bug],
        include_ai_summary: bool,
    ) -> Result<String, String> {
        let mut content = String::new();

        // Title
        content.push_str("# QA Session Summary\n\n");

        // Metadata section
        content.push_str("## Session Information\n\n");
        content.push_str(&format!("- **Session ID:** {}\n", session.id));

        // Parse and format dates
        let started_at = DateTime::parse_from_rfc3339(&session.started_at)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|_| session.started_at.clone());
        content.push_str(&format!("- **Started:** {}\n", started_at));

        if let Some(ended) = &session.ended_at {
            let ended_at = DateTime::parse_from_rfc3339(ended)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|_| ended.clone());
            content.push_str(&format!("- **Ended:** {}\n", ended_at));

            // Calculate duration
            if let (Ok(start), Ok(end)) = (
                DateTime::parse_from_rfc3339(&session.started_at),
                DateTime::parse_from_rfc3339(ended),
            ) {
                let duration = end.signed_duration_since(start);
                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;
                content.push_str(&format!("- **Duration:** {}h {}m\n", hours, minutes));
            }
        } else {
            content.push_str("- **Ended:** In Progress\n");
        }

        content.push_str(&format!("- **Bug Count:** {}\n", bugs.len()));
        content.push_str(&format!("- **Status:** {}\n", session.status.as_str()));

        if let Some(notes) = &session.session_notes {
            if !notes.trim().is_empty() {
                content.push_str(&format!("\n### Session Notes\n\n{}\n", notes));
            }
        }

        content.push('\n');

        // AI-generated overview (optional)
        if include_ai_summary && !bugs.is_empty() {
            if let Ok(ai_summary) = self.generate_ai_overview(bugs) {
                content.push_str("## Overview\n\n");
                content.push_str(&ai_summary);
                content.push_str("\n\n");
            }
        }

        // Bug list section
        if bugs.is_empty() {
            content.push_str("## Bugs Captured\n\nNo bugs captured in this session.\n");
        } else {
            content.push_str("## Bugs Captured\n\n");

            for bug in bugs {
                content.push_str(&format!("### {} - ", bug.display_id));

                if let Some(title) = &bug.title {
                    content.push_str(title);
                } else {
                    content.push_str("(No title)");
                }

                content.push_str("\n\n");

                // Bug metadata
                content.push_str(&format!("- **Type:** {}\n", bug.bug_type.as_str()));
                content.push_str(&format!("- **Status:** {}\n", bug.status.as_str()));

                if let Some(version) = &bug.software_version {
                    content.push_str(&format!("- **Software Version:** {}\n", version));
                }

                // Notes
                if let Some(notes) = &bug.notes {
                    if !notes.trim().is_empty() {
                        content.push_str(&format!("\n**Notes:**\n{}\n", notes));
                    }
                }

                // Description
                if let Some(desc) = &bug.description {
                    if !desc.trim().is_empty() {
                        content.push_str(&format!("\n**Description:**\n{}\n", desc));
                    }
                }

                // AI Description
                if let Some(ai_desc) = &bug.ai_description {
                    if !ai_desc.trim().is_empty() {
                        content.push_str(&format!("\n**AI Description:**\n{}\n", ai_desc));
                    }
                }

                content.push('\n');
            }
        }

        Ok(content)
    }

    /// Generate AI overview of all bugs using Claude CLI
    fn generate_ai_overview(&self, bugs: &[Bug]) -> Result<String, String> {
        // Check if Claude invoker is available
        let invoker = self
            .claude_invoker
            .as_ref()
            .ok_or_else(|| "Claude CLI not configured".to_string())?;

        // Build a summary prompt from all bug data
        let mut prompt = String::new();
        prompt.push_str("You are a QA analyst summarizing a testing session. ");
        prompt.push_str("Below are all the bugs captured in this session. ");
        prompt.push_str("Provide a concise high-level summary (2-3 paragraphs) of the overall quality findings, ");
        prompt.push_str("common themes, and severity of issues encountered.\n\n");

        for bug in bugs {
            prompt.push_str(&format!("**{}**: ", bug.display_id));

            if let Some(title) = &bug.title {
                prompt.push_str(title);
            }

            prompt.push('\n');

            if let Some(ai_desc) = &bug.ai_description {
                prompt.push_str(ai_desc);
            } else if let Some(desc) = &bug.description {
                prompt.push_str(desc);
            } else if let Some(notes) = &bug.notes {
                prompt.push_str(notes);
            }

            prompt.push_str("\n\n");
        }

        prompt.push_str("\nProvide a high-level summary of this testing session's findings.\n");

        // Create request
        let request = ClaudeRequest::new_text(prompt, PromptTask::Custom)
            .with_timeout(120); // 2 minute timeout for summaries

        // Invoke Claude
        let response = invoker
            .invoke(request)
            .map_err(|e| format!("Failed to generate AI summary: {}", e))?;

        Ok(response.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{init_database, BugStatus, BugType, SessionStatus};
    use std::collections::HashMap;
    use std::sync::Mutex as StdMutex;

    // Mock Claude invoker for testing
    struct MockClaudeInvoker {
        should_succeed: bool,
        response_content: String,
    }

    impl ClaudeInvoker for MockClaudeInvoker {
        fn invoke(&self, request: ClaudeRequest) -> Result<crate::claude_cli::ClaudeResponse, crate::claude_cli::ClaudeError> {
            if self.should_succeed {
                Ok(crate::claude_cli::ClaudeResponse {
                    content: self.response_content.clone(),
                    task: request.task,
                    bug_id: request.bug_id,
                })
            } else {
                Err(crate::claude_cli::ClaudeError::InvocationFailed("Mock failure".to_string()))
            }
        }
    }

    // Mock file writer for testing
    struct MockFileWriter {
        files: Arc<StdMutex<HashMap<PathBuf, String>>>,
    }

    impl MockFileWriter {
        fn new() -> Self {
            MockFileWriter {
                files: Arc::new(StdMutex::new(HashMap::new())),
            }
        }

        fn get_written_files(&self) -> HashMap<PathBuf, String> {
            self.files.lock().unwrap().clone()
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

    fn create_test_session(conn: &Connection) -> Session {
        let session = Session {
            id: "session-123".to_string(),
            started_at: "2024-01-15T10:00:00Z".to_string(),
            ended_at: Some("2024-01-15T12:30:00Z".to_string()),
            status: SessionStatus::Ended,
            folder_path: "/tmp/test-session".to_string(),
            session_notes: Some("Test session notes".to_string()),
            environment_json: None,
            original_snip_path: None,
            created_at: "2024-01-15T10:00:00Z".to_string(),
        };

        SessionRepository::new(conn).create(&session).unwrap();
        session
    }

    fn create_test_bugs(conn: &Connection, session_id: &str) -> Vec<Bug> {
        let bugs = vec![
            Bug {
                id: "bug-1".to_string(),
                session_id: session_id.to_string(),
                bug_number: 1,
                display_id: "BUG-001".to_string(),
                bug_type: BugType::Bug,
                title: Some("Login button not responding".to_string()),
                notes: Some("Clicked multiple times, no response".to_string()),
                description: None,
                ai_description: Some("The login button does not respond to clicks.".to_string()),
                status: BugStatus::Captured,
                meeting_id: None,
                software_version: Some("1.2.3".to_string()),
                console_parse_json: None,
                metadata_json: None,
                custom_metadata: None,
                folder_path: "/tmp/test-session/bug_001".to_string(),
                created_at: "2024-01-15T10:15:00Z".to_string(),
                updated_at: "2024-01-15T10:15:00Z".to_string(),
            },
            Bug {
                id: "bug-2".to_string(),
                session_id: session_id.to_string(),
                bug_number: 2,
                display_id: "BUG-002".to_string(),
                bug_type: BugType::Feedback,
                title: Some("Is this behavior expected?".to_string()),
                notes: Some("Form submits without validation".to_string()),
                description: None,
                ai_description: None,
                status: BugStatus::Captured,
                meeting_id: None,
                software_version: None,
                console_parse_json: None,
                metadata_json: None,
                custom_metadata: None,
                folder_path: "/tmp/test-session/bug_002".to_string(),
                created_at: "2024-01-15T11:00:00Z".to_string(),
                updated_at: "2024-01-15T11:00:00Z".to_string(),
            },
        ];

        let bug_repo = BugRepository::new(conn);
        for bug in &bugs {
            bug_repo.create(bug).unwrap();
        }

        bugs
    }

    #[test]
    fn test_generate_summary_basic() {
        let temp_dir = std::env::temp_dir().join(format!("test_summary_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let db_path = temp_dir.join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        init_database(&conn).unwrap();

        let session = create_test_session(&conn);
        let _bugs = create_test_bugs(&conn, &session.id);

        let file_writer = Arc::new(MockFileWriter::new());
        let generator = SessionSummaryGenerator::with_deps(db_path, file_writer.clone(), None);

        let result = generator.generate_summary(&session.id, false);
        assert!(result.is_ok());

        let files = file_writer.get_written_files();
        assert_eq!(files.len(), 1);

        let content = files.values().next().unwrap();
        eprintln!("Generated content:\n{}", content);
        assert!(content.contains("# QA Session Summary"));
        assert!(content.contains("session-123"));
        assert!(content.contains("BUG-001"));
        assert!(content.contains("BUG-002"));
        assert!(content.contains("Login button not responding"));
        assert!(content.contains("**Duration:**"));
        assert!(content.contains("**Bug Count:**"));
    }

    #[test]
    fn test_generate_summary_with_ai() {
        let temp_dir = std::env::temp_dir().join(format!("test_summary_ai_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let db_path = temp_dir.join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        init_database(&conn).unwrap();

        let session = create_test_session(&conn);
        let _bugs = create_test_bugs(&conn, &session.id);

        let file_writer = Arc::new(MockFileWriter::new());
        let mock_claude: Arc<dyn ClaudeInvoker> = Arc::new(MockClaudeInvoker {
            should_succeed: true,
            response_content: "This session found 2 critical issues affecting user login.".to_string(),
        });

        let generator =
            SessionSummaryGenerator::with_deps(db_path, file_writer.clone(), Some(mock_claude));

        let result = generator.generate_summary(&session.id, true);
        assert!(result.is_ok());

        let files = file_writer.get_written_files();
        let content = files.values().next().unwrap();

        assert!(content.contains("# QA Session Summary"));
        assert!(content.contains("## Overview"));
        assert!(content.contains("This session found 2 critical issues"));
    }

    #[test]
    fn test_generate_summary_no_bugs() {
        let temp_dir = std::env::temp_dir().join(format!("test_summary_empty_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let db_path = temp_dir.join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        init_database(&conn).unwrap();

        let session = create_test_session(&conn);

        let file_writer = Arc::new(MockFileWriter::new());
        let generator = SessionSummaryGenerator::with_deps(db_path, file_writer.clone(), None);

        let result = generator.generate_summary(&session.id, false);
        assert!(result.is_ok());

        let files = file_writer.get_written_files();
        let content = files.values().next().unwrap();

        eprintln!("No bugs content:\n{}", content);
        assert!(content.contains("# QA Session Summary"));
        assert!(content.contains("No bugs captured in this session"));
        assert!(content.contains("**Bug Count:**"));
    }

    #[test]
    fn test_duration_calculation() {
        let temp_dir = std::env::temp_dir().join(format!("test_duration_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let db_path = temp_dir.join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        init_database(&conn).unwrap();

        let session = create_test_session(&conn);

        let file_writer = Arc::new(MockFileWriter::new());
        let generator = SessionSummaryGenerator::with_deps(db_path, file_writer.clone(), None);

        let result = generator.generate_summary(&session.id, false);
        assert!(result.is_ok());

        let files = file_writer.get_written_files();
        let content = files.values().next().unwrap();

        eprintln!("Duration test content:\n{}", content);
        // Session was from 10:00 to 12:30, so 2h 30m
        assert!(content.contains("**Duration:**"));
    }
}
