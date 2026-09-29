use super::cdr::CdrReader;
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
        let topic_map: std::collections::HashMap<i64, String> =
            topics.iter().map(|t| (t.id, t.name.clone())).collect();

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

        tracing::info!("Parsed {} events from bag file", mission.event_count());
        Ok(mission)
    }

    fn get_topics(&self, conn: &Connection) -> AnyResult<Vec<RosTopic>> {
        let mut stmt = conn.prepare("SELECT id, name, type FROM topics")?;

        let topics = stmt
            .query_map([], |row| {
                Ok(RosTopic {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    msg_type: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(topics)
    }

    fn get_messages_for_topic(
        &self,
        conn: &Connection,
        topic_id: i64,
    ) -> AnyResult<Vec<RosMessage>> {
        let mut stmt = conn.prepare(
            "SELECT timestamp, data FROM messages WHERE topic_id = ? ORDER BY timestamp",
        )?;

        let messages = stmt
            .query_map([topic_id], |row| {
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
            Err(anyhow::anyhow!(
                "Unsupported message type: {}",
                topic.msg_type
            ))
        }
    }

    /// Real `sensor_msgs/msg/LaserScan` CDR decoding:
    /// `{Header header; float32 angle_min, angle_max, angle_increment,
    /// time_increment, scan_time, range_min, range_max; float32[] ranges,
    /// intensities;}`.
    fn parse_lidar_message(
        &self,
        _topic: &RosTopic,
        msg: &RosMessage,
        timestamp: DateTime<Utc>,
    ) -> AnyResult<MissionEvent> {
        let mut r = CdrReader::new(&msg.data)?;
        let (_stamp, frame_id) = r.read_header()?;
        let angle_min = r.read_f32()?;
        let angle_max = r.read_f32()?;
        let angle_increment = r.read_f32()?;
        let _time_increment = r.read_f32()?;
        let _scan_time = r.read_f32()?;
        let range_min = r.read_f32()?;
        let range_max = r.read_f32()?;
        let ranges = r.read_f32_seq()?;
        let intensities = r.read_f32_seq()?;

        Ok(MissionEvent::LidarScan {
            robot_id: "robot_1".to_string(),
            timestamp,
            data: LidarData {
                ranges,
                intensities: if intensities.is_empty() {
                    None
                } else {
                    Some(intensities)
                },
                frame_id,
                min_angle: angle_min,
                max_angle: angle_max,
                angle_increment,
                range_min,
                range_max,
            },
        })
    }

    /// Real `sensor_msgs/msg/Image` CDR decoding:
    /// `{Header header; uint32 height, width; string encoding; uint8
    /// is_bigendian; uint32 step; uint8[] data;}`.
    fn parse_camera_message(
        &self,
        _topic: &RosTopic,
        msg: &RosMessage,
        timestamp: DateTime<Utc>,
    ) -> AnyResult<MissionEvent> {
        let mut r = CdrReader::new(&msg.data)?;
        let (_stamp, frame_id) = r.read_header()?;
        let height = r.read_u32()?;
        let width = r.read_u32()?;
        let encoding = r.read_string()?;
        let _is_bigendian = r.read_u8()?;
        let _step = r.read_u32()?;
        let image_data = r.read_u8_seq()?;

        Ok(MissionEvent::CameraFrame {
            robot_id: "robot_1".to_string(),
            timestamp,
            data: CameraFrame {
                sensor_id: "camera_1".to_string(),
                frame_id,
                width,
                height,
                encoding,
                image_data,
                camera_info: None,
            },
        })
    }

    /// Real `sensor_msgs/msg/Imu` CDR decoding:
    /// `{Header header; Quaternion orientation; float64[9]
    /// orientation_covariance; Vector3 angular_velocity; float64[9]
    /// angular_velocity_covariance; Vector3 linear_acceleration; float64[9]
    /// linear_acceleration_covariance;}`.
    fn parse_imu_message(
        &self,
        _topic: &RosTopic,
        msg: &RosMessage,
        timestamp: DateTime<Utc>,
    ) -> AnyResult<MissionEvent> {
        let mut r = CdrReader::new(&msg.data)?;
        let (_stamp, frame_id) = r.read_header()?;
        let orientation = r.read_quaternion()?;
        let _orientation_covariance = r.read_f64_array::<9>()?;
        let angular_velocity = r.read_vec3()?;
        let _angular_velocity_covariance = r.read_f64_array::<9>()?;
        let linear_acceleration = r.read_vec3()?;
        let _linear_acceleration_covariance = r.read_f64_array::<9>()?;

        Ok(MissionEvent::IMUData {
            robot_id: "robot_1".to_string(),
            timestamp,
            data: IMUData {
                frame_id,
                linear_acceleration,
                angular_velocity,
                magnetometer: None,
                orientation: Some(orientation),
            },
        })
    }

    /// Real `nav_msgs/msg/Odometry` CDR decoding:
    /// `{Header header; string child_frame_id; PoseWithCovariance pose;
    /// TwistWithCovariance twist;}`, where `PoseWithCovariance = {Pose
    /// pose; float64[36] covariance;}` and `Pose = {Point position;
    /// Quaternion orientation;}`.
    fn parse_odometry_message(
        &self,
        _topic: &RosTopic,
        msg: &RosMessage,
        timestamp: DateTime<Utc>,
    ) -> AnyResult<MissionEvent> {
        let mut r = CdrReader::new(&msg.data)?;
        let (_stamp, frame_id) = r.read_header()?;
        let child_frame_id = r.read_string()?;

        let position = r.read_vec3()?;
        let orientation = r.read_quaternion()?;
        let _pose_covariance = r.read_f64_array::<36>()?;

        let twist_linear = r.read_vec3()?;
        let twist_angular = r.read_vec3()?;
        let _twist_covariance = r.read_f64_array::<36>()?;

        Ok(MissionEvent::OdometryUpdate {
            robot_id: "robot_1".to_string(),
            timestamp,
            data: Odometry {
                frame_id,
                child_frame_id,
                pose: Pose {
                    x: position[0],
                    y: position[1],
                    z: position[2],
                    qx: orientation[0],
                    qy: orientation[1],
                    qz: orientation[2],
                    qw: orientation[3],
                },
                twist_linear,
                twist_angular,
            },
        })
    }

    /// Real `geometry_msgs/msg/PoseWithCovarianceStamped` CDR decoding:
    /// `{Header header; PoseWithCovariance pose;}`. `confidence` is derived
    /// from the real positional covariance (the diagonal x/y/z variance
    /// terms, `covariance[0]`, `covariance[7]`, `covariance[14]` in the
    /// standard row-major 6x6 layout) rather than the previous hardcoded
    /// `0.95` -- higher real positional uncertainty means lower confidence.
    fn parse_pose_message(
        &self,
        _topic: &RosTopic,
        msg: &RosMessage,
        timestamp: DateTime<Utc>,
    ) -> AnyResult<MissionEvent> {
        let mut r = CdrReader::new(&msg.data)?;
        let (_stamp, _frame_id) = r.read_header()?;
        let position = r.read_vec3()?;
        let orientation = r.read_quaternion()?;
        let covariance = r.read_f64_array::<36>()?;

        let position_variance_trace = covariance[0] + covariance[7] + covariance[14];
        let confidence = (1.0 / (1.0 + position_variance_trace)).clamp(0.0, 1.0) as f32;

        Ok(MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp,
            pose: Pose {
                x: position[0],
                y: position[1],
                z: position[2],
                qx: orientation[0],
                qy: orientation[1],
                qz: orientation[2],
                qw: orientation[3],
            },
            confidence: Some(confidence),
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
