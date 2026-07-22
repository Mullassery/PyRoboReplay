//! Perception Failure Analysis
//!
//! Detects and analyzes perception anomalies:
//! - Missed detections (object visible but not detected)
//! - Late detections (detection lag)
//! - False positives
//! - Confidence inconsistencies

use crate::perception::object_detection::{DetectedObject, ObjectClass};
use std::collections::HashMap;

/// Types of perception failures
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FailureType {
    MissedDetection,
    LateDetection,
    FalsePositive,
    ConfidenceDrop,
    TrackingLoss,
    InconsistentDetection,
}

impl std::fmt::Display for FailureType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FailureType::MissedDetection => write!(f, "Missed Detection"),
            FailureType::LateDetection => write!(f, "Late Detection"),
            FailureType::FalsePositive => write!(f, "False Positive"),
            FailureType::ConfidenceDrop => write!(f, "Confidence Drop"),
            FailureType::TrackingLoss => write!(f, "Tracking Loss"),
            FailureType::InconsistentDetection => write!(f, "Inconsistent Detection"),
        }
    }
}

/// Detected perception failure
#[derive(Debug, Clone)]
pub struct PerceptionFailure {
    /// Type of failure
    pub failure_type: FailureType,

    /// Object class involved
    pub object_class: ObjectClass,

    /// Timestamp when failure occurred
    pub timestamp_sec: f32,

    /// Frame number
    pub frame_index: usize,

    /// Confidence before failure
    pub confidence_before: Option<f32>,

    /// Confidence after failure
    pub confidence_after: Option<f32>,

    /// Detection latency (seconds)
    pub detection_latency_sec: Option<f32>,

    /// How critical is this failure
    pub severity: f32, // 0.0-1.0

    /// Description
    pub description: String,

    /// Environmental factors that may have contributed
    pub contributing_factors: Vec<String>,
}

/// Analyzer for perception failures
pub struct PerceptionAnalyzer;

impl PerceptionAnalyzer {
    /// Detect missed detections by comparing consecutive frames
    pub fn detect_missed_detections(
        previous_frame_objects: &[DetectedObject],
        current_frame_objects: &[DetectedObject],
        timestamp_sec: f32,
        frame_index: usize,
    ) -> Vec<PerceptionFailure> {
        let mut failures = Vec::new();

        for prev_obj in previous_frame_objects {
            // Check if this object is still in current frame
            let found = current_frame_objects.iter().any(|obj| {
                obj.class == prev_obj.class
                    && Self::is_same_object(prev_obj, obj)
            });

            if !found && prev_obj.confidence > 0.7 {
                failures.push(PerceptionFailure {
                    failure_type: FailureType::MissedDetection,
                    object_class: prev_obj.class,
                    timestamp_sec,
                    frame_index,
                    confidence_before: Some(prev_obj.confidence),
                    confidence_after: Some(0.0),
                    detection_latency_sec: None,
                    severity: prev_obj.confidence, // Higher confidence = worse miss
                    description: format!(
                        "Object of class {} disappeared after detection",
                        prev_obj.class
                    ),
                    contributing_factors: vec!["Tracking loss".to_string()],
                });
            }
        }

        failures
    }

    /// Detect confidence drops that may indicate perception issues
    pub fn detect_confidence_drops(
        previous_objects: &[DetectedObject],
        current_objects: &[DetectedObject],
        timestamp_sec: f32,
        frame_index: usize,
    ) -> Vec<PerceptionFailure> {
        let mut failures = Vec::new();

        for prev_obj in previous_objects {
            for curr_obj in current_objects {
                if Self::is_same_object(prev_obj, curr_obj) && curr_obj.confidence < prev_obj.confidence {
                    let drop = prev_obj.confidence - curr_obj.confidence;

                    if drop > 0.2 {
                        // Significant drop
                        failures.push(PerceptionFailure {
                            failure_type: FailureType::ConfidenceDrop,
                            object_class: prev_obj.class,
                            timestamp_sec,
                            frame_index,
                            confidence_before: Some(prev_obj.confidence),
                            confidence_after: Some(curr_obj.confidence),
                            detection_latency_sec: None,
                            severity: drop,
                            description: format!(
                                "Confidence dropped from {:.0}% to {:.0}% for {}",
                                prev_obj.confidence * 100.0,
                                curr_obj.confidence * 100.0,
                                prev_obj.class
                            ),
                            contributing_factors: vec![
                                "Object visibility reduced".to_string(),
                                "Detection model uncertainty".to_string(),
                            ],
                        });
                    }
                }
            }
        }

        failures
    }

    /// Detect false positives (detections that don't match ground truth)
    pub fn detect_false_positives(
        detected_objects: &[DetectedObject],
        ground_truth_objects: &[DetectedObject],
        timestamp_sec: f32,
        frame_index: usize,
    ) -> Vec<PerceptionFailure> {
        let mut failures = Vec::new();

        for detection in detected_objects {
            // Check if this detection has a matching ground truth
            let has_match = ground_truth_objects.iter().any(|gt_obj| {
                Self::is_same_object(detection, gt_obj)
            });

            if !has_match && detection.confidence > 0.6 {
                failures.push(PerceptionFailure {
                    failure_type: FailureType::FalsePositive,
                    object_class: detection.class,
                    timestamp_sec,
                    frame_index,
                    confidence_before: None,
                    confidence_after: Some(detection.confidence),
                    detection_latency_sec: None,
                    severity: detection.confidence, // Confident false positives are worse
                    description: format!(
                        "False positive detection of {} with {:.0}% confidence",
                        detection.class,
                        detection.confidence * 100.0
                    ),
                    contributing_factors: vec!["No matching object in scene".to_string()],
                });
            }
        }

        failures
    }

