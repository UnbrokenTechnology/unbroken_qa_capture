use rusqlite::{Connection, Result as SqlResult, params};
use crate::database::models::{Bug, BugType, BugStatus, BugUpdate};

/// Trait defining bug operations
#[allow(dead_code)]
pub trait BugOps {
    fn create(&self, bug: &Bug) -> SqlResult<()>;
    fn get(&self, id: &str) -> SqlResult<Option<Bug>>;
    fn update(&self, bug: &Bug) -> SqlResult<()>;
    fn delete(&self, id: &str) -> SqlResult<()>;
    fn list_by_session(&self, session_id: &str) -> SqlResult<Vec<Bug>>;
    fn update_partial(&self, id: &str, update: &BugUpdate) -> SqlResult<()>;
    fn get_next_bug_number(&self, session_id: &str) -> SqlResult<i32>;
}

/// Bug repository implementation
#[allow(dead_code)]
pub struct BugRepository<'a> {
    conn: &'a Connection,
}

impl<'a> BugRepository<'a> {
    #[allow(dead_code)]
    pub fn new(conn: &'a Connection) -> Self {
        BugRepository { conn }
    }
}

impl<'a> BugOps for BugRepository<'a> {
    fn create(&self, bug: &Bug) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO bugs (id, session_id, bug_number, display_id, type, title, notes, description, ai_description, status, meeting_id, software_version, console_parse_json, metadata_json, custom_metadata, folder_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                bug.id,
                bug.session_id,
                bug.bug_number,
                bug.display_id,
                bug.bug_type.as_str(),
                bug.title,
                bug.notes,
                bug.description,
                bug.ai_description,
                bug.status.as_str(),
                bug.meeting_id,
                bug.software_version,
                bug.console_parse_json,
                bug.metadata_json,
                bug.custom_metadata,
                bug.folder_path,
                bug.created_at,
                bug.updated_at,
            ],
        )?;
        Ok(())
    }

    fn get(&self, id: &str) -> SqlResult<Option<Bug>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, bug_number, display_id, type, title, notes, description, ai_description, status, meeting_id, software_version, console_parse_json, metadata_json, custom_metadata, folder_path, created_at, updated_at
             FROM bugs WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let type_str: String = row.get(4)?;
            let status_str: String = row.get(9)?;
            Ok(Some(Bug {
                id: row.get(0)?,
                session_id: row.get(1)?,
                bug_number: row.get(2)?,
                display_id: row.get(3)?,
                bug_type: BugType::from_str(&type_str).unwrap_or(BugType::Bug),
                title: row.get(5)?,
                notes: row.get(6)?,
                description: row.get(7)?,
                ai_description: row.get(8)?,
                status: BugStatus::from_str(&status_str).unwrap_or(BugStatus::Captured),
                meeting_id: row.get(10)?,
                software_version: row.get(11)?,
                console_parse_json: row.get(12)?,
                metadata_json: row.get(13)?,
                custom_metadata: row.get(14)?,
                folder_path: row.get(15)?,
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn update(&self, bug: &Bug) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE bugs SET session_id = ?2, bug_number = ?3, display_id = ?4, type = ?5, title = ?6, notes = ?7, description = ?8, ai_description = ?9, status = ?10, meeting_id = ?11, software_version = ?12, console_parse_json = ?13, metadata_json = ?14, custom_metadata = ?15, folder_path = ?16, updated_at = datetime('now')
             WHERE id = ?1",
            params![
                bug.id,
                bug.session_id,
                bug.bug_number,
                bug.display_id,
                bug.bug_type.as_str(),
                bug.title,
                bug.notes,
                bug.description,
                bug.ai_description,
                bug.status.as_str(),
                bug.meeting_id,
                bug.software_version,
                bug.console_parse_json,
                bug.metadata_json,
                bug.custom_metadata,
                bug.folder_path,
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM bugs WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn list_by_session(&self, session_id: &str) -> SqlResult<Vec<Bug>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, bug_number, display_id, type, title, notes, description, ai_description, status, meeting_id, software_version, console_parse_json, metadata_json, custom_metadata, folder_path, created_at, updated_at
             FROM bugs WHERE session_id = ?1 ORDER BY bug_number ASC"
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            let type_str: String = row.get(4)?;
            let status_str: String = row.get(9)?;
            Ok(Bug {
                id: row.get(0)?,
                session_id: row.get(1)?,
                bug_number: row.get(2)?,
                display_id: row.get(3)?,
                bug_type: BugType::from_str(&type_str).unwrap_or(BugType::Bug),
                title: row.get(5)?,
                notes: row.get(6)?,
                description: row.get(7)?,
                ai_description: row.get(8)?,
                status: BugStatus::from_str(&status_str).unwrap_or(BugStatus::Captured),
                meeting_id: row.get(10)?,
                software_version: row.get(11)?,
                console_parse_json: row.get(12)?,
                metadata_json: row.get(13)?,
                custom_metadata: row.get(14)?,
                folder_path: row.get(15)?,
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
            })
        })?;

        rows.collect()
    }

    fn update_partial(&self, id: &str, update: &BugUpdate) -> SqlResult<()> {
        // Build dynamic UPDATE query based on which fields are present
        let mut query = String::from("UPDATE bugs SET updated_at = datetime('now')");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref bug_type) = update.bug_type {
            query.push_str(", type = ?");
            params_vec.push(Box::new(bug_type.as_str().to_string()));
        }
        if let Some(ref title) = update.title {
            query.push_str(", title = ?");
            params_vec.push(Box::new(title.clone()));
        }
        if let Some(ref notes) = update.notes {
            query.push_str(", notes = ?");
            params_vec.push(Box::new(notes.clone()));
        }
        if let Some(ref description) = update.description {
            query.push_str(", description = ?");
            params_vec.push(Box::new(description.clone()));
        }
        if let Some(ref ai_description) = update.ai_description {
            query.push_str(", ai_description = ?");
            params_vec.push(Box::new(ai_description.clone()));
        }
        if let Some(ref status) = update.status {
            query.push_str(", status = ?");
            params_vec.push(Box::new(status.as_str().to_string()));
        }
        if let Some(ref meeting_id) = update.meeting_id {
            query.push_str(", meeting_id = ?");
            params_vec.push(Box::new(meeting_id.clone()));
        }
        if let Some(ref software_version) = update.software_version {
            query.push_str(", software_version = ?");
            params_vec.push(Box::new(software_version.clone()));
        }
        if let Some(ref custom_metadata) = update.custom_metadata {
            query.push_str(", custom_metadata = ?");
            params_vec.push(Box::new(custom_metadata.clone()));
        }

        query.push_str(" WHERE id = ?");
        params_vec.push(Box::new(id.to_string()));

        // Convert params_vec to slice of trait objects
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter()
            .map(|p| p.as_ref() as &dyn rusqlite::ToSql)
            .collect();

        self.conn.execute(&query, params_refs.as_slice())?;
        Ok(())
    }

    fn get_next_bug_number(&self, session_id: &str) -> SqlResult<i32> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(MAX(bug_number), 0) + 1 FROM bugs WHERE session_id = ?1"
        )?;

        let next_number: i32 = stmt.query_row(params![session_id], |row| row.get(0))?;
        Ok(next_number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{Database, SessionOps, SessionRepository};
    use crate::database::models::{Session, SessionStatus};

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

    fn create_test_bug(session_id: &str, bug_id: &str, bug_number: i32) -> Bug {
        Bug {
            id: bug_id.to_string(),
            session_id: session_id.to_string(),
            bug_number,
            display_id: format!("Bug-{:02}", bug_number),
            bug_type: BugType::Bug,
            title: Some("Test bug".to_string()),
            notes: Some("Test notes".to_string()),
            description: None,
            ai_description: None,
            status: BugStatus::Captured,
            meeting_id: None,
            software_version: None,
            console_parse_json: None,
            metadata_json: None,
            custom_metadata: None,
            folder_path: format!("/test/bugs/bug-{}", bug_number),
            created_at: "2024-01-01T10:00:00Z".to_string(),
            updated_at: "2024-01-01T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_create_bug() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-1");
        let repo = BugRepository::new(db.connection());
        let bug = create_test_bug("session-1", "bug-1", 1);

        let result = repo.create(&bug);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_bug() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-2");
        let repo = BugRepository::new(db.connection());
        let bug = create_test_bug("session-2", "bug-2", 1);

        repo.create(&bug).unwrap();
        let retrieved = repo.get("bug-2").unwrap();

        assert!(retrieved.is_some());
        let retrieved_bug = retrieved.unwrap();
        assert_eq!(retrieved_bug.id, bug.id);
        assert_eq!(retrieved_bug.bug_number, bug.bug_number);
    }

    #[test]
    fn test_update_bug() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-3");
        let repo = BugRepository::new(db.connection());
        let mut bug = create_test_bug("session-3", "bug-3", 1);

        repo.create(&bug).unwrap();

        bug.title = Some("Updated title".to_string());
        bug.status = BugStatus::Reviewed;
        repo.update(&bug).unwrap();

        let updated = repo.get("bug-3").unwrap().unwrap();
        assert_eq!(updated.title, Some("Updated title".to_string()));
        assert_eq!(updated.status, BugStatus::Reviewed);
    }

    #[test]
    fn test_delete_bug() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-4");
        let repo = BugRepository::new(db.connection());
        let bug = create_test_bug("session-4", "bug-4", 1);

        repo.create(&bug).unwrap();
        repo.delete("bug-4").unwrap();

        let result = repo.get("bug-4").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_by_session() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-5");
        let repo = BugRepository::new(db.connection());

        repo.create(&create_test_bug("session-5", "bug-5", 1)).unwrap();
        repo.create(&create_test_bug("session-5", "bug-6", 2)).unwrap();

        let bugs = repo.list_by_session("session-5").unwrap();
        assert_eq!(bugs.len(), 2);
        assert_eq!(bugs[0].bug_number, 1);
        assert_eq!(bugs[1].bug_number, 2);
    }

    #[test]
    fn test_update_partial() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-6");
        let repo = BugRepository::new(db.connection());
        let bug = create_test_bug("session-6", "bug-7", 1);

        repo.create(&bug).unwrap();

        let update = BugUpdate {
            title: Some("New title".to_string()),
            status: Some(BugStatus::Ready),
            ..Default::default()
        };

        repo.update_partial("bug-7", &update).unwrap();

        let updated = repo.get("bug-7").unwrap().unwrap();
        assert_eq!(updated.title, Some("New title".to_string()));
        assert_eq!(updated.status, BugStatus::Ready);
        assert_eq!(updated.notes, Some("Test notes".to_string())); // Original value preserved
    }

    #[test]
    fn test_get_next_bug_number() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-7");
        let repo = BugRepository::new(db.connection());

        // First bug should be number 1
        let next = repo.get_next_bug_number("session-7").unwrap();
        assert_eq!(next, 1);

        // Create bug with number 1
        repo.create(&create_test_bug("session-7", "bug-8", 1)).unwrap();

        // Next should be 2
        let next = repo.get_next_bug_number("session-7").unwrap();
        assert_eq!(next, 2);
    }

    #[test]
    fn test_update_bug_title() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-8");
        let repo = BugRepository::new(db.connection());
        let bug = create_test_bug("session-8", "bug-title-1", 1);

        repo.create(&bug).unwrap();

        // Update the title via update_partial
        let update = BugUpdate {
            title: Some("Updated title from test".to_string()),
            ..Default::default()
        };
        repo.update_partial("bug-title-1", &update).unwrap();

        let updated = repo.get("bug-title-1").unwrap().unwrap();
        assert_eq!(updated.title, Some("Updated title from test".to_string()));
        // Other fields should remain unchanged
        assert_eq!(updated.notes, Some("Test notes".to_string()));
        assert_eq!(updated.status, BugStatus::Captured);
    }

    #[test]
    fn test_update_bug_title_to_empty() {
        let db = Database::in_memory().unwrap();
        create_test_session(&db, "session-9");
        let repo = BugRepository::new(db.connection());
        let bug = create_test_bug("session-9", "bug-title-2", 1);

        repo.create(&bug).unwrap();

        // Update title to empty string
        let update = BugUpdate {
            title: Some(String::new()),
            ..Default::default()
        };
        repo.update_partial("bug-title-2", &update).unwrap();

        let updated = repo.get("bug-title-2").unwrap().unwrap();
        assert_eq!(updated.title, Some(String::new()));
    }
}
