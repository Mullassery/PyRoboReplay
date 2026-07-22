//! Hidden Explanations: Reconstructing Robot Decision Rationale
//!
//! Generates explanations for robot actions based on what was actually
//! happening, even if the robot didn't perceive it in real time.
//!
//! Core capability: "The robot did X because Y happened, even though
//! the robot may not have known about Y."

use std::collections::HashMap;

/// Discovered fact about the world
#[derive(Debug, Clone)]
pub struct HiddenFact {
    /// What actually happened
    pub fact: String,

    /// When it happened (seconds)
    pub timestamp_sec: f32,

    /// How confident we are in this fact
    pub confidence: f32,

    /// Evidence supporting this fact
    pub evidence: Vec<String>,

    /// Why the robot didn't perceive this
    pub perception_reason: String,
}

/// Causal narrative linking events to robot behavior
#[derive(Debug, Clone)]
pub struct CausalNarrative {
    /// The robot action to explain
    pub robot_action: String,

    /// When the action occurred
    pub action_timestamp_sec: f32,

    /// What actually caused this action (what we discovered in replay)
    pub hidden_cause: HiddenFact,

    /// Timeline of events leading to action
    pub event_chain: Vec<TimelineEvent>,

    /// Alternative explanations (if any)
    pub alternative_explanations: Vec<AlternativeExplanation>,

    /// Most likely explanation
    pub most_likely_cause: String,

    /// Confidence in this explanation
    pub confidence: f32,
}

/// Single event in timeline
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    /// Timestamp
    pub timestamp_sec: f32,

    /// What happened
    pub description: String,

    /// How this relates to the final action
    pub relevance: f32, // 0.0-1.0

    /// Was this visible to the robot?
    pub visible_to_robot: bool,
}

/// Alternative explanation if one exists
#[derive(Debug, Clone)]
pub struct AlternativeExplanation {
    /// What this explanation proposes
    pub explanation: String,

    /// Likelihood (0.0-1.0)
    pub likelihood: f32,

    /// Evidence supporting this alternative
    pub supporting_evidence: Vec<String>,
}

/// Generator of causal narratives
pub struct CausalNarrativeGenerator;

impl CausalNarrativeGenerator {
    /// Generate explanation for a robot action
    pub fn explain_action(
        robot_action: &str,
        action_time: f32,
        scene_timeline: &[(f32, crate::intelligence::scene_reconstruction::RetrospectiveScene)],
        robot_sensor_state: &RobotSensorState,
    ) -> CausalNarrative {
        // Reconstruct events before the action
        let mut event_chain = Vec::new();
        let mut hidden_facts = Vec::new();

        let lookback_window = 2.0; // seconds before action to consider

        for (timestamp, scene) in scene_timeline {
            if (action_time - timestamp) > 0.0 && (action_time - timestamp) <= lookback_window {
                // This event happened before the action
                for obj in &scene.detected_objects {
                    if let Some(distance) = obj.distance_m {
                        if distance < 5.0 {
                        let description =
                            format!("{} detected at {:.1}m", obj.entity_type, distance);
                        let visible_to_robot = obj.in_robot_fov && obj.in_sensor_range;

                        event_chain.push(TimelineEvent {
                            timestamp_sec: *timestamp,
                            description: description.clone(),
                            relevance: 0.8,
                            visible_to_robot,
                        });

                        if !visible_to_robot {
                            hidden_facts.push(HiddenFact {
                                fact: description,
                                timestamp_sec: *timestamp,
                                confidence: scene.reconstruction_confidence,
                                evidence: vec!["Detected in replay camera".to_string()],
                                perception_reason: if !obj.in_robot_fov {
                                    "Outside robot's field of view".to_string()
                                } else {
                                    "Outside effective sensor range".to_string()
                                },
                            });
                        }
                        }
                    }
                }
            }
        }

        // Sort event chain chronologically
        event_chain.sort_by(|a, b| a.timestamp_sec.partial_cmp(&b.timestamp_sec).unwrap());

        // Determine most likely hidden cause
        let hidden_cause = if !hidden_facts.is_empty() {
            hidden_facts[0].clone()
        } else {
            HiddenFact {
                fact: "Unknown environmental factor".to_string(),
                timestamp_sec: action_time,
                confidence: 0.5,
                evidence: vec![],
                perception_reason: "Robot perception did not capture this".to_string(),
            }
        };

        let most_likely_cause = Self::generate_explanation(&robot_action, &hidden_cause);
        let alternatives = Self::generate_alternatives(&robot_action, &event_chain);

        let confidence = Self::compute_confidence(
            &hidden_cause,
            robot_sensor_state,
            &event_chain,
        );

        CausalNarrative {
            robot_action: robot_action.to_string(),
            action_timestamp_sec: action_time,
            hidden_cause,
            event_chain,
            alternative_explanations: alternatives,
            most_likely_cause,
            confidence,
        }
    }

    /// Generate human-readable explanation
    fn generate_explanation(action: &str, fact: &HiddenFact) -> String {
        match action {
            "stopped" | "emergency_stop" => {
                format!(
                    "Robot stopped because: {}. {}",
                    fact.fact, fact.perception_reason
                )
            }
            "slowed" => {
                format!(
                    "Robot slowed due to: {}. {}",
                    fact.fact, fact.perception_reason
                )
            }
            "turned" => {
                format!(
                    "Robot changed direction due to: {}. {}",
                    fact.fact, fact.perception_reason
                )
            }
            _ => format!(
                "Robot {} because: {}.",
                action, fact.fact
            ),
        }
    }

