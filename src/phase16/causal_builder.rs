/// Causal Graph Builder V2 with 5 edge detectors for Phase 16
///
/// Detectors: Temporal Proximity, Magnitude Change, Decision Trigger, Multi-Modal Alignment, Historical Validation

use crate::core::event::MissionEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Vertex in the causal graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vertex {
    pub id: String,
    pub vertex_type: String, // sensor_reading, decision, outcome, environment
    pub timestamp_ns: i64,
    pub attributes: HashMap<String, String>,
    pub confidence: f32,
}

impl Vertex {
    pub fn new(id: String, vertex_type: String, timestamp_ns: i64, confidence: f32) -> Self {
        Vertex {
            id,
            vertex_type,
            timestamp_ns,
            attributes: HashMap::new(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    pub fn with_attributes(mut self, attrs: HashMap<String, String>) -> Self {
        self.attributes = attrs;
        self
    }
}

/// Edge in the causal graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String, // causal, correlation, dependency
    pub confidence: f32,
    pub time_gap_ms: i32,
    pub evidence: Vec<String>,
}

impl Edge {
    pub fn new(source_id: String, target_id: String, edge_type: String, confidence: f32, time_gap_ms: i32) -> Self {
        Edge {
            source_id,
            target_id,
            edge_type,
            confidence: confidence.clamp(0.0, 1.0),
            time_gap_ms,
            evidence: Vec::new(),
        }
    }

    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }
}

/// Complete causal graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraphV2 {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub timestamp_ns: i64,
}

impl CausalGraphV2 {
    pub fn new() -> Self {
        CausalGraphV2 {
            vertices: Vec::new(),
            edges: Vec::new(),
            timestamp_ns: Utc::now().timestamp_nanos_opt().unwrap_or(0),
        }
    }

    /// Check if graph maintains DAG property (no cycles)
    pub fn is_dag(&self) -> bool {
        // Build adjacency for cycle detection
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.source_id.clone())
                .or_insert_with(Vec::new)
                .push(edge.target_id.clone());
        }

        // DFS for cycle detection
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for vertex in &self.vertices {
            if !visited.contains(&vertex.id) {
                if self._has_cycle_dfs(&vertex.id, &adj, &mut visited, &mut rec_stack) {
                    return false;
                }
            }
        }
        true
    }

    fn _has_cycle_dfs(
        &self,
        v: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        visited.insert(v.to_string());
        rec_stack.insert(v.to_string());

        if let Some(neighbors) = adj.get(v) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self._has_cycle_dfs(neighbor, adj, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(v);
        false
    }

    /// Get variance explained by edges (for validation)
    pub fn calculate_variance_explained(&self) -> f32 {
        if self.edges.is_empty() {
            return 0.0;
        }

        let total_confidence: f32 = self.edges.iter().map(|e| e.confidence).sum();
        let avg_confidence = total_confidence / self.edges.len() as f32;

        // Normalized confidence as variance proxy
        (avg_confidence * 100.0).min(100.0) / 100.0
    }
}

/// Trait for edge detectors
pub trait EdgeDetector {
    fn detect(&self, timeline: &[MissionEvent], vertices: &[Vertex]) -> Vec<Edge>;
}

/// Detector 1: Temporal Proximity
pub struct TemporalProximityDetector {
    threshold_ms: i64,
}

impl TemporalProximityDetector {
    pub fn new(threshold_ms: i64) -> Self {
        TemporalProximityDetector {
            threshold_ms: threshold_ms.max(10),
        }
    }
}

impl EdgeDetector for TemporalProximityDetector {
    fn detect(&self, timeline: &[MissionEvent], _vertices: &[Vertex]) -> Vec<Edge> {
        let mut edges = Vec::new();

        for i in 0..timeline.len() {
            let source_time = timeline[i].timestamp();
            for j in (i + 1)..timeline.len() {
                let target_time = timeline[j].timestamp();
                let time_gap = (target_time - source_time).num_milliseconds();

                if time_gap > 0 && time_gap < self.threshold_ms {
                    let decay = 1.0 - (time_gap as f32 / self.threshold_ms as f32);
                    let confidence = 0.5 + (decay * 0.3); // 0.5-0.8 range

                    edges.push(
                        Edge::new(
                            format!("event_{}", i),
                            format!("event_{}", j),
                            "temporal_proximity".to_string(),
                            confidence,
                            time_gap as i32,
                        )
                        .with_evidence(vec![format!("time_gap={}ms", time_gap)])
                    );
                }
            }
        }

        edges
    }
}

/// Detector 2: Magnitude Change
pub struct MagnitudeChangeDetector;

impl EdgeDetector for MagnitudeChangeDetector {
    fn detect(&self, _timeline: &[MissionEvent], _vertices: &[Vertex]) -> Vec<Edge> {
        // Placeholder: Would compare sensor magnitudes
        Vec::new()
    }
}

/// Detector 3: Decision Trigger
pub struct DecisionTriggerDetector;

impl EdgeDetector for DecisionTriggerDetector {
    fn detect(&self, timeline: &[MissionEvent], _vertices: &[Vertex]) -> Vec<Edge> {
        let mut edges = Vec::new();

        for (i, source_event) in timeline.iter().enumerate() {
            for (j, target_event) in timeline.iter().enumerate().skip(i + 1) {
                if let (
                    MissionEvent::ObstacleDetected { robot_id: src_robot, .. },
                    MissionEvent::NavigationDecision { robot_id: tgt_robot, .. },
                ) = (source_event, target_event) {
                    if src_robot == tgt_robot {
                        let time_gap = (target_event.timestamp() - source_event.timestamp()).num_milliseconds();
                        edges.push(
                            Edge::new(
                                format!("event_{}", i),
                                format!("event_{}", j),
                                "decision_trigger".to_string(),
                                0.85,
                                time_gap as i32,
                            )
                            .with_evidence(vec!["obstacle_decision_causality".to_string()])
                        );
                    }
                }
            }
        }

        edges
    }
}

