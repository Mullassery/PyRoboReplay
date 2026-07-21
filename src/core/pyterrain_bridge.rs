use crate::core::event::MissionEvent;
use crate::core::spatial_causality::{SpatialContext, SpatialCausalityAnalyzer};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Obstacle representation from PyTerrainMap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obstacle {
    /// Unique obstacle ID
    pub id: String,
    /// Obstacle center position (x, y, z)
    pub position: (f64, f64, f64),
    /// Obstacle bounding box dimensions
    pub dimensions: (f64, f64, f64),
    /// First observation timestamp
    pub first_seen: DateTime<Utc>,
    /// Last observation timestamp
    pub last_seen: DateTime<Utc>,
    /// Obstacle type (wall, object, vehicle, etc.)
    pub obstacle_type: String,
    /// Confidence in obstacle detection (0.0-1.0)
    pub confidence: f32,
    /// Whether obstacle is dynamic (moving)
    pub is_dynamic: bool,
}

/// Traversability data for a spatial region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversabilityZone {
    /// Zone ID
    pub id: String,
    /// Zone center
    pub center: (f64, f64, f64),
    /// Zone radius
    pub radius_m: f64,
    /// Traversability score (0.0-1.0)
    pub traversability: f32,
    /// Terrain type
    pub terrain_type: String,
    /// Number of successful robot passages
    pub successful_passages: usize,
    /// Number of failed attempts
    pub failed_attempts: usize,
}

/// Spatial knowledge graph from PyTerrainMap
#[derive(Debug, Clone)]
pub struct TerrainKnowledgeGraph {
    /// Obstacles detected during mission
    pub obstacles: HashMap<String, Obstacle>,
    /// Traversability zones
    pub traversability_zones: HashMap<String, TraversabilityZone>,
    /// Coverage map (grid-based)
    pub coverage_grid: CoverageMap,
}

/// Coverage map representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMap {
    /// Grid cell size in meters
    pub cell_size: f64,
    /// Grid width (number of cells)
    pub width: usize,
    /// Grid height (number of cells)
    pub height: usize,
    /// Coverage data per cell (0.0-1.0)
    pub data: Vec<f32>,
}

impl CoverageMap {
    /// Create new coverage map
    pub fn new(cell_size: f64, width: usize, height: usize) -> Self {
        CoverageMap {
            cell_size,
            width,
            height,
            data: vec![0.0; width * height],
        }
    }

    /// Get coverage at position
    pub fn get_coverage(&self, x: f64, y: f64) -> f32 {
        let cell_x = (x / self.cell_size) as usize;
        let cell_y = (y / self.cell_size) as usize;

        if cell_x >= self.width || cell_y >= self.height {
            return 0.0;
        }

        self.data[cell_y * self.width + cell_x]
    }

    /// Update coverage at position
    pub fn update_coverage(&mut self, x: f64, y: f64, coverage: f32) {
        let cell_x = (x / self.cell_size) as usize;
        let cell_y = (y / self.cell_size) as usize;

        if cell_x < self.width && cell_y < self.height {
            let idx = cell_y * self.width + cell_x;
            self.data[idx] = self.data[idx].max(coverage);
        }
    }

    /// Calculate total coverage percentage
    pub fn total_coverage_percentage(&self) -> f32 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().sum::<f32>() / self.data.len() as f32 * 100.0
    }
}

impl TerrainKnowledgeGraph {
    /// Create new terrain knowledge graph
    pub fn new() -> Self {
        TerrainKnowledgeGraph {
            obstacles: HashMap::new(),
            traversability_zones: HashMap::new(),
            coverage_grid: CoverageMap::new(0.1, 100, 100),
        }
    }

    /// Add obstacle to graph
    pub fn add_obstacle(&mut self, obstacle: Obstacle) {
        self.obstacles.insert(obstacle.id.clone(), obstacle);
    }

    /// Add traversability zone
    pub fn add_traversability_zone(&mut self, zone: TraversabilityZone) {
        self.traversability_zones.insert(zone.id.clone(), zone);
    }

    /// Query obstacles at specific timestamp
    pub fn query_obstacles_at_time(&self, timestamp: DateTime<Utc>) -> Vec<&Obstacle> {
        self.obstacles
            .values()
            .filter(|obs| obs.first_seen <= timestamp && obs.last_seen >= timestamp)
            .collect()
    }

    /// Query obstacles near position
    pub fn query_obstacles_near_position(
        &self,
        position: (f64, f64, f64),
        radius_m: f64,
    ) -> Vec<&Obstacle> {
        self.obstacles
            .values()
            .filter(|obs| {
                let dx = obs.position.0 - position.0;
                let dy = obs.position.1 - position.1;
                let dz = obs.position.2 - position.2;
                let distance = (dx * dx + dy * dy + dz * dz).sqrt();
                distance <= radius_m
            })
            .collect()
    }

    /// Query traversability at position
    pub fn query_traversability_at_position(&self, position: (f64, f64, f64)) -> f32 {
        self.traversability_zones
            .values()
            .find(|zone| {
                let dx = zone.center.0 - position.0;
                let dy = zone.center.1 - position.1;
                let dz = zone.center.2 - position.2;
                let distance = (dx * dx + dy * dy + dz * dz).sqrt();
                distance <= zone.radius_m
            })
            .map(|zone| zone.traversability)
            .unwrap_or(0.5) // Default moderate traversability
    }

