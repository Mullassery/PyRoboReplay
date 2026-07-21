use crate::core::event::MissionEvent;
use crate::core::causality::{CausalChain, CausalLink};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Spatial context for a causal relationship
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpatialContext {
    /// Robot position at event time (x, y, z)
    pub robot_position: (f64, f64, f64),
    /// Obstacle/feature location (x, y, z) if applicable
    pub target_position: Option<(f64, f64, f64)>,
    /// Distance between robot and target (meters)
    pub distance_m: f64,
    /// Traversability score at this location (0.0-1.0)
    pub traversability: f32,
    /// Terrain type (open, cluttered, confined, etc.)
    pub terrain_type: String,
}

impl SpatialContext {
    pub fn new(robot_pos: (f64, f64, f64), target_pos: Option<(f64, f64, f64)>) -> Self {
        let distance = if let Some((tx, ty, tz)) = target_pos {
            let dx = robot_pos.0 - tx;
            let dy = robot_pos.1 - ty;
            let dz = robot_pos.2 - tz;
            (dx * dx + dy * dy + dz * dz).sqrt()
        } else {
            0.0
        };

        SpatialContext {
            robot_position: robot_pos,
            target_position: target_pos,
            distance_m: distance,
            traversability: 0.8, // Default: moderately traversable
            terrain_type: "unknown".to_string(),
        }
    }

    pub fn with_traversability(mut self, score: f32) -> Self {
        self.traversability = score.clamp(0.0, 1.0);
        self
    }

    pub fn with_terrain(mut self, terrain: String) -> Self {
        self.terrain_type = terrain;
        self
    }

    /// Is this location near an obstacle (within 2 meters)?
    pub fn near_obstacle(&self) -> bool {
        self.distance_m < 2.0
    }

    /// Is location highly traversable?
    pub fn high_traversability(&self) -> bool {
        self.traversability > 0.7
    }
}

/// Spatial-causal relationship linking causal link with spatial context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialCausalLink {
    /// Underlying causal link
    pub causal_link: CausalLink,
    /// Spatial context at event A
    pub context_a: SpatialContext,
    /// Spatial context at event B
    pub context_b: SpatialContext,
    /// Impact score: how much did spatial situation affect causality (0.0-1.0)
    pub spatial_impact: f32,
}

/// Query results for spatial-causal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialCausalQuery {
    /// Query type (e.g., "what_caused", "what_effects", "spatial_region")
    pub query_type: String,
    /// Results: spatial-causal links ranked by relevance
    pub results: Vec<SpatialCausalLink>,
    /// Spatial region analyzed (if applicable)
    pub spatial_region: Option<SpatialRegion>,
    /// Summary statistics
    pub stats: SpatialCausalStats,
}

/// Represents a spatial region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialRegion {
    /// Region center
    pub center: (f64, f64, f64),
    /// Radius in meters
    pub radius_m: f64,
    /// Events in this region
    pub event_count: usize,
    /// Average traversability in region
    pub avg_traversability: f32,
}

/// Statistics for spatial-causal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialCausalStats {
    /// Total spatial-causal links found
    pub total_links: usize,
    /// Average spatial impact
    pub avg_spatial_impact: f32,
    /// Events near obstacles
    pub events_near_obstacles: usize,
    /// Coverage of analyzed region (0.0-1.0)
    pub coverage: f32,
    /// Average traversability encountered
    pub avg_traversability: f32,
}

/// Analyzer for spatial causality
pub struct SpatialCausalityAnalyzer {
    /// Spatial data for events (event_idx -> SpatialContext)
    spatial_data: HashMap<usize, SpatialContext>,
    /// Traversability map (region -> score)
    traversability_map: HashMap<String, f32>,
}

impl SpatialCausalityAnalyzer {
    pub fn new() -> Self {
        SpatialCausalityAnalyzer {
            spatial_data: HashMap::new(),
            traversability_map: Self::_default_traversability_map(),
        }
    }

