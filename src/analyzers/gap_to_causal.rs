//! Gap Event Adapter
//!
//! Converts RealityGapFinding events into CausalGraph events and causal links.
//! Bridges the gap detection system with the causal reasoning engine.

use crate::analyzers::{RealityGapFinding, MissionAnalysisData, Severity};
use std::collections::HashMap;

/// Represents a gap as a causal event in the mission timeline
#[derive(Debug, Clone)]
pub struct GapCausalEvent {
    /// Event ID for tracking in causal graph
    pub event_id: String,

    /// When the gap was detected (seconds into mission)
    pub timestamp_sec: f32,

    /// What type of gap this is
    pub gap_type: String,

    /// Root cause inferred from gap characteristics
    pub inferred_cause: String,

    /// Downstream effects this gap likely causes
    pub predicted_effects: Vec<String>,

    /// Confidence in causal relationship (0.0-1.0)
    pub causal_confidence: f32,

    /// The original gap finding
    pub source_finding: RealityGapFinding,
}

/// Converts reality gaps into causal event sequences
pub struct GapToCausalAdapter;

impl GapToCausalAdapter {
    /// Convert a single gap finding into causal events
    pub fn gap_to_causal_events(
        gap: &RealityGapFinding,
        mission_id: &str,
    ) -> Vec<GapCausalEvent> {
        match gap.category.as_str() {
            "Mechanical Degradation" => Self::mechanical_degradation_chain(gap, mission_id),
            "Optical Contamination" => Self::optical_contamination_chain(gap, mission_id),
            "Thermal Effects" => Self::thermal_effects_chain(gap, mission_id),
            "Clock Drift" => Self::clock_drift_chain(gap, mission_id),
            "Detection Robustness" => Self::detection_robustness_chain(gap, mission_id),
            _ => Self::generic_gap_chain(gap, mission_id),
        }
    }

    /// Mechanical degradation → response latency → behavioral lag
    fn mechanical_degradation_chain(
        gap: &RealityGapFinding,
        mission_id: &str,
    ) -> Vec<GapCausalEvent> {
        let detection_time = gap.detection_time_sec.unwrap_or(0.0);

        vec![
            GapCausalEvent {
                event_id: format!("{}_mech_wear_{}", mission_id, detection_time as u32),
                timestamp_sec: detection_time,
                gap_type: "Mechanical Wear Detected".to_string(),
                inferred_cause: "Physical component degradation (bearings, joints, wheels)".to_string(),
                predicted_effects: vec![
                    "Response time increase".to_string(),
                    "Oscillation in control loop".to_string(),
                    "Reduced load capacity".to_string(),
                ],
                causal_confidence: gap.confidence * 0.9,
                source_finding: gap.clone(),
            },
            GapCausalEvent {
                event_id: format!("{}_response_lag_{}", mission_id, (detection_time + 0.1) as u32),
                timestamp_sec: detection_time + 0.1,
                gap_type: "Response Time Increase".to_string(),
                inferred_cause: "Mechanical wear reduces system responsiveness".to_string(),
                predicted_effects: vec![
                    "Delayed obstacle avoidance".to_string(),
                    "Planning uncertainty increases".to_string(),
                    "Mission completion time increases".to_string(),
                ],
                causal_confidence: gap.confidence * 0.85,
                source_finding: gap.clone(),
            },
        ]
    }

