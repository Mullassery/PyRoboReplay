//! Multi-Mission Knowledge Accumulation
//!
//! Demonstrates longitudinal learning across missions:
//! - Mission 1: Learn baseline
//! - Mission 2-N: Compare against baseline, detect evolution
//!
//! Shows how persistent knowledge (Phase 10) + spatial grounding (Phase 10.2)
//! enables robot to understand environment dynamics across time.

use crate::knowledge::world_model::{WorldModelManager, Entity, EntityState};
use crate::knowledge::spatial_grounding::{SpatialCoordinates, SpatialGroundingEngine};
use crate::knowledge::longitudinal_reasoning::LongitudinalAnalyzer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mission-level context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionContext {
    /// Mission ID (unique identifier)
    pub mission_id: String,
    /// Timestamp when mission started
    pub start_time_sec: f32,
    /// Environment ID
    pub environment_id: String,
    /// Robot ID
    pub robot_id: String,
}

/// Multi-mission learning trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTrace {
    /// Mission context
    pub mission: MissionContext,
    /// Entities observed in this mission
    pub observed_entities: HashMap<String, (String, f32, SpatialCoordinates)>, // entity_id -> (type, confidence, coords)
    /// Anomalies detected
    pub anomalies_detected: usize,
    /// Insights from comparing to history
    pub longitudinal_insights: Vec<String>,
}

/// Learning progression across missions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningProgression {
    /// Environment ID
    pub environment_id: String,
    /// Mission traces in chronological order
    pub traces: Vec<MissionTrace>,
    /// Cumulative knowledge learned
    pub cumulative_insights: Vec<String>,
}

impl LearningProgression {
    /// Create new learning progression
    pub fn new(environment_id: &str) -> Self {
        LearningProgression {
            environment_id: environment_id.to_string(),
            traces: Vec::new(),
            cumulative_insights: Vec::new(),
        }
    }

    /// Add mission to learning progression
    pub fn add_mission(
        &mut self,
        trace: MissionTrace,
        world_model: &crate::knowledge::world_model::WorldState,
    ) {
        // Generate longitudinal insights
        let insights = LongitudinalAnalyzer::compare_to_baseline(
            world_model,
            trace.observed_entities.len(),
            trace.anomalies_detected,
        );

        let insight_strings: Vec<String> =
            insights.iter().map(|i| i.insight.clone()).collect();

        // Store trace with insights
        let mut enriched_trace = trace.clone();
        enriched_trace.longitudinal_insights = insight_strings.clone();

        self.traces.push(enriched_trace);

        // Accumulate insights
        self.cumulative_insights.extend(insight_strings);
    }

    /// Generate learning summary
    pub fn learning_summary(&self) -> String {
        let mut summary = format!(
            "Learning Progression for {}: {} missions\n",
            self.environment_id,
            self.traces.len()
        );

        for (idx, trace) in self.traces.iter().enumerate() {
            summary.push_str(&format!(
                "\nMission {}: {} ({}ms observed)\n",
                idx + 1,
                trace.mission.mission_id,
                trace.mission.start_time_sec as u32
            ));
            summary.push_str(&format!("  Entities: {}\n", trace.observed_entities.len()));
            summary.push_str(&format!("  Anomalies: {}\n", trace.anomalies_detected));

            if !trace.longitudinal_insights.is_empty() {
                summary.push_str("  Insights:\n");
                for insight in &trace.longitudinal_insights {
                    summary.push_str(&format!("    • {}\n", insight));
                }
            }
        }

        if !self.cumulative_insights.is_empty() {
            summary.push_str("\nCumulative Learning:\n");
            for insight in &self.cumulative_insights {
                summary.push_str(&format!("  • {}\n", insight));
            }
        }

        summary
    }
}

/// Multi-mission learning engine
pub struct MultiMissionLearner {
    /// World model (persistent across missions)
    pub world_model: WorldModelManager,
    /// Spatial grounding (persistent spatial coordinates)
    pub spatial_grounding: SpatialGroundingEngine,
    /// Learning progressions per environment
    pub progressions: HashMap<String, LearningProgression>,
}

impl MultiMissionLearner {
    /// Create new learner
    pub fn new() -> Self {
        MultiMissionLearner {
            world_model: WorldModelManager::new(),
            spatial_grounding: SpatialGroundingEngine::new(),
            progressions: HashMap::new(),
        }
    }

    /// Process mission with persistent learning
    pub fn process_mission(
        &mut self,
        mission: MissionContext,
        observations: Vec<(String, String, f32, SpatialCoordinates)>, // entity_id, type, confidence, coords
        anomalies_count: usize,
    ) -> MissionTrace {
        let env_id = mission.environment_id.clone();

        // Record observations in world model
        for (entity_id, entity_type, confidence, coords) in &observations {
            self.world_model.record_observation(
                &env_id,
                entity_id,
                entity_type,
                &format!("location_{}", env_id),
                mission.start_time_sec,
                &mission.mission_id,
                *confidence,
            );

            // Ground entity spatially
            self.spatial_grounding.update_entity(
                entity_id,
                entity_type,
                coords.clone(),
                *confidence,
            );
        }

        // Build observation map
        let mut observed_entities = HashMap::new();
        for (entity_id, entity_type, confidence, coords) in observations {
            observed_entities.insert(entity_id, (entity_type, confidence, coords));
        }

        // Create mission trace
        let trace = MissionTrace {
            mission,
            observed_entities,
            anomalies_detected: anomalies_count,
            longitudinal_insights: Vec::new(),
        };

        // Get current world state and add to progression
        let world = self.world_model.get_or_create_environment(&env_id);
        let progression = self
            .progressions
            .entry(env_id.clone())
            .or_insert_with(|| LearningProgression::new(&env_id));

        progression.add_mission(trace.clone(), world);

        trace
    }

