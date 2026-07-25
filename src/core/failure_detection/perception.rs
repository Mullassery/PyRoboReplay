/// Perception Failure Detector
///
/// Detects:
/// - Sensor dropout (absence of frames for extended period)
/// - Camera frame loss (missing frames in video stream)
/// - LiDAR interruption (point cloud publishing stops)
/// - Synchronization issues (sensor data misaligned in time)
/// - Low confidence detections

use super::{DetectedFailure, FailureDetector, FailureDomain, FailureSeverity};
use crate::core::timeline_correlation::NormalizedEvent;
use chrono::Duration;

pub struct PerceptionFailureDetector;

impl PerceptionFailureDetector {
    /// Detect sensor dropout: no frames for extended period
    fn detect_sensor_dropout(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();
        const MAX_GAP_MS: i64 = 5000; // 5 second gap is suspicious

        // Check LiDAR
        let lidar_times: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let crate::core::event::MissionEvent::LidarScan { timestamp, .. } = &e.event {
                    Some((*timestamp, e.id.clone()))
                } else {
                    None
                }
            })
            .collect();

        for i in 1..lidar_times.len() {
            let gap_ms = (lidar_times[i].0 - lidar_times[i - 1].0).num_milliseconds();
            if gap_ms > MAX_GAP_MS {
                failures.push(
                    DetectedFailure::new(
                        "sensor_dropout",
                        FailureDomain::Perception,
                        lidar_times[i].0,
                        0.95,
                        FailureSeverity::High,
                        format!("LiDAR dropout: {}ms gap", gap_ms),
                    )
                    .with_event_ids(vec![lidar_times[i].1.clone()]),
                );
            }
        }

        // Check Camera
        let camera_times: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let crate::core::event::MissionEvent::CameraFrame { timestamp, .. } = &e.event {
                    Some((*timestamp, e.id.clone()))
                } else {
                    None
                }
            })
            .collect();

        for i in 1..camera_times.len() {
            let gap_ms = (camera_times[i].0 - camera_times[i - 1].0).num_milliseconds();
            if gap_ms > MAX_GAP_MS {
                failures.push(
                    DetectedFailure::new(
                        "sensor_dropout",
                        FailureDomain::Perception,
                        camera_times[i].0,
                        0.95,
                        FailureSeverity::High,
                        format!("Camera dropout: {}ms gap", gap_ms),
                    )
                    .with_event_ids(vec![camera_times[i].1.clone()]),
                );
            }
        }

        failures
    }

    /// Detect synchronization issues between sensors
    fn detect_sync_mismatch(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();
        const MAX_SKEW_MS: i64 = 100; // 100ms skew is suspicious

        let mut lidar_times: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let crate::core::event::MissionEvent::LidarScan { timestamp, .. } = &e.event {
                    Some((*timestamp, e.id.clone()))
                } else {
                    None
                }
            })
            .collect();

        let mut camera_times: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let crate::core::event::MissionEvent::CameraFrame { timestamp, .. } = &e.event {
                    Some((*timestamp, e.id.clone()))
                } else {
                    None
                }
            })
            .collect();

        lidar_times.sort_by_key(|t| t.0);
        camera_times.sort_by_key(|t| t.0);

        let mut camera_idx = 0;
        for (lidar_time, lidar_id) in &lidar_times {
            while camera_idx < camera_times.len()
                && (camera_times[camera_idx].0 - *lidar_time).num_milliseconds() < -MAX_SKEW_MS
            {
                camera_idx += 1;
            }

            if camera_idx >= camera_times.len() {
                break;
            }

            let skew_ms = (camera_times[camera_idx].0 - *lidar_time)
                .num_milliseconds()
                .abs();

            if skew_ms > MAX_SKEW_MS {
                failures.push(
                    DetectedFailure::new(
                        "sync_mismatch",
                        FailureDomain::Perception,
                        *lidar_time,
                        0.70,
                        FailureSeverity::Medium,
                        format!("LiDAR-Camera sync skew: {}ms", skew_ms),
                    )
                    .with_event_ids(vec![lidar_id.clone()]),
                );
            }
        }

        failures
    }

    /// Detect low confidence detections
    fn detect_low_confidence(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::ObstacleDetected {
                timestamp,
                confidence: Some(conf),
                ..
            } = &event.event
            {
                if *conf < 0.5 {
                    failures.push(
                        DetectedFailure::new(
                            "low_confidence_detection",
                            FailureDomain::Perception,
                            *timestamp,
                            0.80,
                            FailureSeverity::Low,
                            format!("Low confidence obstacle detection: {:.0}%", conf * 100.0),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }
}

impl FailureDetector for PerceptionFailureDetector {
    fn detect(&self, events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut all_failures = Vec::new();

        all_failures.extend(Self::detect_sensor_dropout(events));
        all_failures.extend(Self::detect_sync_mismatch(events));
        all_failures.extend(Self::detect_low_confidence(events));

        all_failures
    }

    fn domain(&self) -> FailureDomain {
        FailureDomain::Perception
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = PerceptionFailureDetector;
        assert_eq!(detector.domain(), FailureDomain::Perception);
    }
}
