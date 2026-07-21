use super::{AdapterError, MissionAdapter};
use crate::core::event::MissionRecord;
use chrono::Utc;
use std::path::Path;

pub struct Ros2Adapter;

impl Ros2Adapter {
    pub fn new() -> Self {
        Self
    }

    /// Parse ROS 2 bag file
    /// This is a stub implementation. Full implementation will:
    /// - Use rosbag2 libraries to read .db3 files
    /// - Extract TF2 transforms, sensor data, navigation topics
    /// - Normalize to universal event model
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

        // Stub: Return empty mission for now
        // Full implementation will parse rosbag2 format
        let mission = MissionRecord::new(format!("ros2_mission_{}", Utc::now().timestamp()));
        Ok(mission)
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
