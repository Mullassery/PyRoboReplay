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
pub struct LidarData {
    pub ranges: Vec<f32>,
    pub intensities: Option<Vec<f32>>,
    pub frame_id: String,
    pub min_angle: f32,
    pub max_angle: f32,
    pub angle_increment: f32,
    pub range_min: f32,
    pub range_max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFrame {
    pub sensor_id: String,
    pub frame_id: String,
    pub width: u32,
    pub height: u32,
    pub encoding: String, // rgb8, mono8, bgr8, etc.
    pub image_data: Vec<u8>,
    pub camera_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IMUData {
    pub frame_id: String,
    pub linear_acceleration: [f64; 3],
    pub angular_velocity: [f64; 3],
    pub magnetometer: Option<[f64; 3]>,
    pub orientation: Option<[f64; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Odometry {
    pub frame_id: String,
    pub child_frame_id: String,
    pub pose: Pose,
    pub twist_linear: [f64; 3],
    pub twist_angular: [f64; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Costmap {
    pub frame_id: String,
    pub resolution: f32,
    pub width: u32,
    pub height: u32,
    pub origin: Pose,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionEvent {
    // Sensor streams (individually replayable)
    LidarScan {
        robot_id: String,
        timestamp: DateTime<Utc>,
        data: LidarData,
    },
    CameraFrame {
        robot_id: String,
        timestamp: DateTime<Utc>,
        data: CameraFrame,
    },
    IMUData {
        robot_id: String,
        timestamp: DateTime<Utc>,
        data: IMUData,
    },
    OdometryUpdate {
        robot_id: String,
        timestamp: DateTime<Utc>,
        data: Odometry,
    },
    CostmapUpdate {
        robot_id: String,
        timestamp: DateTime<Utc>,
        data: Costmap,
    },

    // Processed/fused state
    RobotPose {
        robot_id: String,
        timestamp: DateTime<Utc>,
        pose: Pose,
        confidence: Option<f32>,
    },

    // Navigation & coordination
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
        confidence: Option<f32>,
    },

    // Communication & fleet
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

    // Environmental context
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
            MissionEvent::LidarScan { timestamp, .. }
            | MissionEvent::CameraFrame { timestamp, .. }
            | MissionEvent::IMUData { timestamp, .. }
            | MissionEvent::OdometryUpdate { timestamp, .. }
            | MissionEvent::CostmapUpdate { timestamp, .. }
            | MissionEvent::RobotPose { timestamp, .. }
            | MissionEvent::NavigationDecision { timestamp, .. }
            | MissionEvent::ObstacleDetected { timestamp, .. }
            | MissionEvent::CommunicationEvent { timestamp, .. }
            | MissionEvent::CoordinationEvent { timestamp, .. }
            | MissionEvent::EnvironmentalChange { timestamp, .. }
            | MissionEvent::MissionLifecycle { timestamp, .. } => *timestamp,
        }
    }

    pub fn event_type(&self) -> &str {
        match self {
            MissionEvent::LidarScan { .. } => "lidar_scan",
            MissionEvent::CameraFrame { .. } => "camera_frame",
            MissionEvent::IMUData { .. } => "imu_data",
            MissionEvent::OdometryUpdate { .. } => "odometry_update",
            MissionEvent::CostmapUpdate { .. } => "costmap_update",
            MissionEvent::RobotPose { .. } => "robot_pose",
            MissionEvent::NavigationDecision { .. } => "navigation_decision",
            MissionEvent::ObstacleDetected { .. } => "obstacle_detected",
            MissionEvent::CommunicationEvent { .. } => "communication_event",
            MissionEvent::CoordinationEvent { .. } => "coordination_event",
            MissionEvent::EnvironmentalChange { .. } => "environmental_change",
            MissionEvent::MissionLifecycle { .. } => "mission_lifecycle",
        }
    }

    pub fn robot_id(&self) -> Option<&str> {
        match self {
            MissionEvent::LidarScan { robot_id, .. }
            | MissionEvent::CameraFrame { robot_id, .. }
            | MissionEvent::IMUData { robot_id, .. }
            | MissionEvent::OdometryUpdate { robot_id, .. }
            | MissionEvent::CostmapUpdate { robot_id, .. }
            | MissionEvent::RobotPose { robot_id, .. }
            | MissionEvent::NavigationDecision { robot_id, .. }
            | MissionEvent::ObstacleDetected { robot_id, .. } => Some(robot_id),
            _ => None,
        }
    }

    pub fn sensor_type(&self) -> Option<&str> {
        match self {
            MissionEvent::LidarScan { .. } => Some("lidar"),
            MissionEvent::CameraFrame { .. } => Some("camera"),
            MissionEvent::IMUData { .. } => Some("imu"),
            MissionEvent::OdometryUpdate { .. } => Some("odometry"),
            MissionEvent::CostmapUpdate { .. } => Some("costmap"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            confidence: Some(0.95),
        };
        mission.add_event(event);
        assert_eq!(mission.event_count(), 1);
    }

    #[test]
    fn test_lidar_event() {
        let event = MissionEvent::LidarScan {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            data: LidarData {
                ranges: vec![1.0, 1.5, 2.0],
                intensities: Some(vec![100.0, 150.0, 200.0]),
                frame_id: "laser_frame".to_string(),
                min_angle: -3.14,
                max_angle: 3.14,
                angle_increment: 0.01,
                range_min: 0.0,
                range_max: 30.0,
            },
        };
        assert_eq!(event.event_type(), "lidar_scan");
        assert_eq!(event.sensor_type(), Some("lidar"));
        assert_eq!(event.robot_id(), Some("robot_1"));
    }

    #[test]
    fn test_camera_event() {
        let event = MissionEvent::CameraFrame {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            data: CameraFrame {
                sensor_id: "camera_1".to_string(),
                frame_id: "camera_frame".to_string(),
                width: 640,
                height: 480,
                encoding: "rgb8".to_string(),
                image_data: vec![0u8; 640 * 480 * 3],
                camera_info: None,
            },
        };
        assert_eq!(event.event_type(), "camera_frame");
        assert_eq!(event.sensor_type(), Some("camera"));
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
            confidence: Some(0.95),
        };
        assert_eq!(event.timestamp(), now);
    }
}
