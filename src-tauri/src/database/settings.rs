use rusqlite::{Connection, Result as SqlResult, params};
use crate::database::models::Setting;

/// Trait defining settings operations
#[allow(dead_code)]
pub trait SettingsOps {
    fn set(&self, key: &str, value: &str) -> SqlResult<()>;
    fn get(&self, key: &str) -> SqlResult<Option<String>>;
    fn get_all(&self) -> SqlResult<Vec<Setting>>;
    fn delete(&self, key: &str) -> SqlResult<()>;
    fn exists(&self, key: &str) -> SqlResult<bool>;
}

/// Settings repository implementation
#[allow(dead_code)]
pub struct SettingsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SettingsRepository<'a> {
    #[allow(dead_code)]
    pub fn new(conn: &'a Connection) -> Self {
        SettingsRepository { conn }
    }
}

impl<'a> SettingsOps for SettingsRepository<'a> {
    fn set(&self, key: &str, value: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at)
             VALUES (?1, ?2, datetime('now'))",
            params![key, value],
        )?;
        Ok(())
    }

    fn get(&self, key: &str) -> SqlResult<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM settings WHERE key = ?1"
        )?;

        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    fn get_all(&self) -> SqlResult<Vec<Setting>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value, updated_at FROM settings ORDER BY key ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Setting {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })?;

        rows.collect()
    }

    fn delete(&self, key: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    fn exists(&self, key: &str) -> SqlResult<bool> {
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM settings WHERE key = ?1"
        )?;

        let count: i64 = stmt.query_row(params![key], |row| row.get(0))?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    #[test]
    fn test_set_and_get_setting() {
        let db = Database::in_memory().unwrap();
        let repo = SettingsRepository::new(db.connection());

        repo.set("theme", "dark").unwrap();
        let value = repo.get("theme").unwrap();

        assert!(value.is_some());
        assert_eq!(value.unwrap(), "dark");
    }

    #[test]
    fn test_get_nonexistent_setting() {
        let db = Database::in_memory().unwrap();
        let repo = SettingsRepository::new(db.connection());

        let value = repo.get("nonexistent").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_update_setting() {
        let db = Database::in_memory().unwrap();
        let repo = SettingsRepository::new(db.connection());

        repo.set("language", "en").unwrap();
        repo.set("language", "es").unwrap();

        let value = repo.get("language").unwrap();
        assert_eq!(value.unwrap(), "es");
    }

    #[test]
    fn test_delete_setting() {
        let db = Database::in_memory().unwrap();
        let repo = SettingsRepository::new(db.connection());

        repo.set("temp_setting", "value").unwrap();
        repo.delete("temp_setting").unwrap();

        let value = repo.get("temp_setting").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_get_all_settings() {
        let db = Database::in_memory().unwrap();
        let repo = SettingsRepository::new(db.connection());

        repo.set("setting1", "value1").unwrap();
        repo.set("setting2", "value2").unwrap();
        repo.set("setting3", "value3").unwrap();

        let settings = repo.get_all().unwrap();
        assert_eq!(settings.len(), 3);
        assert_eq!(settings[0].key, "setting1");
        assert_eq!(settings[1].key, "setting2");
        assert_eq!(settings[2].key, "setting3");
    }

    #[test]
    fn test_exists() {
        let db = Database::in_memory().unwrap();
        let repo = SettingsRepository::new(db.connection());

        assert!(!repo.exists("test_key").unwrap());

        repo.set("test_key", "test_value").unwrap();
        assert!(repo.exists("test_key").unwrap());

        repo.delete("test_key").unwrap();
        assert!(!repo.exists("test_key").unwrap());
    }

    #[test]
    fn test_upsert_behavior() {
        let db = Database::in_memory().unwrap();
        let repo = SettingsRepository::new(db.connection());

        // Insert
        repo.set("counter", "1").unwrap();
        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 1);

        // Update (should not create duplicate)
        repo.set("counter", "2").unwrap();
        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(repo.get("counter").unwrap().unwrap(), "2");
    }
}
