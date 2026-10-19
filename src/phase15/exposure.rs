//! Public, JSON-shaped entry point for Phase 15's Nav2 root-cause heuristics.
//!
//! Phase 15 is a set of independent analyzers that each need a small number of
//! measurements. This module defines a single versioned input document and
//! runs every analyzer whose required measurements are present, so the logic
//! is reachable from the CLI and Python without inventing live ROS plumbing.

use crate::phase15::costmap_analyzer::{CostmapAnalyzer, CostmapIssue};
use crate::phase15::dynamic_obstacle_analyzer::{DynamicObstacleAnalyzer, ObstacleIssue};
use crate::phase15::localization_analyzer::{LocalizationAnalyzer, LocalizationIssue};
use crate::phase15::nav2_limitation_detector::{Nav2LimitationDetection, Nav2LimitationDetector};
use crate::phase15::planner_analyzer::{PlannerAnalyzer, PlannerIssue};
use crate::phase15::root_cause_generator::{RootCauseGenerator, RootCauseHypothesis};
use crate::phase15::semantic_gap_analyzer::{SemanticGap, SemanticGapAnalyzer};
use serde::{Deserialize, Serialize};

/// Localization measurements. Each analyzer runs when its fields are present.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalizationMetrics {
    #[serde(default)]
    pub particle_spread_before: Option<f32>,
    #[serde(default)]
    pub particle_spread_after: Option<f32>,
    #[serde(default)]
    pub odometry_error_rate: Option<f32>,
    #[serde(default)]
    pub distance_traveled: Option<f32>,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub detected_features: Option<u32>,
    #[serde(default)]
    pub features_threshold: Option<u32>,
    #[serde(default)]
    pub lighting_conditions: Option<String>,
    #[serde(default)]
    pub noise_before: Option<f32>,
    #[serde(default)]
    pub noise_after: Option<f32>,
    #[serde(default)]
    pub outlier_rate_before: Option<f32>,
    #[serde(default)]
    pub outlier_rate_after: Option<f32>,
    #[serde(default)]
    pub confidence: Option<f32>,
    #[serde(default)]
    pub estimated_error: Option<f32>,
    #[serde(default)]
    pub map_matches_per_scan: Option<u32>,
}

/// Planner measurements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlannerMetrics {
    #[serde(default)]
    pub replanning_frequency: Option<f32>,
    #[serde(default)]
    pub time_since_goal_update: Option<f32>,
    #[serde(default)]
    pub plan_divergence: Option<f32>,
    #[serde(default)]
    pub oscillation_magnitude: Option<f32>,
    #[serde(default)]
    pub plan_attempts: Option<u32>,
    #[serde(default)]
    pub failed_attempts: Option<u32>,
    #[serde(default)]
    pub distance_to_goal: Option<f32>,
    #[serde(default)]
    pub corridor_width: Option<f32>,
    #[serde(default)]
    pub robot_footprint: Option<f32>,
    #[serde(default)]
    pub replan_count_in_corridor: Option<u32>,
    #[serde(default)]
    pub mean_plan_time_ms: Option<f32>,
    #[serde(default)]
    pub success_rate: Option<f32>,
}

/// Costmap measurements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostmapMetrics {
    #[serde(default)]
    pub inflation_radius: Option<f32>,
    #[serde(default)]
    pub robot_footprint: Option<f32>,
    #[serde(default)]
    pub static_layer_size: Option<u32>,
    #[serde(default)]
    pub dynamic_layer_size: Option<u32>,
}

/// Dynamic-obstacle measurements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DynamicObstacleMetrics {
    #[serde(default)]
    pub human_detected: Option<bool>,
    #[serde(default)]
    pub distance_to_human: Option<f32>,
}

/// Semantic-navigation measurements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticMetrics {
    #[serde(default)]
    pub failure_near_door: Option<bool>,
    #[serde(default)]
    pub door_detected: Option<bool>,
}

/// Inputs for Nav2 limitation classification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassificationMetrics {
    #[serde(default)]
    pub failure_type: Option<String>,
    #[serde(default)]
    pub environment_type: Option<String>,
    #[serde(default)]
    pub scale_meters: Option<f32>,
}

/// Inputs for the compact root-cause hypothesis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HypothesisMetrics {
    #[serde(default)]
    pub localization_confidence: Option<f32>,
    #[serde(default)]
    pub planner_quality: Option<f32>,
    #[serde(default)]
    pub costmap_validity: Option<f32>,
}

/// Versioned input document for Phase 15 analyses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Nav2Diagnostics {
    #[serde(default)]
    pub localization: LocalizationMetrics,
    #[serde(default)]
    pub planner: PlannerMetrics,
    #[serde(default)]
    pub costmap: CostmapMetrics,
    #[serde(default)]
    pub dynamic_obstacle: DynamicObstacleMetrics,
    #[serde(default)]
    pub semantic: SemanticMetrics,
    #[serde(default)]
    pub classification: Option<ClassificationMetrics>,
    #[serde(default)]
    pub hypothesis: Option<HypothesisMetrics>,
}

