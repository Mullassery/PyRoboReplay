//! Semantic navigation gap detection

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticGap {
    pub location_type: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

pub struct SemanticGapAnalyzer;

impl SemanticGapAnalyzer {
    pub fn detect_door_navigation(
        failure_near_door: bool,
        door_detected: bool,
    ) -> Option<SemanticGap> {
        if failure_near_door && door_detected {
            return Some(SemanticGap {
                location_type: "door".to_string(),
                confidence: 0.85,
                evidence: vec!["Failure correlates with door detection".to_string()],
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_door_detection() {
        let gap = SemanticGapAnalyzer::detect_door_navigation(true, true);
        assert!(gap.is_some());
    }
}
