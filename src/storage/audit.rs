use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit trail event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    /// Mission created
    MissionCreated,
    /// Mission modified
    MissionModified,
    /// Mission deleted
    MissionDeleted,
    /// Event added
    EventAdded,
    /// Event modified
    EventModified,
    /// Report generated
    ReportGenerated,
    /// Data accessed
    DataAccessed,
    /// Storage backend changed
    BackendChanged,
}

/// Audit trail event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: AuditEventType,
    /// Mission ID
    pub mission_id: String,
    /// User or service performing action
    pub actor: String,
    /// Action description
    pub action: String,
    /// Data hash (for integrity verification)
    pub data_hash: Option<String>,
    /// Audit event ID (for tracking)
    pub event_id: String,
}

impl AuditEvent {
    /// Create new audit event
    pub fn new(
        event_type: AuditEventType,
        mission_id: &str,
        actor: &str,
        action: &str,
    ) -> Self {
        AuditEvent {
            timestamp: Utc::now(),
            event_type,
            mission_id: mission_id.to_string(),
            actor: actor.to_string(),
            action: action.to_string(),
            data_hash: None,
            event_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Set data hash for integrity verification
    pub fn with_hash(mut self, hash: String) -> Self {
        self.data_hash = Some(hash);
        self
    }
}

/// Audit trail storage
pub struct AuditTrail {
    events: Vec<AuditEvent>,
}

impl AuditTrail {
    /// Create new audit trail
    pub fn new() -> Self {
        AuditTrail {
            events: Vec::new(),
        }
    }

    /// Record an audit event
    pub fn record(&mut self, event: AuditEvent) {
        self.events.push(event);
    }

    /// Get all events
    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    /// Get events for a mission
    pub fn events_for_mission(&self, mission_id: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.mission_id == mission_id)
            .collect()
    }

    /// Get events by type
    pub fn events_by_type(&self, event_type: AuditEventType) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| e.event_type == event_type).collect()
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Verify audit trail integrity (simplified)
    pub fn verify_integrity(&self) -> bool {
        // In production, this would verify cryptographic hashes
        // For now, just check that timestamps are ordered
        for i in 1..self.events.len() {
            if self.events[i].timestamp < self.events[i - 1].timestamp {
                return false;
            }
        }
        true
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::MissionCreated,
            "mission_1",
            "robot_service",
            "Created new mission",
        );

        assert_eq!(event.mission_id, "mission_1");
        assert_eq!(event.actor, "robot_service");
        assert_eq!(event.event_type, AuditEventType::MissionCreated);
    }

    #[test]
    fn test_audit_event_with_hash() {
        let event = AuditEvent::new(
            AuditEventType::MissionCreated,
            "mission_1",
            "robot_service",
            "Created mission",
        )
        .with_hash("abc123".to_string());

        assert_eq!(event.data_hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_audit_trail_creation() {
        let trail = AuditTrail::new();
        assert_eq!(trail.event_count(), 0);
    }

    #[test]
    fn test_record_event() {
        let mut trail = AuditTrail::new();
        let event = AuditEvent::new(
            AuditEventType::MissionCreated,
            "mission_1",
            "robot",
            "Created",
        );

        trail.record(event);
        assert_eq!(trail.event_count(), 1);
    }

    #[test]
    fn test_events_for_mission() {
        let mut trail = AuditTrail::new();
        trail.record(AuditEvent::new(
            AuditEventType::MissionCreated,
            "mission_1",
            "robot",
            "Created",
        ));
        trail.record(AuditEvent::new(
            AuditEventType::MissionCreated,
            "mission_2",
            "robot",
            "Created",
        ));

        let events = trail.events_for_mission("mission_1");
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_events_by_type() {
        let mut trail = AuditTrail::new();
        trail.record(AuditEvent::new(
            AuditEventType::MissionCreated,
            "mission_1",
            "robot",
            "Created",
        ));
        trail.record(AuditEvent::new(
            AuditEventType::EventAdded,
            "mission_1",
            "robot",
            "Added event",
        ));

        let created_events = trail.events_by_type(AuditEventType::MissionCreated);
        assert_eq!(created_events.len(), 1);
    }

    #[test]
    fn test_verify_integrity() {
        let mut trail = AuditTrail::new();
        trail.record(AuditEvent::new(
            AuditEventType::MissionCreated,
            "mission_1",
            "robot",
            "Created",
        ));
        trail.record(AuditEvent::new(
            AuditEventType::EventAdded,
            "mission_1",
            "robot",
            "Added",
        ));

        assert!(trail.verify_integrity());
    }
}
