//! Change Detection Engine
//!
//! Compares current observations against historical baseline
//! to identify what changed since the last visit.
//!
//! Enables questions like:
//! - "Has this object moved?"
//! - "Why is the charging station blocked today?"
//! - "What changed since the last visit?"

use crate::knowledge::world_model::{Entity, Location, WorldState};

/// Detected change in environment
#[derive(Debug, Clone)]
pub struct EnvironmentChange {
    /// What changed
    pub change_type: ChangeType,

    /// Entity involved (if any)
    pub entity_id: Option<String>,

    /// Location involved (if any)
    pub location_id: Option<String>,

    /// Description of change
    pub description: String,

    /// Confidence in this change detection
    pub confidence: f32,

    /// Severity of change (0.0-1.0)
    pub severity: f32,

    /// What was the baseline
    pub baseline: String,

    /// What is the current state
    pub current_state: String,

    /// Is this change expected/normal
    pub is_expected: bool,

    /// Potential explanation
    pub potential_cause: Option<String>,
}

/// Types of changes
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    EntityMoved,              // Object changed location
    EntityAdded,              // New object discovered
    EntityRemoved,            // Object no longer present
    EntityStateChanged,       // State changed (blocked/active/etc)
    LocationTrafficChanged,   // Traffic patterns differ
    LocationHazardAdded,      // New hazard detected
    LocationAccessibilityChanged, // Became accessible/blocked
    AnomalyDetected,          // Behavior differs from baseline
    Unknown,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ChangeType::EntityMoved => write!(f, "Entity Moved"),
            ChangeType::EntityAdded => write!(f, "Entity Added"),
            ChangeType::EntityRemoved => write!(f, "Entity Removed"),
            ChangeType::EntityStateChanged => write!(f, "State Changed"),
            ChangeType::LocationTrafficChanged => write!(f, "Traffic Changed"),
            ChangeType::LocationHazardAdded => write!(f, "Hazard Added"),
            ChangeType::LocationAccessibilityChanged => write!(f, "Accessibility Changed"),
            ChangeType::AnomalyDetected => write!(f, "Anomaly Detected"),
            ChangeType::Unknown => write!(f, "Unknown Change"),
        }
    }
}

/// Change detection engine
pub struct ChangeDetector;

impl ChangeDetector {
    /// Detect changes between current observation and historical baseline
    pub fn detect_changes(
        historical_world: &WorldState,
        current_observations: &[(String, String, String)], // (entity_id, entity_type, location_id)
        timestamp_sec: f32,
    ) -> Vec<EnvironmentChange> {
        let mut changes = Vec::new();

        // Check for entities that moved
        for (entity_id, _entity_type, current_location) in current_observations {
            if let Some(entity) = historical_world.entities.get(entity_id) {
                if let Some(last_location) = &entity.current_location {
                    if last_location != current_location {
                        changes.push(EnvironmentChange {
                            change_type: ChangeType::EntityMoved,
                            entity_id: Some(entity_id.clone()),
                            location_id: Some(current_location.clone()),
                            description: format!(
                                "{} moved from {} to {}",
                                entity_id, last_location, current_location
                            ),
                            confidence: 0.95,
                            severity: Self::assess_move_severity(entity),
                            baseline: format!("Location: {}", last_location),
                            current_state: format!("Location: {}", current_location),
                            is_expected: Self::is_expected_move(entity),
                            potential_cause: Some(
                                "Entity mobility or external movement".to_string()
                            ),
                        });
                    }
                }
            } else {
                // New entity
                changes.push(EnvironmentChange {
                    change_type: ChangeType::EntityAdded,
                    entity_id: Some(entity_id.clone()),
                    location_id: Some(current_location.clone()),
                    description: format!("New entity {} discovered at {}", entity_id, current_location),
                    confidence: 0.85,
                    severity: 0.3, // New entities are generally low severity
                    baseline: "Entity not previously observed".to_string(),
                    current_state: format!("Entity at {}", current_location),
                    is_expected: false,
                    potential_cause: Some("New item introduced to environment".to_string()),
                });
            }
        }

        // Check for entities that disappeared
        for (entity_id, entity) in &historical_world.entities {
            if entity.last_observed_sec > (timestamp_sec - 3600.0) {
                // Was observed recently
                if !current_observations.iter().any(|(eid, _, _)| eid == entity_id) {
                    changes.push(EnvironmentChange {
                        change_type: ChangeType::EntityRemoved,
                        entity_id: Some(entity_id.clone()),
                        location_id: entity.current_location.clone(),
                        description: format!(
                            "{} no longer present (last seen {:.0}s ago)",
                            entity_id,
                            timestamp_sec - entity.last_observed_sec
                        ),
                        confidence: 0.88,
                        severity: 0.4,
                        baseline: format!("Entity at {}",
                            entity.current_location.as_ref().unwrap_or(&"unknown".to_string())),
                        current_state: "Entity not detected".to_string(),
                        is_expected: false,
                        potential_cause: Some("Entity removed or out of view".to_string()),
                    });
                }
            }
        }

        changes
    }

