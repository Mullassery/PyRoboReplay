//! Root cause hypothesis generation

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseHypothesis {
    pub category: String,
    pub primary_cause: String,
    pub confidence: f32,
    pub contributing_factors: Vec<String>,
}

pub struct RootCauseGenerator;

impl RootCauseGenerator {
    pub fn generate_hypothesis(
        localization_confidence: f32,
        planner_quality: f32,
        costmap_validity: f32,
    ) -> RootCauseHypothesis {
        let mut primary_cause = "Unknown".to_string();
        let mut confidence = 0.5;
        let mut contributing_factors = Vec::new();

        if localization_confidence < 0.5 {
            primary_cause = "Localization failure".to_string();
            confidence = 1.0 - localization_confidence;
            contributing_factors.push("AMCL divergence likely".to_string());
        } else if planner_quality < 0.6 {
            primary_cause = "Planner oscillation".to_string();
            confidence = 1.0 - planner_quality;
            contributing_factors.push("Excessive replanning detected".to_string());
        } else if costmap_validity < 0.7 {
            primary_cause = "Costmap inflation".to_string();
            confidence = 1.0 - costmap_validity;
            contributing_factors.push("Blocking valid paths".to_string());
        }

        RootCauseHypothesis {
            category: "Navigation".to_string(),
            primary_cause,
            confidence: confidence.clamp(0.0, 1.0),
            contributing_factors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypothesis_generation() {
        let hyp = RootCauseGenerator::generate_hypothesis(0.3, 0.8, 0.8);
        assert!(hyp.confidence > 0.5);
    }
}
