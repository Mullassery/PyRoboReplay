//! Scene Understanding Layer
//!
//! Infers spatial relationships, semantic context, and environmental conditions.
//! Answers: What did the robot understand about its environment?

use crate::perception::object_detection::ObjectClass;
use std::collections::HashMap;

/// Spatial relationship between two objects
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpatialRelationship {
    PersonCrossingPath,
    VehicleApproachingIntersection,
    PalletBlockingAisle,
    VehicleOvertakingRobot,
    PedestrianEnteringSafetyZone,
    ObjectBehindRobot,
    ObjectToLeftOfRobot,
    ObjectToRightOfRobot,
    ObjectAboveRobot,
    ObjectBlockingPath,
    ObjectParallel,
    ObjectApproaching,
    ObjectReceding,
}

impl std::fmt::Display for SpatialRelationship {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SpatialRelationship::PersonCrossingPath => write!(f, "Person crossing path"),
            SpatialRelationship::VehicleApproachingIntersection => {
                write!(f, "Vehicle approaching intersection")
            }
            SpatialRelationship::PalletBlockingAisle => write!(f, "Pallet blocking aisle"),
            SpatialRelationship::VehicleOvertakingRobot => write!(f, "Vehicle overtaking robot"),
            SpatialRelationship::PedestrianEnteringSafetyZone => {
                write!(f, "Pedestrian entering safety zone")
            }
            SpatialRelationship::ObjectBehindRobot => write!(f, "Object behind robot"),
            SpatialRelationship::ObjectToLeftOfRobot => write!(f, "Object to left of robot"),
            SpatialRelationship::ObjectToRightOfRobot => write!(f, "Object to right of robot"),
            SpatialRelationship::ObjectAboveRobot => write!(f, "Object above robot"),
            SpatialRelationship::ObjectBlockingPath => write!(f, "Object blocking path"),
            SpatialRelationship::ObjectParallel => write!(f, "Object parallel to robot"),
            SpatialRelationship::ObjectApproaching => write!(f, "Object approaching"),
            SpatialRelationship::ObjectReceding => write!(f, "Object receding"),
        }
    }
}

/// Semantic scene context
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticContext {
    CrowdedArea,
    OpenCorridor,
    LoadingZone,
    Intersection,
    ConstructionArea,
    WarehouseRack,
    ParkingLot,
    Sidewalk,
    HighTrafficArea,
    RestrictedArea,
}

impl std::fmt::Display for SemanticContext {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SemanticContext::CrowdedArea => write!(f, "Crowded area"),
            SemanticContext::OpenCorridor => write!(f, "Open corridor"),
            SemanticContext::LoadingZone => write!(f, "Loading zone"),
            SemanticContext::Intersection => write!(f, "Intersection"),
            SemanticContext::ConstructionArea => write!(f, "Construction area"),
            SemanticContext::WarehouseRack => write!(f, "Warehouse rack"),
            SemanticContext::ParkingLot => write!(f, "Parking lot"),
            SemanticContext::Sidewalk => write!(f, "Sidewalk"),
            SemanticContext::HighTrafficArea => write!(f, "High traffic area"),
            SemanticContext::RestrictedArea => write!(f, "Restricted area"),
        }
    }
}

/// Environmental conditions affecting perception
#[derive(Debug, Clone)]
pub struct EnvironmentalCondition {
    /// Condition type
    pub condition_type: String,

    /// Severity (0.0-1.0)
    pub severity: f32,

    /// How much this affects detection
    pub detection_impact: f32,

    /// Affected area (percentage of frame)
    pub affected_area_percent: f32,
}

/// Scene understanding output
#[derive(Debug, Clone)]
pub struct SceneUnderstanding {
    /// Timestamp
    pub timestamp_sec: f32,

    /// Spatial relationships identified
    pub spatial_relationships: Vec<(ObjectClass, SpatialRelationship)>,

    /// Semantic context
    pub context: Option<SemanticContext>,

    /// Environmental factors
    pub environmental_conditions: Vec<EnvironmentalCondition>,

    /// Scene complexity score (0.0-1.0)
    pub complexity_score: f32,

    /// Pedestrian density
    pub pedestrian_density: f32,

    /// Dynamic obstacle count
    pub dynamic_obstacle_count: usize,

    /// Occlusion level (0.0-1.0)
    pub occlusion_level: f32,

    /// Visibility score (0.0-1.0)
    pub visibility_score: f32,
}

/// Infers scene understanding from detected objects
pub struct SceneUnderstandingEngine;

