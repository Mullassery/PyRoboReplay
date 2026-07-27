//! Multi-modal data source adapters for Phase 14 temporal fusion
//!
//! Supports ingestion of diverse data sources:
//! - ROS 2 bags (.mcap, .rosbag2)
//! - Linux system logs (syslog, dmesg, journalctl)
//! - Nav2 diagnostic exports
//! - Video files (MP4, MKV, image sequences)
//! - Point clouds (PCD, PLY, LAS)
//! - Annotations (CSV, JSON, YAML)
//! - Sensor calibration files
//! - Environment maps
//! - Robot models (URDF, CAD specs)

use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataSourceType {
    RosBag,
    LinuxLogs,
    Nav2Export,
    Video,
    PointCloud,
    Annotation,
    SensorCalibration,
    EnvironmentMap,
    RobotModel,
}

impl std::fmt::Display for DataSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSourceType::RosBag => write!(f, "ROS Bag"),
            DataSourceType::LinuxLogs => write!(f, "Linux Logs"),
            DataSourceType::Nav2Export => write!(f, "Nav2 Export"),
            DataSourceType::Video => write!(f, "Video"),
            DataSourceType::PointCloud => write!(f, "Point Cloud"),
            DataSourceType::Annotation => write!(f, "Annotation"),
            DataSourceType::SensorCalibration => write!(f, "Sensor Calibration"),
            DataSourceType::EnvironmentMap => write!(f, "Environment Map"),
            DataSourceType::RobotModel => write!(f, "Robot Model"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub msg_type: String,
    pub message_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceData {
    pub source_type: DataSourceType,
    pub topics: Vec<Topic>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<f64>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Failed to load {0}: {1}")]
    LoadFailed(String, String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Missing data: {0}")]
    MissingData(String),
}

pub type AdapterResult<T> = Result<T, AdapterError>;

/// Core trait for all data source adapters
pub trait DataSource: Send + Sync {
    /// Load data from source
    fn load(&self, path: &Path) -> AdapterResult<SourceData>;

    /// Get available topics/channels in source
    fn available_topics(&self) -> AdapterResult<Vec<Topic>>;

    /// Extract time-series stream for a specific topic
    fn extract_stream(&self, topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>>;

    /// Get metadata
    fn metadata(&self) -> std::collections::HashMap<String, String>;

    /// Data source type identifier
    fn source_type(&self) -> DataSourceType;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,  // ROS time in nanoseconds (unified)
    pub value: Vec<u8>,  // Serialized message
    pub topic: String,
}

// ============================================================================
// ROS 2 Bag Adapter
// ============================================================================

pub struct RosBagAdapter {
    path: Option<std::path::PathBuf>,
    topics: Vec<Topic>,
}

impl RosBagAdapter {
    pub fn new() -> Self {
        RosBagAdapter {
            path: None,
            topics: Vec::new(),
        }
    }
}

impl Default for RosBagAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for RosBagAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                "ROS Bag".to_string(),
                format!("File not found: {:?}", path),
            ));
        }

        // TODO: Parse mcap/rosbag2 files
        // Placeholder: return basic structure
        Ok(SourceData {
            source_type: DataSourceType::RosBag,
            topics: self.topics.clone(),
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(self.topics.clone())
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::RosBag
    }
}

// ============================================================================
// Linux Logs Adapter
// ============================================================================

pub struct LinuxLogsAdapter {
    log_type: LogType,
}

#[derive(Debug, Clone, Copy)]
pub enum LogType {
    Syslog,
    Dmesg,
    Journalctl,
}

impl LinuxLogsAdapter {
    pub fn new(log_type: LogType) -> Self {
        LinuxLogsAdapter { log_type }
    }
}

impl DataSource for LinuxLogsAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                format!("{:?} Log", self.log_type),
                format!("File not found: {:?}", path),
            ));
        }

        // TODO: Parse syslog/dmesg with wall-clock timestamps
        Ok(SourceData {
            source_type: DataSourceType::LinuxLogs,
            topics: vec![],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![Topic {
            name: format!("{:?}", self.log_type),
            msg_type: "std_msgs/String".to_string(),
            message_count: 0,
        }])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("log_type".to_string(), format!("{:?}", self.log_type));
        m
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::LinuxLogs
    }
}

// ============================================================================
// Nav2 Export Adapter
// ============================================================================

pub struct Nav2ExportAdapter;

impl Nav2ExportAdapter {
    pub fn new() -> Self {
        Nav2ExportAdapter
    }
}

impl Default for Nav2ExportAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for Nav2ExportAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                "Nav2 Export".to_string(),
                format!("Directory not found: {:?}", path),
            ));
        }

        // TODO: Parse Nav2 costmaps, planner diagnostics, controller state
        Ok(SourceData {
            source_type: DataSourceType::Nav2Export,
            topics: vec![],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![
            Topic {
                name: "costmap".to_string(),
                msg_type: "nav2_msgs/Costmap".to_string(),
                message_count: 0,
            },
            Topic {
                name: "planner_diagnostics".to_string(),
                msg_type: "diagnostic_msgs/DiagnosticArray".to_string(),
                message_count: 0,
            },
        ])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::Nav2Export
    }
}

// ============================================================================
// Video Adapter
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum VideoFormat {
    MP4,
    MKV,
    MOV,
    ImageSequence,
}