    /// Assess severity of an entity moving
    fn assess_move_severity(entity: &Entity) -> f32 {
        match entity.entity_type.as_str() {
            "charging_station" => 0.7,  // High: charging station moving is unusual
            "obstacle" => 0.6,           // Medium-High: obstacles can move
            "pallet" => 0.5,             // Medium: pallets move often
            "person" => 0.3,             // Low: people move constantly
            _ => 0.4,
        }
    }

    /// Is this move expected (normal behavior)
    fn is_expected_move(entity: &Entity) -> bool {
        match entity.entity_type.as_str() {
            "person" => true,       // People moving is normal
            "pallet" => true,       // Pallets move in warehouse
            "obstacle" => false,    // Obstacles shouldn't move
            "charging_station" => false, // Charging station shouldn't move
            _ => false,
        }
    }

    /// Analyze change significance for debugging
    pub fn analyze_change_impact(
        change: &EnvironmentChange,
        failure_timestamp_sec: f32,
        change_timestamp_sec: f32,
    ) -> ChangeImpactAnalysis {
        let time_to_failure = failure_timestamp_sec - change_timestamp_sec;
        let is_causally_relevant = time_to_failure > 0.0 && time_to_failure < 300.0; // Within 5 minutes

        let impact_score = if is_causally_relevant {
            change.severity * 0.9 // High impact if close to failure time
        } else {
            change.severity * 0.3 // Lower impact if far from failure
        };

        ChangeImpactAnalysis {
            change: change.clone(),
            impact_score,
            is_causally_relevant,
            time_to_failure_sec: if time_to_failure > 0.0 {
                Some(time_to_failure)
            } else {
                None
            },
            likely_explanation: if is_causally_relevant {
                format!(
                    "{} likely contributed to failure {} seconds later",
                    change.description, time_to_failure as i32
                )
            } else {
                "Change occurred too far from failure to be directly causal".to_string()
            },
        }
    }
}

/// Analysis of change impact on mission outcome
#[derive(Debug, Clone)]
pub struct ChangeImpactAnalysis {
    pub change: EnvironmentChange,
    pub impact_score: f32,
    pub is_causally_relevant: bool,
    pub time_to_failure_sec: Option<f32>,
    pub likely_explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_world() -> WorldState {
        let mut world = WorldState {
            environment_id: "test".to_string(),
            entities: HashMap::new(),
            locations: HashMap::new(),
            temporal_facts: Vec::new(),
            baseline_observations: Vec::new(),
            known_anomalies: Vec::new(),
            last_updated_sec: 0.0,
        };

        world.entities.insert(
            "pallet_42".to_string(),
            crate::knowledge::world_model::Entity {
                id: "pallet_42".to_string(),
                entity_type: "pallet".to_string(),
                current_location: Some("aisle_3".to_string()),
                known_locations: vec![crate::knowledge::world_model::LocationHistory {
                    location_id: "aisle_3".to_string(),
                    first_seen_sec: 0.0,
                    last_seen_sec: 100.0,
                    observation_count: 5,
                }],
                state: crate::knowledge::world_model::EntityState::Mobile,
                properties: HashMap::new(),
                confidence: 0.95,
                first_observed_sec: 0.0,
                last_observed_sec: 100.0,
                observation_count: 5,
                is_anomalous: false,
            },
        );

        world
    }

    #[test]
    fn test_entity_moved_detection() {
        let world = create_test_world();
        let current_obs = vec![(
            "pallet_42".to_string(),
            "pallet".to_string(),
            "aisle_5".to_string(),
        )];

        let changes = ChangeDetector::detect_changes(&world, &current_obs, 200.0);

        assert!(changes.iter().any(|c| c.change_type == ChangeType::EntityMoved));
    }

    #[test]
    fn test_entity_disappeared_detection() {
        let world = create_test_world();
        let current_obs = vec![]; // No observations

        let changes = ChangeDetector::detect_changes(&world, &current_obs, 200.0);

        assert!(changes.iter().any(|c| c.change_type == ChangeType::EntityRemoved));
    }

    #[test]
    fn test_change_impact_analysis() {
        let change = EnvironmentChange {
            change_type: ChangeType::EntityMoved,
            entity_id: Some("pallet_42".to_string()),
            location_id: Some("aisle_5".to_string()),
            description: "Pallet moved".to_string(),
            confidence: 0.95,
            severity: 0.5,
            baseline: "aisle_3".to_string(),
            current_state: "aisle_5".to_string(),
            is_expected: true,
            potential_cause: None,
        };

        let impact = ChangeDetector::analyze_change_impact(&change, 250.0, 200.0);

        assert!(impact.is_causally_relevant);
        assert!(impact.time_to_failure_sec.is_some());
    }
}
