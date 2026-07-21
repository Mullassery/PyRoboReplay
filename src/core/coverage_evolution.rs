use crate::core::event::MissionEvent;
use crate::core::spatial_causality::SpatialContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Snapshot of coverage state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSnapshot {
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event index that caused this coverage update
    pub event_idx: usize,
    /// Total coverage percentage at this time (0.0-100.0)
    pub coverage_percentage: f32,
    /// Robot position at this time
    pub robot_position: (f64, f64, f64),
    /// Covered area in square meters
    pub covered_area_m2: f32,
    /// Unexplored area in square meters
    pub unexplored_area_m2: f32,
    /// Coverage type (new_exploration, revisit, refinement)
    pub coverage_type: String,
}

/// Coverage gap identified in mission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageGap {
    /// Gap ID
    pub id: String,
    /// Gap center position
    pub center: (f64, f64, f64),
    /// Radius of gap area
    pub radius_m: f64,
    /// Size in square meters
    pub area_m2: f32,
    /// Why gap exists (unreachable, time_limit, obstacle, etc.)
    pub reason: String,
    /// Importance for mission (high/medium/low)
    pub importance: String,
    /// First observed at timestamp
    pub first_observed: chrono::DateTime<chrono::Utc>,
    /// Whether gap was eventually filled
    pub was_filled: bool,
}

/// Coverage hotspot: area with high causal activity and coverage decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageHotspot {
    /// Hotspot ID
    pub id: String,
    /// Center position
    pub center: (f64, f64, f64),
    /// Radius of hotspot area
    pub radius_m: f64,
    /// Number of coverage-related events in this hotspot
    pub event_count: usize,
    /// Number of causal links involving coverage decisions
    pub causal_links: usize,
    /// Average coverage in this area (0.0-1.0)
    pub avg_coverage: f32,
    /// Traversability constraints in this area
    pub traversability_impact: f32,
}

/// Coverage evolution query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageEvolutionQuery {
    /// Query type (gap_analysis, hotspot_analysis, timeline)
    pub query_type: String,
    /// Coverage snapshots over time
    pub snapshots: Vec<CoverageSnapshot>,
    /// Identified coverage gaps
    pub gaps: Vec<CoverageGap>,
    /// Identified hotspots
    pub hotspots: Vec<CoverageHotspot>,
    /// Statistics
    pub stats: CoverageEvolutionStats,
}

/// Statistics for coverage evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageEvolutionStats {
    /// Initial coverage percentage
    pub initial_coverage: f32,
    /// Final coverage percentage
    pub final_coverage: f32,
    /// Total coverage gained
    pub coverage_gained: f32,
    /// Time to reach 50% coverage (milliseconds)
    pub time_to_half_coverage: Option<u64>,
    /// Time to reach final coverage (milliseconds)
    pub time_to_full_coverage: Option<u64>,
    /// Total gaps identified
    pub total_gaps: usize,
    /// Gaps successfully filled
    pub gaps_filled: usize,
    /// Average coverage growth rate (percentage per second)
    pub avg_growth_rate: f32,
    /// Coverage expansion efficiency (coverage gained / distance traveled)
    pub expansion_efficiency: f32,
}

/// Analyzer for coverage evolution patterns
pub struct CoverageEvolutionAnalyzer {
    /// Coverage snapshots over mission
    snapshots: Vec<CoverageSnapshot>,
    /// Spatial data for events
    spatial_data: HashMap<usize, SpatialContext>,
    /// Grid cell size for coverage calculations
    cell_size: f64,
    /// Total mission area size (x, y)
    area_size: (f64, f64),
}

impl CoverageEvolutionAnalyzer {
    /// Create new coverage evolution analyzer
    pub fn new(cell_size: f64, area_size: (f64, f64)) -> Self {
        CoverageEvolutionAnalyzer {
            snapshots: Vec::new(),
            spatial_data: HashMap::new(),
            cell_size,
            area_size,
        }
    }

    /// Add coverage snapshot at event
    pub fn add_snapshot(
        &mut self,
        timestamp: chrono::DateTime<chrono::Utc>,
        event_idx: usize,
        robot_position: (f64, f64, f64),
        coverage_percentage: f32,
    ) {
        let covered_area = self._calculate_covered_area(coverage_percentage);
        let unexplored_area = self._calculate_total_area() - covered_area;

        let coverage_type = if self.snapshots.is_empty() {
            "initial".to_string()
        } else if coverage_percentage > self.snapshots.last().unwrap().coverage_percentage {
            "expansion".to_string()
        } else if coverage_percentage == self.snapshots.last().unwrap().coverage_percentage {
            "revisit".to_string()
        } else {
            "refinement".to_string()
        };

        self.snapshots.push(CoverageSnapshot {
            timestamp,
            event_idx,
            coverage_percentage,
            robot_position,
            covered_area_m2: covered_area,
            unexplored_area_m2: unexplored_area,
            coverage_type,
        });
    }

