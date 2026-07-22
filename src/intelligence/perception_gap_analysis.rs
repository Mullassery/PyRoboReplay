//! Perception Gap Analysis
//!
//! Identifies what the robot failed to perceive.
//! Core question: What was visible to cameras but not to the robot?

use std::collections::HashMap;

/// Gap between what was actually there vs what robot perceived
#[derive(Debug, Clone)]
pub struct PerceptionGap {
    /// Timestamp
    pub timestamp_sec: f32,

    /// Type of gap
    pub gap_type: GapType,

    /// Entity that was missed/misunderstood
    pub entity_type: String,

    /// Confidence in this gap assessment
    pub confidence: f32,

    /// Why the gap occurred
    pub root_cause: String,

    /// Impact of this gap on robot behavior
    pub behavioral_impact: String,

    /// Evidence for this gap
    pub evidence: Vec<String>,

    /// Severity (0.0-1.0)
    pub severity: f32,
}

/// Types of perception gaps
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GapType {
    ObjectNotDetected,      // Object present but not detected
    ObjectMisclassified,    // Detected but wrong type
    DistanceEstimateError,  // Wrong distance estimate
    FieldOfViewGap,         // Object outside effective sensing region
    TemporalLag,            // Detection came too late
    SensorFailure,          // Sensor didn't work
    AmbiguousScene,         // Scene too complex to understand
}

impl std::fmt::Display for GapType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GapType::ObjectNotDetected => write!(f, "Object Not Detected"),
            GapType::ObjectMisclassified => write!(f, "Object Misclassified"),
            GapType::DistanceEstimateError => write!(f, "Distance Estimate Error"),
            GapType::FieldOfViewGap => write!(f, "Field of View Gap"),
            GapType::TemporalLag => write!(f, "Temporal Lag"),
            GapType::SensorFailure => write!(f, "Sensor Failure"),
            GapType::AmbiguousScene => write!(f, "Ambiguous Scene"),
        }
    }
}

/// Analysis of all perception gaps in a mission
#[derive(Debug, Clone)]
pub struct GapAnalysis {
    /// Mission ID
    pub mission_id: String,

    /// All detected gaps
    pub gaps: Vec<PerceptionGap>,

    /// Total time object was visible but undetected
    pub total_missed_time_sec: f32,

    /// Critical gaps (that likely caused failures)
    pub critical_gaps: Vec<PerceptionGap>,

    /// Gap statistics
    pub statistics: GapStatistics,
}

/// Statistics about perception gaps
#[derive(Debug, Clone)]
pub struct GapStatistics {
    /// Total gaps found
    pub total_gaps: usize,

    /// Gaps by type
    pub gaps_by_type: HashMap<String, usize>,

    /// Gaps by entity type
    pub gaps_by_entity: HashMap<String, usize>,

    /// Average severity of gaps
    pub avg_severity: f32,

    /// Percentage of mission time affected by gaps
    pub affected_time_percent: f32,

    /// Most common gap type
    pub most_common_gap: String,
}

/// Analyzer for perception gaps
pub struct PerceptionGapAnalyzer;

impl PerceptionGapAnalyzer {
    /// Analyze gaps between what robot should have perceived vs what it did
    pub fn analyze_perception_gaps(
        actual_scene: &crate::intelligence::scene_reconstruction::RetrospectiveScene,
        robot_sensors: &RobotSensorData,
        robot_behavior_history: &[RobotBehaviorSnapshot],
    ) -> PerceptionGap {
        let mut evidence = Vec::new();

        // Check for objects not in sensor range
        let undetected_objects: Vec<_> = actual_scene
            .detected_objects
            .iter()
            .filter(|obj| !obj.in_sensor_range)
            .collect();

        if !undetected_objects.is_empty() {
            evidence.push(format!(
                "{} objects outside sensor range",
                undetected_objects.len()
            ));
        }

        // Check for objects outside field of view
        let outside_fov: Vec<_> = actual_scene
            .detected_objects
            .iter()
            .filter(|obj| !obj.in_robot_fov)
            .collect();

        if !outside_fov.is_empty() {
            evidence.push(format!("{} objects outside field of view", outside_fov.len()));
        }

        // Determine gap type
        let gap_type = if undetected_objects.len() > 0 {
            GapType::ObjectNotDetected
        } else if outside_fov.len() > 0 {
            GapType::FieldOfViewGap
        } else {
            GapType::AmbiguousScene
        };

        // Assess behavioral impact
        let behavioral_impact = Self::assess_behavioral_impact(
            &gap_type,
            robot_sensors,
            robot_behavior_history,
        );

        // Compute severity
        let severity = Self::compute_severity(&gap_type, &evidence);

        let root_cause = Self::explain_root_cause(&gap_type);

        PerceptionGap {
            timestamp_sec: actual_scene.timestamp_sec,
            gap_type: gap_type.clone(),
            entity_type: actual_scene
                .detected_objects
                .first()
                .map(|o| o.entity_type.clone())
                .unwrap_or_default(),
            confidence: actual_scene.reconstruction_confidence,
            root_cause,
            behavioral_impact,
            evidence,
            severity,
        }
    }

