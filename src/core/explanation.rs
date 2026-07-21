use crate::core::anomaly_detector::Failure;
use std::collections::HashMap;

/// Generates human-readable explanations for failures
pub struct ExplanationGenerator;

impl ExplanationGenerator {
    /// Generate a natural language explanation for a failure
    pub fn explain(failure: &Failure) -> String {
        let base_explanation = match failure.failure_type.as_str() {
            "near_collision" => Self::explain_near_collision(failure),
            "perception_failure" => Self::explain_perception_failure(failure),
            "sensor_dropout" => Self::explain_sensor_dropout(failure),
            "communication_loss" => Self::explain_communication_loss(failure),
            "navigation_deadlock" => Self::explain_navigation_deadlock(failure),
            "localization_loss" => Self::explain_localization_loss(failure),
            "oscillation" => Self::explain_oscillation(failure),
            "costmap_anomaly" => Self::explain_costmap_anomaly(failure),
            _ => "Unknown failure type detected.".to_string(),
        };

        // Add severity context
        let severity_context = match failure.severity.as_str() {
            "critical" => "This is a critical safety issue requiring immediate attention.",
            "high" => "This is a significant issue that may cause mission failure.",
            "medium" => "This issue may impact mission performance but is not critical.",
            "low" => "This is a minor issue that should be monitored.",
            _ => "",
        };

        format!("{} {}", base_explanation, severity_context)
    }

    fn explain_near_collision(failure: &Failure) -> String {
        let min_range = failure
            .evidence
            .get("min_range_m")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let threshold = failure
            .evidence
            .get("threshold_m")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        format!(
            "The robot detected an obstacle at {:.2}m distance (alert threshold: {:.2}m). \
             The LiDAR sensor triggered a collision warning, causing the planner to halt movement. \
             This is typically correct behavior unless the obstacle is a false positive.",
            min_range, threshold
        )
    }

    fn explain_perception_failure(failure: &Failure) -> String {
        let low_conf_count = failure
            .evidence
            .get("low_confidence_count")
            .cloned()
            .unwrap_or_else(|| "multiple".to_string());
        let total = failure
            .evidence
            .get("total_detections")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        format!(
            "The camera perception system produced {} low-confidence detections out of {} total detections. \
             This may indicate poor lighting, occlusion, or objects at the detection boundary. \
             The robot may fail to recognize important objects or generate false positives.",
            low_conf_count, total
        )
    }

    fn explain_sensor_dropout(failure: &Failure) -> String {
        let sensor = failure
            .evidence
            .get("sensor")
            .cloned()
            .unwrap_or_else(|| "a sensor".to_string());
        let gap = failure
            .evidence
            .get("gap_seconds")
            .cloned()
            .unwrap_or_else(|| "several seconds".to_string());

        format!(
            "The {} sensor stopped reporting data for {}. \
             This gap may indicate a hardware failure, network issue, or driver crash. \
             During this period, the robot was flying blind and unable to perceive its environment correctly.",
            sensor, gap
        )
    }

    fn explain_communication_loss(failure: &Failure) -> String {
        let max_gap = failure
            .evidence
            .get("max_gap_s")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let avg_gap = failure
            .evidence
            .get("avg_gap_s")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        format!(
            "Message communication experienced a gap of {} seconds (compared to typical {} second intervals). \
             This suggests temporary network congestion, packet loss, or processing delays. \
             The robot may have missed critical sensor updates or control commands.",
            max_gap, avg_gap
        )
    }

    fn explain_navigation_deadlock(failure: &Failure) -> String {
        let replan_count = failure
            .evidence
            .get("replan_count")
            .cloned()
            .unwrap_or_else(|| "many".to_string());

        format!(
            "The navigation system performed {} path replanning attempts in rapid succession. \
             This indicates the robot detected obstacles, attempted to navigate around them, but kept encountering new obstacles. \
             The robot is likely stuck in a confined space or surrounded by dynamic obstacles it cannot navigate through.",
            replan_count
        )
    }

    fn explain_localization_loss(failure: &Failure) -> String {
        let confidence = failure
            .evidence
            .get("confidence")
            .cloned()
            .unwrap_or_else(|| "low".to_string());

        format!(
            "The robot's localization confidence dropped to {} (below the 50% safe threshold). \
             This means the odometry (position estimate) has high uncertainty. \
             Causes may include: GPS signal loss, wheel slippage, IMU drift, or SLAM system divergence. \
             The robot cannot reliably know where it is.",
            confidence
        )
    }

    fn explain_oscillation(failure: &Failure) -> String {
        let direction_changes = failure
            .evidence
            .get("direction_changes")
            .cloned()
            .unwrap_or_else(|| "many".to_string());
        let velocity = failure
            .evidence
            .get("velocity_m_s")
            .cloned()
            .unwrap_or_else(|| "minimal".to_string());

        format!(
            "The robot moved back and forth {} times without making forward progress (velocity: {} m/s). \
             This oscillating behavior typically means the robot is stuck trying to navigate through a narrow passage, \
             or the planner is oscillating between conflicting objectives (e.g., reaching goal vs. avoiding detected obstacles).",
            direction_changes, velocity
        )
    }

    fn explain_costmap_anomaly(failure: &Failure) -> String {
        let current = failure
            .evidence
            .get("current_obstacles")
            .cloned()
            .unwrap_or_else(|| "many".to_string());
        let previous = failure
            .evidence
            .get("previous_obstacles")
            .cloned()
            .unwrap_or_else(|| "few".to_string());

        format!(
            "The costmap (obstacle map) changed dramatically from {} obstacles to {} obstacles. \
             This sudden change may indicate: a sensor glitch, new dynamic obstacles entering the scene, \
             or a change in sensor filtering parameters. The planning system should re-evaluate navigation decisions.",
            previous, current
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_explain_near_collision() {
        let failure = Failure::new(
            "near_collision".to_string(),
            Utc::now(),
            0.95,
            "high".to_string(),
            "LiDAR detected obstacle at 0.30m".to_string(),
        );

        let explanation = ExplanationGenerator::explain(&failure);
        assert!(explanation.contains("obstacle"));
        assert!(explanation.contains("collision"));
    }

    #[test]
    fn test_explain_includes_severity() {
        let failure = Failure::new(
            "perception_failure".to_string(),
            Utc::now(),
            0.65,
            "critical".to_string(),
            "Low confidence detections".to_string(),
        );

        let explanation = ExplanationGenerator::explain(&failure);
        assert!(explanation.contains("critical"));
    }

    #[test]
    fn test_all_failure_types_have_explanations() {
        let failure_types = vec![
            "near_collision",
            "perception_failure",
            "sensor_dropout",
            "communication_loss",
            "navigation_deadlock",
            "localization_loss",
            "oscillation",
            "costmap_anomaly",
        ];

        for failure_type in failure_types {
            let failure = Failure::new(
                failure_type.to_string(),
                Utc::now(),
                0.75,
                "medium".to_string(),
                "Test failure".to_string(),
            );

            let explanation = ExplanationGenerator::explain(&failure);
            assert!(!explanation.is_empty(), "No explanation for {}", failure_type);
            assert!(!explanation.contains("Unknown"), "Unknown failure type: {}", failure_type);
        }
    }
}