/// Human-readable summaries emitted alongside the structured issues.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Nav2Summaries {
    pub localization: Option<String>,
    pub planner: Option<String>,
    pub costmap: Option<String>,
}

/// Combined Phase 15 output.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Nav2Report {
    pub localization_issues: Vec<LocalizationIssue>,
    pub planner_issues: Vec<PlannerIssue>,
    pub costmap_issues: Vec<CostmapIssue>,
    pub dynamic_obstacle_issues: Vec<ObstacleIssue>,
    pub semantic_gaps: Vec<SemanticGap>,
    pub classification: Option<Nav2LimitationDetection>,
    pub root_cause: Option<RootCauseHypothesis>,
    pub summaries: Nav2Summaries,
}

impl Nav2Report {
    /// True when no analyzer produced an issue, gap, or classification.
    pub fn is_empty(&self) -> bool {
        self.localization_issues.is_empty()
            && self.planner_issues.is_empty()
            && self.costmap_issues.is_empty()
            && self.dynamic_obstacle_issues.is_empty()
            && self.semantic_gaps.is_empty()
            && self.classification.is_none()
            && self.root_cause.is_none()
    }
}

/// Run every Phase 15 analyzer whose required measurements are present.
pub fn analyze_nav2_diagnostics(input: &Nav2Diagnostics) -> Nav2Report {
    let mut report = Nav2Report::default();

    let loc = &input.localization;
    if let (Some(before), Some(after)) = (loc.particle_spread_before, loc.particle_spread_after) {
        report
            .localization_issues
            .extend(LocalizationAnalyzer::analyze_particle_divergence(
                before, after,
            ));
    }
    if let (Some(rate), Some(distance)) = (loc.odometry_error_rate, loc.distance_traveled) {
        report
            .localization_issues
            .extend(LocalizationAnalyzer::analyze_odometry_drift(
                rate,
                distance,
                loc.timestamp.unwrap_or(0),
            ));
    }
    if let (Some(features), Some(threshold)) = (loc.detected_features, loc.features_threshold) {
        report
            .localization_issues
            .extend(LocalizationAnalyzer::analyze_feature_starvation(
                features,
                threshold,
                loc.lighting_conditions.as_deref(),
            ));
    }
    if let (Some(nb), Some(na), Some(ob), Some(oa)) = (
        loc.noise_before,
        loc.noise_after,
        loc.outlier_rate_before,
        loc.outlier_rate_after,
    ) {
        report
            .localization_issues
            .extend(LocalizationAnalyzer::analyze_sensor_degradation(
                nb, na, ob, oa,
            ));
    }
    if let (Some(confidence), Some(error), Some(matches)) = (
        loc.confidence,
        loc.estimated_error,
        loc.map_matches_per_scan,
    ) {
        report.summaries.localization = Some(LocalizationAnalyzer::summarize_localization(
            confidence, error, matches,
        ));
    }

    let planner = &input.planner;
    if let (Some(frequency), Some(since_goal)) =
        (planner.replanning_frequency, planner.time_since_goal_update)
    {
        report
            .planner_issues
            .extend(PlannerAnalyzer::analyze_excessive_replanning(
                frequency, since_goal,
            ));
    }
    if let (Some(divergence), Some(oscillation)) =
        (planner.plan_divergence, planner.oscillation_magnitude)
    {
        report
            .planner_issues
            .extend(PlannerAnalyzer::analyze_route_oscillation(
                divergence,
                oscillation,
            ));
    }
    if let (Some(attempts), Some(failed), Some(distance)) = (
        planner.plan_attempts,
        planner.failed_attempts,
        planner.distance_to_goal,
    ) {
        report
            .planner_issues
            .extend(PlannerAnalyzer::analyze_unreachable_goal(
                attempts, failed, distance,
            ));
    }
    if let (Some(width), Some(footprint), Some(replans)) = (
        planner.corridor_width,
        planner.robot_footprint,
        planner.replan_count_in_corridor,
    ) {
        report
            .planner_issues
            .extend(PlannerAnalyzer::analyze_narrow_corridor(
                width, footprint, replans,
            ));
    }
    if let (Some(plan_time), Some(success), Some(frequency)) = (
        planner.mean_plan_time_ms,
        planner.success_rate,
        planner.replanning_frequency,
    ) {
        report.summaries.planner = Some(PlannerAnalyzer::summarize_planner(
            plan_time, success, frequency,
        ));
    }

    let costmap = &input.costmap;
    if let (Some(inflation), Some(footprint)) = (costmap.inflation_radius, costmap.robot_footprint)
    {
        report
            .costmap_issues
            .extend(CostmapAnalyzer::analyze_inflation(inflation, footprint));
    }
    if let (Some(static_size), Some(dynamic_size), Some(inflation)) = (
        costmap.static_layer_size,
        costmap.dynamic_layer_size,
        costmap.inflation_radius,
    ) {
        report.summaries.costmap = Some(CostmapAnalyzer::summarize_costmap(
            static_size,
            dynamic_size,
            inflation,
        ));
    }

    if let (Some(human), Some(distance)) = (
        input.dynamic_obstacle.human_detected,
        input.dynamic_obstacle.distance_to_human,
    ) {
        report
            .dynamic_obstacle_issues
            .extend(DynamicObstacleAnalyzer::detect_human_obstruction(
                human, distance,
            ));
    }
    if let (Some(near_door), Some(door)) = (
        input.semantic.failure_near_door,
        input.semantic.door_detected,
    ) {
        report
            .semantic_gaps
            .extend(SemanticGapAnalyzer::detect_door_navigation(near_door, door));
    }

    if let Some(classification) = &input.classification {
        if let (Some(failure_type), Some(environment)) = (
            classification.failure_type.as_deref(),
            classification.environment_type.as_deref(),
        ) {
            report.classification = Some(Nav2LimitationDetector::classify(
                failure_type,
                environment,
                classification.scale_meters.unwrap_or(0.0),
            ));
        }
    }

    if let Some(hypothesis) = &input.hypothesis {
        if let (Some(loc_conf), Some(planner_quality), Some(costmap_validity)) = (
            hypothesis.localization_confidence,
            hypothesis.planner_quality,
            hypothesis.costmap_validity,
        ) {
            report.root_cause = Some(RootCauseGenerator::generate_hypothesis(
                loc_conf,
                planner_quality,
                costmap_validity,
            ));
        }
    }

    report
}

