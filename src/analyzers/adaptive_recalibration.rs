//! Adaptive Recalibration Engine
//!
//! Continuously learns from feedback to improve:
//! - Per-robot sensitivity profiles
//! - Environmental impact models
//! - Gap chain probabilities
//! - Prediction accuracy

use crate::analyzers::fleet_learning::{FleetMission, RobotTypeProfile};
use crate::analyzers::predictive_gaps::{PredictiveModel, DegradationCurve};
use crate::analyzers::RealityGapFinding;
use std::collections::HashMap;

/// Feedback on whether a prediction was accurate
#[derive(Debug, Clone)]
pub struct PredictionFeedback {
    /// Which gap was predicted
    pub predicted_gap: String,

    /// Was the prediction correct?
    pub was_accurate: bool,

    /// Did the predicted gap actually occur?
    pub actually_occurred: bool,

    /// How soon after prediction did it occur (seconds)?
    pub time_to_actual_sec: Option<f32>,

    /// Human feedback (if available)
    pub human_feedback: Option<String>,

    /// Robot that this feedback is for
    pub robot_id: String,
}

/// Recalibration metrics for a model
#[derive(Debug, Clone)]
pub struct RecalibrationMetrics {
    /// Accuracy of predictions (correct / total)
    pub prediction_accuracy: f32,

    /// False positive rate (predicted but didn't happen)
    pub false_positive_rate: f32,

    /// False negative rate (happened but didn't predict)
    pub false_negative_rate: f32,

    /// Average lead time in predictions (seconds)
    pub avg_lead_time: f32,

    /// Total predictions made
    pub total_predictions: usize,

    /// Correct predictions
    pub correct_predictions: usize,
}

/// Recalibration strategy
#[derive(Debug, Clone, Copy)]
pub enum RecalibrationStrategy {
    /// Conservative: only update on high-confidence feedback
    Conservative,

    /// Balanced: moderate updates
    Balanced,

    /// Aggressive: quickly adapt to new patterns
    Aggressive,
}

/// Engine for continuously improving models
pub struct AdaptiveRecalibrationEngine;

impl AdaptiveRecalibrationEngine {
    /// Update predictive model based on feedback
    pub fn recalibrate_model(
        model: &mut PredictiveModel,
        feedback: &[PredictionFeedback],
        strategy: RecalibrationStrategy,
    ) -> RecalibrationMetrics {
        let total_predictions = feedback.len();
        let correct_predictions = feedback.iter().filter(|f| f.was_accurate).count();
        let false_positives = feedback.iter().filter(|f| !f.actually_occurred).count();
        let false_negatives = total_predictions - correct_predictions;

        let accuracy = if total_predictions > 0 {
            correct_predictions as f32 / total_predictions as f32
        } else {
            0.5
        };

        let false_positive_rate = if total_predictions > 0 {
            false_positives as f32 / total_predictions as f32
        } else {
            0.0
        };

        let false_negative_rate = if total_predictions > 0 {
            false_negatives as f32 / total_predictions as f32
        } else {
            0.0
        };

        let avg_lead_time: f32 = feedback
            .iter()
            .filter_map(|f| f.time_to_actual_sec)
            .sum::<f32>()
            / feedback.iter().filter(|f| f.time_to_actual_sec.is_some()).count().max(1) as f32;

        // Apply learning rate based on strategy
        let learning_rate = match strategy {
            RecalibrationStrategy::Conservative => 0.1,
            RecalibrationStrategy::Balanced => 0.3,
            RecalibrationStrategy::Aggressive => 0.5,
        };

        // Update model accuracy
        model.model_accuracy = (model.model_accuracy * (1.0 - learning_rate) + accuracy * learning_rate)
            .max(0.0)
            .min(1.0);

        // Update gap chains based on feedback
        Self::update_gap_chains(model, feedback, learning_rate);

        // Update degradation curves
        Self::update_degradation_curves(model, feedback, learning_rate);

        RecalibrationMetrics {
            prediction_accuracy: accuracy,
            false_positive_rate,
            false_negative_rate,
            avg_lead_time,
            total_predictions,
            correct_predictions,
        }
    }

    /// Update gap chain probabilities based on feedback
    fn update_gap_chains(
        model: &mut PredictiveModel,
        feedback: &[PredictionFeedback],
        learning_rate: f32,
    ) {
        for item in feedback {
            if item.actually_occurred {
                // This gap occurred - we may have a valid chain
                // Update probability based on accuracy
                let confidence_boost = if item.was_accurate { learning_rate } else { -learning_rate * 0.5 };

                // Find all chains that predicted this gap
                for (_from_gap, following_gaps) in model.gap_chains.iter_mut() {
                    for (gap_name, rate) in following_gaps.iter_mut() {
                        if gap_name == &item.predicted_gap {
                            *rate = (*rate + confidence_boost).max(0.0).min(1.0);
                        }
                    }
                }
            }
        }
    }

