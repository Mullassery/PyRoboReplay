pub mod configuration;
pub mod linux_log;
pub mod metrics;
pub mod ros2;

use crate::core::event::MissionRecord;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Failed to read file: {0}")]
    FileReadError(String),
    #[error("Failed to parse data: {0}")]
    ParseError(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Timestamp error: {0}")]
    TimestampError(String),
    #[error("IO error: {0}")]
    IoError(String),
}

pub trait MissionAdapter {
    fn read(&self, path: &str) -> Result<MissionRecord, AdapterError>;
    fn adapter_name(&self) -> &str;
}

pub use configuration::ConfigurationAdapter;
pub use linux_log::LinuxLogAdapter;
pub use metrics::MetricsAdapter;
