use rusqlite::Connection;
use std::sync::{Arc, Mutex, MutexGuard};
use std::path::Path;

use super::schema::init_database;

/// Shared database state for Tauri managed state.
///
/// Wraps a `rusqlite::Connection` in `Arc<Mutex<Connection>>` so it can be
/// registered with `app.manage()` and accessed by Tauri commands via
/// `State<DbState>`. WAL mode is enabled for better concurrent read
/// performance.
///
/// # Usage in a Tauri command
///
/// ```rust,ignore
/// #[tauri::command]
/// fn my_command(db: tauri::State<DbState>) -> Result<(), String> {
///     let conn = db.connection();
///     // use conn (MutexGuard<Connection>) …
///     Ok(())
/// }
/// ```
pub struct DbState {
    inner: Arc<Mutex<Connection>>,
}

impl DbState {
    /// Open (or create) the SQLite database at `path`, enable WAL mode, and
    /// initialize the schema.  Returns an error if any of these steps fail.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        // Enable WAL mode for better concurrent read performance.
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

        // Ensure all tables and indices are present.
        init_database(&conn)
            .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

        Ok(DbState {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create an in-memory database (primarily for testing).
    #[cfg(test)]
    pub fn in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

        init_database(&conn)
            .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

        Ok(DbState {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// Acquire an exclusive lock on the underlying connection.
    ///
    /// Callers hold the returned `MutexGuard` for the duration of their
    /// database work and then release it (RAII — the lock is dropped when the
    /// guard goes out of scope).
    pub fn connection(&self) -> MutexGuard<'_, Connection> {
        self.inner.lock().expect("DbState mutex poisoned")
    }

    /// Clone the inner `Arc<Mutex<Connection>>` for use cases that need to
    /// store the handle separately (e.g. background tasks).
    pub fn arc(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_db_state_opens() {
        let state = DbState::in_memory();
        assert!(state.is_ok(), "DbState::in_memory() should succeed");
    }

    #[test]
    fn test_connection_returns_usable_guard() {
        let state = DbState::in_memory().unwrap();
        let conn = state.connection();
        // Execute a trivial query to confirm the connection is live.
        let result: i64 = conn
            .query_row("SELECT 1", [], |row| row.get(0))
            .expect("simple SELECT should succeed");
        assert_eq!(result, 1);
    }

    #[test]
    fn test_schema_initialized_on_open() {
        let state = DbState::in_memory().unwrap();
        let conn = state.connection();

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
    fn test_wal_mode_enabled() {
        let state = DbState::in_memory().unwrap();
        let conn = state.connection();

        // WAL mode on an in-memory database is reported as "memory" by SQLite
        // (in-memory DBs cannot use WAL), but the PRAGMA must not error.
        // For a real file-backed DB it would return "wal".  We just verify
        // the PRAGMA round-trip works without panicking.
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("PRAGMA journal_mode should succeed");
        // Accept both "wal" (file) and "memory" (in-memory) — either is fine.
        assert!(
            mode == "wal" || mode == "memory",
            "Unexpected journal_mode: {}",
            mode
        );
    }

    #[test]
    fn test_shared_connection_arc_clone() {
        let state = DbState::in_memory().unwrap();
        let arc1 = state.arc();
        let arc2 = state.arc();

        // Both arcs should point to the same allocation.
        assert!(Arc::ptr_eq(&arc1, &arc2));
    }

    #[test]
    fn test_multiple_sequential_connection_accesses() {
        let state = DbState::in_memory().unwrap();

        // First access: insert a row.
        {
            let conn = state.connection();
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('test_key', 'test_value')",
                [],
            )
            .expect("INSERT should succeed");
        } // guard dropped here — lock released

        // Second access: read the row back.
        {
            let conn = state.connection();
            let value: String = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = 'test_key'",
                    [],
                    |row| row.get(0),
                )
                .expect("SELECT should find the inserted row");
            assert_eq!(value, "test_value");
        }
    }
}