    /// Get learning summary for environment
    pub fn get_progression_summary(&self, env_id: &str) -> Option<String> {
        self.progressions
            .get(env_id)
            .map(|prog| prog.learning_summary())
    }

    /// Predict entity behavior based on history
    pub fn predict_entity_location(&self, env_id: &str, entity_id: &str) -> Option<String> {
        if let Some(world) = self.world_model.get_environment(env_id) {
            crate::knowledge::longitudinal_reasoning::LongitudinalAnalyzer::predict_based_on_history(
                world, entity_id,
            )
        } else {
            None
        }
    }
}

impl Default for MultiMissionLearner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_progression_creation() {
        let prog = LearningProgression::new("warehouse_1");
        assert_eq!(prog.traces.len(), 0);
        assert_eq!(prog.environment_id, "warehouse_1");
    }

    #[test]
    fn test_multi_mission_learner_creation() {
        let learner = MultiMissionLearner::new();
        assert_eq!(learner.progressions.len(), 0);
    }

    #[test]
    fn test_single_mission_processing() {
        let mut learner = MultiMissionLearner::new();

        let mission = MissionContext {
            mission_id: "mission_001".to_string(),
            start_time_sec: 100.0,
            environment_id: "warehouse_1".to_string(),
            robot_id: "robot_1".to_string(),
        };

        let observations = vec![(
            "pallet_42".to_string(),
            "pallet".to_string(),
            0.95,
            SpatialCoordinates {
                x: 5.0,
                y: 5.0,
                z: 0.0,
            },
        )];

        let trace = learner.process_mission(mission, observations, 0);
        assert_eq!(trace.observed_entities.len(), 1);
        assert!(learner.progressions.contains_key("warehouse_1"));
    }

    #[test]
    fn test_multi_mission_learning() {
        let mut learner = MultiMissionLearner::new();

        // Mission 1: Baseline
        let mission1 = MissionContext {
            mission_id: "mission_001".to_string(),
            start_time_sec: 100.0,
            environment_id: "warehouse_1".to_string(),
            robot_id: "robot_1".to_string(),
        };

        let obs1 = vec![(
            "pallet_42".to_string(),
            "pallet".to_string(),
            0.95,
            SpatialCoordinates {
                x: 5.0,
                y: 5.0,
                z: 0.0,
            },
        )];

        learner.process_mission(mission1, obs1, 0);

        // Mission 2: Entity moved
        let mission2 = MissionContext {
            mission_id: "mission_002".to_string(),
            start_time_sec: 200.0,
            environment_id: "warehouse_1".to_string(),
            robot_id: "robot_1".to_string(),
        };

        let obs2 = vec![(
            "pallet_42".to_string(),
            "pallet".to_string(),
            0.92,
            SpatialCoordinates {
                x: 7.0,
                y: 5.0,
                z: 0.0,
            },
        )];

        learner.process_mission(mission2, obs2, 0);

        // Check learning
        let progression = learner.progressions.get("warehouse_1").unwrap();
        assert_eq!(progression.traces.len(), 2);

        // Check spatial grounding recorded movement
        let grounded = learner.spatial_grounding.entities.get("pallet_42").unwrap();
        assert!(grounded.distance_moved.is_some());
        assert!(grounded.distance_moved.unwrap() > 0.0);
    }

    #[test]
    fn test_longitudinal_prediction() {
        let mut learner = MultiMissionLearner::new();

        // Need 4+ observations for prediction
        for i in 0..4 {
            let mission = MissionContext {
                mission_id: format!("mission_{:03}", i),
                start_time_sec: 100.0 + (i as f32 * 100.0),
                environment_id: "warehouse_1".to_string(),
                robot_id: "robot_1".to_string(),
            };

            let observations = vec![(
                "pallet_42".to_string(),
                "pallet".to_string(),
                0.95,
                SpatialCoordinates {
                    x: 5.0 + (i as f32),
                    y: 5.0,
                    z: 0.0,
                },
            )];

            learner.process_mission(mission, observations, 0);
        }

        let prediction = learner.predict_entity_location("warehouse_1", "pallet_42");
        assert!(prediction.is_some());
    }

    #[test]
    fn test_progression_summary() {
        let mut learner = MultiMissionLearner::new();

        let mission = MissionContext {
            mission_id: "mission_001".to_string(),
            start_time_sec: 100.0,
            environment_id: "warehouse_1".to_string(),
            robot_id: "robot_1".to_string(),
        };

        let observations = vec![(
            "pallet_42".to_string(),
            "pallet".to_string(),
            0.95,
            SpatialCoordinates {
                x: 5.0,
                y: 5.0,
                z: 0.0,
            },
        )];

        learner.process_mission(mission, observations, 0);

        let summary = learner.get_progression_summary("warehouse_1");
        assert!(summary.is_some());
        let summary_text = summary.unwrap();
        assert!(summary_text.contains("warehouse_1"));
        assert!(summary_text.contains("mission_001"));
    }
}
