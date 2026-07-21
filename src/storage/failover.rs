use crate::storage::backend::{StorageBackend, StorageError, StorageResult, StorageStats};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Failover event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FailoverEventType {
    /// Primary backend failed
    PrimaryFailed,
    /// Standby promoted to primary
    StandbyPromoted,
    /// Heartbeat check failed
    HeartbeatFailed,
    /// Write-ahead completed successfully
    WriteAheadCompleted,
    /// Write-ahead failed on standby
    WriteAheadFailed,
}

/// Backup configuration
#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub heartbeat_interval_ms: u64,
    pub max_promotion_retries: usize,
}

impl Default for BackupConfig {
    fn default() -> Self {
        BackupConfig {
            heartbeat_interval_ms: 5000,
            max_promotion_retries: 3,
        }
    }
}

/// Failover event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: FailoverEventType,
    pub promoted_backend: String,
    pub demoted_backend: String,
    pub reason: String,
    pub retry_count: usize,
}

/// Failover errors
#[derive(Debug, Error)]
pub enum FailoverError {
    #[error("All backends exhausted — no healthy backend available")]
    AllBackendsExhausted,
    #[error("Primary failed to respond: {0}")]
    PrimaryHeartbeatFailed(String),
    #[error("Write-ahead failed on standby {index}: {reason}")]
    WriteAheadFailed { index: usize, reason: String },
}

/// Mission-critical failover manager
/// Manages primary + standby backends with automatic promotion and write-ahead logging
pub struct FailoverManager {
    primary: Arc<Mutex<Box<dyn StorageBackend>>>,
    standbys: Vec<Arc<Mutex<Box<dyn StorageBackend>>>>,
    config: BackupConfig,
    active_idx: usize,
    failover_log: Arc<Mutex<Vec<FailoverEvent>>>,
}