    /// Optical contamination → detection confidence drop → planning error
    fn optical_contamination_chain(
        gap: &RealityGapFinding,
        mission_id: &str,
    ) -> Vec<GapCausalEvent> {
        let detection_time = gap.detection_time_sec.unwrap_or(0.0);

        vec![
            GapCausalEvent {
                event_id: format!("{}_optical_contam_{}", mission_id, detection_time as u32),
                timestamp_sec: detection_time,
                gap_type: "Optical Contamination Detected".to_string(),
                inferred_cause: "Water droplets, dust, or debris on camera/sensor lens".to_string(),
                predicted_effects: vec![
                    "Image quality degrades".to_string(),
                    "Detection confidence drops".to_string(),
                    "False negatives increase".to_string(),
                ],
                causal_confidence: gap.confidence * 0.92,
                source_finding: gap.clone(),
            },
            GapCausalEvent {
                event_id: format!("{}_detect_conf_drop_{}", mission_id, (detection_time + 0.05) as u32),
                timestamp_sec: detection_time + 0.05,
                gap_type: "Detection Confidence Degradation".to_string(),
                inferred_cause: "Contamination reduces image quality → object recognition fails".to_string(),
                predicted_effects: vec![
                    "Undetected obstacles possible".to_string(),
                    "Planner assumes safe when dangerous".to_string(),
                    "Collision risk increases".to_string(),
                ],
                causal_confidence: gap.confidence * 0.88,
                source_finding: gap.clone(),
            },
            GapCausalEvent {
                event_id: format!("{}_planning_error_{}", mission_id, (detection_time + 0.2) as u32),
                timestamp_sec: detection_time + 0.2,
                gap_type: "Planner Decision Based on Unreliable Data".to_string(),
                inferred_cause: "Planner unaware that detection confidence is degraded".to_string(),
                predicted_effects: vec![
                    "Unsafe path selection".to_string(),
                    "Collision".to_string(),
                    "Mission failure".to_string(),
                ],
                causal_confidence: gap.confidence * 0.72,
                source_finding: gap.clone(),
            },
        ]
    }

    /// Thermal effects → CPU throttle → latency spike
    fn thermal_effects_chain(
        gap: &RealityGapFinding,
        mission_id: &str,
    ) -> Vec<GapCausalEvent> {
        let detection_time = gap.detection_time_sec.unwrap_or(0.0);

        vec![
            GapCausalEvent {
                event_id: format!("{}_thermal_accumul_{}", mission_id, detection_time as u32),
                timestamp_sec: detection_time,
                gap_type: "Thermal Accumulation".to_string(),
                inferred_cause: "Continuous operation → motor/CPU heat buildup".to_string(),
                predicted_effects: vec![
                    "Temperature > thermal limit".to_string(),
                    "Throttling initiated".to_string(),
                    "Power reduction".to_string(),
                ],
                causal_confidence: gap.confidence * 0.87,
                source_finding: gap.clone(),
            },
            GapCausalEvent {
                event_id: format!("{}_cpu_throttle_{}", mission_id, (detection_time + 0.05) as u32),
                timestamp_sec: detection_time + 0.05,
                gap_type: "CPU Frequency Throttling".to_string(),
                inferred_cause: "Thermal management system reduces CPU frequency".to_string(),
                predicted_effects: vec![
                    "Algorithm execution slows".to_string(),
                    "Latency increases 15-30%".to_string(),
                    "Real-time guarantees broken".to_string(),
                ],
                causal_confidence: gap.confidence * 0.94,
                source_finding: gap.clone(),
            },
            GapCausalEvent {
                event_id: format!("{}_latency_spike_{}", mission_id, (detection_time + 0.1) as u32),
                timestamp_sec: detection_time + 0.1,
                gap_type: "Perception/Planning Latency Spike".to_string(),
                inferred_cause: "CPU throttling cascades to slower perception and planning".to_string(),
                predicted_effects: vec![
                    "Detection runs slower".to_string(),
                    "Planning takes longer".to_string(),
                    "Late avoidance maneuvers".to_string(),
                ],
                causal_confidence: gap.confidence * 0.85,
                source_finding: gap.clone(),
            },
        ]
    }

    /// Clock drift → localization uncertainty → position error
    fn clock_drift_chain(
        gap: &RealityGapFinding,
        mission_id: &str,
    ) -> Vec<GapCausalEvent> {
        let detection_time = gap.detection_time_sec.unwrap_or(0.0);

        vec![
            GapCausalEvent {
                event_id: format!("{}_clock_drift_{}", mission_id, detection_time as u32),
                timestamp_sec: detection_time,
                gap_type: "Sensor Clock Drift Detected".to_string(),
                inferred_cause: "Sensor clock running fast/slow relative to system clock".to_string(),
                predicted_effects: vec![
                    "Timestamp misalignment".to_string(),
                    "Fusion algorithm confusion".to_string(),
                    "Localization divergence".to_string(),
                ],
                causal_confidence: gap.confidence * 0.99,
                source_finding: gap.clone(),
            },
            GapCausalEvent {
                event_id: format!("{}_fusion_error_{}", mission_id, (detection_time + 0.05) as u32),
                timestamp_sec: detection_time + 0.05,
                gap_type: "Fusion Algorithm Error".to_string(),
                inferred_cause: "Mismatched timestamps confuse multi-sensor fusion".to_string(),
                predicted_effects: vec![
                    "Localization filter diverges".to_string(),
                    "Particle filter collapses".to_string(),
                    "Position uncertainty explodes".to_string(),
                ],
                causal_confidence: gap.confidence * 0.92,
                source_finding: gap.clone(),
            },
            GapCausalEvent {
                event_id: format!("{}_position_error_{}", mission_id, (detection_time + 0.5) as u32),
                timestamp_sec: detection_time + 0.5,
                gap_type: "Localization Position Error".to_string(),
                inferred_cause: "Fusion failure → system doesn't know where it is".to_string(),
                predicted_effects: vec![
                    "Navigation commands based on wrong position".to_string(),
                    "Obstacle detection in wrong frame".to_string(),
                    "Collision or navigation failure".to_string(),
                ],
                causal_confidence: gap.confidence * 0.78,
                source_finding: gap.clone(),
            },
        ]
    }

