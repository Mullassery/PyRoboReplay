use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pose {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
    pub qw: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Location {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionEvent {
    RobotPose {
        robot_id: String,
        timestamp: DateTime<Utc>,
        pose: Pose,
    },
    SensorObservation {
        robot_id: String,
        timestamp: DateTime<Utc>,
        sensor_type: String,
        data: serde_json::Value,
    },
    NavigationDecision {
        robot_id: String,
        timestamp: DateTime<Utc>,
        decision_type: String,
        rationale: Option<String>,
    },
    ObstacleDetected {
        robot_id: String,
        timestamp: DateTime<Utc>,
        location: Location,
        obstacle_type: String,
    },
    CommunicationEvent {
        timestamp: DateTime<Utc>,
        from: String,
        to: String,
        event_type: String,
        data: Option<serde_json::Value>,
    },
    CoordinationEvent {
        timestamp: DateTime<Utc>,
        robots_involved: Vec<String>,
        event_type: String,
        data: Option<serde_json::Value>,
    },
    EnvironmentalChange {
        timestamp: DateTime<Utc>,
        location: Location,
        change_type: String,
        description: Option<String>,
    },
    MissionLifecycle {
        timestamp: DateTime<Utc>,
        event_type: String, // start, pause, resume, end
        mission_id: String,
    },
}

impl MissionEvent {
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            MissionEvent::RobotPose { timestamp, .. } => *timestamp,
            MissionEvent::SensorObservation { timestamp, .. } => *timestamp,
            MissionEvent::NavigationDecision { timestamp, .. } => *timestamp,
            MissionEvent::ObstacleDetected { timestamp, .. } => *timestamp,
            MissionEvent::CommunicationEvent { timestamp, .. } => *timestamp,
            MissionEvent::CoordinationEvent { timestamp, .. } => *timestamp,
            MissionEvent::EnvironmentalChange { timestamp, .. } => *timestamp,
            MissionEvent::MissionLifecycle { timestamp, .. } => *timestamp,
        }
    }

    pub fn event_type(&self) -> &str {
        match self {
            MissionEvent::RobotPose { .. } => "robot_pose",
            MissionEvent::SensorObservation { .. } => "sensor_observation",
            MissionEvent::NavigationDecision { .. } => "navigation_decision",
            MissionEvent::ObstacleDetected { .. } => "obstacle_detected",
            MissionEvent::CommunicationEvent { .. } => "communication_event",
            MissionEvent::CoordinationEvent { .. } => "coordination_event",
            MissionEvent::EnvironmentalChange { .. } => "environmental_change",
            MissionEvent::MissionLifecycle { .. } => "mission_lifecycle",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MissionRecord {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub events: Vec<MissionEvent>,
}

impl MissionRecord {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            created_at: Utc::now(),
            events: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: MissionEvent) {
        self.events.push(event);
    }

    pub fn sort_by_timestamp(&mut self) {
        self.events.sort_by_key(|e| e.timestamp());
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        if self.events.is_empty() {
            return None;
        }
        let first = self.events.first()?.timestamp();
        let last = self.events.last()?.timestamp();
        Some(last - first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_record_creation() {
        let mission = MissionRecord::new("test_mission");
        assert_eq!(mission.name, "test_mission");
        assert_eq!(mission.event_count(), 0);
    }

    #[test]
    fn test_add_event() {
        let mut mission = MissionRecord::new("test_mission");
        let event = MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            pose: Pose {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
        };
        mission.add_event(event);
        assert_eq!(mission.event_count(), 1);
    }

    #[test]
    fn test_event_timestamp() {
        let now = Utc::now();
        let event = MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            pose: Pose {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
        };
        assert_eq!(event.timestamp(), now);
    }
}
