use rusqlite::{Connection, Result as SqlResult, params};
use crate::database::models::{Session, SessionStatus, SessionSummary};

/// Trait defining session operations
#[allow(dead_code)]
pub trait SessionOps {
    fn create(&self, session: &Session) -> SqlResult<()>;
    fn get(&self, id: &str) -> SqlResult<Option<Session>>;
    fn update(&self, session: &Session) -> SqlResult<()>;
    fn delete(&self, id: &str) -> SqlResult<()>;
    fn list(&self) -> SqlResult<Vec<Session>>;
    fn get_active_session(&self) -> SqlResult<Option<Session>>;
    fn get_summaries(&self) -> SqlResult<Vec<SessionSummary>>;
    fn update_status(&self, id: &str, status: SessionStatus) -> SqlResult<()>;
}

/// Session repository implementation
#[allow(dead_code)]
pub struct SessionRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SessionRepository<'a> {
    #[allow(dead_code)]
    pub fn new(conn: &'a Connection) -> Self {
        SessionRepository { conn }
    }
}

impl<'a> SessionOps for SessionRepository<'a> {
    fn create(&self, session: &Session) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO sessions (id, started_at, ended_at, status, folder_path, session_notes, environment_json, original_snip_path, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                session.id,
                session.started_at,
                session.ended_at,
                session.status.as_str(),
                session.folder_path,
                session.session_notes,
                session.environment_json,
                session.original_snip_path,
                session.created_at,
            ],
        )?;
        Ok(())
    }

    fn get(&self, id: &str) -> SqlResult<Option<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, started_at, ended_at, status, folder_path, session_notes, environment_json, original_snip_path, created_at
             FROM sessions WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let status_str: String = row.get(3)?;
            Ok(Some(Session {
                id: row.get(0)?,
                started_at: row.get(1)?,
                ended_at: row.get(2)?,
                status: SessionStatus::from_str(&status_str).unwrap_or(SessionStatus::Active),
                folder_path: row.get(4)?,
                session_notes: row.get(5)?,
                environment_json: row.get(6)?,
                original_snip_path: row.get(7)?,
                created_at: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn update(&self, session: &Session) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE sessions SET started_at = ?2, ended_at = ?3, status = ?4, folder_path = ?5,
             session_notes = ?6, environment_json = ?7, original_snip_path = ?8
             WHERE id = ?1",
            params![
                session.id,
                session.started_at,
                session.ended_at,
                session.status.as_str(),
                session.folder_path,
                session.session_notes,
                session.environment_json,
                session.original_snip_path,
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn list(&self) -> SqlResult<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, started_at, ended_at, status, folder_path, session_notes, environment_json, original_snip_path, created_at
             FROM sessions ORDER BY started_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            let status_str: String = row.get(3)?;
            Ok(Session {
                id: row.get(0)?,
                started_at: row.get(1)?,
                ended_at: row.get(2)?,
                status: SessionStatus::from_str(&status_str).unwrap_or(SessionStatus::Active),
                folder_path: row.get(4)?,
                session_notes: row.get(5)?,
                environment_json: row.get(6)?,
                original_snip_path: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    fn get_active_session(&self) -> SqlResult<Option<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, started_at, ended_at, status, folder_path, session_notes, environment_json, original_snip_path, created_at
             FROM sessions WHERE status = 'active' ORDER BY started_at DESC LIMIT 1"
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            let status_str: String = row.get(3)?;
            Ok(Some(Session {
                id: row.get(0)?,
                started_at: row.get(1)?,
                ended_at: row.get(2)?,
                status: SessionStatus::from_str(&status_str).unwrap_or(SessionStatus::Active),
                folder_path: row.get(4)?,
                session_notes: row.get(5)?,
                environment_json: row.get(6)?,
                original_snip_path: row.get(7)?,
                created_at: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_summaries(&self) -> SqlResult<Vec<SessionSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.started_at, s.ended_at, s.status, COUNT(b.id) as bug_count
             FROM sessions s
             LEFT JOIN bugs b ON s.id = b.session_id
             GROUP BY s.id
             ORDER BY s.started_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            let status_str: String = row.get(3)?;
            Ok(SessionSummary {
                id: row.get(0)?,
                started_at: row.get(1)?,
                ended_at: row.get(2)?,
                status: SessionStatus::from_str(&status_str).unwrap_or(SessionStatus::Active),
                bug_count: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    fn update_status(&self, id: &str, status: SessionStatus) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE sessions SET status = ?1 WHERE id = ?2",
            params![status.as_str(), id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    fn create_test_session(id: &str) -> Session {
        Session {
            id: id.to_string(),
            started_at: "2024-01-01T10:00:00Z".to_string(),
            ended_at: None,
            status: SessionStatus::Active,
            folder_path: "/test/sessions/session1".to_string(),
            session_notes: Some("Test notes".to_string()),
            environment_json: Some(r#"{"os":"Windows 11"}"#.to_string()),
            original_snip_path: None,
            created_at: "2024-01-01T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_create_session() {
        let db = Database::in_memory().unwrap();
        let repo = SessionRepository::new(db.connection());
        let session = create_test_session("test-id-1");

        let result = repo.create(&session);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_session() {
        let db = Database::in_memory().unwrap();
        let repo = SessionRepository::new(db.connection());
        let session = create_test_session("test-id-2");

        repo.create(&session).unwrap();
        let retrieved = repo.get("test-id-2").unwrap();

        assert!(retrieved.is_some());
        let retrieved_session = retrieved.unwrap();
        assert_eq!(retrieved_session.id, session.id);
        assert_eq!(retrieved_session.folder_path, session.folder_path);
    }

    #[test]
    fn test_get_nonexistent_session() {
        let db = Database::in_memory().unwrap();
        let repo = SessionRepository::new(db.connection());

        let result = repo.get("nonexistent");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_update_session() {
        let db = Database::in_memory().unwrap();
        let repo = SessionRepository::new(db.connection());
        let mut session = create_test_session("test-id-3");

        repo.create(&session).unwrap();

        session.session_notes = Some("Updated notes".to_string());
        session.status = SessionStatus::Ended;
        repo.update(&session).unwrap();

        let updated = repo.get("test-id-3").unwrap().unwrap();
        assert_eq!(updated.session_notes, Some("Updated notes".to_string()));
        assert_eq!(updated.status, SessionStatus::Ended);
    }

    #[test]
    fn test_delete_session() {
        let db = Database::in_memory().unwrap();
        let repo = SessionRepository::new(db.connection());
        let session = create_test_session("test-id-4");

        repo.create(&session).unwrap();
        repo.delete("test-id-4").unwrap();

        let result = repo.get("test-id-4").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_sessions() {
        let db = Database::in_memory().unwrap();
        let repo = SessionRepository::new(db.connection());

        repo.create(&create_test_session("test-id-5")).unwrap();
        repo.create(&create_test_session("test-id-6")).unwrap();

        let sessions = repo.list().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_get_active_session() {
        let db = Database::in_memory().unwrap();
        let repo = SessionRepository::new(db.connection());

        let mut session1 = create_test_session("test-id-7");
        session1.status = SessionStatus::Ended;
        repo.create(&session1).unwrap();

        let session2 = create_test_session("test-id-8");
        repo.create(&session2).unwrap();

        let active = repo.get_active_session().unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, "test-id-8");
    }

    #[test]
    fn test_update_status() {
        let db = Database::in_memory().unwrap();
        let repo = SessionRepository::new(db.connection());
        let session = create_test_session("test-id-9");

        repo.create(&session).unwrap();
        repo.update_status("test-id-9", SessionStatus::Reviewed).unwrap();

        let updated = repo.get("test-id-9").unwrap().unwrap();
        assert_eq!(updated.status, SessionStatus::Reviewed);
    }

    #[test]
    fn test_get_summaries() {
        let db = Database::in_memory().unwrap();
        let repo = SessionRepository::new(db.connection());

        repo.create(&create_test_session("test-id-10")).unwrap();
        repo.create(&create_test_session("test-id-11")).unwrap();

        let summaries = repo.get_summaries().unwrap();
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].bug_count, 0);
    }
}
