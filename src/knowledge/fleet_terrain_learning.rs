//! Fleet-Level Terrain Learning
//!
//! Multi-robot terrain understanding:
//! - Robot A learns zone X has 90% traversability
//! - Robot B learns zone X has 60% traversability (different weight, time)
//! - Fleet knows: Zone X varies by payload/weather/time
//!
//! Enables cross-robot learning and terrain consensus.

use crate::knowledge::terrain_integration::TerrainZone;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Single robot's traversability observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotTraversabilityObservation {
    /// Robot ID
    pub robot_id: String,
    /// Zone ID
    pub zone_id: String,
    /// Observed traversability (0.0-1.0)
    pub traversability: f32,
    /// Mission ID during observation
    pub mission_id: String,
    /// Timestamp
    pub timestamp_sec: f32,
    /// Confidence in this observation
    pub confidence: f32,
    /// Payload weight (kg, affects traversability)
    pub payload_kg: Option<f32>,
    /// Weather conditions
    pub weather_conditions: HashMap<String, f32>,
}

/// Consensus traversability from fleet observations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainConsensus {
    /// Zone ID
    pub zone_id: String,
    /// Consensus traversability (weighted average)
    pub consensus_traversability: f32,
    /// Standard deviation (uncertainty)
    pub std_dev: f32,
    /// Number of observations contributing
    pub observation_count: usize,
    /// Minimum observed traversability (worst case)
    pub min_traversability: f32,
    /// Maximum observed traversability (best case)
    pub max_traversability: f32,
    /// Confidence in consensus
    pub confidence: f32,
    /// Factors affecting consensus
    pub affecting_factors: Vec<String>,
}

/// Fleet terrain model
pub struct FleetTerrainModel {
    /// All observations per zone
    pub observations: HashMap<String, Vec<RobotTraversabilityObservation>>,
    /// Consensus per zone
    pub consensus: HashMap<String, TerrainConsensus>,
    /// Robot profiles
    pub robot_profiles: HashMap<String, RobotProfile>,
    /// Environment ID
    pub environment_id: String,
}

/// Robot profile for consensus weighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotProfile {
    /// Robot ID
    pub robot_id: String,
    /// Robot type (wheeled, legged, drone, etc.)
    pub robot_type: String,
    /// Sensor reliability (0.0-1.0)
    pub sensor_reliability: f32,
    /// Number of successful missions
    pub successful_missions: usize,
    /// Number of failures
    pub failed_missions: usize,
}

impl RobotProfile {
    /// Get reputation score (0.0-1.0) based on success history
    pub fn reputation_score(&self) -> f32 {
        let total = (self.successful_missions + self.failed_missions) as f32;
        if total == 0.0 {
            0.5 // Unknown robot
        } else {
            self.successful_missions as f32 / total
        }
    }

    /// Observation weight based on robot reliability and reputation
    pub fn observation_weight(&self) -> f32 {
        (self.sensor_reliability * 0.6 + self.reputation_score() * 0.4).max(0.1)
    }
}

impl FleetTerrainModel {
    /// Create new fleet terrain model
    pub fn new(environment_id: &str) -> Self {
        FleetTerrainModel {
            observations: HashMap::new(),
            consensus: HashMap::new(),
            robot_profiles: HashMap::new(),
            environment_id: environment_id.to_string(),
        }
    }

    /// Register robot in fleet
    pub fn register_robot(&mut self, robot: RobotProfile) {
        self.robot_profiles
            .insert(robot.robot_id.clone(), robot);
    }

    /// Record robot's zone traversability observation
    pub fn record_observation(&mut self, observation: RobotTraversabilityObservation) {
        // Store observation
        self.observations
            .entry(observation.zone_id.clone())
            .or_insert_with(Vec::new)
            .push(observation.clone());

        // Recompute consensus for this zone
        self.update_consensus(&observation.zone_id);
    }

