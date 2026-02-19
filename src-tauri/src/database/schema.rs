use rusqlite::{Connection, Result as SqlResult};

/// Initialize the database schema
pub fn init_database(conn: &Connection) -> SqlResult<()> {
    // Create sessions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            folder_path TEXT NOT NULL,
            session_notes TEXT,
            environment_json TEXT,
            original_snip_path TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Create bugs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bugs (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            bug_number INTEGER NOT NULL,
            display_id TEXT NOT NULL,
            type TEXT DEFAULT 'bug',
            title TEXT,
            notes TEXT,
            description TEXT,
            ai_description TEXT,
            status TEXT NOT NULL DEFAULT 'captured',
            meeting_id TEXT,
            software_version TEXT,
            console_parse_json TEXT,
            metadata_json TEXT,
            folder_path TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Create captures table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS captures (
            id TEXT PRIMARY KEY,
            bug_id TEXT REFERENCES bugs(id),
            session_id TEXT NOT NULL REFERENCES sessions(id),
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            annotated_path TEXT,
            file_size_bytes INTEGER,
            is_console_capture BOOLEAN DEFAULT FALSE,
            parsed_content TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Create settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Create profiles table (stores QA testing profiles as JSON blobs)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS profiles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            data TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Create indices
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_bugs_session ON bugs(session_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_captures_bug ON captures(bug_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_captures_session ON captures(session_id)",
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_database_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        let result = init_database(&conn);
        assert!(result.is_ok());

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"sessions".to_string()));
        assert!(tables.contains(&"bugs".to_string()));
        assert!(tables.contains(&"captures".to_string()));
        assert!(tables.contains(&"settings".to_string()));
        assert!(tables.contains(&"profiles".to_string()));
    }

    #[test]
    fn test_init_database_creates_indices() {
        let conn = Connection::open_in_memory().unwrap();
        init_database(&conn).unwrap();

        // Verify indices exist
        let indices: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(indices.contains(&"idx_bugs_session".to_string()));
        assert!(indices.contains(&"idx_captures_bug".to_string()));
        assert!(indices.contains(&"idx_captures_session".to_string()));
    }

    #[test]
    fn test_init_database_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Initialize twice - should not error
        assert!(init_database(&conn).is_ok());
        assert!(init_database(&conn).is_ok());
    }
}