    /// Update degradation curves based on observed progression
    fn update_degradation_curves(
        model: &mut PredictiveModel,
        feedback: &[PredictionFeedback],
        learning_rate: f32,
    ) {
        // Aggregate feedback by gap type
        let mut gap_lead_times: HashMap<String, Vec<f32>> = HashMap::new();

        for item in feedback {
            if let Some(lead_time) = item.time_to_actual_sec {
                gap_lead_times
                    .entry(item.predicted_gap.clone())
                    .or_insert_with(Vec::new)
                    .push(lead_time);
            }
        }

        // Update degradation curves
        for (gap_category, lead_times) in gap_lead_times {
            if let Some(curve) = model.degradation_curves.get_mut(&gap_category) {
                if !lead_times.is_empty() {
                    let avg_lead_time: f32 = lead_times.iter().sum::<f32>() / lead_times.len() as f32;

                    // Adjust missions to critical based on observed lead time
                    let new_missions_to_critical = avg_lead_time / 100.0; // Rough conversion back
                    curve.missions_to_critical =
                        (curve.missions_to_critical * (1.0 - learning_rate)
                            + new_missions_to_critical * learning_rate)
                            .max(1.0);

                    // Adjust degradation rate
                    let degradation_adjustment = (avg_lead_time / 100.0) * learning_rate;
                    curve.degradation_rate = (curve.degradation_rate + degradation_adjustment)
                        .max(0.0)
                        .min(0.2);
                }
            }
        }
    }

    /// Update robot-type profile based on mission outcomes
    pub fn recalibrate_robot_profile(
        profile: &mut RobotTypeProfile,
        recent_missions: &[FleetMission],
        learning_rate: f32,
    ) {
        if recent_missions.is_empty() {
            return;
        }

        // Recompute problematic gaps
        let mut gap_counts: HashMap<String, (usize, f32)> = HashMap::new();
        for mission in recent_missions {
            for gap in &mission.gaps {
                let entry = gap_counts.entry(gap.category.clone()).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += gap.confidence;
            }
        }

        let mut problematic_gaps: Vec<_> = gap_counts
            .into_iter()
            .map(|(name, (count, sum_conf))| (name, count, sum_conf / count as f32))
            .collect();
        problematic_gaps.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Update with learning rate
        for (i, (new_gap, new_count, new_conf)) in problematic_gaps.iter().enumerate() {
            if i < profile.problematic_gaps.len() {
                let old_gap = &profile.problematic_gaps[i];
                let updated_count = ((old_gap.1 as f32 * (1.0 - learning_rate))
                    + (*new_count as f32 * learning_rate)) as usize;
                let updated_conf =
                    old_gap.2 * (1.0 - learning_rate) + new_conf * learning_rate;
                profile.problematic_gaps[i] = (new_gap.clone(), updated_count, updated_conf);
            }
        }

        // Update sensitivity multiplier based on failure rate
        let failure_rate: f32 = recent_missions
            .iter()
            .filter(|m| m.mission_outcome != "success")
            .count() as f32
            / recent_missions.len() as f32;

        if failure_rate > 0.3 {
            // High failure rate → increase sensitivity
            profile.sensitivity_multiplier =
                (profile.sensitivity_multiplier * (1.0 - learning_rate) + 1.3 * learning_rate)
                    .max(0.8)
                    .min(1.5);
        } else if failure_rate < 0.1 {
            // Low failure rate → decrease sensitivity
            profile.sensitivity_multiplier =
                (profile.sensitivity_multiplier * (1.0 - learning_rate) + 0.9 * learning_rate)
                    .max(0.8)
                    .min(1.5);
        }

        // Update recency
        profile.calibration_recency = ((profile.calibration_recency * 0.7) + 1.0).min(1.0);
    }

