//! Predictive Gap Discovery
//!
//! Forecasts which gaps will likely occur next based on:
//! - Historical gap chains (A → B patterns)
//! - Environmental trigger detection
//! - Degradation curves (mechanical wear, thermal aging)
//! - Fleet-wide leading indicators

use crate::analyzers::{RealityGapFinding, RealityDomain};
use std::collections::HashMap;

/// Predicted gap with timing and confidence
#[derive(Debug, Clone)]
pub struct PredictedGap {
    /// Gap category that will likely occur
    pub predicted_category: String,

    /// When this gap is likely to occur (seconds into mission)
    pub predicted_time_sec: f32,

    /// Confidence in this prediction (0.0-1.0)
    pub prediction_confidence: f32,

    /// What evidence led to this prediction
    pub evidence: Vec<String>,

    /// Recommended preventive action
    pub preventive_action: String,

    /// How much earlier can we detect this vs. when it causes failure
    pub detection_lead_time_sec: f32,
}

/// Predictive model for a specific robot/environment
#[derive(Debug, Clone)]
pub struct PredictiveModel {
    /// Robot type this model is calibrated for
    pub robot_type: String,

    /// Known gap chains for this type
    pub gap_chains: HashMap<String, Vec<(String, f32)>>, // gap → [(following_gap, co_occurrence_rate)]

    /// Degradation curves (gap intensity over time)
    pub degradation_curves: HashMap<String, DegradationCurve>,

    /// Environmental triggers (condition → likelihood of gap)
    pub environmental_triggers: HashMap<String, f32>,

    /// Average time between gap detection and failure
    pub detection_to_failure_time: f32,

    /// Model accuracy (fraction of correctly predicted gaps)
    pub model_accuracy: f32,
}

/// How a gap degradation evolves over time
#[derive(Debug, Clone)]
pub struct DegradationCurve {
    /// Gap category
    pub gap_category: String,

    /// Degradation rate (confidence increase per mission)
    pub degradation_rate: f32,

    /// Initial threshold when gap first appears
    pub initial_threshold: f32,

    /// Critical threshold (when gap becomes severe)
    pub critical_threshold: f32,

    /// Missions until critical (estimated)
    pub missions_to_critical: f32,
}

/// Engine for predicting gaps
pub struct PredictiveGapEngine;

impl PredictiveGapEngine {
    /// Build predictive model from historical data
    pub fn build_predictive_model(
        robot_type: &str,
        historical_missions: &[crate::analyzers::fleet_learning::FleetMission],
        gap_chains: &[(String, String, f32)],
    ) -> PredictiveModel {
        let relevant_missions: Vec<_> = historical_missions
            .iter()
            .filter(|m| m.robot_type == robot_type)
            .collect();

        // Build gap chains map
        let mut chains_map: HashMap<String, Vec<(String, f32)>> = HashMap::new();
        for (from, to, rate) in gap_chains {
            chains_map
                .entry(from.clone())
                .or_insert_with(Vec::new)
                .push((to.clone(), *rate));
        }

        // Compute degradation curves
        let degradation_curves = Self::compute_degradation_curves(&relevant_missions);

        // Analyze environmental triggers
        let environmental_triggers = Self::analyze_environmental_triggers(&relevant_missions);

        // Compute detection-to-failure time
        let detection_to_failure_time = Self::compute_detection_to_failure_time(&relevant_missions);

        // Estimate model accuracy (fraction of predicted gaps that occurred)
        let model_accuracy = 0.65; // Start conservative; will improve with calibration

        PredictiveModel {
            robot_type: robot_type.to_string(),
            gap_chains: chains_map,
            degradation_curves,
            environmental_triggers,
            detection_to_failure_time,
            model_accuracy,
        }
    }

