use crate::core::event::MissionEvent;
use chrono::{Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a causal relationship between two events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CausalLink {
    /// Index of the source event (what caused)
    pub source_event_idx: usize,
    /// Index of the target event (what was caused)
    pub target_event_idx: usize,
    /// Type of causal relationship
    pub relationship_type: String,
    /// Confidence score (0.0-1.0) for this causal relationship
    pub confidence: f32,
    /// Time gap between source and target (in milliseconds)
    pub time_gap_ms: i64,
}

impl CausalLink {
    pub fn new(
        source_idx: usize,
        target_idx: usize,
        rel_type: String,
        confidence: f32,
        time_gap_ms: i64,
    ) -> Self {
        CausalLink {
            source_event_idx: source_idx,
            target_event_idx: target_idx,
            relationship_type: rel_type,
            confidence: confidence.clamp(0.0, 1.0),
            time_gap_ms,
        }
    }
}

/// Event dependency graph for causal analysis
pub struct CausalGraph {
    /// All causal links in the mission
    links: Vec<CausalLink>,
    /// Forward edges: event_idx → [causal links where this event is source]
    forward_edges: HashMap<usize, Vec<usize>>,
    /// Backward edges: event_idx → [causal links where this event is target]
    backward_edges: HashMap<usize, Vec<usize>>,
    /// Causality window in milliseconds (default 2000ms = 2 seconds)
    causality_window_ms: i64,
}

impl CausalGraph {
    /// Create new causal graph with default causality window (2 seconds)
    pub fn new() -> Self {
        CausalGraph {
            links: Vec::new(),
            forward_edges: HashMap::new(),
            backward_edges: HashMap::new(),
            causality_window_ms: 2000,
        }
    }

    /// Create with custom causality window
    pub fn with_window(causality_window_ms: i64) -> Self {
        CausalGraph {
            links: Vec::new(),
            forward_edges: HashMap::new(),
            backward_edges: HashMap::new(),
            causality_window_ms: causality_window_ms.max(100),
        }
    }

    /// Add a causal link to the graph
    pub fn add_link(&mut self, link: CausalLink) {
        let link_idx = self.links.len();

        self.forward_edges
            .entry(link.source_event_idx)
            .or_insert_with(Vec::new)
            .push(link_idx);

        self.backward_edges
            .entry(link.target_event_idx)
            .or_insert_with(Vec::new)
            .push(link_idx);

        self.links.push(link);
    }

    /// Get all events that directly caused the given event
    pub fn get_direct_causes(&self, event_idx: usize) -> Vec<&CausalLink> {
        self.backward_edges
            .get(&event_idx)
            .map(|indices| {
                indices
                    .iter()
                    .map(|&idx| &self.links[idx])
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all events directly caused by the given event
    pub fn get_direct_effects(&self, event_idx: usize) -> Vec<&CausalLink> {
        self.forward_edges
            .get(&event_idx)
            .map(|indices| {
                indices
                    .iter()
                    .map(|&idx| &self.links[idx])
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Trace causal chain backwards (what caused this event recursively)
    pub fn trace_causes(&self, event_idx: usize, max_depth: usize) -> Vec<CausalChain> {
        let mut chains = Vec::new();
        self._trace_causes_recursive(event_idx, vec![event_idx], max_depth, &mut chains);
        chains
    }

    fn _trace_causes_recursive(
        &self,
        current_idx: usize,
        path: Vec<usize>,
        max_depth: usize,
        chains: &mut Vec<CausalChain>,
    ) {
        if max_depth == 0 {
            chains.push(CausalChain {
                event_chain: path,
                total_confidence: 1.0,
            });
            return;
        }

        let causes = self.get_direct_causes(current_idx);
        if causes.is_empty() {
            chains.push(CausalChain {
                event_chain: path,
                total_confidence: 1.0,
            });
        } else {
            for cause in causes {
                let mut new_path = path.clone();
                new_path.insert(0, cause.source_event_idx);
                self._trace_causes_recursive(
                    cause.source_event_idx,
                    new_path,
                    max_depth - 1,
                    chains,
                );
            }
        }
    }

    /// Get all links
    pub fn links(&self) -> &[CausalLink] {
        &self.links
    }

    /// Causality window in milliseconds
    pub fn causality_window_ms(&self) -> i64 {
        self.causality_window_ms
    }
}

/// Represents a causal chain (sequence of events)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalChain {
    /// Indices of events in the chain (earliest to latest)
    pub event_chain: Vec<usize>,
    /// Cumulative confidence of the chain
    pub total_confidence: f32,
}

impl CausalChain {
    pub fn new(events: Vec<usize>, confidence: f32) -> Self {
        CausalChain {
            event_chain: events,
            total_confidence: confidence.clamp(0.0, 1.0),
        }
    }

    pub fn length(&self) -> usize {
        self.event_chain.len()
    }
}

/// Builder for constructing causal graphs from event sequences
pub struct CausalGraphBuilder {
    events: Vec<MissionEvent>,
    causality_window_ms: i64,
}

impl CausalGraphBuilder {
    pub fn new(events: Vec<MissionEvent>) -> Self {
        CausalGraphBuilder {
            events,
            causality_window_ms: 2000,
        }
    }

    pub fn with_window(mut self, window_ms: i64) -> Self {
        self.causality_window_ms = window_ms.max(100);
        self
    }

    /// Build causal graph using heuristic rules
    pub fn build(self) -> CausalGraph {
        let mut graph = CausalGraph::with_window(self.causality_window_ms);

        for (i, target_event) in self.events.iter().enumerate() {
            let target_time = target_event.timestamp();
            let window_start = target_time - Duration::milliseconds(self.causality_window_ms);

            // Find potential causes (events within causality window before this event)
            for (j, source_event) in self.events.iter().enumerate() {
                if i == j {
                    continue;
                }

                let source_time = source_event.timestamp();
                if source_time >= window_start && source_time < target_time {
                    if let Some(link) = self._infer_causal_link(j, source_event, i, target_event) {
                        graph.add_link(link);
                    }
                }
            }
        }

        graph
    }

    /// Infer causal relationship between two events using heuristics
    fn _infer_causal_link(
        &self,
        source_idx: usize,
        source_event: &MissionEvent,
        target_idx: usize,
        target_event: &MissionEvent,
    ) -> Option<CausalLink> {
        let source_time = source_event.timestamp();
        let target_time = target_event.timestamp();
        let time_gap_ms = (target_time - source_time).num_milliseconds();

        match (source_event, target_event) {
            // Obstacle detected → Navigation decision
            (MissionEvent::ObstacleDetected { robot_id: src_robot, location: _, confidence: src_conf, .. },
             MissionEvent::NavigationDecision { robot_id: tgt_robot, .. }) => {
                if src_robot == tgt_robot {
                    return Some(CausalLink::new(
                        source_idx,
                        target_idx,
                        "obstacle_triggered_nav".to_string(),
                        src_conf.unwrap_or(0.8),
                        time_gap_ms,
                    ));
                }
            }
            // Lidar spike (high range variation) → Obstacle detection
            (MissionEvent::LidarScan { robot_id: src_robot, .. },
             MissionEvent::ObstacleDetected { robot_id: tgt_robot, confidence: tgt_conf, .. }) => {
                if src_robot == tgt_robot {
                    return Some(CausalLink::new(
                        source_idx,
                        target_idx,
                        "lidar_detected_obstacle".to_string(),
                        tgt_conf.unwrap_or(0.7),
                        time_gap_ms,
                    ));
                }
            }
            // Costmap update → Navigation decision
            (MissionEvent::CostmapUpdate { robot_id: src_robot, .. },
             MissionEvent::NavigationDecision { robot_id: tgt_robot, .. }) => {
                if src_robot == tgt_robot {
                    return Some(CausalLink::new(
                        source_idx,
                        target_idx,
                        "costmap_influenced_nav".to_string(),
                        0.6,
                        time_gap_ms,
                    ));
                }
            }
            // IMU spike → Odometry change
            (MissionEvent::IMUData { robot_id: src_robot, data: imu_data, .. },
             MissionEvent::OdometryUpdate { robot_id: tgt_robot, .. }) => {
                if src_robot == tgt_robot {
                    let accel_magnitude =
                        (imu_data.linear_acceleration[0].powi(2)
                            + imu_data.linear_acceleration[1].powi(2)
                            + imu_data.linear_acceleration[2].powi(2))
                        .sqrt();

                    // Higher confidence if IMU showed significant acceleration
                    let confidence = if accel_magnitude > 2.0 { 0.9 } else { 0.5 };

                    return Some(CausalLink::new(
                        source_idx,
                        target_idx,
                        "imu_caused_motion".to_string(),
                        confidence,
                        time_gap_ms,
                    ));
                }
            }
            _ => {}
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_link_creation() {
        let link = CausalLink::new(0, 1, "test".to_string(), 0.8, 500);
        assert_eq!(link.source_event_idx, 0);
        assert_eq!(link.target_event_idx, 1);
        assert_eq!(link.confidence, 0.8);
        assert_eq!(link.time_gap_ms, 500);
    }

    #[test]
    fn test_confidence_clamping() {
        let link1 = CausalLink::new(0, 1, "test".to_string(), 1.5, 100);
        assert_eq!(link1.confidence, 1.0);

        let link2 = CausalLink::new(0, 1, "test".to_string(), -0.5, 100);
        assert_eq!(link2.confidence, 0.0);
    }

    #[test]
    fn test_causal_graph_creation() {
        let graph = CausalGraph::new();
        assert_eq!(graph.causality_window_ms(), 2000);
        assert_eq!(graph.links().len(), 0);
    }

    #[test]
    fn test_add_link() {
        let mut graph = CausalGraph::new();
        let link = CausalLink::new(0, 1, "test".to_string(), 0.8, 500);
        graph.add_link(link);

        assert_eq!(graph.links().len(), 1);
        assert_eq!(graph.get_direct_causes(1).len(), 1);
        assert_eq!(graph.get_direct_effects(0).len(), 1);
    }

    #[test]
    fn test_causal_chain() {
        let chain = CausalChain::new(vec![0, 1, 2], 0.9);
        assert_eq!(chain.length(), 3);
        assert_eq!(chain.total_confidence, 0.9);
    }

    #[test]
    fn test_graph_builder_window() {
        let events = Vec::new();
        let builder = CausalGraphBuilder::new(events).with_window(5000);
        let graph = builder.build();
        assert_eq!(graph.causality_window_ms(), 5000);
    }

    #[test]
    fn test_trace_causes_empty_chain() {
        let mut graph = CausalGraph::new();
        let link = CausalLink::new(0, 1, "test".to_string(), 0.8, 500);
        graph.add_link(link);

        let chains = graph.trace_causes(0, 3);
        assert!(chains.len() > 0);
    }
}