    /// Compute consensus traversability from all observations
    fn update_consensus(&mut self, zone_id: &str) {
        if let Some(observations) = self.observations.get(zone_id) {
            if observations.is_empty() {
                return;
            }

            let mut weighted_sum = 0.0;
            let mut total_weight = 0.0;
            let mut min_trav = f32::INFINITY;
            let mut max_trav = f32::NEG_INFINITY;

            for obs in observations {
                let robot_weight = self
                    .robot_profiles
                    .get(&obs.robot_id)
                    .map(|r| r.observation_weight())
                    .unwrap_or(0.5);

                let weight = robot_weight * obs.confidence;
                weighted_sum += obs.traversability * weight;
                total_weight += weight;

                min_trav = min_trav.min(obs.traversability);
                max_trav = max_trav.max(obs.traversability);
            }

            let consensus_traversability = if total_weight > 0.0 {
                weighted_sum / total_weight
            } else {
                0.5
            };

            // Compute standard deviation
            let mut variance = 0.0;
            for obs in observations {
                let diff = obs.traversability - consensus_traversability;
                variance += diff * diff;
            }
            let std_dev = (variance / observations.len() as f32).sqrt();

            // Identify factors
            let mut affecting_factors = Vec::new();
            let has_payload_variance = observations
                .iter()
                .filter_map(|o| o.payload_kg)
                .collect::<Vec<_>>()
                .len()
                > 1;
            if has_payload_variance {
                affecting_factors.push("payload_dependent".to_string());
            }

            let weather_factors: std::collections::HashSet<String> = observations
                .iter()
                .flat_map(|o| o.weather_conditions.keys().cloned())
                .collect();
            for factor in weather_factors {
                affecting_factors.push(format!("weather_{}", factor));
            }

            let confidence = 1.0 - (std_dev * 0.5).min(1.0); // High std_dev = low confidence

            let consensus = TerrainConsensus {
                zone_id: zone_id.to_string(),
                consensus_traversability,
                std_dev,
                observation_count: observations.len(),
                min_traversability: min_trav,
                max_traversability: max_trav,
                confidence,
                affecting_factors,
            };

            self.consensus.insert(zone_id.to_string(), consensus);
        }
    }

    /// Get consensus for zone
    pub fn get_consensus(&self, zone_id: &str) -> Option<&TerrainConsensus> {
        self.consensus.get(zone_id)
    }

    /// Detect anomalous observations (outliers)
    pub fn find_anomalies(&self, zone_id: &str) -> Vec<RobotTraversabilityObservation> {
        let mut anomalies = Vec::new();

        if let Some(consensus) = self.consensus.get(zone_id) {
            if let Some(observations) = self.observations.get(zone_id) {
                for obs in observations {
                    let deviation = (obs.traversability - consensus.consensus_traversability).abs();
                    // Anomaly if 2+ standard deviations away
                    if deviation > consensus.std_dev * 2.0 {
                        anomalies.push(obs.clone());
                    }
                }
            }
        }

        anomalies
    }

    /// Recommend traversability threshold for zone
    pub fn recommend_threshold(&self, zone_id: &str, safety_margin: f32) -> Option<f32> {
        self.consensus.get(zone_id).map(|c| {
            // Recommend lower bound with safety margin
            (c.consensus_traversability - c.std_dev * safety_margin).max(0.0)
        })
    }

