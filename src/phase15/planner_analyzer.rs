//! Planner failure analysis
//!
//! Detects and categorizes planner issues:
//! - Path oscillation (replanning too frequently)
//! - Deadlock (cannot find valid path)
//! - Excessive backtracking
//! - Narrow corridor navigation failures
//! - Local minima traps

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlannerCause {
    /// Planner generating many contradictory routes
    ExcessiveReplanning,
    /// Robot oscillates between forward and backward plans
    RouteOscillation,
    /// Cannot find valid path despite trying
    LocalMinima,
    /// Multiple waypoint revisions without progress
    WaypointThrashing,
    /// Critic weights causing suboptimal solutions
    CriticWeightIssue,
    /// Goal unreachable given costmap
    UnreachableGoal,
    /// Narrow corridor causing frequent replans
    NarrowCorridorIssue,
}

impl std::fmt::Display for PlannerCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlannerCause::ExcessiveReplanning => write!(f, "Excessive Replanning"),
            PlannerCause::RouteOscillation => write!(f, "Route Oscillation"),
            PlannerCause::LocalMinima => write!(f, "Local Minima"),
            PlannerCause::WaypointThrashing => write!(f, "Waypoint Thrashing"),
            PlannerCause::CriticWeightIssue => write!(f, "Critic Weight Issue"),
            PlannerCause::UnreachableGoal => write!(f, "Unreachable Goal"),
            PlannerCause::NarrowCorridorIssue => write!(f, "Narrow Corridor Issue"),
        }
    }
}

/// Planner issue with diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerIssue {
    pub cause: PlannerCause,
    pub confidence: f32,
    pub replanning_frequency: f32,  // Hz
    pub oscillation_magnitude: f32, // meters
    pub evidence: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Analyzes planner failures
pub struct PlannerAnalyzer;

impl PlannerAnalyzer {
    /// Analyze excessive replanning
    pub fn analyze_excessive_replanning(
        replanning_frequency: f32,  // Hz
        time_since_goal_update: f32, // seconds
    ) -> Option<PlannerIssue> {
        if replanning_frequency > 0.5 && time_since_goal_update > 5.0 {
            return Some(PlannerIssue {
                cause: PlannerCause::ExcessiveReplanning,
                confidence: 0.85,
                replanning_frequency,
                oscillation_magnitude: 0.0,
                evidence: vec![
                    format!("Planner replanning at {:.2} Hz", replanning_frequency),
                    format!("Goal unchanged for {:.1} seconds", time_since_goal_update),
                    "Suggests environment instability or poor planning parameters".to_string(),
                ],
                recommendations: vec![
                    "Increase planner update period (default 0.05s too frequent)".to_string(),
                    "Increase path commitment cost in critic weights".to_string(),
                    "Decrease costmap update frequency".to_string(),
                    "Verify dynamic obstacle update rate".to_string(),
                ],
            });
        }
        None
    }

    /// Analyze route oscillation
    pub fn analyze_route_oscillation(
        plan_divergence: f32,  // average distance between consecutive plans
        oscillation_magnitude: f32, // peak-to-peak in meters
    ) -> Option<PlannerIssue> {
        if plan_divergence > 0.3 && oscillation_magnitude > 0.5 {
            return Some(PlannerIssue {
                cause: PlannerCause::RouteOscillation,
                confidence: 0.82,
                replanning_frequency: 0.0,
                oscillation_magnitude,
                evidence: vec![
                    format!("Average plan divergence: {:.2}m", plan_divergence),
                    format!("Oscillation magnitude: {:.2}m peak-to-peak", oscillation_magnitude),
                    "Robot tracing back and forth rather than progressing".to_string(),
                ],
                recommendations: vec![
                    "Reduce oscillation with path smoothing (curve_radius increase)".to_string(),
                    "Increase cost of plan changes in critic".to_string(),
                    "Debug costmap inflation (may be falsely blocking paths)".to_string(),
                    "Consider trajectory rollout instead of DWA".to_string(),
                ],
            });
        }
        None
    }

