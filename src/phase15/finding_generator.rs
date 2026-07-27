//! Structured finding and recommendation generation

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseFinding {
    pub category: String,
    pub confidence: f32,
    pub evidence_trail: Vec<String>,
    pub recommendations: Vec<Recommendation>,
    pub nav2_limitation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub tier: RecommendationTier,
    pub title: String,
    pub description: String,
    pub effort_days: f32,
    pub impact: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationTier {
    Tuning,
    Capability,
    Architecture,
}

pub struct FindingGenerator;

impl FindingGenerator {
    pub fn generate_finding(
        root_cause: String,
        confidence: f32,
        evidence: Vec<String>,
    ) -> RootCauseFinding {
        RootCauseFinding {
            category: "Navigation".to_string(),
            confidence,
            evidence_trail: evidence,
            recommendations: vec![
                Recommendation {
                    tier: RecommendationTier::Tuning,
                    title: "Tune planner parameters".to_string(),
                    description: "Adjust critic weights and update frequencies".to_string(),
                    effort_days: 1.0,
                    impact: 0.3,
                },
            ],
            nav2_limitation: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_generation() {
        let finding = FindingGenerator::generate_finding(
            "Localization divergence".to_string(),
            0.85,
            vec!["Particle spread increased".to_string()],
        );
        assert!(finding.confidence > 0.8);
    }
}
