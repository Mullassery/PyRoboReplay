//! Failure detection via pattern recognition
//!
//! Identifies common navigation failure patterns using heuristics:
//! - Sudden motion stops
//! - Localization divergence
//! - Path oscillation
//! - Recovery loop execution
//! - Timeout/deadline exceeded
//! - Safety constraint violations

use crate::phase14::timeline_indexing::{TimelineEvent, Modality};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureType {
    /// Robot stopped moving unexpectedly
    SuddenStop,
    /// Localization confidence dropped sharply
    LocalizationDivergence,
    /// Planner generated oscillating/contradictory paths
    PathOscillation,
    /// Recovery behavior triggered (backtrack, clear costmap, etc.)
    RecoveryTriggered,
    /// Mission exceeded time budget
    TimeoutExceeded,
    /// Predicted collision too close
    SafetyCritical,
    /// Cannot find valid path to goal
    PlanningDeadlock,
    /// Robot spinning in place (yaw oscillation)
    SpinningInPlace,
}

impl std::fmt::Display for FailureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureType::SuddenStop => write!(f, "Sudden Stop"),
            FailureType::LocalizationDivergence => write!(f, "Localization Divergence"),
            FailureType::PathOscillation => write!(f, "Path Oscillation"),
            FailureType::RecoveryTriggered => write!(f, "Recovery Triggered"),
            FailureType::TimeoutExceeded => write!(f, "Timeout Exceeded"),
            FailureType::SafetyCritical => write!(f, "Safety Critical"),
            FailureType::PlanningDeadlock => write!(f, "Planning Deadlock"),
            FailureType::SpinningInPlace => write!(f, "Spinning In Place"),
        }
    }
}

/// Detected failure pattern with timing and confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub failure_type: FailureType,
    /// When failure occurred (ROS time in ns)
    pub timestamp: i64,
    /// Confidence in this detection (0-1)
    pub confidence: f32,
    /// Duration of failure condition (ns)
    pub duration: i64,
    /// Supporting evidence (metric names and values)
    pub evidence: Vec<(String, f32)>,
}

/// Detects failures from timeline events
pub struct FailureDetector;

impl FailureDetector {
    /// Detect sudden stops: cmd_vel suddenly becomes zero
    pub fn detect_sudden_stop(
        cmd_vel_before: f32,
        cmd_vel_after: f32,
        timestamp: i64,
    ) -> Option<FailurePattern> {
        if cmd_vel_before > 0.1 && cmd_vel_after < 0.01 {
            return Some(FailurePattern {
                failure_type: FailureType::SuddenStop,
                timestamp,
                confidence: 0.85,
                duration: 0,
                evidence: vec![
                    ("cmd_vel_before".to_string(), cmd_vel_before),
                    ("cmd_vel_after".to_string(), cmd_vel_after),
                ],
            });
        }
        None
    }

    /// Detect localization divergence: pose confidence drops + error increases
    pub fn detect_localization_divergence(
        confidence_before: f32,
        confidence_after: f32,
        error_before: f32,
        error_after: f32,
        timestamp: i64,
    ) -> Option<FailurePattern> {
        let conf_drop = confidence_before - confidence_after;
        let error_rise = error_after - error_before;

        if conf_drop > 0.3 && error_rise > 0.2 {
            return Some(FailurePattern {
                failure_type: FailureType::LocalizationDivergence,
                timestamp,
                confidence: 0.90,
                duration: 0,
                evidence: vec![
                    ("confidence_drop".to_string(), conf_drop),
                    ("error_rise".to_string(), error_rise),
                ],
            });
        }
        None
    }

    /// Detect path oscillation: planner generates many contradictory plans rapidly
    pub fn detect_path_oscillation(
        plan_count: u32,
        time_window_ms: u32,
        average_plan_divergence: f32,
        timestamp: i64,
    ) -> Option<FailurePattern> {
        let planning_freq = (plan_count as f32 / time_window_ms as f32) * 1000.0; // Hz

        if planning_freq > 0.5 && average_plan_divergence > 0.4 {
            return Some(FailurePattern {
                failure_type: FailureType::PathOscillation,
                timestamp,
                confidence: 0.80,
                duration: time_window_ms as i64 * 1_000_000,
                evidence: vec![
                    ("planning_frequency".to_string(), planning_freq),
                    ("plan_divergence".to_string(), average_plan_divergence),
                ],
            });
        }
        None
    }

    /// Detect recovery behavior: specific log messages or costmap clear events
    pub fn detect_recovery_triggered(
        log_message: &str,
        timestamp: i64,
    ) -> Option<FailurePattern> {
        let recovery_keywords = [
            "recovery",
            "backtrack",
            "clear_costmap",
            "spin",
            "forward",
            "dynamic_reconfigure",
        ];

        let triggered = recovery_keywords.iter()
            .any(|kw| log_message.to_lowercase().contains(kw));

        if triggered {
            return Some(FailurePattern {
                failure_type: FailureType::RecoveryTriggered,
                timestamp,
                confidence: 0.95,
                duration: 0,
                evidence: vec![
                    ("log_message_length".to_string(), log_message.len() as f32),
                ],
            });
        }
        None
    }