    /// Generate alternative explanations
    fn generate_alternatives(
        _action: &str,
        event_chain: &[TimelineEvent],
    ) -> Vec<AlternativeExplanation> {
        let mut alternatives = Vec::new();

        // If there are sensor readings, consider sensor-based explanation
        if event_chain.iter().any(|e| e.visible_to_robot) {
            alternatives.push(AlternativeExplanation {
                explanation: "Robot detected obstacle via onboard sensors".to_string(),
                likelihood: 0.6,
                supporting_evidence: vec!["Robot behavior change correlates with event".to_string()],
            });
        }

        // Consider planner oscillation
        alternatives.push(AlternativeExplanation {
            explanation: "Navigation planning algorithm behavior".to_string(),
            likelihood: 0.2,
            supporting_evidence: vec!["Behavior could be autonomous behavior planning".to_string()],
        });

        alternatives
    }

    /// Compute confidence in the explanation
    fn compute_confidence(
        hidden_cause: &HiddenFact,
        _robot_state: &RobotSensorState,
        event_chain: &[TimelineEvent],
    ) -> f32 {
        let mut score = hidden_cause.confidence;

        // Increase confidence if multiple events in chain
        let temporal_score = (event_chain.len() as f32 * 0.05).min(0.3);
        score += temporal_score;

        score.min(1.0)
    }
}

/// Robot sensor state during operation
#[derive(Debug, Clone)]
pub struct RobotSensorState {
    pub ultrasonic_active: bool,
    pub lidar_active: bool,
    pub camera_streaming: bool,
    pub imu_active: bool,
}

/// Examples of hidden explanations
pub mod examples {
    use super::*;

    /// Example: Robot stopped due to invisible pedestrian
    pub fn example_invisible_pedestrian() -> CausalNarrative {
        CausalNarrative {
            robot_action: "stopped".to_string(),
            action_timestamp_sec: 305.5,
            hidden_cause: HiddenFact {
                fact: "Pedestrian crossed robot path".to_string(),
                timestamp_sec: 305.2,
                confidence: 0.94,
                evidence: vec!["Person visible in camera at crossing position".to_string()],
                perception_reason: "Robot lacked object detection capability; ultrasonic may have detected but interpretation unclear".to_string(),
            },
            event_chain: vec![
                TimelineEvent {
                    timestamp_sec: 305.0,
                    description: "Pedestrian enters frame at distance 2.5m".to_string(),
                    relevance: 0.7,
                    visible_to_robot: false,
                },
                TimelineEvent {
                    timestamp_sec: 305.2,
                    description: "Pedestrian trajectory crosses robot path".to_string(),
                    relevance: 0.95,
                    visible_to_robot: false,
                },
                TimelineEvent {
                    timestamp_sec: 305.5,
                    description: "Robot emergency stop triggered".to_string(),
                    relevance: 1.0,
                    visible_to_robot: true,
                },
            ],
            alternative_explanations: vec![
                AlternativeExplanation {
                    explanation: "Ultrasonic detected obstacle and triggered stop".to_string(),
                    likelihood: 0.8,
                    supporting_evidence: vec!["Ultrasonic sensor active during this period".to_string()],
                },
            ],
            most_likely_cause: "Pedestrian crossing caused emergency stop. Robot likely detected via ultrasonic sensor, not vision.".to_string(),
            confidence: 0.85,
        }
    }

    /// Example: Robot collided with pallet robot never saw
    pub fn example_invisible_pallet_collision() -> CausalNarrative {
        CausalNarrative {
            robot_action: "collision".to_string(),
            action_timestamp_sec: 612.8,
            hidden_cause: HiddenFact {
                fact: "Pallet blocking aisle, outside robot's sensor range".to_string(),
                timestamp_sec: 610.0,
                confidence: 0.91,
                evidence: vec!["Pallet clearly visible in replay camera from frames 10-12s before collision".to_string()],
                perception_reason: "Pallet positioned at height where ultrasonic sensor aimed above; camera not processed for object detection".to_string(),
            },
            event_chain: vec![
                TimelineEvent {
                    timestamp_sec: 610.0,
                    description: "Pallet appears in camera frame".to_string(),
                    relevance: 0.9,
                    visible_to_robot: false,
                },
                TimelineEvent {
                    timestamp_sec: 610.5,
                    description: "Pallet remains in frame for 2.5 seconds".to_string(),
                    relevance: 0.85,
                    visible_to_robot: false,
                },
                TimelineEvent {
                    timestamp_sec: 612.5,
                    description: "Robot enters pallet location".to_string(),
                    relevance: 1.0,
                    visible_to_robot: true,
                },
                TimelineEvent {
                    timestamp_sec: 612.8,
                    description: "Physical collision detected (IMU spike)".to_string(),
                    relevance: 1.0,
                    visible_to_robot: true,
                },
            ],
            alternative_explanations: vec![],
            most_likely_cause: "Robot collided with pallet it never perceived. Pallet was visible in replay but outside robot's effective sensing region.".to_string(),
            confidence: 0.91,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_pedestrian_explanation() {
        let narrative = examples::example_invisible_pedestrian();

        assert_eq!(narrative.robot_action, "stopped");
        assert!(narrative.confidence > 0.8);
        assert!(!narrative.event_chain.is_empty());
    }

    #[test]
    fn test_example_pallet_collision() {
        let narrative = examples::example_invisible_pallet_collision();

        assert_eq!(narrative.robot_action, "collision");
        assert!(narrative.confidence > 0.9);
        assert!(narrative.most_likely_cause.contains("pallet"));
    }

    #[test]
    fn test_alternative_explanations() {
        let narrative = examples::example_invisible_pedestrian();

        assert!(!narrative.alternative_explanations.is_empty());
        assert!(narrative.alternative_explanations[0].likelihood > 0.0);
    }
}
