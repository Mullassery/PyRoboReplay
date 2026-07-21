pub mod backend;
pub mod inmemory;
pub mod audit;
pub mod sqlite;
pub mod backends;
pub mod failover;

pub use backend::{StorageBackend, StorageConfig, StorageError, StorageResult};
pub use inmemory::InMemoryBackend;
pub use audit::{AuditTrail, AuditEvent, AuditEventType};
pub use sqlite::SqliteBackend;
pub use backends::{PostgresBackend, BigQueryBackend, S3Backend};
pub use failover::{FailoverManager, FailoverEvent, FailoverEventType, BackupConfig, FailoverError};