pub struct VideoAdapter {
    format: VideoFormat,
    fps: f32,
}

impl VideoAdapter {
    pub fn new(format: VideoFormat, fps: f32) -> Self {
        VideoAdapter { format, fps }
    }
}

impl DataSource for VideoAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                format!("Video ({:?})", self.format),
                format!("File not found: {:?}", path),
            ));
        }

        // TODO: Extract video metadata (duration, resolution, frame count)
        Ok(SourceData {
            source_type: DataSourceType::Video,
            topics: vec![Topic {
                name: "video".to_string(),
                msg_type: "sensor_msgs/Image".to_string(),
                message_count: 0,
            }],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![Topic {
            name: "video".to_string(),
            msg_type: "sensor_msgs/Image".to_string(),
            message_count: 0,
        }])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("format".to_string(), format!("{:?}", self.format));
        m.insert("fps".to_string(), self.fps.to_string());
        m
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::Video
    }
}

// ============================================================================
// Point Cloud Adapter
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum PointCloudFormat {
    PCD,
    PLY,
    LAS,
}

pub struct PointCloudAdapter {
    format: PointCloudFormat,
}

impl PointCloudAdapter {
    pub fn new(format: PointCloudFormat) -> Self {
        PointCloudAdapter { format }
    }
}

impl DataSource for PointCloudAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                format!("Point Cloud ({:?})", self.format),
                format!("File not found: {:?}", path),
            ));
        }

        Ok(SourceData {
            source_type: DataSourceType::PointCloud,
            topics: vec![Topic {
                name: "pointcloud".to_string(),
                msg_type: "sensor_msgs/PointCloud2".to_string(),
                message_count: 0,
            }],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![Topic {
            name: "pointcloud".to_string(),
            msg_type: "sensor_msgs/PointCloud2".to_string(),
            message_count: 0,
        }])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("format".to_string(), format!("{:?}", self.format));
        m
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::PointCloud
    }
}

// ============================================================================
// Annotation Adapter
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum AnnotationFormat {
    CSV,
    JSON,
    YAML,
}

pub struct AnnotationAdapter {
    format: AnnotationFormat,
}

impl AnnotationAdapter {
    pub fn new(format: AnnotationFormat) -> Self {
        AnnotationAdapter { format }
    }
}

impl DataSource for AnnotationAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                format!("Annotation ({:?})", self.format),
                format!("File not found: {:?}", path),
            ));
        }

        Ok(SourceData {
            source_type: DataSourceType::Annotation,
            topics: vec![Topic {
                name: "annotations".to_string(),
                msg_type: "std_msgs/String".to_string(),
                message_count: 0,
            }],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![Topic {
            name: "annotations".to_string(),
            msg_type: "std_msgs/String".to_string(),
            message_count: 0,
        }])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("format".to_string(), format!("{:?}", self.format));
        m
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::Annotation
    }
}

// ============================================================================
// Sensor Calibration Adapter
// ============================================================================

pub struct SensorCalibrationAdapter;

impl SensorCalibrationAdapter {
    pub fn new() -> Self {
        SensorCalibrationAdapter
    }
}

impl Default for SensorCalibrationAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for SensorCalibrationAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                "Sensor Calibration".to_string(),
                format!("File not found: {:?}", path),
            ));
        }

        Ok(SourceData {
            source_type: DataSourceType::SensorCalibration,
            topics: vec![],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::SensorCalibration
    }
}

// ============================================================================
// Environment Map Adapter
// ============================================================================

pub struct EnvironmentMapAdapter;

impl EnvironmentMapAdapter {
    pub fn new() -> Self {
        EnvironmentMapAdapter
    }
}

impl Default for EnvironmentMapAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for EnvironmentMapAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                "Environment Map".to_string(),
                format!("File not found: {:?}", path),
            ));
        }

        Ok(SourceData {
            source_type: DataSourceType::EnvironmentMap,
            topics: vec![],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::EnvironmentMap
    }
}

// ============================================================================
// Robot Model Adapter
// ============================================================================

pub struct RobotModelAdapter;

impl RobotModelAdapter {
    pub fn new() -> Self {
        RobotModelAdapter
    }
}

impl Default for RobotModelAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for RobotModelAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                "Robot Model".to_string(),
                format!("File not found: {:?}", path),
            ));
        }

        Ok(SourceData {
            source_type: DataSourceType::RobotModel,
            topics: vec![],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::RobotModel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ros_bag_adapter_creation() {
        let adapter = RosBagAdapter::new();
        assert_eq!(adapter.source_type(), DataSourceType::RosBag);
    }

    #[test]
    fn test_linux_logs_adapter_creation() {
        let adapter = LinuxLogsAdapter::new(LogType::Syslog);
        assert_eq!(adapter.source_type(), DataSourceType::LinuxLogs);
    }

    #[test]
    fn test_video_adapter_metadata() {
        let adapter = VideoAdapter::new(VideoFormat::MP4, 30.0);
        let metadata = adapter.metadata();
        assert!(metadata.contains_key("fps"));
    }

    #[test]
    fn test_missing_file_error() {
        let adapter = RosBagAdapter::new();
        let result = adapter.load(Path::new("/nonexistent/file.bag"));
        assert!(result.is_err());
    }
}
