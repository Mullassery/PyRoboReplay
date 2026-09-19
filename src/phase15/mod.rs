//! Phase 15: Root Cause Inference Engine
//!
//! Builds on Phase 14's temporal fusion to generate AI-powered root cause analysis.
//! Analyzes navigation failures across 7 dimensions:
//! - Localization (AMCL divergence, odometry drift, sensor degradation)
//! - Planner (oscillation, deadlock, excessive replanning)
//! - Costmap (inflation, layer conflicts, false blockages)
//! - Dynamic obstacles (prediction gaps, collision avoidance failures)
//! - Semantic gaps (occupancy-grid limitations)
//! - Environmental context (multi-floor, scale, warehouse-specific)
//! - Controller stability (tracking error, command oscillation)
//!
//! Outputs structured findings with:
//! - Root cause category and confidence (0-1)
//! - Evidence trails with supporting data
//! - Nav2 architectural limitation classification
//! - Actionable recommendations (tuning, capability, architecture)

pub mod costmap_analyzer;
pub mod dynamic_obstacle_analyzer;
pub mod failure_detector;
pub mod finding_generator;
pub mod localization_analyzer;
pub mod nav2_limitation_detector;
pub mod planner_analyzer;
pub mod root_cause_generator;
pub mod semantic_gap_analyzer;

// Re-exports for public API
pub use costmap_analyzer::{CostmapAnalyzer, CostmapIssue};
pub use dynamic_obstacle_analyzer::{DynamicObstacleAnalyzer, ObstacleIssue};
pub use failure_detector::{FailureDetector, FailurePattern, FailureType};
pub use finding_generator::{FindingGenerator, Recommendation, RootCauseFinding};
pub use localization_analyzer::{LocalizationAnalyzer, LocalizationIssue};
pub use nav2_limitation_detector::{
    Nav2Limitation, Nav2LimitationDetection, Nav2LimitationDetector,
};
pub use planner_analyzer::{PlannerAnalyzer, PlannerIssue};
pub use root_cause_generator::{RootCauseGenerator, RootCauseHypothesis};
pub use semantic_gap_analyzer::{SemanticGap, SemanticGapAnalyzer};

#[cfg(test)]
mod tests {

    #[test]
    fn test_phase15_modules_accessible() {
        let _: () = ();
    }
}
