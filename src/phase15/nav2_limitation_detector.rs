//! Detect genuine Nav2 architectural limitations vs tuning/environment issues

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Nav2Limitation {
    /// Occupancy-grid navigation insufficient (semantic, topological needed)
    SemanticNavigationRequired,
    /// AMCL insufficient for environment (visual SLAM needed)
    LocalizationArchitectureLimitation,
    /// Single-level planner insufficient (hierarchical needed)
    HierarchicalPlanningNeeded,
    /// Occupancy-grid approach inherent limitation
    OccupancyGridLimitation,
    /// Not a Nav2 limitation (tuning or environment)
    TuningIssue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nav2LimitationDetection {
    pub limitation: Nav2Limitation,
    pub confidence: f32,
    pub explanation: String,
}

pub struct Nav2LimitationDetector;

impl Nav2LimitationDetector {
    pub fn classify(
        failure_type: &str,
        environment_type: &str,
        scale_meters: f32,
    ) -> Nav2LimitationDetection {
        let (limitation, confidence) = match (failure_type, environment_type) {
            ("door_crossing", _) => (Nav2Limitation::SemanticNavigationRequired, 0.92),
            ("elevator_coordination", _) => (Nav2Limitation::SemanticNavigationRequired, 0.95),
            ("desk_cluster_navigation", _) => (Nav2Limitation::SemanticNavigationRequired, 0.85),
            ("multi_floor", _) => (Nav2Limitation::SemanticNavigationRequired, 0.90),
            (_, "low_light") => (Nav2Limitation::LocalizationArchitectureLimitation, 0.88),
            (_, "feature_sparse") => (Nav2Limitation::LocalizationArchitectureLimitation, 0.82),
            (_, "warehouse") if scale_meters > 500.0 => {
                (Nav2Limitation::HierarchicalPlanningNeeded, 0.80)
            },
            (_, "outdoor_gps_denied") => (Nav2Limitation::LocalizationArchitectureLimitation, 0.85),
            _ => (Nav2Limitation::TuningIssue, 0.60),
        };

        let explanation = match limitation {
            Nav2Limitation::SemanticNavigationRequired => {
                "Failure indicates occupancy-grid navigation insufficient. \
                 Consider semantic mapping, object-centric navigation, or VLM-assisted planning."
                    .to_string()
            },
            Nav2Limitation::LocalizationArchitectureLimitation => {
                "AMCL with grid-based features insufficient. \
                 Deploy visual SLAM, VIO, or multi-sensor fusion."
                    .to_string()
            },
            Nav2Limitation::HierarchicalPlanningNeeded => {
                "Single-level planner unsuitable for large-scale environments. \
                 Implement hierarchical multi-level planning."
                    .to_string()
            },
            Nav2Limitation::OccupancyGridLimitation => {
                "Occupancy-grid approach inherent limitation. \
                 Consider topological or hybrid representations."
                    .to_string()
            },
            Nav2Limitation::TuningIssue => {
                "Failure likely solvable through parameter tuning or environment adaptation."
                    .to_string()
            },
        };

        Nav2LimitationDetection {
            limitation,
            confidence,
            explanation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_limitation_detection() {
        let result = Nav2LimitationDetector::classify("door_crossing", "office", 100.0);
        assert_eq!(result.limitation, Nav2Limitation::SemanticNavigationRequired);
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_tuning_classification() {
        let result = Nav2LimitationDetector::classify("path_oscillation", "open_space", 50.0);
        assert_eq!(result.limitation, Nav2Limitation::TuningIssue);
    }
}
