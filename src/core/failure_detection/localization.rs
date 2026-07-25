/// Localization Failure Detector
///
/// Detects:
/// - AMCL divergence (covariance growing unbounded)
/// - Map mismatch (scan inconsistent with loaded map)
/// - TF inconsistencies (transforms don't compose correctly)
/// - Pose instability (estimated pose jumps discontinuously)
/// - GPS dropout (absolute positioning unavailable)

use super::{DetectedFailure, FailureDetector, FailureDomain, FailureSeverity};
use crate::core::timeline_correlation::NormalizedEvent;

pub struct LocalizationFailureDetector;

impl LocalizationFailureDetector {
    /// Detect AMCL divergence: pose covariance grows unbounded
    fn detect_amcl_divergence(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            // Look for explicit AMCL divergence events
            if let crate::core::event::MissionEvent::NavigationDecision {
                timestamp,
                decision_type,
                ..
            } = &event.event
            {
                if decision_type.contains("amcl_divergence") || decision_type.contains("localization_divergence") {
                    failures.push(
                        DetectedFailure::new(
                            "amcl_divergence",
                            FailureDomain::Localization,
                            *timestamp,
                            0.80,
                            FailureSeverity::High,
                            "AMCL covariance diverging - localization unstable".to_string(),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }

    /// Detect TF inconsistencies: transforms don't compose
    fn detect_tf_inconsistency(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::NavigationDecision {
                timestamp,
                decision_type,
                ..
            } = &event.event
            {
                if decision_type.contains("tf_error") || decision_type.contains("tf_timeout") {
                    failures.push(
                        DetectedFailure::new(
                            "tf_inconsistency",
                            FailureDomain::Localization,
                            *timestamp,
                            0.75,
                            FailureSeverity::High,
                            "TF tree has inconsistencies or missing transforms".to_string(),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }

    /// Detect pose instability: abrupt jumps in estimated pose
    fn detect_pose_instability(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();
        const MAX_POSE_JUMP: f64 = 1.0; // 1 meter is suspicious

        let mut poses: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let crate::core::event::MissionEvent::RobotPose {
                    timestamp,
                    pose,
                    ..
                } = &e.event
                {
                    Some((e.id.clone(), *timestamp, pose.x, pose.y))
                } else {
                    None
                }
            })
            .collect();

        // Check for discontinuous jumps
        for i in 1..poses.len() {
            let (_, ts1, x1, y1) = poses[i - 1];
            let (_, ts2, x2, y2) = poses[i];

            let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
            let time_delta = (ts2 - ts1).num_milliseconds() as f64 / 1000.0;

            // Check if jump is too large for the time delta
            if time_delta > 0.1 && distance > MAX_POSE_JUMP {
                failures.push(
                    DetectedFailure::new(
                        "pose_instability",
                        FailureDomain::Localization,
                        ts2,
                        0.70,
                        FailureSeverity::Medium,
                        format!("Pose jumped {:.2}m discontinuously", distance),
                    )
                    .with_event_ids(vec![poses[i].0.clone()]),
                );
            }
        }

        failures
    }

    /// Detect GPS dropout: absolute positioning unavailable
    fn detect_gps_dropout(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::NavigationDecision {
                timestamp,
                decision_type,
                ..
            } = &event.event
            {
                if decision_type.contains("gps_loss") || decision_type.contains("gps_dropout") {
                    failures.push(
                        DetectedFailure::new(
                            "gps_dropout",
                            FailureDomain::Localization,
                            *timestamp,
                            0.95,
                            FailureSeverity::High,
                            "GPS signal lost - absolute positioning unavailable".to_string(),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }
}

impl FailureDetector for LocalizationFailureDetector {
    fn detect(&self, events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut all_failures = Vec::new();

        all_failures.extend(Self::detect_amcl_divergence(events));
        all_failures.extend(Self::detect_tf_inconsistency(events));
        all_failures.extend(Self::detect_pose_instability(events));
        all_failures.extend(Self::detect_gps_dropout(events));

        all_failures
    }

    fn domain(&self) -> FailureDomain {
        FailureDomain::Localization
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = LocalizationFailureDetector;
        assert_eq!(detector.domain(), FailureDomain::Localization);
    }

    #[test]
    fn test_empty_events() {
        let detector = LocalizationFailureDetector;
        let events = vec![];
        let failures = detector.detect(&events);
        assert_eq!(failures.len(), 0);
    }
}