/// Detector 4: Multi-Modal Alignment
pub struct MultiModalDetector;

impl EdgeDetector for MultiModalDetector {
    fn detect(&self, _timeline: &[MissionEvent], _vertices: &[Vertex]) -> Vec<Edge> {
        // Placeholder: Would correlate multiple modalities
        Vec::new()
    }
}

/// Detector 5: Historical Validation
pub struct HistoricalDetector {
    fleet_data: Vec<Vec<MissionEvent>>,
}

impl HistoricalDetector {
    pub fn new(fleet_data: Vec<Vec<MissionEvent>>) -> Self {
        HistoricalDetector { fleet_data }
    }
}

impl EdgeDetector for HistoricalDetector {
    fn detect(&self, _timeline: &[MissionEvent], _vertices: &[Vertex]) -> Vec<Edge> {
        // Placeholder: Would validate edges across fleet
        Vec::new()
    }
}

/// Causal Graph Builder V2 with multiple detectors
pub struct CausalGraphBuilderV2 {
    timeline: Vec<MissionEvent>,
    detectors: Vec<Box<dyn EdgeDetector>>,
}

impl CausalGraphBuilderV2 {
    pub fn new(timeline: Vec<MissionEvent>) -> Self {
        let mut detectors: Vec<Box<dyn EdgeDetector>> = Vec::new();

        // Add all 5 detectors
        detectors.push(Box::new(TemporalProximityDetector::new(2000)));
        detectors.push(Box::new(MagnitudeChangeDetector));
        detectors.push(Box::new(DecisionTriggerDetector));
        detectors.push(Box::new(MultiModalDetector));
        detectors.push(Box::new(HistoricalDetector::new(Vec::new())));

        CausalGraphBuilderV2 {
            timeline,
            detectors,
        }
    }

    pub fn build(self) -> CausalGraphV2 {
        let mut graph = CausalGraphV2::new();

        // Extract vertices from timeline
        graph.vertices = self._extract_vertices();

        // Detect edges using all detectors
        for detector in &self.detectors {
            let edges = detector.detect(&self.timeline, &graph.vertices);
            graph.edges.extend(edges);
        }

        // Prune low-confidence edges and enforce DAG
        graph.edges.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        self._enforce_dag(&mut graph);

        graph
    }

    fn _extract_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::new();

        for (idx, event) in self.timeline.iter().enumerate() {
            let ts_ns = event.timestamp().timestamp_nanos_opt().unwrap_or(0);
            let confidence = match event {
                MissionEvent::ObstacleDetected { confidence: Some(c), .. } => *c,
                _ => 0.8,
            };

            let vertex = Vertex::new(
                format!("event_{}", idx),
                "sensor_reading".to_string(),
                ts_ns,
                confidence,
            );

            vertices.push(vertex);
        }

        vertices
    }

    fn _enforce_dag(&self, graph: &mut CausalGraphV2) {
        while !graph.is_dag() {
            // Find lowest-confidence edge creating cycle, remove it
            graph.edges.sort_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap());
            if let Some(edge) = graph.edges.first() {
                graph.edges.remove(0);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let vertex = Vertex::new("v1".to_string(), "sensor_reading".to_string(), 1000, 0.85);
        assert_eq!(vertex.id, "v1");
        assert_eq!(vertex.confidence, 0.85);
    }

    #[test]
    fn test_edge_creation() {
        let edge = Edge::new(
            "v1".to_string(),
            "v2".to_string(),
            "causal".to_string(),
            0.9,
            500,
        );
        assert_eq!(edge.source_id, "v1");
        assert_eq!(edge.confidence, 0.9);
    }

    #[test]
    fn test_graph_is_dag() {
        let mut graph = CausalGraphV2::new();
        graph.vertices.push(Vertex::new("v1".to_string(), "sensor".to_string(), 0, 0.8));
        graph.vertices.push(Vertex::new("v2".to_string(), "sensor".to_string(), 100, 0.8));
        graph.edges.push(Edge::new("v1".to_string(), "v2".to_string(), "causal".to_string(), 0.8, 100));

        assert!(graph.is_dag());
    }

    #[test]
    fn test_temporal_proximity_detector() {
        use crate::core::event::LidarData;
        let base_time = Utc::now();

        let lidar_data = LidarData {
            ranges: vec![1.0, 1.5, 2.0],
            intensities: None,
            frame_id: "laser".to_string(),
            min_angle: -1.57,
            max_angle: 1.57,
            angle_increment: 0.01,
            range_min: 0.1,
            range_max: 30.0,
        };

        let timeline = vec![
            MissionEvent::LidarScan {
                robot_id: "r1".to_string(),
                timestamp: base_time,
                data: lidar_data.clone(),
            },
            MissionEvent::LidarScan {
                robot_id: "r1".to_string(),
                timestamp: base_time + Duration::milliseconds(500),
                data: lidar_data,
            },
        ];

        let detector = TemporalProximityDetector::new(1000);
        let edges = detector.detect(&timeline, &[]);

        assert!(!edges.is_empty());
        assert!(edges[0].confidence > 0.5);
    }
}