    /// Predict next gaps for a specific robot given current state
    pub fn predict_next_gaps(
        model: &PredictiveModel,
        current_gaps: &[RealityGapFinding],
        environmental_conditions: &HashMap<String, f32>,
        missions_completed: usize,
    ) -> Vec<PredictedGap> {
        let mut predictions = Vec::new();

        // Prediction strategy 1: Chain-based (if A exists, B likely follows)
        for current_gap in current_gaps {
            if let Some(following_gaps) = model.gap_chains.get(&current_gap.category) {
                for (following_category, co_occurrence_rate) in following_gaps {
                    let time_to_next = Self::estimate_time_to_gap(
                        following_category,
                        model,
                        *co_occurrence_rate,
                    );

                    predictions.push(PredictedGap {
                        predicted_category: following_category.clone(),
                        predicted_time_sec: time_to_next,
                        prediction_confidence: *co_occurrence_rate * current_gap.confidence,
                        evidence: vec![format!(
                            "Chain pattern: {} → {} (rate: {:.0}%)",
                            current_gap.category,
                            following_category,
                            co_occurrence_rate * 100.0
                        )],
                        preventive_action: format!("Monitor {} closely", following_category),
                        detection_lead_time_sec: model.detection_to_failure_time,
                    });
                }
            }
        }

        // Prediction strategy 2: Degradation-based (if curve is steep, critical soon)
        for (gap_category, curve) in &model.degradation_curves {
            // Check if any current gap matches this degradation curve
            if let Some(current_gap) = current_gaps.iter().find(|g| g.category == *gap_category) {
                let missions_remaining = if curve.degradation_rate > 0.0 {
                    ((curve.critical_threshold - current_gap.confidence) / curve.degradation_rate).max(0.0)
                } else {
                    f32::INFINITY
                };

                if missions_remaining < 10.0 {
                    predictions.push(PredictedGap {
                        predicted_category: gap_category.clone(),
                        predicted_time_sec: missions_remaining * 100.0, // Rough estimate
                        prediction_confidence: (1.0 - (missions_remaining / 10.0).min(1.0)) * 0.8,
                        evidence: vec![format!(
                            "Degradation curve: {} confidence growing at {:.2}/mission, critical in {:.1} missions",
                            gap_category, curve.degradation_rate, missions_remaining
                        )],
                        preventive_action: format!("Service {} before next {}+ missions", gap_category, missions_remaining as u32),
                        detection_lead_time_sec: model.detection_to_failure_time,
                    });
                }
            }
        }

        // Prediction strategy 3: Environmental trigger-based
        for (env_factor, trigger_likelihood) in &model.environmental_triggers {
            if let Some(env_value) = environmental_conditions.get(env_factor) {
                if *env_value > 0.5 && *trigger_likelihood > 0.6 {
                    // This environment likely triggers a gap
                    let triggered_gap = format!("{}_(env_triggered)", env_factor);
                    predictions.push(PredictedGap {
                        predicted_category: triggered_gap,
                        predicted_time_sec: 50.0, // Moderate-term prediction
                        prediction_confidence: *trigger_likelihood * 0.7,
                        evidence: vec![format!(
                            "Environmental trigger: {} present ({:.0}%), historically triggers gaps at {:.0}% rate",
                            env_factor, env_value * 100.0, trigger_likelihood * 100.0
                        )],
                        preventive_action: format!("Increase monitoring for {} effects", env_factor),
                        detection_lead_time_sec: model.detection_to_failure_time,
                    });
                }
            }
        }

        // Sort by confidence and time
        predictions.sort_by(|a, b| {
            b.prediction_confidence
                .partial_cmp(&a.prediction_confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.predicted_time_sec
                        .partial_cmp(&b.predicted_time_sec)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        predictions.truncate(5);
        predictions
    }

    /// Compute degradation curves from historical data
    fn compute_degradation_curves(
        missions: &[&crate::analyzers::fleet_learning::FleetMission],
    ) -> HashMap<String, DegradationCurve> {
        let mut curves = HashMap::new();
        let mut gap_progression: HashMap<String, Vec<f32>> = HashMap::new();

        // Collect confidence progression for each gap
        for mission in missions {
            for gap in &mission.gaps {
                gap_progression
                    .entry(gap.category.clone())
                    .or_insert_with(Vec::new)
                    .push(gap.confidence);
            }
        }

        // Compute curves
        for (gap_category, confidences) in gap_progression {
            if confidences.len() < 2 {
                continue;
            }

            let avg_confidence: f32 = confidences.iter().sum::<f32>() / confidences.len() as f32;
            let degradation_rate = if confidences.len() > 1 {
                (confidences[confidences.len() - 1] - confidences[0]) / (confidences.len() - 1) as f32
            } else {
                0.0
            };

            let critical_threshold = 0.85;
            let initial_threshold = 0.4;

            let missions_to_critical = if degradation_rate > 0.0 {
                (critical_threshold - avg_confidence) / degradation_rate.max(0.01)
            } else {
                f32::INFINITY
            };

            curves.insert(
                gap_category.clone(),
                DegradationCurve {
                    gap_category,
                    degradation_rate,
                    initial_threshold,
                    critical_threshold,
                    missions_to_critical,
                },
            );
        }

        curves
    }

    /// Estimate time until predicted gap appears
    fn estimate_time_to_gap(
        gap_category: &str,
        model: &PredictiveModel,
        co_occurrence_rate: f32,
    ) -> f32 {
        // If we have degradation info, use it
        if let Some(curve) = model.degradation_curves.get(gap_category) {
            curve.missions_to_critical * 100.0 // Convert to seconds (rough)
        } else {
            // Default: high co-occurrence → sooner
            100.0 - (co_occurrence_rate * 50.0)
        }
    }

    /// Analyze which environmental conditions trigger gaps
    fn analyze_environmental_triggers(
        missions: &[&crate::analyzers::fleet_learning::FleetMission],
    ) -> HashMap<String, f32> {
        let mut triggers = HashMap::new();

        for mission in missions {
            if mission.gaps.is_empty() {
                continue; // No gaps, skip
            }

            // If gaps occurred, which environments were present?
            for (env_factor, env_value) in &mission.environmental_conditions {
                if *env_value > 0.5 {
                    triggers
                        .entry(env_factor.clone())
                        .and_modify(|rate| *rate = (*rate + 1.0) / 2.0)
                        .or_insert(1.0);
                }
            }
        }

        triggers
    }

    /// Compute average time between gap detection and actual failure
    fn compute_detection_to_failure_time(
        missions: &[&crate::analyzers::fleet_learning::FleetMission],
    ) -> f32 {
        if missions.is_empty() {
            return 30.0; // Default: 30 seconds
        }

        let failed_missions: Vec<_> = missions
            .iter()
            .filter(|m| m.mission_outcome != "success")
            .collect();

        if failed_missions.is_empty() {
            return 30.0;
        }

        // Estimate: if failure occurred, gaps likely appeared ~20-40s before
        let avg_gap_time: f32 = failed_missions
            .iter()
            .flat_map(|m| m.gaps.iter())
            .filter_map(|g| g.detection_time_sec)
            .sum::<f32>()
            / failed_missions.iter().flat_map(|m| &m.gaps).count().max(1) as f32;

        (50.0 - avg_gap_time).max(10.0).min(60.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzers::fleet_learning::FleetMission;
    use crate::analyzers::{Evidence, Severity};
    use std::collections::HashMap;

    fn create_test_gap(category: &str, confidence: f32, time: f32) -> RealityGapFinding {
        RealityGapFinding {
            domain: RealityDomain::Sensor,
            category: category.to_string(),
            finding_type: "Test".to_string(),
            severity: Severity::Medium,
            confidence,
            reality_gap_score: 0.7,
            description: "Test gap".to_string(),
            evidence: vec![Evidence {
                signal: "test".to_string(),
                value: 0.5,
                timestamp: time,
                confidence: 0.8,
            }],
            metrics: HashMap::new(),
            sim_recreation_suggestion: "Test".to_string(),
            remediation: "Test".to_string(),
            detection_time_sec: Some(time),
        }
    }

    fn create_test_mission(outcome: &str, gaps: Vec<RealityGapFinding>) -> FleetMission {
        FleetMission {
            robot_id: "robot_1".to_string(),
            robot_type: "mobile_robot".to_string(),
            timestamp_sec: 1000.0,
            gaps,
            environmental_conditions: {
                let mut map = HashMap::new();
                map.insert("temperature".to_string(), 25.0);
                map
            },
            mission_outcome: outcome.to_string(),
        }
    }

    #[test]
    fn test_build_predictive_model() {
        let missions = vec![
            create_test_mission("success", vec![create_test_gap("Optical", 0.60, 30.0)]),
            create_test_mission("success", vec![create_test_gap("Optical", 0.75, 30.0)]),
            create_test_mission(
                "partial_failure",
                vec![
                    create_test_gap("Optical", 0.85, 30.0),
                    create_test_gap("Detection", 0.80, 40.0),
                ],
            ),
        ];

        let gap_chains = vec![("Optical".to_string(), "Detection".to_string(), 0.5)];

        let model = PredictiveGapEngine::build_predictive_model("mobile_robot", &missions, &gap_chains);

        assert_eq!(model.robot_type, "mobile_robot");
        assert!(!model.gap_chains.is_empty());
    }

    #[test]
    fn test_predict_next_gaps() {
        let missions = vec![create_test_mission(
            "success",
            vec![create_test_gap("Optical", 0.80, 30.0)],
        )];

        let gap_chains = vec![("Optical".to_string(), "Detection".to_string(), 0.75)];

        let model = PredictiveGapEngine::build_predictive_model("mobile_robot", &missions, &gap_chains);

        let current_gaps = vec![create_test_gap("Optical", 0.85, 30.0)];
        let env_conditions = HashMap::new();

        let predictions = PredictiveGapEngine::predict_next_gaps(&model, &current_gaps, &env_conditions, 0);

        assert!(!predictions.is_empty());
        assert!(predictions[0].prediction_confidence > 0.0);
    }

    #[test]
    fn test_degradation_curve_computation() {
        let missions = vec![
            create_test_mission("success", vec![create_test_gap("Thermal", 0.50, 30.0)]),
            create_test_mission("success", vec![create_test_gap("Thermal", 0.65, 30.0)]),
            create_test_mission("success", vec![create_test_gap("Thermal", 0.80, 30.0)]),
        ];

        let model = PredictiveGapEngine::build_predictive_model("mobile_robot", &missions, &[]);

        assert!(!model.degradation_curves.is_empty());
    }
}