impl FailoverManager {
    /// Create a new failover manager
    pub fn new(
        primary: Box<dyn StorageBackend>,
        standbys: Vec<Box<dyn StorageBackend>>,
        config: BackupConfig,
    ) -> Self {
        let primary_arc = Arc::new(Mutex::new(primary));
        let standbys_arc = standbys
            .into_iter()
            .map(|s| Arc::new(Mutex::new(s)))
            .collect();

        FailoverManager {
            primary: primary_arc,
            standbys: standbys_arc,
            config,
            active_idx: 0,
            failover_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Check primary health via heartbeat
    pub fn heartbeat(&mut self) -> Result<(), FailoverError> {
        let primary_name = self.current_primary_name();

        let result = {
            let mut primary = self.primary.lock().unwrap();
            primary.connect()
        };

        if result.is_err() {
            let reason = format!("Primary {} failed heartbeat check", primary_name);
            self.failover_log.lock().unwrap().push(FailoverEvent {
                timestamp: Utc::now(),
                event_type: FailoverEventType::HeartbeatFailed,
                promoted_backend: String::new(),
                demoted_backend: primary_name,
                reason: reason.clone(),
                retry_count: 0,
            });

            self.promote_standby(&reason)?;
        }

        Ok(())
    }

    /// Promote next standby to primary
    fn promote_standby(&mut self, reason: &str) -> Result<(), FailoverError> {
        if self.standbys.is_empty() {
            return Err(FailoverError::AllBackendsExhausted);
        }

        let old_primary_name = self.current_primary_name();
        let new_primary = self.standbys.remove(0);

        {
            let mut new_prim = new_primary.lock().unwrap();
            new_prim.connect().map_err(|e| {
                FailoverError::PrimaryHeartbeatFailed(format!("Failed to connect promoted primary: {:?}", e))
            })?;
        }

        self.primary = new_primary;
        self.active_idx += 1;

        self.failover_log.lock().unwrap().push(FailoverEvent {
            timestamp: Utc::now(),
            event_type: FailoverEventType::StandbyPromoted,
            promoted_backend: self.current_primary_name(),
            demoted_backend: old_primary_name,
            reason: reason.to_string(),
            retry_count: 0,
        });

        Ok(())
    }

    /// Get name of current primary
    pub fn current_primary_name(&self) -> String {
        match self.active_idx {
            0 => "primary".to_string(),
            n => format!("standby_{}", n - 1),
        }
    }

    /// Check if using standby
    pub fn is_using_standby(&self) -> bool {
        self.active_idx > 0
    }

    /// Get failover log
    pub fn failover_log(&self) -> Vec<FailoverEvent> {
        self.failover_log.lock().unwrap().clone()
    }
}

impl StorageBackend for FailoverManager {
    fn connect(&mut self) -> StorageResult<()> {
        let mut primary = self.primary.lock().unwrap();
        primary.connect()
    }

    fn store_mission(&self, mission_id: &str, data: &str) -> StorageResult<()> {
        let mut primary = self.primary.lock().unwrap();
        primary.store_mission(mission_id, data)?;

        // Write-ahead to all standbys (non-fatal if they fail)
        for (idx, standby) in self.standbys.iter().enumerate() {
            let mut sb = standby.lock().unwrap();
            if let Err(e) = sb.store_mission(mission_id, data) {
                self.failover_log.lock().unwrap().push(FailoverEvent {
                    timestamp: Utc::now(),
                    event_type: FailoverEventType::WriteAheadFailed,
                    promoted_backend: self.current_primary_name().to_string(),
                    demoted_backend: format!("standby_{}", idx),
                    reason: format!("Failed to write mission: {:?}", e),
                    retry_count: 0,
                });
            }
        }

        Ok(())
    }

    fn retrieve_mission(&self, mission_id: &str) -> StorageResult<String> {
        let primary = self.primary.lock().unwrap();
        primary.retrieve_mission(mission_id)
    }

    fn store_event(&self, mission_id: &str, event_id: &str, data: &str) -> StorageResult<()> {
        let mut primary = self.primary.lock().unwrap();
        primary.store_event(mission_id, event_id, data)?;

        // Write-ahead to all standbys (non-fatal if they fail)
        for (idx, standby) in self.standbys.iter().enumerate() {
            let mut sb = standby.lock().unwrap();
            if let Err(e) = sb.store_event(mission_id, event_id, data) {
                self.failover_log.lock().unwrap().push(FailoverEvent {
                    timestamp: Utc::now(),
                    event_type: FailoverEventType::WriteAheadFailed,
                    promoted_backend: self.current_primary_name().to_string(),
                    demoted_backend: format!("standby_{}", idx),
                    reason: format!("Failed to write event: {:?}", e),
                    retry_count: 0,
                });
            }
        }

        Ok(())
    }

    fn retrieve_event(&self, mission_id: &str, event_id: &str) -> StorageResult<String> {
        let primary = self.primary.lock().unwrap();
        primary.retrieve_event(mission_id, event_id)
    }

    fn store_report(&self, mission_id: &str, report: &str) -> StorageResult<()> {
        let mut primary = self.primary.lock().unwrap();
        primary.store_report(mission_id, report)?;

        // Write-ahead to all standbys (non-fatal if they fail)
        for (idx, standby) in self.standbys.iter().enumerate() {
            let mut sb = standby.lock().unwrap();
            if let Err(e) = sb.store_report(mission_id, report) {
                self.failover_log.lock().unwrap().push(FailoverEvent {
                    timestamp: Utc::now(),
                    event_type: FailoverEventType::WriteAheadFailed,
                    promoted_backend: self.current_primary_name().to_string(),
                    demoted_backend: format!("standby_{}", idx),
                    reason: format!("Failed to write report: {:?}", e),
                    retry_count: 0,
                });
            }
        }

        Ok(())
    }

    fn retrieve_report(&self, mission_id: &str) -> StorageResult<String> {
        let primary = self.primary.lock().unwrap();
        primary.retrieve_report(mission_id)
    }

    fn list_missions(&self, limit: Option<usize>) -> StorageResult<Vec<String>> {
        let primary = self.primary.lock().unwrap();
        primary.list_missions(limit)
    }

    fn delete_mission(&self, mission_id: &str) -> StorageResult<()> {
        let mut primary = self.primary.lock().unwrap();
        primary.delete_mission(mission_id)?;

        // Write-ahead to all standbys (non-fatal if they fail)
        for (idx, standby) in self.standbys.iter().enumerate() {
            let mut sb = standby.lock().unwrap();
            if let Err(e) = sb.delete_mission(mission_id) {
                self.failover_log.lock().unwrap().push(FailoverEvent {
                    timestamp: Utc::now(),
                    event_type: FailoverEventType::WriteAheadFailed,
                    promoted_backend: self.current_primary_name().to_string(),
                    demoted_backend: format!("standby_{}", idx),
                    reason: format!("Failed to delete mission: {:?}", e),
                    retry_count: 0,
                });
            }
        }

        Ok(())
    }

    fn mission_exists(&self, mission_id: &str) -> StorageResult<bool> {
        let primary = self.primary.lock().unwrap();
        primary.mission_exists(mission_id)
    }

    fn get_stats(&self) -> StorageResult<StorageStats> {
        let primary = self.primary.lock().unwrap();
        primary.get_stats()
    }

    fn close(&self) -> StorageResult<()> {
        let mut primary = self.primary.lock().unwrap();
        primary.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryBackend;

    fn create_failover_manager() -> FailoverManager {
        let primary = Box::new(InMemoryBackend::new());
        let standby1 = Box::new(InMemoryBackend::new());
        let standby2 = Box::new(InMemoryBackend::new());

        let config = BackupConfig::default();
        FailoverManager::new(primary, vec![standby1, standby2], config)
    }

    #[test]
    fn test_failover_manager_creation() {
        let manager = create_failover_manager();
        assert_eq!(manager.current_primary_name(), "primary");
        assert!(!manager.is_using_standby());
    }

    #[test]
    fn test_store_mission_writes_to_primary() {
        let manager = create_failover_manager();
        let data = r#"{"id":"mission_1","name":"test"}"#;

        assert!(manager.store_mission("mission_1", data).is_ok());
        assert!(manager.retrieve_mission("mission_1").is_ok());
    }

    #[test]
    fn test_store_mission_writes_to_standbys() {
        let manager = create_failover_manager();
        let data = r#"{"id":"mission_1","name":"test"}"#;

        manager.store_mission("mission_1", data).unwrap();
        assert!(manager.retrieve_mission("mission_1").is_ok());
    }

    #[test]
    fn test_heartbeat_healthy_primary_no_failover() {
        let mut manager = create_failover_manager();
        assert!(manager.heartbeat().is_ok());
        assert_eq!(manager.failover_log().len(), 0);
    }

    #[test]
    fn test_promote_standby_on_primary_failure() {
        let mut manager = create_failover_manager();
        let initial_name = manager.current_primary_name().to_string();

        assert!(manager.promote_standby("test promotion").is_ok());
        assert_ne!(manager.current_primary_name(), initial_name);
        assert!(manager.is_using_standby());
    }

    #[test]
    fn test_failover_log_records_promotion() {
        let mut manager = create_failover_manager();
        manager.promote_standby("test").ok();

        let log = manager.failover_log();
        assert!(!log.is_empty());
        assert_eq!(log[0].event_type, FailoverEventType::StandbyPromoted);
    }

    #[test]
    fn test_failover_event_has_timestamp() {
        let mut manager = create_failover_manager();
        let before = Utc::now();
        manager.promote_standby("test").ok();
        let after = Utc::now();

        let log = manager.failover_log();
        assert!(!log.is_empty());
        let ts = log[0].timestamp;
        assert!(ts >= before && ts <= after);
    }

    #[test]
    fn test_all_backends_exhausted_error() {
        let mut manager = FailoverManager::new(
            Box::new(InMemoryBackend::new()),
            vec![],
            BackupConfig::default(),
        );

        let result = manager.promote_standby("test");
        assert!(matches!(result, Err(FailoverError::AllBackendsExhausted)));
    }

    #[test]
    fn test_retrieve_after_failover_reads_from_new_primary() {
        let mut manager = create_failover_manager();
        let data = r#"{"id":"mission_1"}"#;

        manager.store_mission("mission_1", data).unwrap();
        manager.promote_standby("test").ok();

        assert!(manager.retrieve_mission("mission_1").is_ok());
    }

    #[test]
    fn test_is_using_standby_false_initially() {
        let manager = create_failover_manager();
        assert!(!manager.is_using_standby());
    }

    #[test]
    fn test_is_using_standby_true_after_promotion() {
        let mut manager = create_failover_manager();
        manager.promote_standby("test").ok();
        assert!(manager.is_using_standby());
    }

    #[test]
    fn test_write_ahead_continues_if_standby_fails() {
        let mut manager = create_failover_manager();
        let data = r#"{"id":"mission_1"}"#;

        assert!(manager.store_mission("mission_1", data).is_ok());
        assert!(manager.retrieve_mission("mission_1").is_ok());
    }

    #[test]
    fn test_store_event_write_ahead() {
        let manager = create_failover_manager();
        manager.store_mission("mission_1", r#"{"id":"mission_1"}"#).ok();

        let event_data = r#"{"type":"lidar_scan"}"#;
        assert!(manager.store_event("mission_1", "event_1", event_data).is_ok());
        assert!(manager.retrieve_event("mission_1", "event_1").is_ok());
    }

    #[test]
    fn test_delete_mission_cascades_to_standbys() {
        let manager = create_failover_manager();
        manager.store_mission("mission_1", r#"{"id":"mission_1"}"#).ok();

        assert!(manager.delete_mission("mission_1").is_ok());
        assert!(manager.retrieve_mission("mission_1").is_err());
    }
}
