//! Persistent World Model
//!
//! Maintains knowledge about:
//! - Objects and their properties
//! - Locations and spatial relationships
//! - Temporal facts and state changes
//! - Historical observations
//!
//! Schema inspired by Google Open Knowledge Framework (OKF)
//! but adapted for robot replay and observability.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete world state across all time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// Environment ID (building, floor, warehouse, etc.)
    pub environment_id: String,

    /// All known entities in this environment
    pub entities: HashMap<String, Entity>,

    /// All known locations
    pub locations: HashMap<String, Location>,

    /// Temporal facts (what happened when)
    pub temporal_facts: Vec<TemporalFact>,

    /// Baseline observations (normal state)
    pub baseline_observations: Vec<Observation>,

    /// Historical anomalies
    pub known_anomalies: Vec<AnomalyRecord>,

    /// Last updated timestamp
    pub last_updated_sec: f32,
}

/// An entity (object, person, robot, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier (e.g., "pallet_42", "door_3b", "charging_station_main")
    pub id: String,

    /// Entity type
    pub entity_type: String, // "pallet", "door", "charging_station", "obstacle", etc.

    /// Current known location
    pub current_location: Option<String>, // Location ID

    /// Historical locations (where has it been seen)
    pub known_locations: Vec<LocationHistory>,

    /// State (fixed/mobile, active/inactive, accessible/blocked)
    pub state: EntityState,

    /// Properties (size, color, hazard_level, etc.)
    pub properties: HashMap<String, String>,

    /// How confident are we about this entity
    pub confidence: f32,

    /// When was this entity first observed
    pub first_observed_sec: f32,

    /// When was this entity last observed
    pub last_observed_sec: f32,

    /// How many times has this entity been observed
    pub observation_count: usize,

    /// Is this entity anomalous (behaving differently than baseline)
    pub is_anomalous: bool,
}

/// State of an entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityState {
    Fixed,           // Doesn't move (wall, door, charging station)
    Mobile,          // Can move (person, other robot, obstacle)
    Active,          // In use or relevant
    Inactive,        // Not in use
    Blocked,         // Inaccessible
    Unknown,
}

/// Historical location of entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationHistory {
    pub location_id: String,
    pub first_seen_sec: f32,
    pub last_seen_sec: f32,
    pub observation_count: usize,
}

/// Historical event at a location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub event: String,
    pub timestamp_sec: f32,
}

/// A location in the environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Unique identifier (e.g., "aisle_3", "entrance", "charging_area")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Location type
    pub location_type: String, // "aisle", "intersection", "storage", "entrance", etc.

    /// Spatial coordinates (if available)
    pub coordinates: Option<(f32, f32, f32)>, // x, y, z

    /// Typical traffic patterns (low/medium/high)
    pub typical_traffic: String,

    /// Known hazards in this location
    pub known_hazards: Vec<String>,

    /// Historical events at this location
    pub event_history: Vec<EventRecord>,

    /// Confidence in our knowledge about this location
    pub confidence: f32,

    /// When was this location last visited
    pub last_visited_sec: f32,

    /// How many times has this location been visited
    pub visit_count: usize,
}

/// A temporal fact (what happened when)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalFact {
    /// What happened
    pub fact: String,

    /// When it happened (seconds)
    pub timestamp_sec: f32,

    /// Which mission did this occur in
    pub mission_id: String,

    /// Entity involved (if any)
    pub entity_id: Option<String>,

    /// Location involved (if any)
    pub location_id: Option<String>,

    /// How reliable is this fact
    pub confidence: f32,

    /// What was anomalous about this
    pub anomaly_notes: Option<String>,
}

/// A single observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// What was observed
    pub observation: String,

    /// Where was it observed
    pub location_id: String,

    /// When was it observed
    pub timestamp_sec: f32,

    /// Mission ID
    pub mission_id: String,

    /// Confidence in observation
    pub confidence: f32,

    /// Is this observation aligned with baseline
    pub is_baseline_aligned: bool,

    /// Deviation from baseline (if any)
    pub baseline_deviation: Option<f32>,
}

/// Historical anomaly record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRecord {
    /// What was anomalous
    pub anomaly: String,

    /// When was it first detected
    pub first_detected_sec: f32,

    /// When was it last detected
    pub last_detected_sec: f32,

    /// How many times has this anomaly occurred
    pub occurrence_count: usize,

    /// Severity of anomaly (0.0-1.0)
    pub severity: f32,

    /// Has this anomaly been explained
    pub is_explained: bool,

    /// Known causes for this anomaly
    pub known_causes: Vec<String>,
}

impl Default for Observation {
    fn default() -> Self {
        Observation {
            observation: String::new(),
            location_id: String::new(),
            timestamp_sec: 0.0,
            mission_id: String::new(),
            confidence: 0.0,
            is_baseline_aligned: false,
            baseline_deviation: None,
        }
    }
}

impl Default for AnomalyRecord {
    fn default() -> Self {
        AnomalyRecord {
            anomaly: String::new(),
            first_detected_sec: 0.0,
            last_detected_sec: 0.0,
            occurrence_count: 0,
            severity: 0.0,
            is_explained: false,
            known_causes: Vec::new(),
        }
    }
}

/// World model builder and manager
pub struct WorldModelManager {
    states: HashMap<String, WorldState>,
}

impl WorldModelManager {
    /// Create new world model manager
    pub fn new() -> Self {
        WorldModelManager {
            states: HashMap::new(),
        }
    }

