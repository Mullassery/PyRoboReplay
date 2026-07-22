//! Spatial Grounding: Embedding Persistent Knowledge in PyTerrainMap
//!
//! Grounds world model entities (from Phase 10) with spatial coordinates
//! from PyTerrainMap, enabling:
//! - "Object moved 2.3m northwest" (not just "moved")
//! - Distance-aware anomaly detection
//! - Traversability-informed change impact
//! - Spatial-temporal entity tracking

use crate::knowledge::world_model::Entity;
use serde::{Deserialize, Serialize};

/// Spatial coordinates for entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialCoordinates {
    /// X coordinate (meters)
    pub x: f32,
    /// Y coordinate (meters)
    pub y: f32,
    /// Z coordinate (meters, elevation)
    pub z: f32,
}

impl SpatialCoordinates {
    /// Calculate Euclidean distance to another point
    pub fn distance_to(&self, other: &SpatialCoordinates) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculate bearing (direction) to another point in degrees
    pub fn bearing_to(&self, other: &SpatialCoordinates) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let angle = dy.atan2(dx);
        (angle.to_degrees() + 360.0) % 360.0
    }

    /// Get direction name from bearing (N, NE, E, SE, S, SW, W, NW)
    pub fn direction_name(bearing: f32) -> &'static str {
        match (bearing + 22.5) as u32 / 45 {
            0 => "N",
            1 => "NE",
            2 => "E",
            3 => "SE",
            4 => "S",
            5 => "SW",
            6 => "W",
            7 => "NW",
            _ => "unknown",
        }
    }
}

/// Entity with spatial grounding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedEntity {
    /// Reference to entity from world model
    pub entity_id: String,
    /// Entity type
    pub entity_type: String,
    /// Current spatial coordinates
    pub coordinates: Option<SpatialCoordinates>,
    /// Previous coordinates (for movement detection)
    pub previous_coordinates: Option<SpatialCoordinates>,
    /// Distance moved (meters)
    pub distance_moved: Option<f32>,
    /// Direction moved (bearing in degrees)
    pub direction_moved: Option<f32>,
    /// Human-readable direction (N, NE, E, SE, S, SW, W, NW)
    pub direction_name: Option<String>,
    /// Confidence in spatial grounding (0.0-1.0)
    pub spatial_confidence: f32,
}

impl GroundedEntity {
    /// Create new grounded entity from world model entity
    pub fn from_entity(entity: &Entity) -> Self {
        GroundedEntity {
            entity_id: entity.id.clone(),
            entity_type: entity.entity_type.clone(),
            coordinates: None,
            previous_coordinates: None,
            distance_moved: None,
            direction_moved: None,
            direction_name: None,
            spatial_confidence: 0.0,
        }
    }

    /// Update entity position and compute movement
    pub fn update_position(
        &mut self,
        new_coords: SpatialCoordinates,
        confidence: f32,
    ) {
        let distance = if let Some(prev) = &self.coordinates {
            Some(prev.distance_to(&new_coords))
        } else {
            None
        };

        let bearing = if let Some(prev) = &self.coordinates {
            Some(prev.bearing_to(&new_coords))
        } else {
            None
        };

        // Store previous before updating
        self.previous_coordinates = self.coordinates.clone();
        self.distance_moved = distance;
        self.direction_moved = bearing;
        self.direction_name = bearing.map(|b| SpatialCoordinates::direction_name(b).to_string());
        self.coordinates = Some(new_coords);
        self.spatial_confidence = confidence;
    }

    /// Generate human-readable movement description
    pub fn movement_description(&self) -> Option<String> {
        match (&self.distance_moved, &self.direction_name) {
            (Some(dist), Some(dir)) if *dist > 0.01 => {
                Some(format!("moved {:.2}m {}", dist, dir))
            }
            (Some(dist), Some(dir)) if *dist > 0.001 => {
                Some(format!("shifted {:.3}m {}", dist, dir))
            }
            _ => None,
        }
    }
}

/// Spatial-temporal trend tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialTemporalTrend {
    /// Entity ID
    pub entity_id: String,
    /// Positions over time
    pub trajectory: Vec<(f32, SpatialCoordinates)>, // (timestamp, coordinates)
    /// Trend direction (moving away, approaching, stationary, cycling)
    pub trend: String,
    /// Average velocity (meters per second)
    pub avg_velocity: f32,
    /// Max distance traveled in trend period
    pub max_distance_traveled: f32,
}

