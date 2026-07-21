use serde::{Deserialize, Serialize};
use std::fmt;

/// Storage operation result type
pub type StorageResult<T> = Result<T, StorageError>;

/// Storage operation errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageError {
    /// Connection failed
    ConnectionFailed(String),
    /// Write operation failed
    WriteFailed(String),
    /// Read operation failed
    ReadFailed(String),
    /// Item not found
    NotFound(String),
    /// Serialization error
    SerializationError(String),
    /// Backend-specific error
    BackendError(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            StorageError::WriteFailed(msg) => write!(f, "Write failed: {}", msg),
            StorageError::ReadFailed(msg) => write!(f, "Read failed: {}", msg),
            StorageError::NotFound(msg) => write!(f, "Not found: {}", msg),
            StorageError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::BackendError(msg) => write!(f, "Backend error: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Backend type (inmemory, sqlite, postgresql, bigquery, s3)
    pub backend_type: String,
    /// Connection string or path
    pub connection_string: String,
    /// Enable audit trail
    pub enable_audit: bool,
    /// Enable encryption
    pub enable_encryption: bool,
    /// Retention period in days (0 = infinite)
    pub retention_days: u32,
    /// Max concurrent connections
    pub max_connections: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            backend_type: "inmemory".to_string(),
            connection_string: String::new(),
            enable_audit: true,
            enable_encryption: false,
            retention_days: 0,
            max_connections: 10,
        }
    }
}

/// Generic storage backend trait
pub trait StorageBackend: Send + Sync {
    /// Connect to storage backend
    fn connect(&mut self) -> StorageResult<()>;

    /// Store mission data
    fn store_mission(&self, mission_id: &str, data: &str) -> StorageResult<()>;

    /// Retrieve mission data
    fn retrieve_mission(&self, mission_id: &str) -> StorageResult<String>;

    /// Store event data
    fn store_event(&self, mission_id: &str, event_id: &str, data: &str) -> StorageResult<()>;

    /// Retrieve event data
    fn retrieve_event(&self, mission_id: &str, event_id: &str) -> StorageResult<String>;

    /// Store diagnostic report
    fn store_report(&self, mission_id: &str, report: &str) -> StorageResult<()>;

    /// Retrieve diagnostic report
    fn retrieve_report(&self, mission_id: &str) -> StorageResult<String>;

    /// List missions (optional limit)
    fn list_missions(&self, limit: Option<usize>) -> StorageResult<Vec<String>>;

    /// Delete mission and all associated data
    fn delete_mission(&self, mission_id: &str) -> StorageResult<()>;

    /// Check if mission exists
    fn mission_exists(&self, mission_id: &str) -> StorageResult<bool>;

    /// Get storage statistics
    fn get_stats(&self) -> StorageResult<StorageStats>;

    /// Close connection
    fn close(&self) -> StorageResult<()>;
}

/// Storage backend statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total missions stored
    pub total_missions: u64,
    /// Total events stored
    pub total_events: u64,
    /// Total reports stored
    pub total_reports: u64,
    /// Storage size in bytes (if available)
    pub storage_size_bytes: Option<u64>,
    /// Connection status
    pub connected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.backend_type, "inmemory");
        assert!(config.enable_audit);
    }

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::ConnectionFailed("test".to_string());
        assert_eq!(err.to_string(), "Connection failed: test");
    }

    #[test]
    fn test_storage_stats() {
        let stats = StorageStats {
            total_missions: 10,
            total_events: 1000,
            total_reports: 10,
            storage_size_bytes: Some(1024 * 1024),
            connected: true,
        };
        assert_eq!(stats.total_missions, 10);
        assert!(stats.connected);
    }
}