    /// Explain why the gap occurred
    fn explain_root_cause(gap_type: &GapType) -> String {
        match gap_type {
            GapType::ObjectNotDetected => {
                "Object present but robot's detection system did not trigger".to_string()
            }
            GapType::FieldOfViewGap => {
                "Object outside robot's camera/sensor field of view".to_string()
            }
            GapType::DistanceEstimateError => {
                "Distance estimation was inaccurate; object appeared closer/farther than actual"
                    .to_string()
            }
            GapType::TemporalLag => {
                "Object detected but after significant delay; robot reacted too late".to_string()
            }
            GapType::SensorFailure => "Sensor malfunction or degradation".to_string(),
            GapType::ObjectMisclassified => "Object type was incorrectly identified".to_string(),
            GapType::AmbiguousScene => "Scene was too complex; ambiguous what was present"
                .to_string(),
        }
    }

    /// Assess how this gap affected robot behavior
    fn assess_behavioral_impact(
        gap_type: &GapType,
        _sensors: &RobotSensorData,
        behavior_history: &[RobotBehaviorSnapshot],
    ) -> String {
        // Check if there was unexpected behavior
        let has_unusual_behavior = behavior_history
            .iter()
            .any(|b| b.behavior == "stopped" || b.behavior == "emergency_stop");

        if has_unusual_behavior {
            match gap_type {
                GapType::ObjectNotDetected => {
                    "Robot stopped despite object not being detected; likely other sensor triggered"
                        .to_string()
                }
                GapType::FieldOfViewGap => {
                    "Robot behavior triggered by object outside field of view (other sensor)"
                        .to_string()
                }
                _ => "Robot behavior impacted by perception gap".to_string(),
            }
        } else {
            "Gap did not trigger observable behavior change".to_string()
        }
    }

    /// Compute severity of gap
    fn compute_severity(gap_type: &GapType, evidence: &[String]) -> f32 {
        let base_severity = match gap_type {
            GapType::ObjectNotDetected => 0.9,
            GapType::FieldOfViewGap => 0.8,
            GapType::DistanceEstimateError => 0.7,
            GapType::TemporalLag => 0.85,
            GapType::SensorFailure => 0.95,
            GapType::ObjectMisclassified => 0.6,
            GapType::AmbiguousScene => 0.5,
        };

        // Reduce severity if evidence is scarce
        let evidence_factor = (evidence.len() as f32 * 0.1).min(0.3);

        (base_severity - evidence_factor).max(0.0).min(1.0)
    }

    /// Find all critical gaps that likely caused failures
    pub fn find_critical_gaps(gaps: &[PerceptionGap]) -> Vec<PerceptionGap> {
        gaps.iter()
            .filter(|g| {
                g.severity > 0.75 && g.gap_type != GapType::AmbiguousScene
            })
            .cloned()
            .collect()
    }