impl SceneUnderstandingEngine {
    /// Analyze scene from detected objects
    pub fn analyze_scene(
        objects: &[(ObjectClass, f32, (f32, f32, f32))], // class, confidence, position
        robot_position: (f32, f32, f32),
        safety_radius: f32,
    ) -> SceneUnderstanding {
        let mut spatial_relationships = Vec::new();
        let mut dynamic_obstacle_count = 0;
        let mut pedestrian_count = 0;

        // Analyze each object
        for (class, _conf, pos) in objects {
            // Calculate distance to robot
            let distance = ((pos.0 - robot_position.0).powi(2)
                + (pos.1 - robot_position.1).powi(2))
                .sqrt();

            // Check spatial relationships
            match class {
                ObjectClass::Person => {
                    pedestrian_count += 1;
                    if distance < safety_radius {
                        spatial_relationships
                            .push((*class, SpatialRelationship::PedestrianEnteringSafetyZone));
                    }
                    if Self::is_crossing_path(pos, robot_position) {
                        spatial_relationships
                            .push((*class, SpatialRelationship::PersonCrossingPath));
                    }
                }
                ObjectClass::Vehicle => {
                    dynamic_obstacle_count += 1;
                    if Self::is_approaching(pos, robot_position) {
                        spatial_relationships
                            .push((*class, SpatialRelationship::VehicleApproachingIntersection));
                    }
                }
                ObjectClass::DynamicObstacle => {
                    dynamic_obstacle_count += 1;
                    if Self::is_blocking_path(pos, robot_position) {
                        spatial_relationships
                            .push((*class, SpatialRelationship::ObjectBlockingPath));
                    }
                }
                _ => {}
            }
        }

        // Infer semantic context
        let context = Self::infer_context(pedestrian_count, dynamic_obstacle_count);

        // Compute complexity
        let complexity_score = Self::compute_complexity(pedestrian_count, dynamic_obstacle_count);

        SceneUnderstanding {
            timestamp_sec: 0.0,
            spatial_relationships,
            context,
            environmental_conditions: Vec::new(),
            complexity_score,
            pedestrian_density: (pedestrian_count as f32).min(10.0) / 10.0,
            dynamic_obstacle_count,
            occlusion_level: 0.0,
            visibility_score: 1.0,
        }
    }

    /// Check if object is crossing robot path
    fn is_crossing_path(obj_pos: &(f32, f32, f32), robot_pos: (f32, f32, f32)) -> bool {
        let cross_threshold = 1.5; // meters
        (obj_pos.0 - robot_pos.0).abs() < cross_threshold
    }

    /// Check if object is approaching
    fn is_approaching(obj_pos: &(f32, f32, f32), robot_pos: (f32, f32, f32)) -> bool {
        let distance = ((obj_pos.0 - robot_pos.0).powi(2) + (obj_pos.1 - robot_pos.1).powi(2)).sqrt();
        distance < 5.0 // Within 5 meters
    }

    /// Check if object is blocking path
    fn is_blocking_path(obj_pos: &(f32, f32, f32), robot_pos: (f32, f32, f32)) -> bool {
        let forward_threshold = 3.0; // meters ahead
        obj_pos.0 > robot_pos.0 && (obj_pos.0 - robot_pos.0) < forward_threshold
    }

    /// Infer semantic context from scene composition
    fn infer_context(pedestrian_count: usize, dynamic_obstacles: usize) -> Option<SemanticContext> {
        if pedestrian_count > 5 {
            Some(SemanticContext::CrowdedArea)
        } else if pedestrian_count > 0 && dynamic_obstacles > 0 {
            Some(SemanticContext::HighTrafficArea)
        } else if dynamic_obstacles > 0 {
            Some(SemanticContext::Intersection)
        } else {
            Some(SemanticContext::OpenCorridor)
        }
    }

    /// Compute scene complexity
    fn compute_complexity(pedestrian_count: usize, dynamic_obstacles: usize) -> f32 {
        let base = (pedestrian_count as f32 * 0.3 + dynamic_obstacles as f32 * 0.2).min(1.0);
        (base + 0.2).min(1.0) // Add base complexity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_analysis_empty() {
        let scene = SceneUnderstandingEngine::analyze_scene(&[], (0.0, 0.0, 0.0), 2.0);

        assert_eq!(scene.pedestrian_density, 0.0);
        assert_eq!(scene.dynamic_obstacle_count, 0);
    }

    #[test]
    fn test_scene_analysis_with_pedestrian() {
        let objects = vec![(ObjectClass::Person, 0.95, (1.0, 0.0, 0.0))];

        let scene = SceneUnderstandingEngine::analyze_scene(&objects, (0.0, 0.0, 0.0), 2.0);

        assert!(scene.pedestrian_density > 0.0);
    }

    #[test]
    fn test_crowded_scene_context() {
        let mut objects = Vec::new();
        for i in 0..6 {
            objects.push((ObjectClass::Person, 0.95, (i as f32, 0.0, 0.0)));
        }

        let scene = SceneUnderstandingEngine::analyze_scene(&objects, (0.0, 0.0, 0.0), 2.0);

        assert_eq!(scene.context, Some(SemanticContext::CrowdedArea));
    }
}