    /// Detection robustness → false negatives → missed obstacles
    fn detection_robustness_chain(
        gap: &RealityGapFinding,
        mission_id: &str,
    ) -> Vec<GapCausalEvent> {
        let detection_time = gap.detection_time_sec.unwrap_or(0.0);

        vec![
            GapCausalEvent {
                event_id: format!("{}_detect_robust_{}", mission_id, detection_time as u32),
                timestamp_sec: detection_time,
                gap_type: "Detection Robustness Issue".to_string(),
                inferred_cause: "Model confidence unreliable in out-of-distribution scenarios".to_string(),
                predicted_effects: vec![
                    "False negatives (missed objects)".to_string(),
                    "False positives (ghost detections)".to_string(),
                    "Confidence poorly calibrated".to_string(),
                ],
                causal_confidence: gap.confidence * 0.86,
                source_finding: gap.clone(),
            },
            GapCausalEvent {
                event_id: format!("{}_false_negative_{}", mission_id, (detection_time + 0.1) as u32),
                timestamp_sec: detection_time + 0.1,
                gap_type: "Obstacle False Negative".to_string(),
                inferred_cause: "Robustness issue → actual obstacle not detected".to_string(),
                predicted_effects: vec![
                    "Planner doesn't know about obstacle".to_string(),
                    "Path passes through obstacle".to_string(),
                    "Collision imminent".to_string(),
                ],
                causal_confidence: gap.confidence * 0.82,
                source_finding: gap.clone(),
            },
        ]
    }

    /// Generic gap chain for unrecognized types
    fn generic_gap_chain(
        gap: &RealityGapFinding,
        mission_id: &str,
    ) -> Vec<GapCausalEvent> {
        let detection_time = gap.detection_time_sec.unwrap_or(0.0);

        vec![GapCausalEvent {
            event_id: format!("{}_{}_gap_{}", mission_id, gap.category, detection_time as u32),
            timestamp_sec: detection_time,
            gap_type: gap.finding_type.clone(),
            inferred_cause: gap.description.clone(),
            predicted_effects: vec!["System behavior altered".to_string()],
            causal_confidence: gap.confidence,
            source_finding: gap.clone(),
        }]
    }

    /// Create causal links showing how gaps chain together
    pub fn infer_gap_causal_links(
        gaps: &[RealityGapFinding],
    ) -> Vec<GapCausalLink> {
        let mut links = Vec::new();

        // Pattern 1: Sensor degradation → detection failure
        for gap in gaps {
            if gap.category.contains("Optical") || gap.category.contains("Sensor") {
                for other_gap in gaps {
                    if other_gap.category.contains("Detection") {
                        if gap.detection_time_sec < other_gap.detection_time_sec {
                            links.push(GapCausalLink {
                                source_gap: gap.category.clone(),
                                target_gap: other_gap.category.clone(),
                                causal_relationship: "Sensor degradation reduces perception quality".to_string(),
                                confidence: 0.85,
                                time_gap_sec: (other_gap.detection_time_sec.unwrap_or(0.0)
                                    - gap.detection_time_sec.unwrap_or(0.0))
                                    .abs(),
                            });
                        }
                    }
                }
            }
        }

        // Pattern 2: Thermal → latency → detection
        for gap in gaps {
            if gap.category.contains("Thermal") {
                for other_gap in gaps {
                    if other_gap.category.contains("Detection") {
                        links.push(GapCausalLink {
                            source_gap: gap.category.clone(),
                            target_gap: other_gap.category.clone(),
                            causal_relationship: "Thermal throttling → latency → late detection".to_string(),
                            confidence: 0.72,
                            time_gap_sec: (other_gap.detection_time_sec.unwrap_or(0.0)
                                - gap.detection_time_sec.unwrap_or(0.0))
                                .abs(),
                        });
                    }
                }
            }
        }

        // Pattern 3: Clock drift → localization → navigation
        for gap in gaps {
            if gap.category.contains("Clock") {
                for other_gap in gaps {
                    if other_gap.category.contains("Mechanical") {
                        links.push(GapCausalLink {
                            source_gap: gap.category.clone(),
                            target_gap: other_gap.category.clone(),
                            causal_relationship: "Clock misalignment confuses motion estimation".to_string(),
                            confidence: 0.68,
                            time_gap_sec: (other_gap.detection_time_sec.unwrap_or(0.0)
                                - gap.detection_time_sec.unwrap_or(0.0))
                                .abs(),
                        });
                    }
                }
            }
        }

        links
    }
}

