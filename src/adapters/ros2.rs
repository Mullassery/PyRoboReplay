use super::{AdapterError, MissionAdapter};
use crate::core::event::*;
use anyhow::Result as AnyResult;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::Path;

pub struct Ros2Adapter;

#[derive(Debug, Clone)]
struct RosTopic {
    id: i64,
    name: String,
    msg_type: String,
}

#[derive(Debug, Clone)]
struct RosMessage {
    timestamp: i64,
    data: Vec<u8>,
    topic_id: i64,
}

impl Ros2Adapter {
    pub fn new() -> Self {
        Self
    }

    /// Parse ROS 2 bag file (.db3 SQLite format)
    /// Extracts common topics and converts to universal events
    pub fn parse_bag_file(&self, path: &str) -> Result<MissionRecord, AdapterError> {
        if !Path::new(path).exists() {
            return Err(AdapterError::FileReadError(format!(
                "Bag file not found: {}",
                path
            )));
        }

        if !path.ends_with(".bag") && !path.ends_with(".db3") {
            return Err(AdapterError::InvalidFormat(
                "Expected .bag or .db3 file".to_string(),
            ));
        }

        self.parse_db3_file(path)
            .map_err(|e| AdapterError::ParseError(e.to_string()))
    }

    fn parse_db3_file(&self, path: &str) -> AnyResult<MissionRecord> {
        let conn = Connection::open(path)?;

        // Get mission name from file
        let filename = Path::new(path)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("ros2_mission");

        let mut mission = MissionRecord::new(format!("{} ({})", filename, Utc::now().timestamp()));

        // Get topics from database
        let topics = self.get_topics(&conn)?;
        tracing::info!("Found {} topics in bag", topics.len());

        // Build a map of topic_id -> topic_name for reference
        let topic_map: std::collections::HashMap<i64, String> = topics
            .iter()
            .map(|t| (t.id, t.name.clone()))
            .collect();

        // Get messages and convert to events
        for topic in &topics {
            let messages = self.get_messages_for_topic(&conn, topic.id)?;
            tracing::debug!("Topic {}: found {} messages", topic.name, messages.len());

            for msg in messages {
                if let Ok(event) = self.parse_message(topic, &msg, &topic_map) {
                    mission.add_event(event);
                }
            }
        }

        // Sort events by timestamp
        mission.sort_by_timestamp();

        tracing::info!(
            "Parsed {} events from bag file",
            mission.event_count()
        );
        Ok(mission)
    }

