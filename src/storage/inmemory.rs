use super::backend::{StorageBackend, StorageError, StorageResult, StorageStats};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory storage backend for testing and development
pub struct InMemoryBackend {
    missions: Arc<RwLock<HashMap<String, MissionData>>>,
    connected: Arc<RwLock<bool>>,
}

struct MissionData {
    mission_data: String,
    events: HashMap<String, String>,
    report: Option<String>,
}

impl InMemoryBackend {
    /// Create new in-memory backend
    pub fn new() -> Self {
        InMemoryBackend {
            missions: Arc::new(RwLock::new(HashMap::new())),
            connected: Arc::new(RwLock::new(false)),
        }
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for InMemoryBackend {
    fn connect(&mut self) -> StorageResult<()> {
        let mut connected = self
            .connected
            .write()
            .map_err(|_| StorageError::ConnectionFailed("Lock poisoned".to_string()))?;
        *connected = true;
        Ok(())
    }

    fn store_mission(&self, mission_id: &str, data: &str) -> StorageResult<()> {
        let mut missions = self
            .missions
            .write()
            .map_err(|_| StorageError::WriteFailed("Lock poisoned".to_string()))?;

        missions.insert(
            mission_id.to_string(),
            MissionData {
                mission_data: data.to_string(),
                events: HashMap::new(),
                report: None,
            },
        );

        Ok(())
    }

    fn retrieve_mission(&self, mission_id: &str) -> StorageResult<String> {
        let missions = self
            .missions
            .read()
            .map_err(|_| StorageError::ReadFailed("Lock poisoned".to_string()))?;

        missions
            .get(mission_id)
            .map(|m| m.mission_data.clone())
            .ok_or_else(|| StorageError::NotFound(format!("Mission {} not found", mission_id)))
    }

    fn store_event(&self, mission_id: &str, event_id: &str, data: &str) -> StorageResult<()> {
        let mut missions = self
            .missions
            .write()
            .map_err(|_| StorageError::WriteFailed("Lock poisoned".to_string()))?;

        let mission = missions
            .get_mut(mission_id)
            .ok_or_else(|| StorageError::WriteFailed(format!("Mission {} not found", mission_id)))?;

        mission.events.insert(event_id.to_string(), data.to_string());
        Ok(())
    }

    fn retrieve_event(&self, mission_id: &str, event_id: &str) -> StorageResult<String> {
        let missions = self
            .missions
            .read()
            .map_err(|_| StorageError::ReadFailed("Lock poisoned".to_string()))?;

        let mission = missions
            .get(mission_id)
            .ok_or_else(|| StorageError::NotFound(format!("Mission {} not found", mission_id)))?;

        mission
            .events
            .get(event_id)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(format!("Event {} not found", event_id)))
    }

    fn store_report(&self, mission_id: &str, report: &str) -> StorageResult<()> {
        let mut missions = self
            .missions
            .write()
            .map_err(|_| StorageError::WriteFailed("Lock poisoned".to_string()))?;

        let mission = missions
            .get_mut(mission_id)
            .ok_or_else(|| StorageError::WriteFailed(format!("Mission {} not found", mission_id)))?;

        mission.report = Some(report.to_string());
        Ok(())
    }

    fn retrieve_report(&self, mission_id: &str) -> StorageResult<String> {
        let missions = self
            .missions
            .read()
            .map_err(|_| StorageError::ReadFailed("Lock poisoned".to_string()))?;

        let mission = missions
            .get(mission_id)
            .ok_or_else(|| StorageError::NotFound(format!("Mission {} not found", mission_id)))?;

        mission
            .report
            .clone()
            .ok_or_else(|| StorageError::NotFound(format!("Report for {} not found", mission_id)))
    }

    fn list_missions(&self, limit: Option<usize>) -> StorageResult<Vec<String>> {
        let missions = self
            .missions
            .read()
            .map_err(|_| StorageError::ReadFailed("Lock poisoned".to_string()))?;

        let mut mission_ids: Vec<String> = missions.keys().cloned().collect();
        mission_ids.sort();

        if let Some(limit) = limit {
            mission_ids.truncate(limit);
        }

        Ok(mission_ids)
    }

    fn delete_mission(&self, mission_id: &str) -> StorageResult<()> {
        let mut missions = self
            .missions
            .write()
            .map_err(|_| StorageError::WriteFailed("Lock poisoned".to_string()))?;

        missions.remove(mission_id);
        Ok(())
    }

    fn mission_exists(&self, mission_id: &str) -> StorageResult<bool> {
        let missions = self
            .missions
            .read()
            .map_err(|_| StorageError::ReadFailed("Lock poisoned".to_string()))?;

        Ok(missions.contains_key(mission_id))
    }

    fn get_stats(&self) -> StorageResult<StorageStats> {
        let missions = self
            .missions
            .read()
            .map_err(|_| StorageError::ReadFailed("Lock poisoned".to_string()))?;

        let connected = self
            .connected
            .read()
            .map_err(|_| StorageError::ReadFailed("Lock poisoned".to_string()))?;

        let total_events: u64 = missions.values().map(|m| m.events.len() as u64).sum();
        let total_reports: u64 = missions.values().filter(|m| m.report.is_some()).count() as u64;

        Ok(StorageStats {
            total_missions: missions.len() as u64,
            total_events,
            total_reports,
            storage_size_bytes: None,
            connected: *connected,
        })
    }

    fn close(&self) -> StorageResult<()> {
        let mut connected = self
            .connected
            .write()
            .map_err(|_| StorageError::ConnectionFailed("Lock poisoned".to_string()))?;
        *connected = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inmemory_backend_creation() {
        let backend = InMemoryBackend::new();
        assert_eq!(backend.missions.read().unwrap().len(), 0);
    }

    #[test]
    fn test_connect() {
        let mut backend = InMemoryBackend::new();
        assert!(backend.connect().is_ok());
    }

    #[test]
    fn test_store_and_retrieve_mission() {
        let backend = InMemoryBackend::new();
        let mission_data = "test mission data";

        assert!(backend.store_mission("mission_1", mission_data).is_ok());
        let retrieved = backend.retrieve_mission("mission_1").unwrap();
        assert_eq!(retrieved, mission_data);
    }

    #[test]
    fn test_store_and_retrieve_event() {
        let backend = InMemoryBackend::new();
        backend.store_mission("mission_1", "data").unwrap();

        let event_data = "event data";
        assert!(backend.store_event("mission_1", "event_1", event_data).is_ok());
        let retrieved = backend.retrieve_event("mission_1", "event_1").unwrap();
        assert_eq!(retrieved, event_data);
    }

    #[test]
    fn test_store_and_retrieve_report() {
        let backend = InMemoryBackend::new();
        backend.store_mission("mission_1", "data").unwrap();

        let report_data = "diagnostic report";
        assert!(backend.store_report("mission_1", report_data).is_ok());
        let retrieved = backend.retrieve_report("mission_1").unwrap();
        assert_eq!(retrieved, report_data);
    }

    #[test]
    fn test_list_missions() {
        let backend = InMemoryBackend::new();
        backend.store_mission("mission_1", "data1").unwrap();
        backend.store_mission("mission_2", "data2").unwrap();

        let missions = backend.list_missions(None).unwrap();
        assert_eq!(missions.len(), 2);
    }

    #[test]
    fn test_mission_exists() {
        let backend = InMemoryBackend::new();
        backend.store_mission("mission_1", "data").unwrap();

        assert!(backend.mission_exists("mission_1").unwrap());
        assert!(!backend.mission_exists("mission_2").unwrap());
    }

    #[test]
    fn test_delete_mission() {
        let backend = InMemoryBackend::new();
        backend.store_mission("mission_1", "data").unwrap();
        assert!(backend.mission_exists("mission_1").unwrap());

        backend.delete_mission("mission_1").unwrap();
        assert!(!backend.mission_exists("mission_1").unwrap());
    }

    #[test]
    fn test_get_stats() {
        let backend = InMemoryBackend::new();
        let mut backend_mut = InMemoryBackend::new();
        backend_mut.connect().unwrap();

        backend_mut.store_mission("mission_1", "data").unwrap();
        backend_mut.store_event("mission_1", "event_1", "event").unwrap();
        backend_mut.store_report("mission_1", "report").unwrap();

        let stats = backend_mut.get_stats().unwrap();
        assert_eq!(stats.total_missions, 1);
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.total_reports, 1);
    }
}