impl SpatialTemporalTrend {
    /// Analyze spatial-temporal trend from trajectory
    pub fn from_trajectory(
        entity_id: &str,
        trajectory: Vec<(f32, SpatialCoordinates)>,
    ) -> Self {
        let mut trend = "stationary".to_string();
        let mut avg_velocity = 0.0;
        let mut max_distance = 0.0;

        if trajectory.len() < 2 {
            return SpatialTemporalTrend {
                entity_id: entity_id.to_string(),
                trajectory,
                trend,
                avg_velocity,
                max_distance_traveled: max_distance,
            };
        }

        let first_pos = &trajectory[0].1;
        let last_pos = &trajectory[trajectory.len() - 1].1;
        let first_time = trajectory[0].0;
        let last_time = trajectory[trajectory.len() - 1].0;

        let total_distance = first_pos.distance_to(last_pos);
        let time_delta = last_time - first_time;

        if time_delta > 0.0 {
            avg_velocity = total_distance / time_delta;
        }

        // Calculate max distance between any two points
        for i in 0..trajectory.len() {
            for j in i + 1..trajectory.len() {
                let dist = trajectory[i].1.distance_to(&trajectory[j].1);
                if dist > max_distance {
                    max_distance = dist;
                }
            }
        }

        // Classify trend
        if avg_velocity < 0.01 && max_distance < 0.1 {
            trend = "stationary".to_string();
        } else if total_distance > 0.1 && time_delta > 0.0 {
            // Check if returning to origin (cycling)
            let final_distance = first_pos.distance_to(last_pos);
            if final_distance < 0.2 && max_distance > 1.0 {
                trend = "cycling".to_string();
            } else if final_distance > 0.5 {
                trend = "moving_away".to_string();
            } else if final_distance < 0.2 {
                trend = "approaching".to_string();
            }
        }

        SpatialTemporalTrend {
            entity_id: entity_id.to_string(),
            trajectory,
            trend,
            avg_velocity,
            max_distance_traveled: max_distance,
        }
    }
}

/// Spatial-aware anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAnomaly {
    /// Entity ID
    pub entity_id: String,
    /// Anomaly type
    pub anomaly_type: String, // "unexpected_movement", "wrong_location", "excessive_velocity", etc.
    /// Baseline coordinates (expected)
    pub baseline: Option<SpatialCoordinates>,
    /// Current coordinates (observed)
    pub current: SpatialCoordinates,
    /// Deviation distance (meters)
    pub deviation_meters: f32,
    /// Deviation percentage from baseline area
    pub deviation_percentage: f32,
    /// Severity (0.0-1.0)
    pub severity: f32,
}

/// Spatial grounding engine
pub struct SpatialGroundingEngine {
    /// Grounded entities
    pub entities: std::collections::HashMap<String, GroundedEntity>,
    /// Spatial-temporal trends
    pub trends: std::collections::HashMap<String, SpatialTemporalTrend>,
    /// Spatial anomalies
    pub anomalies: Vec<SpatialAnomaly>,
}

impl SpatialGroundingEngine {
    /// Create new engine
    pub fn new() -> Self {
        SpatialGroundingEngine {
            entities: std::collections::HashMap::new(),
            trends: std::collections::HashMap::new(),
            anomalies: Vec::new(),
        }
    }

    /// Add or update grounded entity
    pub fn update_entity(
        &mut self,
        entity_id: &str,
        entity_type: &str,
        coordinates: SpatialCoordinates,
        confidence: f32,
    ) {
        let mut grounded = self
            .entities
            .entry(entity_id.to_string())
            .or_insert_with(|| GroundedEntity {
                entity_id: entity_id.to_string(),
                entity_type: entity_type.to_string(),
                coordinates: None,
                previous_coordinates: None,
                distance_moved: None,
                direction_moved: None,
                direction_name: None,
                spatial_confidence: 0.0,
            });

        grounded.update_position(coordinates, confidence);
    }

    /// Record spatial-temporal trajectory
    pub fn record_trajectory(
        &mut self,
        entity_id: &str,
        timestamp: f32,
        coordinates: SpatialCoordinates,
    ) {
        self.entities
            .entry(entity_id.to_string())
            .or_insert_with(|| GroundedEntity {
                entity_id: entity_id.to_string(),
                entity_type: "unknown".to_string(),
                coordinates: None,
                previous_coordinates: None,
                distance_moved: None,
                direction_moved: None,
                direction_name: None,
                spatial_confidence: 0.0,
            });

        // Build trajectory for trend analysis
        if let Some(trend) = self.trends.get_mut(entity_id) {
            trend.trajectory.push((timestamp, coordinates));
        } else {
            let trend = SpatialTemporalTrend::from_trajectory(
                entity_id,
                vec![(timestamp, coordinates)],
            );
            self.trends.insert(entity_id.to_string(), trend);
        }
    }