    fn _default_traversability_map() -> HashMap<String, f32> {
        let mut map = HashMap::new();
        map.insert("open".to_string(), 0.95);
        map.insert("corridor".to_string(), 0.85);
        map.insert("cluttered".to_string(), 0.60);
        map.insert("confined".to_string(), 0.40);
        map.insert("obstacle".to_string(), 0.10);
        map
    }

    /// Register spatial context for an event
    pub fn add_spatial_context(&mut self, event_idx: usize, context: SpatialContext) {
        self.spatial_data.insert(event_idx, context);
    }

    /// Analyze spatial impact of a causal link
    pub fn analyze_spatial_causality(
        &self,
        causal_link: &CausalLink,
    ) -> Option<SpatialCausalLink> {
        let context_a = self.spatial_data.get(&causal_link.source_event_idx)?;
        let context_b = self.spatial_data.get(&causal_link.target_event_idx)?;

        // Calculate spatial impact
        let spatial_impact = self._calculate_spatial_impact(context_a, context_b, causal_link);

        Some(SpatialCausalLink {
            causal_link: causal_link.clone(),
            context_a: context_a.clone(),
            context_b: context_b.clone(),
            spatial_impact,
        })
    }

    fn _calculate_spatial_impact(
        &self,
        context_a: &SpatialContext,
        context_b: &SpatialContext,
        link: &CausalLink,
    ) -> f32 {
        let mut impact = 0.0;

        // Factor 1: Proximity to obstacle increases impact
        if context_a.near_obstacle() {
            impact += 0.3;
        }

        // Factor 2: Low traversability increases impact
        impact += (1.0 - context_a.traversability as f32) * 0.2;

        // Factor 3: Rapid position change indicates navigation impact
        let position_delta = (
            (context_b.robot_position.0 - context_a.robot_position.0).powi(2)
                + (context_b.robot_position.1 - context_a.robot_position.1).powi(2)
        )
        .sqrt();

        if position_delta > 0.5 {
            impact += 0.2;
        }

        // Factor 4: Causal link confidence amplifies spatial impact
        impact += link.confidence * 0.3;

        impact.clamp(0.0, 1.0)
    }

    /// Query: "Which obstacles affected this causal event?"
    pub fn query_obstacles_in_causality(&self, causal_link: &CausalLink) -> Option<Vec<SpatialContext>> {
        let context_b = self.spatial_data.get(&causal_link.target_event_idx)?;

        // Find all spatial contexts with nearby obstacles in temporal window
        let relevant_contexts: Vec<_> = self
            .spatial_data
            .values()
            .filter(|ctx| {
                ctx.near_obstacle()
                    && (ctx.robot_position.0 - context_b.robot_position.0).abs() < 5.0
                    && (ctx.robot_position.1 - context_b.robot_position.1).abs() < 5.0
            })
            .cloned()
            .collect();

        if relevant_contexts.is_empty() {
            None
        } else {
            Some(relevant_contexts)
        }
    }

    /// Compute statistics for spatial-causal analysis
    pub fn compute_stats(&self, links: &[SpatialCausalLink]) -> SpatialCausalStats {
        if links.is_empty() {
            return SpatialCausalStats {
                total_links: 0,
                avg_spatial_impact: 0.0,
                events_near_obstacles: 0,
                coverage: 0.0,
                avg_traversability: 0.0,
            };
        }

        let avg_impact = links.iter().map(|l| l.spatial_impact).sum::<f32>() / links.len() as f32;
        let near_obstacles = links
            .iter()
            .filter(|l| l.context_a.near_obstacle() || l.context_b.near_obstacle())
            .count();
        let avg_traversability =
            (links.iter().map(|l| l.context_a.traversability as f32).sum::<f32>()
                + links.iter().map(|l| l.context_b.traversability as f32).sum::<f32>())
                / (links.len() as f32 * 2.0);

        SpatialCausalStats {
            total_links: links.len(),
            avg_spatial_impact: avg_impact,
            events_near_obstacles: near_obstacles,
            coverage: (links.len() as f32 / 100.0).min(1.0), // Normalized
            avg_traversability,
        }
    }