    /// Generate gap analysis report
    pub fn analyze_mission(
        mission_id: &str,
        scenes: &[crate::intelligence::scene_reconstruction::RetrospectiveScene],
        robot_data: &RobotOperatingData,
    ) -> GapAnalysis {
        let mut all_gaps = Vec::new();
        let mut total_missed_time = 0.0;

        for scene in scenes {
            let sensor_data = RobotSensorData {
                ultrasonic_range: robot_data.sensor_ranges.get("ultrasonic").copied(),
                lidar_range: robot_data.sensor_ranges.get("lidar").copied(),
                camera_fov: robot_data.sensor_ranges.get("camera_fov").copied(),
            };

            let behavior_snapshot = RobotBehaviorSnapshot {
                behavior: robot_data.behavior_at_time(scene.timestamp_sec),
                velocity: 0.0,
            };

            let gap = Self::analyze_perception_gaps(scene, &sensor_data, &[behavior_snapshot]);

            if gap.severity > 0.5 {
                total_missed_time += 0.033; // Assume 30fps
                all_gaps.push(gap);
            }
        }

        let critical_gaps = Self::find_critical_gaps(&all_gaps);

        let mut gaps_by_type: HashMap<String, usize> = HashMap::new();
        let mut gaps_by_entity: HashMap<String, usize> = HashMap::new();

        for gap in &all_gaps {
            *gaps_by_type
                .entry(gap.gap_type.to_string())
                .or_insert(0) += 1;
            *gaps_by_entity
                .entry(gap.entity_type.clone())
                .or_insert(0) += 1;
        }

        let avg_severity: f32 = if all_gaps.is_empty() {
            0.0
        } else {
            all_gaps.iter().map(|g| g.severity).sum::<f32>() / all_gaps.len() as f32
        };

        let affected_time_percent = if scenes.is_empty() {
            0.0
        } else {
            (total_missed_time / (scenes.len() as f32 * 0.033)) * 100.0
        };

        let most_common_gap = gaps_by_type
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(gap_type, _)| gap_type.clone())
            .unwrap_or_default();

        let total_gaps_count = all_gaps.len();

        GapAnalysis {
            mission_id: mission_id.to_string(),
            gaps: all_gaps,
            total_missed_time_sec: total_missed_time,
            critical_gaps,
            statistics: GapStatistics {
                total_gaps: total_gaps_count,
                gaps_by_type,
                gaps_by_entity,
                avg_severity,
                affected_time_percent,
                most_common_gap,
            },
        }
    }
}

/// Robot's sensor capabilities
#[derive(Debug, Clone)]
pub struct RobotSensorData {
    pub ultrasonic_range: Option<f32>,
    pub lidar_range: Option<f32>,
    pub camera_fov: Option<f32>,
}

/// Snapshot of robot behavior
#[derive(Debug, Clone)]
pub struct RobotBehaviorSnapshot {
    pub behavior: String,
    pub velocity: f32,
}

/// Robot operating data for analysis
#[derive(Debug, Clone)]
pub struct RobotOperatingData {
    pub sensor_ranges: HashMap<String, f32>,
    pub behaviors: Vec<(f32, String)>, // (timestamp, behavior)
}

impl RobotOperatingData {
    pub fn behavior_at_time(&self, timestamp: f32) -> String {
        self.behaviors
            .iter()
            .rev()
            .find(|(t, _)| *t <= timestamp)
            .map(|(_, b)| b.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_perception_gap_detection() {
        let robot_data = RobotOperatingData {
            sensor_ranges: {
                let mut map = HashMap::new();
                map.insert("ultrasonic".to_string(), 3.0);
                map.insert("camera_fov".to_string(), 60.0);
                map
            },
            behaviors: vec![(0.0, "moving".to_string())],
        };

        assert_eq!(robot_data.behavior_at_time(0.5), "moving");
    }

    #[test]
    fn test_gap_severity_calculation() {
        let evidence = vec!["Object beyond sensor range".to_string()];
        let severity =
            PerceptionGapAnalyzer::compute_severity(&GapType::ObjectNotDetected, &evidence);

        assert!(severity > 0.7 && severity < 1.0);
    }

    #[test]
    fn test_critical_gap_filtering() {
        let gaps = vec![
            PerceptionGap {
                timestamp_sec: 1.0,
                gap_type: GapType::ObjectNotDetected,
                entity_type: "pallet".to_string(),
                confidence: 0.9,
                root_cause: "Out of range".to_string(),
                behavioral_impact: "Collision".to_string(),
                evidence: vec![],
                severity: 0.9,
            },
            PerceptionGap {
                timestamp_sec: 2.0,
                gap_type: GapType::AmbiguousScene,
                entity_type: "unknown".to_string(),
                confidence: 0.5,
                root_cause: "Complex scene".to_string(),
                behavioral_impact: "Minor".to_string(),
                evidence: vec![],
                severity: 0.4,
            },
        ];

        let critical = PerceptionGapAnalyzer::find_critical_gaps(&gaps);

        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].severity, 0.9);
    }
}