    /// Analyze root causes of mispredictions
    pub fn analyze_prediction_errors(
        feedback: &[PredictionFeedback],
    ) -> HashMap<String, ErrorAnalysis> {
        let mut error_analysis: HashMap<String, ErrorAnalysis> = HashMap::new();

        for item in feedback {
            let analysis = error_analysis
                .entry(item.predicted_gap.clone())
                .or_insert_with(|| ErrorAnalysis {
                    gap_category: item.predicted_gap.clone(),
                    false_positives: 0,
                    false_negatives: 0,
                    avg_prediction_error_sec: 0.0,
                    recommended_adjustment: "None".to_string(),
                });

            if !item.was_accurate {
                if !item.actually_occurred {
                    analysis.false_positives += 1;
                } else {
                    analysis.false_negatives += 1;
                }
            }

            if let Some(lead_time) = item.time_to_actual_sec {
                analysis.avg_prediction_error_sec =
                    (analysis.avg_prediction_error_sec + lead_time.abs()) / 2.0;
            }
        }

        // Generate recommendations
        for analysis in error_analysis.values_mut() {
            analysis.recommended_adjustment = if analysis.false_positives > analysis.false_negatives {
                format!("Increase threshold by {:.1}%", 5.0 + analysis.false_positives as f32)
            } else if analysis.false_negatives > analysis.false_positives {
                format!("Decrease threshold by {:.1}%", 5.0 + analysis.false_negatives as f32)
            } else {
                "Threshold appears calibrated".to_string()
            };
        }

        error_analysis
    }
}

/// Analysis of why a prediction was wrong
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    /// Gap category
    pub gap_category: String,

    /// Number of false positives (predicted but didn't happen)
    pub false_positives: usize,

    /// Number of false negatives (didn't predict but happened)
    pub false_negatives: usize,

    /// Average error in timing prediction (seconds)
    pub avg_prediction_error_sec: f32,

    /// Recommended threshold adjustment
    pub recommended_adjustment: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzers::{Evidence, RealityDomain, Severity};
    use std::collections::HashMap;

    fn create_test_feedback(gap: &str, was_accurate: bool, occurred: bool) -> PredictionFeedback {
        PredictionFeedback {
            predicted_gap: gap.to_string(),
            was_accurate,
            actually_occurred: occurred,
            time_to_actual_sec: if occurred { Some(50.0) } else { None },
            human_feedback: None,
            robot_id: "robot_1".to_string(),
        }
    }

    fn create_test_model() -> PredictiveModel {
        let mut gap_chains = HashMap::new();
        gap_chains.insert(
            "Optical".to_string(),
            vec![("Detection".to_string(), 0.7)],
        );

        PredictiveModel {
            robot_type: "mobile_robot".to_string(),
            gap_chains,
            degradation_curves: HashMap::new(),
            environmental_triggers: HashMap::new(),
            detection_to_failure_time: 30.0,
            model_accuracy: 0.6,
        }
    }

    #[test]
    fn test_recalibration_metrics() {
        let feedback = vec![
            create_test_feedback("Optical", true, true),
            create_test_feedback("Thermal", true, true),
            create_test_feedback("Clock", false, false),
        ];

        let mut model = create_test_model();
        let metrics = AdaptiveRecalibrationEngine::recalibrate_model(
            &mut model,
            &feedback,
            RecalibrationStrategy::Balanced,
        );

        assert_eq!(metrics.total_predictions, 3);
        assert!(metrics.prediction_accuracy > 0.0);
    }

    #[test]
    fn test_conservative_learning() {
        let feedback = vec![
            create_test_feedback("Optical", true, true),
            create_test_feedback("Detection", false, true),
        ];

        let mut model = create_test_model();
        let original_accuracy = model.model_accuracy;

        AdaptiveRecalibrationEngine::recalibrate_model(
            &mut model,
            &feedback,
            RecalibrationStrategy::Conservative,
        );

        // Conservative should make small updates
        assert!((model.model_accuracy - original_accuracy).abs() < 0.2);
    }

    #[test]
    fn test_aggressive_learning() {
        let feedback = vec![create_test_feedback("Optical", true, true); 5];

        let mut model = create_test_model();
        let original_accuracy = model.model_accuracy;

        AdaptiveRecalibrationEngine::recalibrate_model(
            &mut model,
            &feedback,
            RecalibrationStrategy::Aggressive,
        );

        // Aggressive should make larger updates when predictions are correct
        assert!(model.model_accuracy > original_accuracy);
    }

    #[test]
    fn test_error_analysis() {
        let feedback = vec![
            create_test_feedback("Optical", true, true),
            create_test_feedback("Optical", false, false), // false positive
            create_test_feedback("Optical", false, true),  // false negative
        ];

        let analysis = AdaptiveRecalibrationEngine::analyze_prediction_errors(&feedback);

        assert!(analysis.contains_key("Optical"));
        let optical_analysis = &analysis["Optical"];
        assert_eq!(optical_analysis.false_positives, 1);
        assert_eq!(optical_analysis.false_negatives, 1);
    }
}