/// Render a compact text report.
pub fn nav2_report_text(report: &Nav2Report) -> String {
    let mut text = String::from("Nav2 diagnostics\n");
    if let Some(cause) = &report.root_cause {
        text.push_str(&format!(
            "Root cause: {} ({:.0}% confidence)\n",
            cause.primary_cause,
            cause.confidence * 100.0
        ));
    }
    if let Some(classification) = &report.classification {
        text.push_str(&format!(
            "Classification: {:?} ({:.0}% confidence)\n",
            classification.limitation,
            classification.confidence * 100.0
        ));
    }
    for (label, count) in [
        ("Localization issues", report.localization_issues.len()),
        ("Planner issues", report.planner_issues.len()),
        ("Costmap issues", report.costmap_issues.len()),
        (
            "Dynamic-obstacle issues",
            report.dynamic_obstacle_issues.len(),
        ),
        ("Semantic gaps", report.semantic_gaps.len()),
    ] {
        text.push_str(&format!("{label}: {count}\n"));
    }
    if report.is_empty() {
        text.push_str("No Phase 15 findings for the supplied measurements.\n");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_all_present_analyzers() {
        let input: Nav2Diagnostics = serde_json::from_str(
            r#"{
              "localization": {
                "particle_spread_before": 0.3,
                "particle_spread_after": 1.8,
                "confidence": 0.3,
                "estimated_error": 0.9,
                "map_matches_per_scan": 4
              },
              "planner": {
                "replanning_frequency": 1.0,
                "time_since_goal_update": 10.0,
                "mean_plan_time_ms": 250.0,
                "success_rate": 0.4
              },
              "costmap": {
                "inflation_radius": 0.5,
                "robot_footprint": 0.2,
                "static_layer_size": 100,
                "dynamic_layer_size": 0
              },
              "dynamic_obstacle": {"human_detected": true, "distance_to_human": 1.2},
              "semantic": {"failure_near_door": true, "door_detected": true},
              "classification": {
                "failure_type": "door_crossing",
                "environment_type": "office",
                "scale_meters": 50.0
              },
              "hypothesis": {
                "localization_confidence": 0.3,
                "planner_quality": 0.9,
                "costmap_validity": 0.9
              }
            }"#,
        )
        .unwrap();

        let report = analyze_nav2_diagnostics(&input);
        assert_eq!(report.localization_issues.len(), 1);
        assert!(!report.planner_issues.is_empty());
        assert_eq!(report.costmap_issues.len(), 1);
        assert_eq!(report.dynamic_obstacle_issues.len(), 1);
        assert_eq!(report.semantic_gaps.len(), 1);
        assert!(report.classification.is_some());
        assert!(report.root_cause.is_some());
        assert!(report.summaries.localization.is_some());
        assert!(report.summaries.planner.is_some());
        assert!(report.summaries.costmap.is_some());
        assert!(nav2_report_text(&report).contains("Root cause"));
    }

    #[test]
    fn empty_measurements_produce_no_findings() {
        let report = analyze_nav2_diagnostics(&Nav2Diagnostics::default());
        assert!(report.is_empty());
    }

    #[test]
    fn partial_measurements_do_not_panic() {
        let input = Nav2Diagnostics {
            localization: LocalizationMetrics {
                particle_spread_before: Some(0.5),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(analyze_nav2_diagnostics(&input).is_empty());
    }
}
