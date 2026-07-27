//! Dynamic obstacle failure analysis

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObstacleCause {
    PredictionFailure,
    HumanInterference,
    MovingObstacle,
    UnexpectedDynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObstacleIssue {
    pub cause: ObstacleCause,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

pub struct DynamicObstacleAnalyzer;

impl DynamicObstacleAnalyzer {
    pub fn detect_human_obstruction(
        human_detected: bool,
        distance_to_human: f32,
    ) -> Option<ObstacleIssue> {
        if human_detected && distance_to_human < 2.0 {
            return Some(ObstacleIssue {
                cause: ObstacleCause::HumanInterference,
                confidence: 0.90,
                evidence: vec![
                    format!("Human detected at {:.2}m", distance_to_human),
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
    fn test_human_detection() {
        let issue = DynamicObstacleAnalyzer::detect_human_obstruction(true, 1.5);
        assert!(issue.is_some());
    }
}