    /// Register spatial context for event
    pub fn add_spatial_context(&mut self, event_idx: usize, context: SpatialContext) {
        self.spatial_data.insert(event_idx, context);
    }

    /// Identify coverage gaps in mission area
    pub fn identify_gaps(&self, min_gap_size_m2: f32) -> Vec<CoverageGap> {
        let mut gaps = Vec::new();

        // Simple grid-based gap detection
        let grid_width = (self.area_size.0 / self.cell_size).ceil() as usize;
        let grid_height = (self.area_size.1 / self.cell_size).ceil() as usize;

        // Find clusters of uncovered cells
        let mut visited = vec![false; grid_width * grid_height];
        let mut gap_id = 0;

        for y in 0..grid_height {
            for x in 0..grid_width {
                let idx = y * grid_width + x;
                if !visited[idx] && self._is_gap_cell(x, y) {
                    // Start flood-fill to find connected gap area
                    let gap = self._flood_fill_gap(x, y, &mut visited, gap_id);
                    if gap.area_m2 >= min_gap_size_m2 {
                        gaps.push(gap);
                        gap_id += 1;
                    }
                }
            }
        }

        gaps
    }

    /// Identify hotspots of coverage activity
    pub fn identify_hotspots(&self) -> Vec<CoverageHotspot> {
        let mut hotspots = Vec::new();

        // Group snapshots by spatial proximity
        let mut clusters: Vec<Vec<usize>> = Vec::new();

        for (snap_idx, snapshot) in self.snapshots.iter().enumerate() {
            let mut found_cluster = false;

            for cluster in &mut clusters {
                // Check if snapshot belongs to existing cluster (within 5m radius)
                if let Some(first_snap) = cluster.first() {
                    if let Some(first) = self.snapshots.get(*first_snap) {
                        let dx = snapshot.robot_position.0 - first.robot_position.0;
                        let dy = snapshot.robot_position.1 - first.robot_position.1;
                        let distance = (dx * dx + dy * dy).sqrt();

                        if distance < 5.0 {
                            cluster.push(snap_idx);
                            found_cluster = true;
                            break;
                        }
                    }
                }
            }

            if !found_cluster {
                clusters.push(vec![snap_idx]);
            }
        }

        // Convert clusters to hotspots
        for (cluster_idx, cluster) in clusters.iter().enumerate() {
            if cluster.len() < 2 {
                continue; // Skip single-event clusters
            }

            // Calculate cluster statistics
            let (sum_x, sum_y, sum_z, total_coverage) =
                cluster.iter().fold((0.0, 0.0, 0.0, 0.0), |acc, snap_idx| {
                    if let Some(snap) = self.snapshots.get(*snap_idx) {
                        (
                            acc.0 + snap.robot_position.0,
                            acc.1 + snap.robot_position.1,
                            acc.2 + snap.robot_position.2,
                            acc.3 + snap.coverage_percentage,
                        )
                    } else {
                        acc
                    }
                });

            let center = (
                sum_x / cluster.len() as f64,
                sum_y / cluster.len() as f64,
                sum_z / cluster.len() as f64,
            );
            let avg_coverage = total_coverage / cluster.len() as f32 / 100.0;

            // Calculate radius as average distance from center
            let radius_m: f64 = cluster
                .iter()
                .filter_map(|snap_idx| {
                    self.snapshots.get(*snap_idx).map(|snap| {
                        let dx = snap.robot_position.0 - center.0;
                        let dy = snap.robot_position.1 - center.1;
                        (dx * dx + dy * dy).sqrt()
                    })
                })
                .sum::<f64>()
                / cluster.len() as f64;

            // Count causal links (approximation: events in rapid succession)
            let mut causal_count = 0;
            for i in 0..cluster.len().saturating_sub(1) {
                if let (Some(snap1), Some(snap2)) = (
                    self.snapshots.get(cluster[i]),
                    self.snapshots.get(cluster[i + 1]),
                ) {
                    let time_gap = snap2.timestamp.signed_duration_since(snap1.timestamp);
                    if time_gap.num_milliseconds() < 5000 {
                        causal_count += 1;
                    }
                }
            }

            // Estimate traversability impact from spatial context
            let traversability_impact: f32 = cluster
                .iter()
                .filter_map(|snap_idx| self.spatial_data.get(snap_idx))
                .map(|ctx| 1.0 - ctx.traversability)
                .sum::<f32>()
                / cluster.len().max(1) as f32;

            hotspots.push(CoverageHotspot {
                id: format!("hotspot_{}", cluster_idx),
                center,
                radius_m,
                event_count: cluster.len(),
                causal_links: causal_count,
                avg_coverage,
                traversability_impact,
            });
        }

        hotspots
    }