    /// Get fleet summary for zone
    pub fn zone_summary(&self, zone_id: &str) -> Option<String> {
        self.consensus.get(zone_id).map(|c| {
            let mut summary = format!("Zone {}: ", zone_id);
            summary.push_str(&format!(
                "Traversability {:.0}%±{:.0}% ({})",
                c.consensus_traversability * 100.0,
                c.std_dev * 100.0,
                c.observation_count
            ));

            if !c.affecting_factors.is_empty() {
                summary.push_str(&format!(" | Factors: {}", c.affecting_factors.join(", ")));
            }

            summary
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robot_profile_reputation() {
        let robot = RobotProfile {
            robot_id: "robot_1".to_string(),
            robot_type: "wheeled".to_string(),
            sensor_reliability: 0.9,
            successful_missions: 10,
            failed_missions: 2,
        };

        let reputation = robot.reputation_score();
        assert!((reputation - (10.0 / 12.0)).abs() < 0.01);
    }

    #[test]
    fn test_observation_weight() {
        let robot = RobotProfile {
            robot_id: "robot_1".to_string(),
            robot_type: "wheeled".to_string(),
            sensor_reliability: 0.8,
            successful_missions: 10,
            failed_missions: 2,
        };

        let weight = robot.observation_weight();
        assert!(weight > 0.5);
    }

    #[test]
    fn test_fleet_model_creation() {
        let model = FleetTerrainModel::new("warehouse_1");
        assert_eq!(model.environment_id, "warehouse_1");
    }

    #[test]
    fn test_robot_registration() {
        let mut model = FleetTerrainModel::new("warehouse_1");
        let robot = RobotProfile {
            robot_id: "robot_1".to_string(),
            robot_type: "wheeled".to_string(),
            sensor_reliability: 0.9,
            successful_missions: 10,
            failed_missions: 2,
        };
        model.register_robot(robot);

        assert!(model.robot_profiles.contains_key("robot_1"));
    }

    #[test]
    fn test_observation_recording() {
        let mut model = FleetTerrainModel::new("warehouse_1");
        let robot = RobotProfile {
            robot_id: "robot_1".to_string(),
            robot_type: "wheeled".to_string(),
            sensor_reliability: 0.9,
            successful_missions: 10,
            failed_missions: 2,
        };
        model.register_robot(robot);

        let obs = RobotTraversabilityObservation {
            robot_id: "robot_1".to_string(),
            zone_id: "zone_1".to_string(),
            traversability: 0.8,
            mission_id: "mission_001".to_string(),
            timestamp_sec: 100.0,
            confidence: 0.95,
            payload_kg: Some(10.0),
            weather_conditions: HashMap::new(),
        };

        model.record_observation(obs);
        assert!(model.observations.contains_key("zone_1"));
    }

    #[test]
    fn test_consensus_computation() {
        let mut model = FleetTerrainModel::new("warehouse_1");
        let robot = RobotProfile {
            robot_id: "robot_1".to_string(),
            robot_type: "wheeled".to_string(),
            sensor_reliability: 0.95,
            successful_missions: 20,
            failed_missions: 0,
        };
        model.register_robot(robot);

        let obs = RobotTraversabilityObservation {
            robot_id: "robot_1".to_string(),
            zone_id: "zone_1".to_string(),
            traversability: 0.85,
            mission_id: "mission_001".to_string(),
            timestamp_sec: 100.0,
            confidence: 0.95,
            payload_kg: None,
            weather_conditions: HashMap::new(),
        };

        model.record_observation(obs);

        let consensus = model.get_consensus("zone_1");
        assert!(consensus.is_some());
        assert!((consensus.unwrap().consensus_traversability - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut model = FleetTerrainModel::new("warehouse_1");
        let robot = RobotProfile {
            robot_id: "robot_1".to_string(),
            robot_type: "wheeled".to_string(),
            sensor_reliability: 0.95,
            successful_missions: 20,
            failed_missions: 0,
        };
        model.register_robot(robot);

        // Add multiple normal observations
        for i in 0..5 {
            model.record_observation(RobotTraversabilityObservation {
                robot_id: "robot_1".to_string(),
                zone_id: "zone_1".to_string(),
                traversability: 0.85,
                mission_id: format!("mission_{:03}", i),
                timestamp_sec: 100.0 + (i as f32 * 10.0),
                confidence: 0.95,
                payload_kg: None,
                weather_conditions: HashMap::new(),
            });
        }

        // Add extreme outlier
        model.record_observation(RobotTraversabilityObservation {
            robot_id: "robot_1".to_string(),
            zone_id: "zone_1".to_string(),
            traversability: 0.0, // Extremely different
            mission_id: "mission_outlier".to_string(),
            timestamp_sec: 200.0,
            confidence: 0.95,
            payload_kg: None,
            weather_conditions: HashMap::new(),
        });

        let anomalies = model.find_anomalies("zone_1");
        assert!(!anomalies.is_empty());
    }

    #[test]
    fn test_zone_summary() {
        let mut model = FleetTerrainModel::new("warehouse_1");
        let robot = RobotProfile {
            robot_id: "robot_1".to_string(),
            robot_type: "wheeled".to_string(),
            sensor_reliability: 0.95,
            successful_missions: 10,
            failed_missions: 1,
        };
        model.register_robot(robot);

        model.record_observation(RobotTraversabilityObservation {
            robot_id: "robot_1".to_string(),
            zone_id: "zone_1".to_string(),
            traversability: 0.85,
            mission_id: "mission_001".to_string(),
            timestamp_sec: 100.0,
            confidence: 0.95,
            payload_kg: None,
            weather_conditions: HashMap::new(),
        });

        let summary = model.zone_summary("zone_1");
        assert!(summary.is_some());
        assert!(summary.unwrap().contains("zone_1"));
    }
}