    /// Get or create world state for environment
    pub fn get_or_create_environment(&mut self, env_id: &str) -> &mut WorldState {
        self.states.entry(env_id.to_string()).or_insert_with(|| {
            WorldState {
                environment_id: env_id.to_string(),
                entities: HashMap::new(),
                locations: HashMap::new(),
                temporal_facts: Vec::new(),
                baseline_observations: Vec::new(),
                known_anomalies: Vec::new(),
                last_updated_sec: 0.0,
            }
        });

        self.states.get_mut(env_id).unwrap()
    }

    /// Record an observation into world state
    pub fn record_observation(
        &mut self,
        env_id: &str,
        entity_id: &str,
        entity_type: &str,
        location_id: &str,
        timestamp_sec: f32,
        mission_id: &str,
        confidence: f32,
    ) {
        let world = self.get_or_create_environment(env_id);

        // Update or create entity
        let entity = world
            .entities
            .entry(entity_id.to_string())
            .or_insert_with(|| Entity {
                id: entity_id.to_string(),
                entity_type: entity_type.to_string(),
                current_location: None,
                known_locations: Vec::new(),
                state: EntityState::Unknown,
                properties: HashMap::new(),
                confidence: 0.0,
                first_observed_sec: timestamp_sec,
                last_observed_sec: timestamp_sec,
                observation_count: 0,
                is_anomalous: false,
            });

        // Update entity
        entity.current_location = Some(location_id.to_string());
        entity.last_observed_sec = timestamp_sec;
        entity.observation_count += 1;
        entity.confidence = (entity.confidence * 0.8 + confidence * 0.2).min(1.0);

        // Track location history
        if !entity
            .known_locations
            .iter()
            .any(|h| h.location_id == location_id)
        {
            entity.known_locations.push(LocationHistory {
                location_id: location_id.to_string(),
                first_seen_sec: timestamp_sec,
                last_seen_sec: timestamp_sec,
                observation_count: 1,
            });
        } else {
            for history in entity.known_locations.iter_mut() {
                if history.location_id == location_id {
                    history.last_seen_sec = timestamp_sec;
                    history.observation_count += 1;
                }
            }
        }

        // Update location
        world
            .locations
            .entry(location_id.to_string())
            .or_insert_with(|| Location {
                id: location_id.to_string(),
                name: format!("Location {}", location_id),
                location_type: "unknown".to_string(),
                coordinates: None,
                typical_traffic: "unknown".to_string(),
                known_hazards: Vec::new(),
                event_history: Vec::new(),
                confidence: 0.5,
                last_visited_sec: timestamp_sec,
                visit_count: 0,
            });

        if let Some(location) = world.locations.get_mut(location_id) {
            location.last_visited_sec = timestamp_sec;
            location.visit_count += 1;
        }

        world.last_updated_sec = timestamp_sec;
    }

    /// Record temporal fact
    pub fn record_fact(
        &mut self,
        env_id: &str,
        fact: &str,
        timestamp_sec: f32,
        mission_id: &str,
        entity_id: Option<&str>,
        location_id: Option<&str>,
    ) {
        let world = self.get_or_create_environment(env_id);

        world.temporal_facts.push(TemporalFact {
            fact: fact.to_string(),
            timestamp_sec,
            mission_id: mission_id.to_string(),
            entity_id: entity_id.map(|s| s.to_string()),
            location_id: location_id.map(|s| s.to_string()),
            confidence: 0.8,
            anomaly_notes: None,
        });
    }

    /// Get entity knowledge
    pub fn get_entity(&self, env_id: &str, entity_id: &str) -> Option<&Entity> {
        self.states
            .get(env_id)
            .and_then(|w| w.entities.get(entity_id))
    }

    /// Get location knowledge
    pub fn get_location(&self, env_id: &str, location_id: &str) -> Option<&Location> {
        self.states
            .get(env_id)
            .and_then(|w| w.locations.get(location_id))
    }
}

impl Default for WorldModelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_model_creation() {
        let manager = WorldModelManager::new();
        assert_eq!(manager.states.len(), 0);
    }

    #[test]
    fn test_entity_recording() {
        let mut manager = WorldModelManager::new();

        manager.record_observation(
            "warehouse_1",
            "pallet_42",
            "pallet",
            "aisle_3",
            100.0,
            "mission_001",
            0.95,
        );

        let entity = manager.get_entity("warehouse_1", "pallet_42");
        assert!(entity.is_some());
        assert_eq!(entity.unwrap().observation_count, 1);
    }

    #[test]
    fn test_temporal_facts() {
        let mut manager = WorldModelManager::new();

        manager.record_fact(
            "warehouse_1",
            "Pallet moved to new location",
            105.0,
            "mission_001",
            Some("pallet_42"),
            Some("aisle_3"),
        );

        let world = manager.get_or_create_environment("warehouse_1");
        assert_eq!(world.temporal_facts.len(), 1);
    }

    #[test]
    fn test_multiple_observations() {
        let mut manager = WorldModelManager::new();

        // Mission 1: See pallet at location A
        manager.record_observation(
            "warehouse_1",
            "pallet_42",
            "pallet",
            "aisle_3",
            100.0,
            "mission_001",
            0.95,
        );

        // Mission 2: See same pallet at location B
        manager.record_observation(
            "warehouse_1",
            "pallet_42",
            "pallet",
            "aisle_5",
            200.0,
            "mission_002",
            0.92,
        );

        let entity = manager.get_entity("warehouse_1", "pallet_42").unwrap();
        assert_eq!(entity.observation_count, 2);
        assert_eq!(entity.known_locations.len(), 2);
    }
}