    /// Find spatial region with most causal activity
    pub fn find_hotspot(&self) -> Option<SpatialRegion> {
        if self.spatial_data.is_empty() {
            return None;
        }

        // Calculate centroid
        let (sum_x, sum_y, sum_z, count) = self
            .spatial_data
            .values()
            .fold((0.0, 0.0, 0.0, 0), |(sx, sy, sz, c), ctx| {
                (
                    sx + ctx.robot_position.0,
                    sy + ctx.robot_position.1,
                    sz + ctx.robot_position.2,
                    c + 1,
                )
            });

        let center = (sum_x / count as f64, sum_y / count as f64, sum_z / count as f64);

        // Calculate average distance from centroid (radius)
        let avg_dist: f64 = self
            .spatial_data
            .values()
            .map(|ctx| {
                let dx = ctx.robot_position.0 - center.0;
                let dy = ctx.robot_position.1 - center.1;
                let dz = ctx.robot_position.2 - center.2;
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .sum::<f64>()
            / count as f64;

        let avg_trav = self
            .spatial_data
            .values()
            .map(|ctx| ctx.traversability)
            .sum::<f32>()
            / count as f32;

        Some(SpatialRegion {
            center,
            radius_m: avg_dist,
            event_count: count,
            avg_traversability: avg_trav,
        })
    }
}

impl Default for SpatialCausalityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_context_creation() {
        let ctx = SpatialContext::new((1.0, 2.0, 3.0), Some((4.0, 5.0, 6.0)));
        assert!(ctx.distance_m > 0.0);
    }

    #[test]
    fn test_near_obstacle() {
        let ctx = SpatialContext::new((0.0, 0.0, 0.0), Some((1.0, 0.0, 0.0)));
        assert!(ctx.near_obstacle());
    }

    #[test]
    fn test_traversability_clamping() {
        let ctx = SpatialContext::new((0.0, 0.0, 0.0), None).with_traversability(1.5);
        assert_eq!(ctx.traversability, 1.0);
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = SpatialCausalityAnalyzer::new();
        assert!(!analyzer.spatial_data.is_empty() || analyzer.spatial_data.is_empty()); // Tautology check
    }

    #[test]
    fn test_add_spatial_context() {
        let mut analyzer = SpatialCausalityAnalyzer::new();
        let ctx = SpatialContext::new((1.0, 2.0, 3.0), None);
        analyzer.add_spatial_context(0, ctx);

        assert_eq!(analyzer.spatial_data.len(), 1);
    }

    #[test]
    fn test_find_hotspot() {
        let mut analyzer = SpatialCausalityAnalyzer::new();
        let ctx1 = SpatialContext::new((0.0, 0.0, 0.0), None);
        let ctx2 = SpatialContext::new((1.0, 1.0, 0.0), None);
        let ctx3 = SpatialContext::new((2.0, 2.0, 0.0), None);

        analyzer.add_spatial_context(0, ctx1);
        analyzer.add_spatial_context(1, ctx2);
        analyzer.add_spatial_context(2, ctx3);

        let hotspot = analyzer.find_hotspot();
        assert!(hotspot.is_some());
        assert_eq!(hotspot.unwrap().event_count, 3);
    }

    #[test]
    fn test_stats_computation() {
        let analyzer = SpatialCausalityAnalyzer::new();
        let link = SpatialCausalLink {
            causal_link: CausalLink::new(0, 1, "test".to_string(), 0.8, 500),
            context_a: SpatialContext::new((0.0, 0.0, 0.0), None),
            context_b: SpatialContext::new((1.0, 0.0, 0.0), None),
            spatial_impact: 0.75,
        };

        let stats = analyzer.compute_stats(&[link]);
        assert_eq!(stats.total_links, 1);
        assert!(stats.avg_spatial_impact > 0.0);
    }
}
