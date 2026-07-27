//! Temporal synchronization engine for Phase 14
//!
//! Handles time model detection, clock offset computation, and clock skew
//! correction across heterogeneous data sources with different time models:
//! - ROS: nanosecond absolute timestamps
//! - Linux: wall-clock timestamps (syslog with timezone)
//! - Video: frame numbers or embedded timestamps
//! - Annotations: operator event timeline
//! - Sensors: device-specific clocks

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use chrono::{DateTime, Utc, Duration};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeModel {
    /// ROS nanosecond absolute timestamps (Reference epoch: T0)
    RosNanoseconds,
    /// Wall-clock absolute timestamps (syslog style with timezone)
    WallClock,
    /// Video frame number (convert via FPS)
    FrameNumber { fps: u32 },
    /// Operator event sequence (incrementing)
    OperatorSequence,
    /// Device-specific clock (unknown epoch)
    DeviceClock,
}

impl std::fmt::Display for TimeModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeModel::RosNanoseconds => write!(f, "ROS Nanoseconds"),
            TimeModel::WallClock => write!(f, "Wall-Clock"),
            TimeModel::FrameNumber { fps } => write!(f, "Frame Number ({}fps)", fps),
            TimeModel::OperatorSequence => write!(f, "Operator Sequence"),
            TimeModel::DeviceClock => write!(f, "Device Clock"),
        }
    }
}

/// Offset from local time model to unified timeline (ROS nanoseconds)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClockOffset {
    /// Offset in nanoseconds (add to local time to get unified time)
    pub offset_ns: i64,
    /// Confidence in this offset (0-1, based on evidence strength)
    pub confidence: f32,
    /// Method used to compute this offset
    pub method: OffsetMethod,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OffsetMethod {
    /// Direct ROS bag time (reference)
    Reference,
    /// Computed from syslog timestamp correlation
    SyslogCorrelation,
    /// Computed from video sync point (beep, flash, etc.)
    VideoSyncPoint,
    /// Computed from motion correlation (robot movement timestamps)
    MotionCorrelation,
    /// Estimated from NTP drift in logs
    NtpDrift,
    /// Manual override
    ManualCalibration,
}

impl std::fmt::Display for OffsetMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OffsetMethod::Reference => write!(f, "Reference (ROS Bag)"),
            OffsetMethod::SyslogCorrelation => write!(f, "Syslog Correlation"),
            OffsetMethod::VideoSyncPoint => write!(f, "Video Sync Point"),
            OffsetMethod::MotionCorrelation => write!(f, "Motion Correlation"),
            OffsetMethod::NtpDrift => write!(f, "NTP Drift Estimation"),
            OffsetMethod::ManualCalibration => write!(f, "Manual Calibration"),
        }
    }
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Failed to detect time model: {0}")]
    TimeModelDetectionFailed(String),

    #[error("Clock offset computation failed: {0}")]
    OffsetComputationFailed(String),

    #[error("Insufficient evidence for sync: {0}")]
    InsufficientEvidence(String),

    #[error("Inconsistent timestamps detected: {0}")]
    InconsistentTimestamps(String),
}

pub type SyncResult<T> = Result<T, SyncError>;

