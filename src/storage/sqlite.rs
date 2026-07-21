use crate::storage::backend::{StorageBackend, StorageError, StorageResult, StorageStats};
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex, RwLock};

pub struct SqliteBackend {
    conn: Arc<Mutex<Option<Connection>>>,
    db_path: String,
    connected: Arc<RwLock<bool>>,
}

impl SqliteBackend {
    pub fn new(db_path: &str) -> Self {
        SqliteBackend {
            conn: Arc::new(Mutex::new(None)),
            db_path: db_path.to_string(),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    #[cfg(test)]
    pub fn in_memory() -> Self {
        SqliteBackend::new(":memory:")
    }

    fn map_rusqlite_err(e: rusqlite::Error, context: &str) -> StorageError {
        match e {
            rusqlite::Error::QueryReturnedNoRows => StorageError::NotFound(context.to_string()),
            _ => StorageError::BackendError(format!("{}: {}", context, e)),
        }
    }
}

impl StorageBackend for SqliteBackend {
    fn connect(&mut self) -> StorageResult<()> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))?;

        // Enable WAL mode for concurrent readers/writers
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA synchronous=NORMAL;")
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))?;

        // Create schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS missions (
                mission_id TEXT PRIMARY KEY,
                data       TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS events (
                mission_id TEXT NOT NULL,
                event_id   TEXT NOT NULL,
                data       TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (mission_id, event_id),
                FOREIGN KEY (mission_id) REFERENCES missions(mission_id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS reports (
                mission_id TEXT PRIMARY KEY,
                report     TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (mission_id) REFERENCES missions(mission_id) ON DELETE CASCADE
            );",
        )
        .map_err(|e| StorageError::ConnectionFailed(e.to_string()))?;

        *self.conn.lock().unwrap() = Some(conn);
        *self.connected.write().unwrap() = true;
        Ok(())
    }

    fn store_mission(&self, mission_id: &str, data: &str) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .unwrap();
        let conn = conn.as_ref().ok_or_else(|| {
            StorageError::ConnectionFailed("Database not connected".to_string())
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO missions (mission_id, data, created_at) VALUES (?1, ?2, datetime('now'))",
            params![mission_id, data],
        )
        .map_err(|e| Self::map_rusqlite_err(e, "Failed to store mission"))?;

        Ok(())
    }

    fn retrieve_mission(&self, mission_id: &str) -> StorageResult<String> {
        let conn = self
            .conn
            .lock()
            .unwrap();
        let conn = conn.as_ref().ok_or_else(|| {
            StorageError::ConnectionFailed("Database not connected".to_string())
        })?;

        conn.query_row(
            "SELECT data FROM missions WHERE mission_id = ?1",
            params![mission_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| Self::map_rusqlite_err(e, "Failed to retrieve mission"))?
        .ok_or_else(|| StorageError::NotFound(mission_id.to_string()))
    }

    fn store_event(&self, mission_id: &str, event_id: &str, data: &str) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .unwrap();
        let conn = conn.as_ref().ok_or_else(|| {
            StorageError::ConnectionFailed("Database not connected".to_string())
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO events (mission_id, event_id, data, created_at) VALUES (?1, ?2, ?3, datetime('now'))",
            params![mission_id, event_id, data],
        )
        .map_err(|e| Self::map_rusqlite_err(e, "Failed to store event"))?;

        Ok(())
    }

    fn retrieve_event(&self, mission_id: &str, event_id: &str) -> StorageResult<String> {
        let conn = self
            .conn
            .lock()
            .unwrap();
        let conn = conn.as_ref().ok_or_else(|| {
            StorageError::ConnectionFailed("Database not connected".to_string())
        })?;

        conn.query_row(
            "SELECT data FROM events WHERE mission_id = ?1 AND event_id = ?2",
            params![mission_id, event_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| Self::map_rusqlite_err(e, "Failed to retrieve event"))?
        .ok_or_else(|| StorageError::NotFound(format!("{}/{}", mission_id, event_id)))
    }

    fn store_report(&self, mission_id: &str, report: &str) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .unwrap();
        let conn = conn.as_ref().ok_or_else(|| {
            StorageError::ConnectionFailed("Database not connected".to_string())
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO reports (mission_id, report, created_at) VALUES (?1, ?2, datetime('now'))",
            params![mission_id, report],
        )
        .map_err(|e| Self::map_rusqlite_err(e, "Failed to store report"))?;

        Ok(())
    }

    fn retrieve_report(&self, mission_id: &str) -> StorageResult<String> {
        let conn = self
            .conn
            .lock()
            .unwrap();
        let conn = conn.as_ref().ok_or_else(|| {
            StorageError::ConnectionFailed("Database not connected".to_string())
        })?;

        conn.query_row(
            "SELECT report FROM reports WHERE mission_id = ?1",
            params![mission_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| Self::map_rusqlite_err(e, "Failed to retrieve report"))?
        .ok_or_else(|| StorageError::NotFound(mission_id.to_string()))
    }

    fn list_missions(&self, limit: Option<usize>) -> StorageResult<Vec<String>> {
        let conn = self
            .conn
            .lock()
            .unwrap();
        let conn = conn.as_ref().ok_or_else(|| {
            StorageError::ConnectionFailed("Database not connected".to_string())
        })?;

        let query = if let Some(l) = limit {
            format!("SELECT mission_id FROM missions ORDER BY created_at DESC LIMIT {}", l)
        } else {
            "SELECT mission_id FROM missions ORDER BY created_at DESC".to_string()
        };

        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| Self::map_rusqlite_err(e, "Failed to prepare list_missions"))?;

        let missions = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| Self::map_rusqlite_err(e, "Failed to list missions"))?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| Self::map_rusqlite_err(e, "Failed to collect missions"))?;

        Ok(missions)
    }

    fn delete_mission(&self, mission_id: &str) -> StorageResult<()> {
        let conn = self
            .conn
            .lock()
            .unwrap();
        let conn = conn.as_ref().ok_or_else(|| {
            StorageError::ConnectionFailed("Database not connected".to_string())
        })?;

        conn.execute(
            "DELETE FROM missions WHERE mission_id = ?1",
            params![mission_id],
        )
        .map_err(|e| Self::map_rusqlite_err(e, "Failed to delete mission"))?;

        Ok(())
    }

    fn mission_exists(&self, mission_id: &str) -> StorageResult<bool> {
        let conn = self
            .conn
            .lock()
            .unwrap();
        let conn = conn.as_ref().ok_or_else(|| {
            StorageError::ConnectionFailed("Database not connected".to_string())
        })?;

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM missions WHERE mission_id = ?1",
                params![mission_id],
                |row| row.get(0),
            )
            .map_err(|e| Self::map_rusqlite_err(e, "Failed to check mission exists"))?;

        Ok(count > 0)
    }

    fn get_stats(&self) -> StorageResult<StorageStats> {
        let conn = self
            .conn
            .lock()
            .unwrap();
        let conn = conn.as_ref().ok_or_else(|| {
            StorageError::ConnectionFailed("Database not connected".to_string())
        })?;

        let total_missions: u64 = conn
            .query_row("SELECT COUNT(*) FROM missions", [], |row| row.get(0))
            .map_err(|e| Self::map_rusqlite_err(e, "Failed to count missions"))?;

        let total_events: u64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .map_err(|e| Self::map_rusqlite_err(e, "Failed to count events"))?;

        let total_reports: u64 = conn
            .query_row("SELECT COUNT(*) FROM reports", [], |row| row.get(0))
            .map_err(|e| Self::map_rusqlite_err(e, "Failed to count reports"))?;

        let storage_size_bytes = conn
            .query_row(
                "SELECT COALESCE(page_count * page_size, 0) FROM pragma_page_count(), pragma_page_size()",
                [],
                |row| row.get::<_, u64>(0),
            )
            .ok();

        Ok(StorageStats {
            total_missions,
            total_events,
            total_reports,
            storage_size_bytes,
            connected: *self.connected.read().unwrap(),
        })
    }

    fn close(&self) -> StorageResult<()> {
        let mut conn = self.conn.lock().unwrap();
        conn.take();
        *self.connected.write().unwrap() = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_backend_creation() {
        let backend = SqliteBackend::new("test.db");
        assert_eq!(backend.db_path, "test.db");
        let connected = *backend.connected.read().unwrap();
        assert!(!connected);
    }

    #[test]
    fn test_connect_creates_schema() {
        let mut backend = SqliteBackend::in_memory();
        assert!(backend.connect().is_ok());

        let connected = *backend.connected.read().unwrap();
        assert!(connected);

        let conn = backend.conn.lock().unwrap();
        let conn = conn.as_ref().unwrap();
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(tables.contains(&"missions".to_string()));
        assert!(tables.contains(&"events".to_string()));
        assert!(tables.contains(&"reports".to_string()));
    }

    #[test]
    fn test_store_and_retrieve_mission() {
        let mut backend = SqliteBackend::in_memory();
        backend.connect().unwrap();

        let data = r#"{"id": "mission_1", "name": "Test Mission"}"#;
        backend.store_mission("mission_1", data).unwrap();

        let retrieved = backend.retrieve_mission("mission_1").unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_retrieve_nonexistent_mission_returns_not_found() {
        let mut backend = SqliteBackend::in_memory();
        backend.connect().unwrap();

        match backend.retrieve_mission("nonexistent") {
            Err(StorageError::NotFound(_)) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_store_and_retrieve_event() {
        let mut backend = SqliteBackend::in_memory();
        backend.connect().unwrap();

        backend
            .store_mission("mission_1", r#"{"id":"mission_1"}"#)
            .unwrap();

        let event_data = r#"{"type": "lidar_scan", "timestamp": "2024-01-01T12:00:00Z"}"#;
        backend
            .store_event("mission_1", "event_1", event_data)
            .unwrap();

        let retrieved = backend.retrieve_event("mission_1", "event_1").unwrap();
        assert_eq!(retrieved, event_data);
    }

    #[test]
    fn test_store_and_retrieve_report() {
        let mut backend = SqliteBackend::in_memory();
        backend.connect().unwrap();

        backend
            .store_mission("mission_1", r#"{"id":"mission_1"}"#)
            .unwrap();

        let report = r#"{"failure_type": "navigation_deadlock", "confidence": 0.85}"#;
        backend.store_report("mission_1", report).unwrap();

        let retrieved = backend.retrieve_report("mission_1").unwrap();
        assert_eq!(retrieved, report);
    }

    #[test]
    fn test_list_missions_with_limit() {
        let mut backend = SqliteBackend::in_memory();
        backend.connect().unwrap();

        for i in 0..5 {
            backend
                .store_mission(&format!("mission_{}", i), &format!(r#"{{"id":"mission_{}"}}"#, i))
                .unwrap();
        }

        let missions = backend.list_missions(Some(3)).unwrap();
        assert_eq!(missions.len(), 3);
    }

    #[test]
    fn test_mission_exists() {
        let mut backend = SqliteBackend::in_memory();
        backend.connect().unwrap();

        backend
            .store_mission("mission_1", r#"{"id":"mission_1"}"#)
            .unwrap();

        assert!(backend.mission_exists("mission_1").unwrap());
        assert!(!backend.mission_exists("nonexistent").unwrap());
    }

    #[test]
    fn test_delete_mission_cascades_events_and_reports() {
        let mut backend = SqliteBackend::in_memory();
        backend.connect().unwrap();

        backend
            .store_mission("mission_1", r#"{"id":"mission_1"}"#)
            .unwrap();
        backend
            .store_event("mission_1", "event_1", r#"{"type":"lidar"}"#)
            .unwrap();
        backend
            .store_report("mission_1", r#"{"report":"data"}"#)
            .unwrap();

        backend.delete_mission("mission_1").unwrap();

        assert!(!backend.mission_exists("mission_1").unwrap());
        match backend.retrieve_event("mission_1", "event_1") {
            Err(StorageError::NotFound(_)) => (),
            _ => panic!("Event should not exist after mission deletion"),
        }
        match backend.retrieve_report("mission_1") {
            Err(StorageError::NotFound(_)) => (),
            _ => panic!("Report should not exist after mission deletion"),
        }
    }

    #[test]
    fn test_get_stats_accuracy() {
        let mut backend = SqliteBackend::in_memory();
        backend.connect().unwrap();

        backend
            .store_mission("mission_1", r#"{"id":"mission_1"}"#)
            .unwrap();
        backend
            .store_mission("mission_2", r#"{"id":"mission_2"}"#)
            .unwrap();
        backend
            .store_event("mission_1", "event_1", r#"{"type":"lidar"}"#)
            .unwrap();
        backend
            .store_event("mission_1", "event_2", r#"{"type":"camera"}"#)
            .unwrap();
        backend
            .store_report("mission_1", r#"{"report":"data"}"#)
            .unwrap();

        let stats = backend.get_stats().unwrap();
        assert_eq!(stats.total_missions, 2);
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.total_reports, 1);
        assert!(stats.connected);
        assert!(stats.storage_size_bytes.is_some());
    }
}