    /// Analyze goal reachability
    pub fn analyze_unreachable_goal(
        plan_attempts: u32,
        failed_attempts: u32,
        distance_to_goal: f32,
    ) -> Option<PlannerIssue> {
        let failure_rate = if plan_attempts > 0 {
            failed_attempts as f32 / plan_attempts as f32
        } else {
            0.0
        };

        if failure_rate > 0.7 && failed_attempts > 5 {
            return Some(PlannerIssue {
                cause: PlannerCause::UnreachableGoal,
                confidence: 0.90,
                replanning_frequency: 0.0,
                oscillation_magnitude: distance_to_goal,
                evidence: vec![
                    format!("Failed {}/{} plan attempts ({:.0}%)",
                            failed_attempts, plan_attempts, failure_rate * 100.0),
                    format!("Distance to goal: {:.2}m", distance_to_goal),
                ],
                recommendations: vec![
                    "Verify goal is reachable (collision-free path exists)".to_string(),
                    "Check costmap inflation isn't blocking valid paths".to_string(),
                    "Increase max planning time if timeout is culprit".to_string(),
                    "Relax goal tolerance if robot is close enough".to_string(),
                ],
            });
        }
        None
    }

    /// Analyze narrow corridor navigation
    pub fn analyze_narrow_corridor(
        corridor_width: f32,
        robot_footprint: f32,
        replan_count_in_corridor: u32,
    ) -> Option<PlannerIssue> {
        let clearance = corridor_width - robot_footprint;

        if clearance < 0.3 && replan_count_in_corridor > 3 {
            return Some(PlannerIssue {
                cause: PlannerCause::NarrowCorridorIssue,
                confidence: 0.80,
                replanning_frequency: replan_count_in_corridor as f32 / 30.0,  // Assuming 30 seconds
                oscillation_magnitude: clearance,
                evidence: vec![
                    format!("Corridor width: {:.2}m, Robot footprint: {:.2}m, Clearance: {:.2}m",
                            corridor_width, robot_footprint, clearance),
                    format!("Replanned {} times while traversing corridor", replan_count_in_corridor),
                ],
                recommendations: vec![
                    "Reduce costmap inflation radius (currently too conservative)".to_string(),
                    "Deploy tighter trajectory rollouts for narrow spaces".to_string(),
                    "Add spatial awareness to reduce replanning in tight spaces".to_string(),
                    "Consider side-ways entry strategies for very narrow passages".to_string(),
                ],
            });
        }
        None
    }

    /// Summarize planner health
    pub fn summarize_planner(
        mean_plan_time_ms: f32,
        success_rate: f32,
        replanning_frequency: f32,
    ) -> String {
        let mut summary = String::new();

        if success_rate < 0.5 {
            summary.push_str("❌ CRITICAL: Planner failing most plan attempts\n");
        } else if success_rate < 0.8 {
            summary.push_str("⚠️  WARNING: Planner success rate below 80%\n");
        } else {
            summary.push_str("✅ Planner nominal\n");
        }

        if replanning_frequency > 0.5 {
            summary.push_str(&format!("   High replanning frequency: {:.2} Hz\n", replanning_frequency));
        }

        if mean_plan_time_ms > 200.0 {
            summary.push_str(&format!("   Slow planning: {:.0}ms average\n", mean_plan_time_ms));
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excessive_replanning_detection() {
        let issue = PlannerAnalyzer::analyze_excessive_replanning(1.0, 10.0);
        assert!(issue.is_some());
        let i = issue.unwrap();
        assert_eq!(i.cause, PlannerCause::ExcessiveReplanning);
    }

    #[test]
    fn test_route_oscillation_detection() {
        let issue = PlannerAnalyzer::analyze_route_oscillation(0.5, 0.8);
        assert!(issue.is_some());
    }

    #[test]
    fn test_unreachable_goal_detection() {
        let issue = PlannerAnalyzer::analyze_unreachable_goal(10, 8, 5.0);
        assert!(issue.is_some());
    }

    #[test]
    fn test_narrow_corridor_detection() {
        let issue = PlannerAnalyzer::analyze_narrow_corridor(0.7, 0.5, 5);
        assert!(issue.is_some());
        let i = issue.unwrap();
        assert_eq!(i.cause, PlannerCause::NarrowCorridorIssue);
    }

    #[test]
    fn test_summarize_planner_critical() {
        let summary = PlannerAnalyzer::summarize_planner(50.0, 0.3, 0.2);
        assert!(summary.contains("CRITICAL"));
    }
}