/// Report on temporal synchronization process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
    pub reference_epoch: i64,  // ROS time in ns
    pub global_start_time: i64,
    pub global_end_time: i64,
    pub source_offsets: BTreeMap<String, ClockOffset>,
    pub detected_ntp_issues: Vec<NtpIssue>,
    pub sync_quality: f32,  // 0-1, overall confidence
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtpIssue {
    pub timestamp: i64,
    pub drift_rate: f32,  // ns/s
    pub severity: IssueSeverity,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IssueSeverity {
    Minor,
    Moderate,
    Severe,
}

// ============================================================================
// Time Model Detection
// ============================================================================

pub struct TimeModelDetector;

impl TimeModelDetector {
    /// Detect time model from sample timestamps
    pub fn detect(sample_timestamps: &[i64]) -> SyncResult<TimeModel> {
        if sample_timestamps.is_empty() {
            return Err(SyncError::TimeModelDetectionFailed(
                "No timestamps provided".to_string(),
            ));
        }

        // If timestamps are large (10^18+), likely ROS nanoseconds
        if sample_timestamps.iter().all(|&t| t > 1e17 as i64) {
            return Ok(TimeModel::RosNanoseconds);
        }

        // If timestamps fit in 32-bit frame count range
        if sample_timestamps.iter().all(|&t| t < 1_000_000) {
            return Ok(TimeModel::FrameNumber { fps: 30 }); // Default assumption
        }

        // If timestamps are small integers (sequence), likely operator sequence
        if sample_timestamps.len() > 1 {
            let diffs: Vec<_> = sample_timestamps.windows(2)
                .map(|w| w[1] - w[0])
                .collect();

            if diffs.iter().all(|&d| d > 0 && d < 1000) {
                return Ok(TimeModel::OperatorSequence);
            }
        }

        Ok(TimeModel::DeviceClock)
    }

    /// Detect from wall-clock syslog entries
    pub fn detect_wall_clock(log_samples: &[String]) -> SyncResult<TimeModel> {
        // Check if any sample contains ISO8601 or RFC3339 timestamp
        for sample in log_samples {
            if sample.contains('T') && sample.contains('Z') {
                return Ok(TimeModel::WallClock);
            }
        }

        Err(SyncError::TimeModelDetectionFailed(
            "No wall-clock timestamps found in logs".to_string(),
        ))
    }
}

// ============================================================================
// Clock Offset Computation
// ============================================================================

pub struct OffsetComputer;

impl OffsetComputer {
    /// Compute offset from ROS reference (T0)
    pub fn compute_reference_offset() -> ClockOffset {
        ClockOffset {
            offset_ns: 0,
            confidence: 1.0,
            method: OffsetMethod::Reference,
        }
    }

    /// Compute offset from syslog timestamp to ROS time
    pub fn from_syslog(
        ros_timestamp_ns: i64,
        syslog_timestamp: DateTime<Utc>,
    ) -> ClockOffset {
        let syslog_ns = syslog_timestamp.timestamp_nanos_opt()
            .unwrap_or(0);

        let offset_ns = ros_timestamp_ns - syslog_ns;

        ClockOffset {
            offset_ns,
            confidence: 0.85,  // syslog timestamps are reliable but may have clock skew
            method: OffsetMethod::SyslogCorrelation,
        }
    }

    /// Compute offset from frame number and FPS
    pub fn from_frame_number(
        ros_timestamp_ns: i64,
        frame_number: u32,
        fps: f32,
    ) -> ClockOffset {
        let frame_time_ns = (frame_number as f64 / fps as f64 * 1e9) as i64;
        let offset_ns = ros_timestamp_ns - frame_time_ns;

        ClockOffset {
            offset_ns,
            confidence: 0.75,  // Video frame rates can drift
            method: OffsetMethod::VideoSyncPoint,
        }
    }

    /// Detect NTP drift from consecutive syslog entries
    pub fn detect_ntp_drift(entries: &[(DateTime<Utc>, String)]) -> Vec<NtpIssue> {
        let mut issues = Vec::new();

        if entries.len() < 2 {
            return issues;
        }

        for window in entries.windows(2) {
            let (t1, _) = &window[0];
            let (t2, _) = &window[1];

            let time_diff = (*t2 - *t1).num_seconds() as f32;
            if time_diff > 0.0 {
                // Calculate expected vs actual timestamps
                // Simplified: just check for monotonicity
                if *t2 < *t1 {
                    issues.push(NtpIssue {
                        timestamp: t1.timestamp_nanos_opt().unwrap_or(0),
                        drift_rate: -1000.0,  // Negative drift
                        severity: IssueSeverity::Severe,
                        description: "Clock went backwards (NTP adjustment?)".to_string(),
                    });
                }
            }
        }

        issues
    }
}

// ============================================================================
// Temporal Sync Engine
// ============================================================================

#[derive(Clone)]
pub struct TemporalSyncEngine {
    reference_time: i64,
    offsets: BTreeMap<String, ClockOffset>,
    report: Option<SyncReport>,
}

impl TemporalSyncEngine {
    pub fn new(reference_time: i64) -> Self {
        let mut offsets = BTreeMap::new();
        offsets.insert(
            "ros_bag".to_string(),
            OffsetComputer::compute_reference_offset(),
        );

        TemporalSyncEngine {
            reference_time,
            offsets,
            report: None,
        }
    }

    /// Register a time model with offset
    pub fn register_source(&mut self, source_name: String, offset: ClockOffset) {
        self.offsets.insert(source_name, offset);
    }

    /// Convert local timestamp to unified timeline
    pub fn to_unified_time(&self, source_name: &str, local_time: i64) -> SyncResult<i64> {
        let offset = self.offsets.get(source_name)
            .ok_or_else(|| SyncError::OffsetComputationFailed(
                format!("Unknown source: {}", source_name),
            ))?;

        Ok(local_time + offset.offset_ns)
    }

    /// Convert unified time back to source-local time
    pub fn from_unified_time(&self, source_name: &str, unified_time: i64) -> SyncResult<i64> {
        let offset = self.offsets.get(source_name)
            .ok_or_else(|| SyncError::OffsetComputationFailed(
                format!("Unknown source: {}", source_name),
            ))?;

        Ok(unified_time - offset.offset_ns)
    }

    /// Generate synchronization report
    pub fn report(&mut self, start_time: i64, end_time: i64) -> SyncReport {
        let confidence_scores: Vec<f32> = self.offsets.values()
            .map(|o| o.confidence)
            .collect();

        let sync_quality = if confidence_scores.is_empty() {
            0.0
        } else {
            confidence_scores.iter().sum::<f32>() / confidence_scores.len() as f32
        };

        let mut issues = Vec::new();

        // Flag sources with low confidence
        for (source, offset) in &self.offsets {
            if offset.confidence < 0.7 {
                issues.push(format!(
                    "{}: Low confidence ({:.0}%) - {}",
                    source, offset.confidence * 100.0, offset.method
                ));
            }
        }

        let report = SyncReport {
            reference_epoch: self.reference_time,
            global_start_time: start_time,
            global_end_time: end_time,
            source_offsets: self.offsets.clone(),
            detected_ntp_issues: Vec::new(),
            sync_quality,
            issues,
        };

        self.report = Some(report.clone());
        report
    }

    /// Get alignment statistics
    pub fn alignment_stats(&self) -> AlignmentStats {
        AlignmentStats {
            num_sources: self.offsets.len(),
            average_confidence: self.offsets.values()
                .map(|o| o.confidence)
                .sum::<f32>() / self.offsets.len().max(1) as f32,
            offset_range: self.offsets.values()
                .map(|o| o.offset_ns)
                .fold((i64::MAX, i64::MIN), |(min, max), val| {
                    (min.min(val), max.max(val))
                }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentStats {
    pub num_sources: usize,
    pub average_confidence: f32,
    pub offset_range: (i64, i64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_model_detection_ros_nanoseconds() {
        // ROS timestamps are nanoseconds since epoch (typically 1.6e18 for 2021+)
        let samples = vec![1600000000000000000i64, 1600000000000000100, 1600000000000000200];
        let model = TimeModelDetector::detect(&samples).unwrap();
        assert_eq!(model, TimeModel::RosNanoseconds);
    }

    #[test]
    fn test_time_model_detection_frame_number() {
        let samples = vec![0i64, 1, 2, 3, 4];
        let model = TimeModelDetector::detect(&samples).unwrap();
        if let TimeModel::FrameNumber { fps } = model {
            assert_eq!(fps, 30);
        } else {
            panic!("Expected FrameNumber model");
        }
    }

    #[test]
    fn test_clock_offset_reference() {
        let offset = OffsetComputer::compute_reference_offset();
        assert_eq!(offset.offset_ns, 0);
        assert_eq!(offset.confidence, 1.0);
    }

    #[test]
    fn test_temporal_sync_engine_creation() {
        let engine = TemporalSyncEngine::new(1000000000);
        assert!(engine.offsets.contains_key("ros_bag"));
    }

    #[test]
    fn test_temporal_sync_engine_register_source() {
        let mut engine = TemporalSyncEngine::new(1000000000);
        let offset = ClockOffset {
            offset_ns: 1000,
            confidence: 0.95,
            method: OffsetMethod::SyslogCorrelation,
        };
        engine.register_source("syslog".to_string(), offset);
        assert!(engine.offsets.contains_key("syslog"));
    }

    #[test]
    fn test_to_unified_time() {
        let mut engine = TemporalSyncEngine::new(1000000000);
        let offset = ClockOffset {
            offset_ns: 5000,
            confidence: 0.95,
            method: OffsetMethod::ManualCalibration,
        };
        engine.register_source("syslog".to_string(), offset);

        let unified = engine.to_unified_time("syslog", 10000).unwrap();
        assert_eq!(unified, 15000);
    }

    #[test]
    fn test_from_unified_time() {
        let mut engine = TemporalSyncEngine::new(1000000000);
        let offset = ClockOffset {
            offset_ns: 5000,
            confidence: 0.95,
            method: OffsetMethod::ManualCalibration,
        };
        engine.register_source("syslog".to_string(), offset);

        let local = engine.from_unified_time("syslog", 15000).unwrap();
        assert_eq!(local, 10000);
    }

    #[test]
    fn test_sync_report_generation() {
        let mut engine = TemporalSyncEngine::new(1000000000);
        let report = engine.report(1000000000, 2000000000);
        assert_eq!(report.reference_epoch, 1000000000);
        assert!(report.sync_quality > 0.0);
    }
}
