//! Phase 11: PyTerrainMap Integration
//!
//! Unifies persistent world knowledge (Phase 10) with spatial-terrain intelligence.
//! Enables: "Entity X at coordinates Y has traversability Z, will move to high-traffic area W"
//!
//! Bridges:
//! - World Knowledge (entities, locations) + Terrain (obstacles, zones)
//! - Spatial Grounding (x,y,z) + Traversability (0.0-1.0 per zone)
//! - Multi-Mission Learning + Terrain Evolution
//!
//! Answers:
//! - "Why did robot fail? Pallet in zero-traversability zone"
//! - "Will this happen again? Zone traversability declining"
//! - "What changed? New obstacle in high-traffic area"

use crate::knowledge::world_model::{Entity, EntityState};
use crate::knowledge::spatial_grounding::SpatialCoordinates;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Terrain zone information (from PyTerrainMap)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainZone {
    /// Zone ID (e.g., "aisle_3_north")
    pub zone_id: String,
    /// Zone center coordinates
    pub center: SpatialCoordinates,
    /// Zone radius (meters)
    pub radius_m: f32,
    /// Terrain type (open, confined, stairs, wet, etc.)
    pub terrain_type: String,
    /// Current traversability (0.0-1.0)
    pub traversability: f32,
    /// Success count (robot passages)
    pub successful_passages: usize,
    /// Failure count
    pub failed_attempts: usize,
    /// Environmental conditions affecting traversability
    pub environmental_factors: HashMap<String, f32>,
    /// Last update time
    pub last_updated_sec: f32,
}

impl TerrainZone {
    /// Calculate success rate for this zone
    pub fn success_rate(&self) -> f32 {
        let total = (self.successful_passages + self.failed_attempts) as f32;
        if total == 0.0 {
            0.5 // Unknown zone
        } else {
            self.successful_passages as f32 / total
        }
    }

    /// Assess if entity position is safe in this zone
    pub fn is_entity_safe(&self, entity_state: &EntityState) -> bool {
        match entity_state {
            EntityState::Fixed => self.traversability > 0.5, // Static objects need moderate space
            EntityState::Mobile => self.traversability > 0.7, // Mobile objects need good space
            EntityState::Active => self.traversability > 0.6,  // Active objects need decent space
            EntityState::Blocked => true,                      // Already blocked, state known
            _ => false,
        }
    }

    /// Distance from coordinates to zone center
    pub fn distance_from(&self, coords: &SpatialCoordinates) -> f32 {
        self.center.distance_to(coords)
    }

    /// Check if coordinates are within zone
    pub fn contains(&self, coords: &SpatialCoordinates) -> bool {
        self.distance_from(coords) <= self.radius_m
    }
}

/// Obstacle information from terrain (mapped to entity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainObstacle {
    /// Obstacle ID
    pub obstacle_id: String,
    /// Coordinates
    pub coordinates: SpatialCoordinates,
    /// Dimensions (width, height, depth)
    pub dimensions: (f32, f32, f32),
    /// Obstacle type
    pub obstacle_type: String,
    /// Is dynamic (moving)
    pub is_dynamic: bool,
    /// Detection confidence
    pub confidence: f32,
    /// Zone it's currently in
    pub zone_id: Option<String>,
    /// First observed time
    pub first_observed_sec: f32,
    /// Last observed time
    pub last_observed_sec: f32,
}

/// Entity-terrain relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTerrainContext {
    /// Entity ID
    pub entity_id: String,
    /// Current coordinates
    pub coordinates: SpatialCoordinates,
    /// Current zone (if any)
    pub zone_id: Option<String>,
    /// Traversability at this location
    pub local_traversability: f32,
    /// Is location safe for entity type
    pub is_safe: bool,
    /// Risk factors (high-traffic, low-traversability, etc.)
    pub risk_factors: Vec<String>,
    /// Recommended actions
    pub recommendations: Vec<String>,
}

/// Terrain-aware mission context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainMissionContext {
    /// Mission ID
    pub mission_id: String,
    /// Zones traversed
    pub zones_visited: Vec<String>,
    /// Average traversability during mission
    pub avg_traversability: f32,
    /// Zones with declining traversability
    pub declining_zones: Vec<(String, f32, f32)>, // zone_id, prev, current
    /// Obstacles encountered
    pub obstacles_encountered: Vec<String>,
    /// Environmental changes detected
    pub environmental_changes: HashMap<String, f32>,
}