    /// Detect spatial anomalies
    pub fn detect_anomalies(
        &mut self,
        entity_id: &str,
        baseline: Option<SpatialCoordinates>,
        current: SpatialCoordinates,
    ) {
        if let Some(baseline_coords) = baseline {
            let deviation = baseline_coords.distance_to(&current);
            let severity = (deviation / 10.0).min(1.0); // Normalize to 0-1 range

            if deviation > 0.5 {
                self.anomalies.push(SpatialAnomaly {
                    entity_id: entity_id.to_string(),
                    anomaly_type: "unexpected_movement".to_string(),
                    baseline: Some(baseline_coords),
                    current,
                    deviation_meters: deviation,
                    deviation_percentage: (deviation / 10.0) * 100.0,
                    severity,
                });
            }
        }
    }
}

impl Default for SpatialGroundingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_coordinates() {
        let p1 = SpatialCoordinates {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let p2 = SpatialCoordinates {
            x: 3.0,
            y: 4.0,
            z: 0.0,
        };

        let dist = p1.distance_to(&p2);
        assert!((dist - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_bearing_calculation() {
        let p1 = SpatialCoordinates {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let p2 = SpatialCoordinates {
            x: 1.0,
            y: 1.0,
            z: 0.0,
        };

        let bearing = p1.bearing_to(&p2);
        assert!((bearing - 45.0).abs() < 1.0);
    }

    #[test]
    fn test_direction_name() {
        assert_eq!(SpatialCoordinates::direction_name(0.0), "N");
        assert_eq!(SpatialCoordinates::direction_name(45.0), "NE");
        assert_eq!(SpatialCoordinates::direction_name(90.0), "E");
        assert_eq!(SpatialCoordinates::direction_name(180.0), "S");
    }

    #[test]
    fn test_grounded_entity_movement() {
        let mut entity = GroundedEntity {
            entity_id: "pallet_1".to_string(),
            entity_type: "pallet".to_string(),
            coordinates: None,
            previous_coordinates: None,
            distance_moved: None,
            direction_moved: None,
            direction_name: None,
            spatial_confidence: 0.0,
        };

        let pos1 = SpatialCoordinates {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        entity.update_position(pos1, 0.95);

        let pos2 = SpatialCoordinates {
            x: 3.0,
            y: 4.0,
            z: 0.0,
        };
        entity.update_position(pos2, 0.95);

        assert!(entity.distance_moved.is_some());
        assert!((entity.distance_moved.unwrap() - 5.0).abs() < 0.01);
        assert!(entity.movement_description().is_some());
    }

    #[test]
    fn test_spatial_grounding_engine() {
        let mut engine = SpatialGroundingEngine::new();

        let pos = SpatialCoordinates {
            x: 5.0,
            y: 5.0,
            z: 0.0,
        };
        engine.update_entity("pallet_42", "pallet", pos, 0.95);

        assert_eq!(engine.entities.len(), 1);
        assert!(engine.entities.contains_key("pallet_42"));
    }

    #[test]
    fn test_spatial_anomaly_detection() {
        let mut engine = SpatialGroundingEngine::new();

        let baseline = SpatialCoordinates {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let current = SpatialCoordinates {
            x: 2.0,
            y: 2.0,
            z: 0.0,
        };

        engine.detect_anomalies("pallet_1", Some(baseline), current);
        assert_eq!(engine.anomalies.len(), 1);
    }

    #[test]
    fn test_spatial_temporal_trend() {
        let trajectory = vec![
            (0.0, SpatialCoordinates { x: 0.0, y: 0.0, z: 0.0 }),
            (1.0, SpatialCoordinates { x: 1.0, y: 0.0, z: 0.0 }),
            (2.0, SpatialCoordinates { x: 2.0, y: 0.0, z: 0.0 }),
        ];

        let trend = SpatialTemporalTrend::from_trajectory("entity_1", trajectory);
        assert_eq!(trend.trend, "moving_away");
        assert!(trend.avg_velocity > 0.0);
    }
}