    /// Check if two objects are likely the same (by position proximity)
    fn is_same_object(obj1: &DetectedObject, obj2: &DetectedObject) -> bool {
        if obj1.class != obj2.class {
            return false;
        }

        // If both have 3D positions, check proximity
        if let (Some(pos1), Some(pos2)) = (obj1.position_3d, obj2.position_3d) {
            let distance = ((pos1.0 - pos2.0).powi(2) + (pos1.1 - pos2.1).powi(2)).sqrt();
            distance < 1.0 // Same object within 1 meter
        } else {
            // Fall back to bounding box proximity
            let x_diff = (obj1.bbox.x - obj2.bbox.x).abs();
            let y_diff = (obj1.bbox.y - obj2.bbox.y).abs();
            x_diff < 50.0 && y_diff < 50.0 // Similar bounding box
        }
    }

    /// Compute failure statistics
    pub fn compute_failure_stats(failures: &[PerceptionFailure]) -> FailureStatistics {
        let mut by_type: HashMap<FailureType, usize> = HashMap::new();
        let mut by_class: HashMap<ObjectClass, usize> = HashMap::new();
        let mut total_severity = 0.0;
        let mut high_severity_failures = 0;

        for failure in failures {
            *by_type.entry(failure.failure_type.clone()).or_insert(0) += 1;
            *by_class.entry(failure.object_class).or_insert(0) += 1;
            total_severity += failure.severity;

            if failure.severity > 0.8 {
                high_severity_failures += 1;
            }
        }

        let avg_severity = if failures.is_empty() {
            0.0
        } else {
            total_severity / failures.len() as f32
        };

        FailureStatistics {
            total_failures: failures.len(),
            failures_by_type: by_type,
            failures_by_class: by_class,
            avg_severity,
            high_severity_count: high_severity_failures,
        }
    }
}

/// Statistics about perception failures
#[derive(Debug, Clone)]
pub struct FailureStatistics {
    /// Total failures detected
    pub total_failures: usize,

    /// Failures by type
    pub failures_by_type: HashMap<FailureType, usize>,

    /// Failures by object class
    pub failures_by_class: HashMap<ObjectClass, usize>,

    /// Average severity
    pub avg_severity: f32,

    /// High-severity failures
    pub high_severity_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perception::object_detection::BoundingBox;
    use std::collections::HashMap;

    fn create_test_detection(
        class: ObjectClass,
        confidence: f32,
        position: (f32, f32, f32),
    ) -> DetectedObject {
        DetectedObject {
            id: 1,
            class,
            confidence,
            bbox: BoundingBox {
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 100.0,
            },
            distance_m: Some(2.5),
            velocity_ms: None,
            position_3d: Some(position),
            trajectory_id: None,
            attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_missed_detection() {
        let prev_objects = vec![create_test_detection(ObjectClass::Person, 0.95, (2.0, 0.0, 0.0))];
        let curr_objects = vec![];

        let failures =
            PerceptionAnalyzer::detect_missed_detections(&prev_objects, &curr_objects, 1.0, 1);

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].failure_type, FailureType::MissedDetection);
    }

    #[test]
    fn test_confidence_drop() {
        let prev_objects = vec![create_test_detection(ObjectClass::Vehicle, 0.95, (5.0, 0.0, 0.0))];
        let curr_objects =
            vec![create_test_detection(ObjectClass::Vehicle, 0.70, (5.1, 0.0, 0.0))];

        let failures =
            PerceptionAnalyzer::detect_confidence_drops(&prev_objects, &curr_objects, 1.0, 1);

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].failure_type, FailureType::ConfidenceDrop);
    }

    #[test]
    fn test_failure_statistics() {
        let failures = vec![
            PerceptionFailure {
                failure_type: FailureType::MissedDetection,
                object_class: ObjectClass::Person,
                timestamp_sec: 1.0,
                frame_index: 1,
                confidence_before: Some(0.95),
                confidence_after: Some(0.0),
                detection_latency_sec: None,
                severity: 0.95,
                description: "Test".to_string(),
                contributing_factors: vec![],
            },
            PerceptionFailure {
                failure_type: FailureType::FalsePositive,
                object_class: ObjectClass::Vehicle,
                timestamp_sec: 2.0,
                frame_index: 2,
                confidence_before: None,
                confidence_after: Some(0.7),
                detection_latency_sec: None,
                severity: 0.7,
                description: "Test".to_string(),
                contributing_factors: vec![],
            },
        ];

        let stats = PerceptionAnalyzer::compute_failure_stats(&failures);

        assert_eq!(stats.total_failures, 2);
        assert!(stats.avg_severity > 0.7);
    }
}