    /// Calculate coverage evolution between timestamps
    pub fn coverage_evolution(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> CoverageEvolution {
        let initial_coverage = self.coverage_grid.total_coverage_percentage();

        CoverageEvolution {
            start_time,
            end_time,
            initial_coverage,
            final_coverage: initial_coverage, // Would be updated by actual exploration
            coverage_gained: 0.0,
            new_obstacles_found: self
                .obstacles
                .values()
                .filter(|obs| obs.first_seen >= start_time && obs.first_seen <= end_time)
                .count(),
        }
    }
}

impl Default for TerrainKnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Coverage evolution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageEvolution {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_coverage: f32,
    pub final_coverage: f32,
    pub coverage_gained: f32,
    pub new_obstacles_found: usize,
}

/// Bridge between spatial causality and PyTerrainMap
pub struct PyTerrainBridge {
    /// Terrain knowledge graph
    pub knowledge_graph: TerrainKnowledgeGraph,
    /// Event timestamp to spatial context cache
    spatial_cache: HashMap<usize, SpatialContext>,
}

impl PyTerrainBridge {
    pub fn new(graph: TerrainKnowledgeGraph) -> Self {
        PyTerrainBridge {
            knowledge_graph: graph,
            spatial_cache: HashMap::new(),
        }
    }

    /// Enrich spatial context with terrain data for an event
    pub fn enrich_spatial_context(
        &mut self,
        event_idx: usize,
        event: &MissionEvent,
        mut context: SpatialContext,
    ) -> SpatialContext {
        // Update traversability based on terrain data
        let traversability =
            self.knowledge_graph.query_traversability_at_position(context.robot_position);
        context = context.with_traversability(traversability as f32);

        // Determine terrain type from nearby obstacles
        let nearby_obstacles = self
            .knowledge_graph
            .query_obstacles_near_position(context.robot_position, 3.0);
        let terrain = if nearby_obstacles.is_empty() {
            "open".to_string()
        } else if nearby_obstacles.len() > 3 {
            "cluttered".to_string()
        } else if nearby_obstacles.iter().any(|obs| obs.is_dynamic) {
            "dynamic".to_string()
        } else {
            "confined".to_string()
        };

        context = context.with_terrain(terrain);

        // Cache result
        self.spatial_cache.insert(event_idx, context.clone());

        context
    }

    /// Apply terrain knowledge to spatial causality analyzer
    pub fn apply_to_analyzer(
        &mut self,
        analyzer: &mut SpatialCausalityAnalyzer,
        events: &[MissionEvent],
    ) {
        for (idx, event) in events.iter().enumerate() {
            // Extract position from event if applicable
            if let Some(position) = self._extract_position(event) {
                let base_context = SpatialContext::new(position, None);
                let enriched = self.enrich_spatial_context(idx, event, base_context);
                analyzer.add_spatial_context(idx, enriched);
            }
        }
    }

    fn _extract_position(&self, event: &MissionEvent) -> Option<(f64, f64, f64)> {
        match event {
            MissionEvent::RobotPose { pose, .. } => Some((pose.x, pose.y, pose.z)),
            MissionEvent::OdometryUpdate { data, .. } => Some((data.pose.x, data.pose.y, data.pose.z)),
            _ => None,
        }
    }

    /// Generate spatial-causal summary report
    pub fn generate_report(&self) -> String {
        format!(
            "PyTerrainMap Bridge Report\n\
             ─────────────────────────────────\n\
             Obstacles tracked: {}\n\
             Traversability zones: {}\n\
             Coverage: {:.1}%\n\
             Cached contexts: {}",
            self.knowledge_graph.obstacles.len(),
            self.knowledge_graph.traversability_zones.len(),
            self.knowledge_graph.coverage_grid.total_coverage_percentage(),
            self.spatial_cache.len()
        )
    }
}

impl Default for PyTerrainBridge {
    fn default() -> Self {
        Self::new(TerrainKnowledgeGraph::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_map_creation() {
        let map = CoverageMap::new(0.1, 100, 100);
        assert_eq!(map.width, 100);
        assert_eq!(map.height, 100);
    }

    #[test]
    fn test_coverage_update() {
        let mut map = CoverageMap::new(0.1, 100, 100);
        map.update_coverage(5.0, 5.0, 0.8);
        assert!(map.get_coverage(5.0, 5.0) > 0.0);
    }

    #[test]
    fn test_terrain_graph_creation() {
        let graph = TerrainKnowledgeGraph::new();
        assert_eq!(graph.obstacles.len(), 0);
        assert_eq!(graph.traversability_zones.len(), 0);
    }

    #[test]
    fn test_add_obstacle() {
        let mut graph = TerrainKnowledgeGraph::new();
        let obstacle = Obstacle {
            id: "obs_1".to_string(),
            position: (1.0, 2.0, 0.0),
            dimensions: (0.5, 0.5, 1.0),
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            obstacle_type: "wall".to_string(),
            confidence: 0.9,
            is_dynamic: false,
        };

        graph.add_obstacle(obstacle);
        assert_eq!(graph.obstacles.len(), 1);
    }

    #[test]
    fn test_query_obstacles_near_position() {
        let mut graph = TerrainKnowledgeGraph::new();
        let obstacle = Obstacle {
            id: "obs_1".to_string(),
            position: (1.0, 1.0, 0.0),
            dimensions: (0.5, 0.5, 1.0),
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            obstacle_type: "wall".to_string(),
            confidence: 0.9,
            is_dynamic: false,
        };

        graph.add_obstacle(obstacle);

        let nearby = graph.query_obstacles_near_position((1.0, 1.0, 0.0), 2.0);
        assert_eq!(nearby.len(), 1);
    }

    #[test]
    fn test_bridge_creation() {
        let graph = TerrainKnowledgeGraph::new();
        let _bridge = PyTerrainBridge::new(graph);
        // Successfully created
    }

    #[test]
    fn test_coverage_percentage() {
        let mut map = CoverageMap::new(0.1, 10, 10);
        map.data = vec![0.5; 100]; // All cells at 50%
        assert_eq!(map.total_coverage_percentage(), 50.0);
    }
}