    fn get_topics(&self, conn: &Connection) -> AnyResult<Vec<RosTopic>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, type FROM topics"
        )?;

        let topics = stmt.query_map([], |row| {
            Ok(RosTopic {
                id: row.get(0)?,
                name: row.get(1)?,
                msg_type: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(topics)
    }

    fn get_messages_for_topic(&self, conn: &Connection, topic_id: i64) -> AnyResult<Vec<RosMessage>> {
        let mut stmt = conn.prepare(
            "SELECT timestamp, data FROM messages WHERE topic_id = ? ORDER BY timestamp"
        )?;

        let messages = stmt.query_map([topic_id], |row| {
            Ok(RosMessage {
                timestamp: row.get(0)?,
                data: row.get(1)?,
                topic_id,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    fn parse_message(
        &self,
        topic: &RosTopic,
        msg: &RosMessage,
        _topic_map: &std::collections::HashMap<i64, String>,
    ) -> AnyResult<MissionEvent> {
        let timestamp = DateTime::<Utc>::from_timestamp_nanos(msg.timestamp);

        // Parse based on topic name patterns
        if topic.name.contains("scan") || topic.name.contains("lidar") {
            self.parse_lidar_message(topic, msg, timestamp)
        } else if topic.name.contains("camera") || topic.name.contains("image") {
            self.parse_camera_message(topic, msg, timestamp)
        } else if topic.name.contains("imu") {
            self.parse_imu_message(topic, msg, timestamp)
        } else if topic.name.contains("odom") || topic.name.contains("odometry") {
            self.parse_odometry_message(topic, msg, timestamp)
        } else if topic.name.contains("pose") {
            self.parse_pose_message(topic, msg, timestamp)
        } else {
            Err(anyhow::anyhow!("Unsupported message type: {}", topic.msg_type))
        }
    }

    fn parse_lidar_message(
        &self,
        _topic: &RosTopic,
        _msg: &RosMessage,
        timestamp: DateTime<Utc>,
    ) -> AnyResult<MissionEvent> {
        // Stub: return minimal lidar event
        Ok(MissionEvent::LidarScan {
            robot_id: "robot_1".to_string(),
            timestamp,
            data: LidarData {
                ranges: vec![],
                intensities: None,
                frame_id: "laser_frame".to_string(),
                min_angle: -std::f32::consts::PI,
                max_angle: std::f32::consts::PI,
                angle_increment: 0.01,
                range_min: 0.0,
                range_max: 30.0,
            },
        })
    }

    fn parse_camera_message(
        &self,
        _topic: &RosTopic,
        _msg: &RosMessage,
        timestamp: DateTime<Utc>,
    ) -> AnyResult<MissionEvent> {
        // Stub: return minimal camera frame
        Ok(MissionEvent::CameraFrame {
            robot_id: "robot_1".to_string(),
            timestamp,
            data: CameraFrame {
                sensor_id: "camera_1".to_string(),
                frame_id: "camera_frame".to_string(),
                width: 640,
                height: 480,
                encoding: "rgb8".to_string(),
                image_data: vec![],
                camera_info: None,
            },
        })
    }

    fn parse_imu_message(
        &self,
        _topic: &RosTopic,
        _msg: &RosMessage,
        timestamp: DateTime<Utc>,
    ) -> AnyResult<MissionEvent> {
        Ok(MissionEvent::IMUData {
            robot_id: "robot_1".to_string(),
            timestamp,
            data: IMUData {
                frame_id: "imu_frame".to_string(),
                linear_acceleration: [0.0, 0.0, 0.0],
                angular_velocity: [0.0, 0.0, 0.0],
                magnetometer: None,
                orientation: None,
            },
        })
    }

    fn parse_odometry_message(
        &self,
        _topic: &RosTopic,
        _msg: &RosMessage,
        timestamp: DateTime<Utc>,
    ) -> AnyResult<MissionEvent> {
        Ok(MissionEvent::OdometryUpdate {
            robot_id: "robot_1".to_string(),
            timestamp,
            data: Odometry {
                frame_id: "odom".to_string(),
                child_frame_id: "base_link".to_string(),
                pose: Pose {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    qx: 0.0,
                    qy: 0.0,
                    qz: 0.0,
                    qw: 1.0,
                },
                twist_linear: [0.0, 0.0, 0.0],
                twist_angular: [0.0, 0.0, 0.0],
            },
        })
    }

    fn parse_pose_message(
        &self,
        _topic: &RosTopic,
        _msg: &RosMessage,
        timestamp: DateTime<Utc>,
    ) -> AnyResult<MissionEvent> {
        Ok(MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp,
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
        })
    }
}

impl Default for Ros2Adapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MissionAdapter for Ros2Adapter {
    fn read(&self, path: &str) -> Result<MissionRecord, AdapterError> {
        self.parse_bag_file(path)
    }

    fn adapter_name(&self) -> &str {
        "ros2_adapter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ros2_adapter_creation() {
        let adapter = Ros2Adapter::new();
        assert_eq!(adapter.adapter_name(), "ros2_adapter");
    }

    #[test]
    fn test_ros2_adapter_missing_file() {
        let adapter = Ros2Adapter::new();
        let result = adapter.read("/nonexistent/path.bag");
        assert!(result.is_err());
    }

    #[test]
    fn test_ros2_adapter_invalid_format() {
        let adapter = Ros2Adapter::new();
        let result = adapter.read("/tmp/invalid.txt");
        assert!(result.is_err());
    }
}
