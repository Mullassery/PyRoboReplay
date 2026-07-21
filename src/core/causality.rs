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

    /// Query: "What caused this event?"
    /// Returns ranked list of causal hypotheses with confidence scores
    pub fn query_what_caused(
        &self,
        target_event_idx: usize,
        events: &[MissionEvent],
    ) -> CausalQuery {
        let direct_causes = self.get_direct_causes(target_event_idx);
        let total_links = direct_causes.len();

        let mut hypotheses = Vec::new();

        for cause_link in direct_causes {
            let cause_chain = self.trace_causes(cause_link.source_event_idx, 10);

            for chain in cause_chain {
                let total_time_gap = if let (Some(first_event), Some(last_event)) =
                    (events.get(chain.event_chain[0]), events.get(*chain.event_chain.last().unwrap()))
                {
                    (last_event.timestamp() - first_event.timestamp()).num_milliseconds()
                } else {
                    cause_link.time_gap_ms
                };

                let explanation = self._generate_explanation(
                    &chain.event_chain,
                    events,
                    &cause_link.relationship_type,
                );

                hypotheses.push(CausalHypothesis {
                    chain,
                    confidence: cause_link.confidence,
                    explanation,
                    direct_cause_type: Some(cause_link.relationship_type.clone()),
                    total_time_gap_ms: total_time_gap,
                });
            }
        }

        // Sort by confidence (descending)
        hypotheses.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        CausalQuery {
            target_event_idx,
            hypotheses,
            total_links_found: total_links,
        }
    }

    /// Query: "What did this event cause?"
    /// Returns ranked list of effects (events caused by this event)
    pub fn query_what_effects(
        &self,
        source_event_idx: usize,
        events: &[MissionEvent],
    ) -> CausalQuery {
        let direct_effects = self.get_direct_effects(source_event_idx);
        let total_links = direct_effects.len();

        let mut hypotheses = Vec::new();

        for effect_link in direct_effects {
            let effect_chain = self.trace_causes(effect_link.target_event_idx, 10);

            for chain in effect_chain {
                let total_time_gap = if let (Some(first_event), Some(last_event)) =
                    (events.get(source_event_idx), events.get(*chain.event_chain.last().unwrap()))
                {
                    (last_event.timestamp() - first_event.timestamp()).num_milliseconds()
                } else {
                    effect_link.time_gap_ms
                };

                let explanation = self._generate_explanation_forward(
                    source_event_idx,
                    &chain.event_chain,
                    events,
                    &effect_link.relationship_type,
                );

                hypotheses.push(CausalHypothesis {
                    chain,
                    confidence: effect_link.confidence,
                    explanation,
                    direct_cause_type: Some(effect_link.relationship_type.clone()),
                    total_time_gap_ms: total_time_gap,
                });
            }
        }

        // Sort by confidence (descending)
        hypotheses.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        CausalQuery {
            target_event_idx: source_event_idx,
            hypotheses,
            total_links_found: total_links,
        }
    }

    fn _generate_explanation(
        &self,
        event_chain: &[usize],
        events: &[MissionEvent],
        relationship_type: &str,
    ) -> String {
        if event_chain.is_empty() {
            return "Unknown causal chain".to_string();
        }

        let chain_desc = event_chain
            .iter()
            .take(3)
            .map(|&idx| events.get(idx).map(|e| e.event_type()).unwrap_or("?"))
            .collect::<Vec<_>>()
            .join(" → ");

        let suffix = if event_chain.len() > 3 {
            format!(" → ... ({} total)", event_chain.len())
        } else {
            String::new()
        };

        match relationship_type {
            "obstacle_triggered_nav" => {
                format!("Obstacle detection triggered navigation: {}{}", chain_desc, suffix)
            }
            "lidar_detected_obstacle" => {
                format!(
                    "Lidar scan detected obstacle leading to: {}{}",
                    chain_desc, suffix
                )
            }
            "costmap_influenced_nav" => {
                format!(
                    "Costmap update influenced navigation decision: {}{}",
                    chain_desc, suffix
                )
            }
            "imu_caused_motion" => {
                format!("IMU acceleration caused motion change: {}{}", chain_desc, suffix)
            }
            _ => {
                format!("Causal relationship ({}): {}{}", relationship_type, chain_desc, suffix)
            }
        }
    }

    fn _generate_explanation_forward(
        &self,
        _source_idx: usize,
        event_chain: &[usize],
        events: &[MissionEvent],
        relationship_type: &str,
    ) -> String {
        if event_chain.is_empty() {
            return "Unknown downstream effects".to_string();
        }

        let chain_desc = event_chain
            .iter()
            .take(3)
            .map(|&idx| events.get(idx).map(|e| e.event_type()).unwrap_or("?"))
            .collect::<Vec<_>>()
            .join(" → ");

        let suffix = if event_chain.len() > 3 {
            format!(" → ... ({} total)", event_chain.len())
        } else {
            String::new()
        };

        match relationship_type {
            "obstacle_triggered_nav" => {
                format!("Triggered navigation decision: {}{}", chain_desc, suffix)
            }
            "lidar_detected_obstacle" => {
                format!("Led to obstacle detection: {}{}", chain_desc, suffix)
            }
            "costmap_influenced_nav" => {
                format!("Influenced navigation: {}{}", chain_desc, suffix)
            }
            "imu_caused_motion" => {
                format!("Caused motion changes: {}{}", chain_desc, suffix)
            }
            _ => {
                format!(
                    "Caused downstream effects ({}): {}{}",
                    relationship_type, chain_desc, suffix
                )
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Result of a causal query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalQuery {
    /// The event that was queried
    pub target_event_idx: usize,
    /// Ranked list of causal hypotheses (sorted by confidence)
    pub hypotheses: Vec<CausalHypothesis>,
    /// Total number of causal links found
    pub total_links_found: usize,
}

/// A hypothesis about what caused an event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CausalHypothesis {
    /// The causal chain leading to the event
    pub chain: CausalChain,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Human-readable explanation
    pub explanation: String,
    /// Relationship type of the direct cause
    pub direct_cause_type: Option<String>,
    /// Time from root cause to target event
    pub total_time_gap_ms: i64,
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

    #[test]
    fn test_causal_hypothesis_creation() {
        let hypothesis = CausalHypothesis {
            chain: CausalChain::new(vec![0, 1], 0.9),
            confidence: 0.9,
            explanation: "Test cause".to_string(),
            direct_cause_type: Some("test_type".to_string()),
            total_time_gap_ms: 500,
        };

        assert_eq!(hypothesis.confidence, 0.9);
        assert_eq!(hypothesis.chain.length(), 2);
    }

    #[test]
    fn test_query_what_caused() {
        use crate::core::event::Location;

        let base_time = chrono::Utc::now();
        let events = vec![
            MissionEvent::ObstacleDetected {
                robot_id: "robot_1".to_string(),
                timestamp: base_time,
                location: Location {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                obstacle_type: "static".to_string(),
                confidence: Some(0.95),
            },
            MissionEvent::NavigationDecision {
                robot_id: "robot_1".to_string(),
                timestamp: base_time + chrono::Duration::milliseconds(500),
                decision_type: "path_replan".to_string(),
                rationale: None,
            },
        ];

        let mut graph = CausalGraph::new();
        let link = CausalLink::new(0, 1, "obstacle_triggered_nav".to_string(), 0.95, 500);
        graph.add_link(link);

        let query = graph.query_what_caused(1, &events);
        assert_eq!(query.target_event_idx, 1);
        assert_eq!(query.total_links_found, 1);
        assert!(query.hypotheses.len() > 0);
    }

    #[test]
    fn test_query_what_effects() {
        use crate::core::event::Location;

        let base_time = chrono::Utc::now();
        let events = vec![
            MissionEvent::ObstacleDetected {
                robot_id: "robot_1".to_string(),
                timestamp: base_time,
                location: Location {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                obstacle_type: "static".to_string(),
                confidence: Some(0.95),
            },
            MissionEvent::NavigationDecision {
                robot_id: "robot_1".to_string(),
                timestamp: base_time + chrono::Duration::milliseconds(500),
                decision_type: "path_replan".to_string(),
                rationale: None,
            },
        ];

        let mut graph = CausalGraph::new();
        let link = CausalLink::new(0, 1, "obstacle_triggered_nav".to_string(), 0.95, 500);
        graph.add_link(link);

        let query = graph.query_what_effects(0, &events);
        assert_eq!(query.target_event_idx, 0);
        assert_eq!(query.total_links_found, 1);
        assert!(query.hypotheses.len() > 0);
    }

    #[test]
    fn test_hypothesis_ranking() {
        let mut hypotheses = vec![
            CausalHypothesis {
                chain: CausalChain::new(vec![0], 0.5),
                confidence: 0.5,
                explanation: "Low confidence".to_string(),
                direct_cause_type: Some("type1".to_string()),
                total_time_gap_ms: 1000,
            },
            CausalHypothesis {
                chain: CausalChain::new(vec![1], 0.9),
                confidence: 0.9,
                explanation: "High confidence".to_string(),
                direct_cause_type: Some("type2".to_string()),
                total_time_gap_ms: 500,
            },
        ];

        hypotheses.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        assert_eq!(hypotheses[0].confidence, 0.9);
        assert_eq!(hypotheses[1].confidence, 0.5);
    }
}
