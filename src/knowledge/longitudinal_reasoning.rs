//! Longitudinal Reasoning: Understanding Evolution Across Missions
//!
//! Compares current mission against historical context to enable
//! higher-level reasoning like "What changed?" and "Is this anomalous?"

use crate::knowledge::world_model::WorldState;

/// Cross-mission reasoning result
#[derive(Debug, Clone)]
pub struct LongitudinalInsight {
    /// What insight was discovered
    pub insight: String,

    /// How many missions of history informed this
    pub history_depth: usize,

    /// Confidence in this insight
    pub confidence: f32,

    /// Is this insight anomalous
    pub is_anomalous: bool,

    /// Suggested explanation
    pub explanation: String,

    /// Recommended action
    pub recommendation: Option<String>,
}

/// Longitudinal analyzer
pub struct LongitudinalAnalyzer;

impl LongitudinalAnalyzer {
    /// Analyze current mission against historical baseline
    pub fn compare_to_baseline(
        historical: &WorldState,
        current_observations_count: usize,
        anomalies_detected: usize,
    ) -> Vec<LongitudinalInsight> {
        let mut insights = Vec::new();

        let baseline_observation_count = historical.baseline_observations.len();
        if baseline_observation_count == 0 {
            return vec![LongitudinalInsight {
                insight: "Insufficient historical data to compare".to_string(),
                history_depth: 0,
                confidence: 0.0,
                is_anomalous: false,
                explanation: "First mission in this environment".to_string(),
                recommendation: None,
            }];
        }

        // Check if current observations deviate from baseline
        let observation_ratio =
            current_observations_count as f32 / baseline_observation_count as f32;
        if observation_ratio > 1.5 {
            insights.push(LongitudinalInsight {
                insight: format!(
                    "Unusually high observation count: {:.0}% above baseline",
                    (observation_ratio - 1.0) * 100.0
                ),
                history_depth: historical.baseline_observations.len(),
                confidence: 0.8,
                is_anomalous: true,
                explanation: "Environment contains more objects than typical".to_string(),
                recommendation: Some("Investigate for new obstacles or hazards".to_string()),
            });
        } else if observation_ratio < 0.7 {
            insights.push(LongitudinalInsight {
                insight: "Fewer observations than baseline".to_string(),
                history_depth: historical.baseline_observations.len(),
                confidence: 0.75,
                is_anomalous: true,
                explanation: "Some known entities not detected".to_string(),
                recommendation: Some("Check if entities have been moved or removed".to_string()),
            });
        }

        // Anomaly spike detection
        let baseline_anomalies = historical.known_anomalies.len();
        if anomalies_detected > baseline_anomalies * 2 {
            insights.push(LongitudinalInsight {
                insight: format!(
                    "Anomaly spike: {} detected vs {} historical average",
                    anomalies_detected, baseline_anomalies
                ),
                history_depth: historical.known_anomalies.len(),
                confidence: 0.85,
                is_anomalous: true,
                explanation: "Environment is significantly different from norm".to_string(),
                recommendation: Some("Conduct full environmental assessment".to_string()),
            });
        }

        insights
    }

    /// Predict expected behavior based on history
    pub fn predict_based_on_history(
        historical: &WorldState,
        entity_id: &str,
    ) -> Option<String> {
        if let Some(entity) = historical.entities.get(entity_id) {
            if entity.observation_count > 3 {
                // Sufficient history
                let primary_location = entity
                    .known_locations
                    .iter()
                    .max_by_key(|loc| loc.observation_count)
                    .map(|loc| loc.location_id.clone());

                return primary_location.map(|loc| {
                    format!(
                        "Based on {} observations, {} is typically at {}",
                        entity.observation_count, entity_id, loc
                    )
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_world() -> WorldState {
        WorldState {
            environment_id: "test".to_string(),
            entities: HashMap::new(),
            locations: HashMap::new(),
            temporal_facts: Vec::new(),
            baseline_observations: vec![
                Default::default(),
                Default::default(),
                Default::default(),
            ],
            known_anomalies: vec![Default::default()],
            last_updated_sec: 0.0,
        }
    }

    #[test]
    fn test_baseline_comparison() {
        let world = create_test_world();
        let insights = LongitudinalAnalyzer::compare_to_baseline(&world, 6, 3);

        assert!(!insights.is_empty());
    }
}
