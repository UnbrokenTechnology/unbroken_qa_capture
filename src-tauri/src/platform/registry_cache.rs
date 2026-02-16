//! Persistent cache for Windows registry values to enable crash recovery.
//!
//! This module provides SQLite-backed storage for caching the original registry
//! values before modification. If the application crashes before restoration,
//! the cached values can be used to restore the registry on next startup.

use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::error::{PlatformError, Result};

#[allow(dead_code)]
const DB_NAME: &str = "registry_cache.db";

/// Database-backed cache for registry redirect state.
///
/// This cache persists the original registry value to SQLite, enabling crash
/// recovery. If the app terminates before restoring the registry, the next
/// startup can detect and restore the stale redirect.
#[allow(dead_code)]
pub struct RegistryCache {
    conn: Mutex<Connection>,
}

#[allow(dead_code)]
impl RegistryCache {
    /// Creates a new registry cache using the specified SQLite database file.
    ///
    /// The database is created if it doesn't exist. The schema is automatically
    /// initialized on first use.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file (e.g., `app_data/registry_cache.db`)
    ///
    /// # Errors
    ///
    /// Returns `PlatformError::FileSystemError` if the database cannot be created or opened.
    pub fn new(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| PlatformError::FileSystemError {
                path: parent.display().to_string(),
                operation: "create_dir_all".to_string(),
                message: format!("Failed to create database directory: {}", e),
            })?;
        }

        let conn = Connection::open(db_path).map_err(|e| PlatformError::FileSystemError {
            path: db_path.display().to_string(),
            operation: "open_database".to_string(),
            message: format!("Failed to open database: {}", e),
        })?;

        let cache = Self {
            conn: Mutex::new(conn),
        };

        cache.initialize_schema()?;
        Ok(cache)
    }

    /// Initializes the database schema if not already present.
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| PlatformError::FileSystemError {
            path: DB_NAME.to_string(),
            operation: "lock".to_string(),
            message: format!("Failed to acquire database lock: {}", e),
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS registry_redirects (
                id INTEGER PRIMARY KEY,
                registry_key TEXT NOT NULL,
                original_value TEXT NOT NULL,
                redirected_value TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| PlatformError::FileSystemError {
            path: DB_NAME.to_string(),
            operation: "create_schema".to_string(),
            message: format!("Failed to create schema: {}", e),
        })?;

        Ok(())
    }

    /// Caches the original registry value before redirection.
    ///
    /// # Arguments
    ///
    /// * `registry_key` - The full registry key path (e.g., "HKCU\\Software\\...")
    /// * `original_value` - The original value before modification
    /// * `redirected_value` - The new value being written to the registry
    ///
    /// # Errors
    ///
    /// Returns `PlatformError::FileSystemError` if the database write fails.
    pub fn cache_redirect(
        &self,
        registry_key: &str,
        original_value: &Path,
        redirected_value: &Path,
    ) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| PlatformError::FileSystemError {
            path: DB_NAME.to_string(),
            operation: "lock".to_string(),
            message: format!("Failed to acquire database lock: {}", e),
        })?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Delete any existing entry for this key first
        conn.execute(
            "DELETE FROM registry_redirects WHERE registry_key = ?1",
            params![registry_key],
        )
        .map_err(|e| PlatformError::FileSystemError {
            path: DB_NAME.to_string(),
            operation: "delete".to_string(),
            message: format!("Failed to delete old cache entry: {}", e),
        })?;

        // Insert new entry
        conn.execute(
            "INSERT INTO registry_redirects (registry_key, original_value, redirected_value, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                registry_key,
                original_value.to_string_lossy().as_ref(),
                redirected_value.to_string_lossy().as_ref(),
                timestamp
            ],
        )
        .map_err(|e| PlatformError::FileSystemError {
            path: DB_NAME.to_string(),
            operation: "insert".to_string(),
            message: format!("Failed to cache redirect: {}", e),
        })?;

        Ok(())
    }

    /// Retrieves the cached original value for a registry key.
    ///
    /// # Arguments
    ///
    /// * `registry_key` - The registry key to look up
    ///
    /// # Returns
    ///
    /// `Some(PathBuf)` with the original value if a cached redirect exists,
    /// or `None` if no redirect is cached for this key.
    ///
    /// # Errors
    ///
    /// Returns `PlatformError::FileSystemError` if the database read fails.
    pub fn get_cached_original(&self, registry_key: &str) -> Result<Option<PathBuf>> {
        let conn = self.conn.lock().map_err(|e| PlatformError::FileSystemError {
            path: DB_NAME.to_string(),
            operation: "lock".to_string(),
            message: format!("Failed to acquire database lock: {}", e),
        })?;

        let mut stmt = conn
            .prepare("SELECT original_value FROM registry_redirects WHERE registry_key = ?1")
            .map_err(|e| PlatformError::FileSystemError {
                path: DB_NAME.to_string(),
                operation: "prepare_query".to_string(),
                message: format!("Failed to prepare query: {}", e),
            })?;

        let mut rows = stmt.query(params![registry_key]).map_err(|e| {
            PlatformError::FileSystemError {
                path: DB_NAME.to_string(),
                operation: "query".to_string(),
                message: format!("Failed to query cache: {}", e),
            }
        })?;

        if let Some(row) = rows.next().map_err(|e| PlatformError::FileSystemError {
            path: DB_NAME.to_string(),
            operation: "read_row".to_string(),
            message: format!("Failed to read row: {}", e),
        })? {
            let value: String = row.get(0).map_err(|e| PlatformError::FileSystemError {
                path: DB_NAME.to_string(),
                operation: "read_value".to_string(),
                message: format!("Failed to read value: {}", e),
            })?;
            Ok(Some(PathBuf::from(value)))
        } else {
            Ok(None)
        }
    }

    /// Clears the cached redirect for a registry key after successful restoration.
    ///
    /// # Arguments
    ///
    /// * `registry_key` - The registry key to clear from the cache
    ///
    /// # Errors
    ///
    /// Returns `PlatformError::FileSystemError` if the database write fails.
    pub fn clear_redirect(&self, registry_key: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| PlatformError::FileSystemError {
            path: DB_NAME.to_string(),
            operation: "lock".to_string(),
            message: format!("Failed to acquire database lock: {}", e),
        })?;

        conn.execute(
            "DELETE FROM registry_redirects WHERE registry_key = ?1",
            params![registry_key],
        )
        .map_err(|e| PlatformError::FileSystemError {
            path: DB_NAME.to_string(),
            operation: "delete".to_string(),
            message: format!("Failed to clear cache: {}", e),
        })?;

        Ok(())
    }

    /// Lists all active (uncompleted) redirects in the cache.
    ///
    /// This is used on app startup to detect stale redirects that need restoration.
    ///
    /// # Returns
    ///
    /// A vector of tuples: `(registry_key, original_value, redirected_value)`.
    ///
    /// # Errors
    ///
    /// Returns `PlatformError::FileSystemError` if the database read fails.
    pub fn list_active_redirects(&self) -> Result<Vec<(String, PathBuf, PathBuf)>> {
        let conn = self.conn.lock().map_err(|e| PlatformError::FileSystemError {
            path: DB_NAME.to_string(),
            operation: "lock".to_string(),
            message: format!("Failed to acquire database lock: {}", e),
        })?;

        let mut stmt = conn
            .prepare("SELECT registry_key, original_value, redirected_value FROM registry_redirects")
            .map_err(|e| PlatformError::FileSystemError {
                path: DB_NAME.to_string(),
                operation: "prepare_query".to_string(),
                message: format!("Failed to prepare query: {}", e),
            })?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| PlatformError::FileSystemError {
                path: DB_NAME.to_string(),
                operation: "query_map".to_string(),
                message: format!("Failed to query redirects: {}", e),
            })?;

        let mut redirects = Vec::new();
        for row in rows {
            let (key, original, redirected) = row.map_err(|e| PlatformError::FileSystemError {
                path: DB_NAME.to_string(),
                operation: "read_row".to_string(),
                message: format!("Failed to read row: {}", e),
            })?;
            redirects.push((key, PathBuf::from(original), PathBuf::from(redirected)));
        }

        Ok(redirects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_cache_and_retrieve() {
        let temp_dir = std::env::temp_dir().join("registry_cache_test");
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        let cache = RegistryCache::new(&db_path).unwrap();

        let registry_key = "HKCU\\Software\\Test";
        let original = PathBuf::from("C:\\Original");
        let redirected = PathBuf::from("C:\\Redirected");

        // Cache a redirect
        cache
            .cache_redirect(registry_key, &original, &redirected)
            .unwrap();

        // Retrieve it
        let retrieved = cache.get_cached_original(registry_key).unwrap();
        assert_eq!(retrieved, Some(original));

        // Clear it
        cache.clear_redirect(registry_key).unwrap();

        // Should be gone
        let retrieved = cache.get_cached_original(registry_key).unwrap();
        assert_eq!(retrieved, None);

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_list_active_redirects() {
        let temp_dir = std::env::temp_dir().join("registry_cache_test2");
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        let cache = RegistryCache::new(&db_path).unwrap();

        // Add multiple redirects
        cache
            .cache_redirect("HKCU\\Key1", &PathBuf::from("C:\\A"), &PathBuf::from("C:\\B"))
            .unwrap();
        cache
            .cache_redirect("HKCU\\Key2", &PathBuf::from("C:\\C"), &PathBuf::from("C:\\D"))
            .unwrap();

        let redirects = cache.list_active_redirects().unwrap();
        assert_eq!(redirects.len(), 2);

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_update_existing_redirect() {
        let temp_dir = std::env::temp_dir().join("registry_cache_test3");
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        let cache = RegistryCache::new(&db_path).unwrap();

        let registry_key = "HKCU\\Software\\Test";

        // Cache first redirect
        cache
            .cache_redirect(
                registry_key,
                &PathBuf::from("C:\\Original"),
                &PathBuf::from("C:\\Redirect1"),
            )
            .unwrap();

        // Update with new redirect (should replace, not duplicate)
        cache
            .cache_redirect(
                registry_key,
                &PathBuf::from("C:\\Original"),
                &PathBuf::from("C:\\Redirect2"),
            )
            .unwrap();

        let redirects = cache.list_active_redirects().unwrap();
        assert_eq!(redirects.len(), 1); // Should be only one entry

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