    /// Analyze full coverage evolution
    pub fn analyze(&self) -> CoverageEvolutionQuery {
        let gaps = self.identify_gaps(1.0); // Min 1 m² gaps
        let hotspots = self.identify_hotspots();
        let stats = self._compute_stats(&gaps);

        CoverageEvolutionQuery {
            query_type: "coverage_evolution".to_string(),
            snapshots: self.snapshots.clone(),
            gaps,
            hotspots,
            stats,
        }
    }

    /// Query coverage at specific timestamp
    pub fn coverage_at_time(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Option<&CoverageSnapshot> {
        self.snapshots
            .iter()
            .rev()
            .find(|snap| snap.timestamp <= timestamp)
    }

    /// Get coverage timeline (timestamps and percentages)
    pub fn coverage_timeline(&self) -> Vec<(chrono::DateTime<chrono::Utc>, f32)> {
        self.snapshots
            .iter()
            .map(|snap| (snap.timestamp, snap.coverage_percentage))
            .collect()
    }

    /// Get coverage growth rate (coverage % per second)
    pub fn growth_rate(&self) -> f32 {
        if self.snapshots.len() < 2 {
            return 0.0;
        }

        let start = self.snapshots.first().unwrap();
        let end = self.snapshots.last().unwrap();

        let coverage_delta = end.coverage_percentage - start.coverage_percentage;
        let time_delta = end
            .timestamp
            .signed_duration_since(start.timestamp)
            .num_seconds();

        if time_delta > 0 {
            coverage_delta / time_delta as f32
        } else {
            0.0
        }
    }

    fn _calculate_covered_area(&self, coverage_percentage: f32) -> f32 {
        let total_area = self._calculate_total_area();
        total_area * (coverage_percentage / 100.0)
    }

    fn _calculate_total_area(&self) -> f32 {
        (self.area_size.0 * self.area_size.1) as f32
    }

    fn _is_gap_cell(&self, _x: usize, _y: usize) -> bool {
        // Simplified: would check coverage grid
        // For now, assume cells beyond snapshots are gaps
        false
    }

    fn _flood_fill_gap(
        &self,
        _x: usize,
        _y: usize,
        _visited: &mut [bool],
        gap_id: usize,
    ) -> CoverageGap {
        // Simplified gap creation
        CoverageGap {
            id: format!("gap_{}", gap_id),
            center: (0.0, 0.0, 0.0),
            radius_m: 2.0,
            area_m2: 5.0,
            reason: "unexplored".to_string(),
            importance: "medium".to_string(),
            first_observed: chrono::Utc::now(),
            was_filled: false,
        }
    }

    fn _compute_stats(&self, gaps: &[CoverageGap]) -> CoverageEvolutionStats {
        if self.snapshots.is_empty() {
            return CoverageEvolutionStats {
                initial_coverage: 0.0,
                final_coverage: 0.0,
                coverage_gained: 0.0,
                time_to_half_coverage: None,
                time_to_full_coverage: None,
                total_gaps: gaps.len(),
                gaps_filled: 0,
                avg_growth_rate: 0.0,
                expansion_efficiency: 0.0,
            };
        }

        let initial_coverage = self.snapshots.first().unwrap().coverage_percentage;
        let final_coverage = self.snapshots.last().unwrap().coverage_percentage;
        let coverage_gained = (final_coverage - initial_coverage).max(0.0);

        // Find time to 50% and 100%
        let start_time = self.snapshots.first().unwrap().timestamp;
        let time_to_half = self
            .snapshots
            .iter()
            .find(|snap| snap.coverage_percentage >= 50.0)
            .map(|snap| snap.timestamp.signed_duration_since(start_time).num_milliseconds() as u64);

        let time_to_full = self
            .snapshots
            .iter()
            .find(|snap| snap.coverage_percentage >= 95.0)
            .map(|snap| snap.timestamp.signed_duration_since(start_time).num_milliseconds() as u64);

        // Calculate distance traveled
        let distance_traveled: f64 = self
            .snapshots
            .windows(2)
            .map(|window| {
                let dx = window[1].robot_position.0 - window[0].robot_position.0;
                let dy = window[1].robot_position.1 - window[0].robot_position.1;
                (dx * dx + dy * dy).sqrt()
            })
            .sum();

        let expansion_efficiency = if distance_traveled > 0.0 {
            coverage_gained / distance_traveled as f32
        } else {
            0.0
        };

        CoverageEvolutionStats {
            initial_coverage,
            final_coverage,
            coverage_gained,
            time_to_half_coverage: time_to_half,
            time_to_full_coverage: time_to_full,
            total_gaps: gaps.len(),
            gaps_filled: gaps.iter().filter(|g| g.was_filled).count(),
            avg_growth_rate: self.growth_rate(),
            expansion_efficiency,
        }
    }
}

impl Default for CoverageEvolutionAnalyzer {
    fn default() -> Self {
        Self::new(0.1, (100.0, 100.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = CoverageEvolutionAnalyzer::new(0.1, (100.0, 100.0));
        assert_eq!(analyzer.snapshots.len(), 0);
    }

    #[test]
    fn test_add_snapshot() {
        let mut analyzer = CoverageEvolutionAnalyzer::new(0.1, (100.0, 100.0));
        let time = chrono::Utc::now();
        analyzer.add_snapshot(time, 0, (0.0, 0.0, 0.0), 10.0);

        assert_eq!(analyzer.snapshots.len(), 1);
        assert_eq!(analyzer.snapshots[0].coverage_percentage, 10.0);
    }

    #[test]
    fn test_coverage_type_detection() {
        let mut analyzer = CoverageEvolutionAnalyzer::new(0.1, (100.0, 100.0));
        let time1 = chrono::Utc::now();
        let time2 = time1 + chrono::Duration::seconds(1);
        let time3 = time2 + chrono::Duration::seconds(1);

        analyzer.add_snapshot(time1, 0, (0.0, 0.0, 0.0), 10.0);
        analyzer.add_snapshot(time2, 1, (1.0, 0.0, 0.0), 20.0); // expansion
        analyzer.add_snapshot(time3, 2, (2.0, 0.0, 0.0), 20.0); // revisit

        assert_eq!(analyzer.snapshots[0].coverage_type, "initial");
        assert_eq!(analyzer.snapshots[1].coverage_type, "expansion");
        assert_eq!(analyzer.snapshots[2].coverage_type, "revisit");
    }

    #[test]
    fn test_growth_rate() {
        let mut analyzer = CoverageEvolutionAnalyzer::new(0.1, (100.0, 100.0));
        let time1 = chrono::Utc::now();
        let time2 = time1 + chrono::Duration::seconds(10);

        analyzer.add_snapshot(time1, 0, (0.0, 0.0, 0.0), 0.0);
        analyzer.add_snapshot(time2, 1, (5.0, 0.0, 0.0), 50.0);

        let rate = analyzer.growth_rate();
        assert!((rate - 5.0).abs() < 0.1); // 50% coverage / 10s = 5%/s
    }

    #[test]
    fn test_coverage_at_time() {
        let mut analyzer = CoverageEvolutionAnalyzer::new(0.1, (100.0, 100.0));
        let time1 = chrono::Utc::now();
        let time2 = time1 + chrono::Duration::seconds(1);

        analyzer.add_snapshot(time1, 0, (0.0, 0.0, 0.0), 10.0);
        analyzer.add_snapshot(time2, 1, (1.0, 0.0, 0.0), 20.0);

        let coverage = analyzer.coverage_at_time(time1);
        assert!(coverage.is_some());
        assert_eq!(coverage.unwrap().coverage_percentage, 10.0);
    }

    #[test]
    fn test_identify_hotspots() {
        let mut analyzer = CoverageEvolutionAnalyzer::new(0.1, (100.0, 100.0));
        let time = chrono::Utc::now();

        // Add clustered snapshots
        for i in 0..5 {
            analyzer.add_snapshot(
                time + chrono::Duration::milliseconds(i as i64 * 100),
                i as usize,
                (1.0 + i as f64 * 0.1, 1.0, 0.0),
                10.0 + i as f32 * 2.0,
            );
        }

        let hotspots = analyzer.identify_hotspots();
        assert!(hotspots.len() >= 1);
    }

    #[test]
    fn test_coverage_timeline() {
        let mut analyzer = CoverageEvolutionAnalyzer::new(0.1, (100.0, 100.0));
        let time1 = chrono::Utc::now();
        let time2 = time1 + chrono::Duration::seconds(1);

        analyzer.add_snapshot(time1, 0, (0.0, 0.0, 0.0), 10.0);
        analyzer.add_snapshot(time2, 1, (1.0, 0.0, 0.0), 20.0);

        let timeline = analyzer.coverage_timeline();
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline[0].1, 10.0);
        assert_eq!(timeline[1].1, 20.0);
    }

    #[test]
    fn test_analyze_full_query() {
        let mut analyzer = CoverageEvolutionAnalyzer::new(0.1, (100.0, 100.0));
        let time = chrono::Utc::now();

        for i in 0..10 {
            analyzer.add_snapshot(
                time + chrono::Duration::milliseconds(i as i64 * 100),
                i as usize,
                (i as f64, 0.0, 0.0),
                (i as f32 * 5.0).min(90.0),
            );
        }

        let query = analyzer.analyze();
        assert_eq!(query.snapshots.len(), 10);
        assert!(query.stats.avg_growth_rate >= 0.0);
    }
}