/// Link between two gaps in a causal chain
#[derive(Debug, Clone)]
pub struct GapCausalLink {
    pub source_gap: String,
    pub target_gap: String,
    pub causal_relationship: String,
    pub confidence: f32,
    pub time_gap_sec: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_gap(category: &str, confidence: f32) -> RealityGapFinding {
        RealityGapFinding {
            domain: crate::analyzers::RealityDomain::Physical,
            category: category.to_string(),
            finding_type: format!("Test {}", category),
            severity: Severity::Medium,
            confidence,
            reality_gap_score: 0.7,
            description: "Test gap".to_string(),
            evidence: vec![],
            metrics: HashMap::new(),
            sim_recreation_suggestion: "Test".to_string(),
            remediation: "Test".to_string(),
            detection_time_sec: Some(100.0),
        }
    }

    #[test]
    fn test_mechanical_degradation_chain() {
        let gap = create_test_gap("Mechanical Degradation", 0.8);
        let events = GapToCausalAdapter::gap_to_causal_events(&gap, "test_mission");

        assert_eq!(events.len(), 2); // Wear detected + response lag
        assert!(events[0].gap_type.contains("Mechanical Wear"));
        assert!(events[1].gap_type.contains("Response Time"));
    }

    #[test]
    fn test_optical_contamination_chain() {
        let gap = create_test_gap("Optical Contamination", 0.75);
        let events = GapToCausalAdapter::gap_to_causal_events(&gap, "test_mission");

        assert_eq!(events.len(), 3); // Contamination + confidence drop + planning error
        assert!(events[2].gap_type.contains("Planner Decision"));
    }

    #[test]
    fn test_thermal_effects_chain() {
        let gap = create_test_gap("Thermal Effects", 0.82);
        let events = GapToCausalAdapter::gap_to_causal_events(&gap, "test_mission");

        assert_eq!(events.len(), 3); // Accumulation + throttle + latency
        assert!(events[1].gap_type.contains("Throttling"));
    }

    #[test]
    fn test_clock_drift_chain() {
        let gap = create_test_gap("Clock Drift", 0.9);
        let events = GapToCausalAdapter::gap_to_causal_events(&gap, "test_mission");

        assert_eq!(events.len(), 3); // Drift + fusion error + position error
        assert!(events[0].causal_confidence > 0.85);
    }

    #[test]
    fn test_gap_causal_links() {
        let gaps = vec![
            create_test_gap("Optical Contamination", 0.8),
            create_test_gap("Detection Robustness", 0.75),
            create_test_gap("Thermal Effects", 0.82),
        ];

        let links = GapToCausalAdapter::infer_gap_causal_links(&gaps);
        assert!(links.len() > 0); // Should find causal relationships

        // Verify links have reasonable confidence
        for link in &links {
            assert!(link.confidence > 0.6);
            assert!(link.confidence < 1.0);
        }
    }

    #[test]
    fn test_confidence_propagation() {
        let gap = create_test_gap("Mechanical Degradation", 0.9);
        let events = GapToCausalAdapter::gap_to_causal_events(&gap, "test_mission");

        // Confidence should decrease through causal chain
        assert!(events[0].causal_confidence > events[1].causal_confidence);
    }
}