/// Terrain knowledge integration engine
pub struct TerrainIntegrationEngine {
    /// All known terrain zones
    pub zones: HashMap<String, TerrainZone>,
    /// All known terrain obstacles
    pub obstacles: HashMap<String, TerrainObstacle>,
    /// Entity-terrain relationships
    pub entity_contexts: HashMap<String, EntityTerrainContext>,
    /// Mission terrain contexts
    pub mission_contexts: Vec<TerrainMissionContext>,
    /// Zone traversability history
    pub zone_history: HashMap<String, Vec<(f32, f32)>>, // zone_id -> (timestamp, traversability)
}

impl TerrainIntegrationEngine {
    /// Create new integration engine
    pub fn new() -> Self {
        TerrainIntegrationEngine {
            zones: HashMap::new(),
            obstacles: HashMap::new(),
            entity_contexts: HashMap::new(),
            mission_contexts: Vec::new(),
            zone_history: HashMap::new(),
        }
    }

    /// Register terrain zone from PyTerrainMap
    pub fn register_zone(&mut self, zone: TerrainZone) {
        // Track history
        let history = self
            .zone_history
            .entry(zone.zone_id.clone())
            .or_insert_with(Vec::new);
        history.push((zone.last_updated_sec, zone.traversability));

        self.zones.insert(zone.zone_id.clone(), zone);
    }

    /// Register terrain obstacle
    pub fn register_obstacle(&mut self, obstacle: TerrainObstacle) {
        self.obstacles
            .insert(obstacle.obstacle_id.clone(), obstacle);
    }

    /// Update entity with terrain context
    pub fn update_entity_context(
        &mut self,
        entity_id: &str,
        entity_state: &EntityState,
        coordinates: SpatialCoordinates,
    ) {
        // Find zone containing entity
        let mut zone_id = None;
        let mut local_traversability = 0.5; // Unknown default

        for (zid, zone) in &self.zones {
            if zone.contains(&coordinates) {
                zone_id = Some(zid.clone());
                local_traversability = zone.traversability;
                break;
            }
        }

        // Determine safety
        let is_safe = if let Some(zone_id_ref) = &zone_id {
            if let Some(zone) = self.zones.get(zone_id_ref) {
                zone.is_entity_safe(entity_state)
            } else {
                false
            }
        } else {
            local_traversability > 0.5
        };

        // Identify risk factors
        let mut risk_factors = Vec::new();
        if local_traversability < 0.3 {
            risk_factors.push("low_traversability".to_string());
        }
        if local_traversability > 0.8 && matches!(entity_state, EntityState::Mobile) {
            risk_factors.push("high_traffic_zone".to_string());
        }
        if zone_id.is_none() {
            risk_factors.push("unknown_zone".to_string());
        }

        // Generate recommendations
        let mut recommendations = Vec::new();
        if !is_safe {
            recommendations.push(format!(
                "Consider moving {} away from low-traversability zone",
                entity_id
            ));
        }
        if local_traversability < 0.5 {
            recommendations.push("Verify entity stability in this zone".to_string());
        }

        let context = EntityTerrainContext {
            entity_id: entity_id.to_string(),
            coordinates,
            zone_id,
            local_traversability,
            is_safe,
            risk_factors,
            recommendations,
        };

        self.entity_contexts
            .insert(entity_id.to_string(), context);
    }

    /// Detect zone traversability changes
    pub fn detect_zone_changes(&self, zone_id: &str) -> Option<(f32, f32)> {
        if let Some(history) = self.zone_history.get(zone_id) {
            if history.len() < 2 {
                return None;
            }
            let prev = history[history.len() - 2].1;
            let current = history[history.len() - 1].1;
            if (prev - current).abs() > 0.1 {
                return Some((prev, current));
            }
        }
        None
    }

    /// Assess mission terrain context
    pub fn create_mission_context(
        &self,
        mission_id: &str,
        zones_visited: Vec<String>,
    ) -> TerrainMissionContext {
        let mut total_traversability = 0.0;
        let mut declining_zones = Vec::new();

        for zone_id in &zones_visited {
            if let Some(zone) = self.zones.get(zone_id) {
                total_traversability += zone.traversability;

                if let Some((prev, current)) = self.detect_zone_changes(zone_id) {
                    if current < prev {
                        declining_zones.push((zone_id.clone(), prev, current));
                    }
                }
            }
        }

        let avg_traversability = if !zones_visited.is_empty() {
            total_traversability / zones_visited.len() as f32
        } else {
            0.5
        };

        TerrainMissionContext {
            mission_id: mission_id.to_string(),
            zones_visited,
            avg_traversability,
            declining_zones,
            obstacles_encountered: Vec::new(),
            environmental_changes: HashMap::new(),
        }
    }

