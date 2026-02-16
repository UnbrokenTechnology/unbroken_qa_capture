mod models;
mod schema;
mod session;
mod bug;
mod capture;
mod settings;

// Public exports for external module use
#[allow(unused_imports)]
pub use models::*;
#[allow(unused_imports)]
pub use schema::init_database;
#[allow(unused_imports)]
pub use session::{SessionOps, SessionRepository};
#[allow(unused_imports)]
pub use bug::{BugOps, BugRepository};
#[allow(unused_imports)]
pub use capture::{CaptureOps, CaptureRepository};
#[allow(unused_imports)]
pub use settings::{SettingsOps, SettingsRepository};

use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

/// Database connection manager
pub struct Database {
    #[allow(dead_code)]
    conn: Connection,
}

impl Database {
    /// Create a new database connection
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(path: P) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        schema::init_database(&conn)?;
        Ok(Database { conn })
    }

    /// Create an in-memory database (for testing)
    #[allow(dead_code)]
    pub fn in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        schema::init_database(&conn)?;
        Ok(Database { conn })
    }

    /// Get a reference to the underlying connection
    #[allow(dead_code)]
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_in_memory_database() {
        let db = Database::in_memory();
        assert!(db.is_ok());
    }

    #[test]
    fn test_schema_initialized() {
        let db = Database::in_memory().unwrap();

        // Verify tables exist
        let tables: Vec<String> = db
            .connection()
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
    }
}