    /// Detect timeout: mission duration exceeded budget
    pub fn detect_timeout(
        mission_duration_ms: u32,
        budget_ms: u32,
        timestamp: i64,
    ) -> Option<FailurePattern> {
        if mission_duration_ms > budget_ms {
            let overage = mission_duration_ms - budget_ms;
            return Some(FailurePattern {
                failure_type: FailureType::TimeoutExceeded,
                timestamp,
                confidence: 1.0,
                duration: overage as i64 * 1_000_000,
                evidence: vec![
                    ("mission_duration_ms".to_string(), mission_duration_ms as f32),
                    ("budget_ms".to_string(), budget_ms as f32),
                    ("overage_ms".to_string(), overage as f32),
                ],
            });
        }
        None
    }

    /// Detect safety critical: predicted collision distance < threshold
    pub fn detect_safety_critical(
        collision_distance_m: f32,
        safety_margin_m: f32,
        timestamp: i64,
    ) -> Option<FailurePattern> {
        if collision_distance_m < safety_margin_m {
            let margin_violation = safety_margin_m - collision_distance_m;
            return Some(FailurePattern {
                failure_type: FailureType::SafetyCritical,
                timestamp,
                confidence: 0.98,
                duration: 0,
                evidence: vec![
                    ("collision_distance".to_string(), collision_distance_m),
                    ("safety_margin".to_string(), safety_margin_m),
                    ("violation".to_string(), margin_violation),
                ],
            });
        }
        None
    }

    /// Detect planning deadlock: repeated failed plan attempts
    pub fn detect_planning_deadlock(
        failed_plan_attempts: u32,
        time_window_ms: u32,
        timestamp: i64,
    ) -> Option<FailurePattern> {
        if failed_plan_attempts > 5 && time_window_ms < 5000 {
            return Some(FailurePattern {
                failure_type: FailureType::PlanningDeadlock,
                timestamp,
                confidence: 0.85,
                duration: time_window_ms as i64 * 1_000_000,
                evidence: vec![
                    ("failed_attempts".to_string(), failed_plan_attempts as f32),
                    ("time_window_ms".to_string(), time_window_ms as f32),
                ],
            });
        }
        None
    }

    /// Detect spinning: yaw changing rapidly but x,y position static
    pub fn detect_spinning_in_place(
        yaw_rate: f32,
        linear_velocity: f32,
        timestamp: i64,
    ) -> Option<FailurePattern> {
        if yaw_rate.abs() > 2.0 && linear_velocity < 0.05 {
            return Some(FailurePattern {
                failure_type: FailureType::SpinningInPlace,
                timestamp,
                confidence: 0.80,
                duration: 0,
                evidence: vec![
                    ("yaw_rate".to_string(), yaw_rate),
                    ("linear_velocity".to_string(), linear_velocity),
                ],
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_sudden_stop() {
        let pattern = FailureDetector::detect_sudden_stop(1.5, 0.0, 1000);
        assert!(pattern.is_some());
        let p = pattern.unwrap();
        assert_eq!(p.failure_type, FailureType::SuddenStop);
        assert!(p.confidence > 0.8);
    }

    #[test]
    fn test_detect_localization_divergence() {
        let pattern = FailureDetector::detect_localization_divergence(0.95, 0.50, 0.1, 0.5, 1000);
        assert!(pattern.is_some());
        let p = pattern.unwrap();
        assert_eq!(p.failure_type, FailureType::LocalizationDivergence);
    }

    #[test]
    fn test_detect_path_oscillation() {
        let pattern = FailureDetector::detect_path_oscillation(5, 5000, 0.6, 1000);
        assert!(pattern.is_some());
    }

    #[test]
    fn test_detect_recovery_triggered() {
        let pattern = FailureDetector::detect_recovery_triggered("Triggering recovery behavior: backtrack", 1000);
        assert!(pattern.is_some());
    }

    #[test]
    fn test_detect_timeout() {
        let pattern = FailureDetector::detect_timeout(6000, 5000, 1000);
        assert!(pattern.is_some());
        let p = pattern.unwrap();
        assert_eq!(p.failure_type, FailureType::TimeoutExceeded);
    }

    #[test]
    fn test_detect_safety_critical() {
        let pattern = FailureDetector::detect_safety_critical(0.1, 0.5, 1000);
        assert!(pattern.is_some());
    }

    #[test]
    fn test_detect_spinning() {
        let pattern = FailureDetector::detect_spinning_in_place(3.0, 0.02, 1000);
        assert!(pattern.is_some());
    }
}