    /// Get entity terrain risk assessment
    pub fn assess_entity_risk(&self, entity_id: &str) -> Option<f32> {
        self.entity_contexts.get(entity_id).map(|ctx| {
            let mut risk = 0.0;

            // Low traversability = high risk
            risk += (1.0 - ctx.local_traversability) * 0.4;

            // Unknown zone = medium risk
            if ctx.zone_id.is_none() {
                risk += 0.3;
            }

            // Multiple risk factors = compound risk
            risk += (ctx.risk_factors.len() as f32) * 0.1;

            risk.min(1.0)
        })
    }

    /// Get zone trending (improving/declining)
    pub fn get_zone_trend(&self, zone_id: &str) -> String {
        if let Some(history) = self.zone_history.get(zone_id) {
            if history.len() < 2 {
                return "insufficient_data".to_string();
            }

            let mut improving = 0;
            let mut declining = 0;

            for i in 1..history.len() {
                if history[i].1 > history[i - 1].1 {
                    improving += 1;
                } else if history[i].1 < history[i - 1].1 {
                    declining += 1;
                }
            }

            if declining > improving {
                "declining".to_string()
            } else if improving > declining {
                "improving".to_string()
            } else {
                "stable".to_string()
            }
        } else {
            "unknown".to_string()
        }
    }
}

impl Default for TerrainIntegrationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_zone() -> TerrainZone {
        TerrainZone {
            zone_id: "test_zone".to_string(),
            center: SpatialCoordinates {
                x: 5.0,
                y: 5.0,
                z: 0.0,
            },
            radius_m: 3.0,
            terrain_type: "open".to_string(),
            traversability: 0.9,
            successful_passages: 20,
            failed_attempts: 2,
            environmental_factors: HashMap::new(),
            last_updated_sec: 100.0,
        }
    }

    #[test]
    fn test_terrain_zone_creation() {
        let zone = create_test_zone();
        assert_eq!(zone.zone_id, "test_zone");
        assert_eq!(zone.traversability, 0.9);
    }

    #[test]
    fn test_success_rate() {
        let zone = create_test_zone();
        let rate = zone.success_rate();
        assert!((rate - 0.909).abs() < 0.01);
    }

    #[test]
    fn test_zone_contains() {
        let zone = create_test_zone();
        let in_zone = SpatialCoordinates {
            x: 5.0,
            y: 5.0,
            z: 0.0,
        };
        assert!(zone.contains(&in_zone));

        let out_zone = SpatialCoordinates {
            x: 10.0,
            y: 10.0,
            z: 0.0,
        };
        assert!(!zone.contains(&out_zone));
    }

    #[test]
    fn test_entity_safety() {
        let zone = create_test_zone();
        assert!(zone.is_entity_safe(&EntityState::Fixed));
        assert!(zone.is_entity_safe(&EntityState::Mobile));
    }

    #[test]
    fn test_integration_engine() {
        let mut engine = TerrainIntegrationEngine::new();
        let zone = create_test_zone();
        engine.register_zone(zone);

        assert_eq!(engine.zones.len(), 1);
    }

    #[test]
    fn test_entity_terrain_context() {
        let mut engine = TerrainIntegrationEngine::new();
        let zone = create_test_zone();
        engine.register_zone(zone);

        let coords = SpatialCoordinates {
            x: 5.0,
            y: 5.0,
            z: 0.0,
        };
        engine.update_entity_context("entity_1", &EntityState::Mobile, coords);

        assert!(engine.entity_contexts.contains_key("entity_1"));
    }

    #[test]
    fn test_mission_context() {
        let mut engine = TerrainIntegrationEngine::new();
        let zone = create_test_zone();
        engine.register_zone(zone);

        let mission_ctx = engine.create_mission_context("mission_001", vec!["test_zone".to_string()]);
        assert_eq!(mission_ctx.mission_id, "mission_001");
        assert!((mission_ctx.avg_traversability - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_entity_risk_assessment() {
        let mut engine = TerrainIntegrationEngine::new();
        let zone = create_test_zone();
        engine.register_zone(zone);

        let coords = SpatialCoordinates {
            x: 5.0,
            y: 5.0,
            z: 0.0,
        };
        engine.update_entity_context("entity_1", &EntityState::Mobile, coords);

        let risk = engine.assess_entity_risk("entity_1");
        assert!(risk.is_some());
        assert!(risk.unwrap() < 0.2); // Safe zone = low risk
    }

    #[test]
    fn test_zone_trend() {
        let mut engine = TerrainIntegrationEngine::new();
        let zone = create_test_zone();
        engine.register_zone(zone);

        let trend = engine.get_zone_trend("test_zone");
        assert_eq!(trend, "insufficient_data");
    }
}
