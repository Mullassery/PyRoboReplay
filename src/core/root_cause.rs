use crate::core::causality::{CausalGraph, CausalLink};
use crate::core::event::MissionEvent;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Root cause hypothesis for a failure event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseHypothesis {
    /// Unique hypothesis ID
    pub id: String,
    /// Root cause event index
    pub root_event_idx: usize,
    /// Root cause event type
    pub root_event_type: String,
    /// Causal chain from root to failure (event indices)
    pub causal_chain: Vec<usize>,
    /// Total chain length (hops from root to failure)
    pub chain_length: usize,
    /// Confidence in this hypothesis (0.0-1.0)
    pub confidence: f32,
    /// Explanation of the causal sequence
    pub explanation: String,
    /// Alternative causes that could have led to same outcome
    pub alternative_causes: Vec<String>,
    /// Criticality: how essential was this event to failure (0.0-1.0)
    pub criticality: f32,
}

/// Root cause analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    /// Target failure event index
    pub failure_event_idx: usize,
    /// Target failure event type
    pub failure_event_type: String,
    /// Ranked hypotheses (sorted by confidence descending)
    pub hypotheses: Vec<RootCauseHypothesis>,
    /// Most likely root cause (if identified)
    pub most_likely_cause: Option<RootCauseHypothesis>,
    /// Diagnostic confidence (0.0-1.0, higher = more certain)
    pub diagnostic_confidence: f32,
    /// Analysis statistics
    pub stats: DiagnosticStats,
}

/// Failure mode classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMode {
    /// Failure type (navigation_deadlock, collision, timeout, coverage_gap, etc.)
    pub failure_type: String,
    /// Severity (critical, high, medium, low)
    pub severity: String,
    /// First occurrence index
    pub first_event_idx: usize,
    /// Associated events (timeline of failure)
    pub event_chain: Vec<usize>,
    /// Detection confidence (0.0-1.0)
    pub confidence: f32,
}

/// Diagnostic statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticStats {
    /// Total hypotheses generated
    pub total_hypotheses: usize,
    /// Hypotheses with confidence > 0.8
    pub high_confidence_hypotheses: usize,
    /// Average causal chain length
    pub avg_chain_length: f32,
    /// Maximum chain length found
    pub max_chain_length: usize,
    /// Consensus among hypotheses (how similar are they?)
    pub consensus_score: f32,
}

/// Root cause analyzer
pub struct RootCauseAnalyzer {
    /// Mission events
    events: Vec<MissionEvent>,
    /// Causal graph
    causal_graph: Option<CausalGraph>,
    /// Detected failure modes
    failure_modes: Vec<FailureMode>,
    /// Root cause hypotheses cache
    hypotheses_cache: HashMap<usize, Vec<RootCauseHypothesis>>,
}

impl RootCauseAnalyzer {
    /// Create new root cause analyzer
    pub fn new(events: Vec<MissionEvent>) -> Self {
        RootCauseAnalyzer {
            events,
            causal_graph: None,
            failure_modes: Vec::new(),
            hypotheses_cache: HashMap::new(),
        }
    }

    /// Set causal graph for analysis
    pub fn with_causal_graph(mut self, graph: CausalGraph) -> Self {
        self.causal_graph = Some(graph);
        self
    }

    /// Add detected failure mode
    pub fn add_failure_mode(&mut self, mode: FailureMode) {
        self.failure_modes.push(mode);
    }

    /// Analyze root causes for a specific failure event
    pub fn analyze_failure(&self, failure_event_idx: usize) -> Option<RootCauseAnalysis> {
        let graph = self.causal_graph.as_ref()?;
        let failure_event = self.events.get(failure_event_idx)?;

        // Trace back causal chain from failure
        let hypotheses = self._trace_root_causes(failure_event_idx, graph);

        if hypotheses.is_empty() {
            return None;
        }

        // Calculate statistics
        let stats = self._compute_stats(&hypotheses);

        // Most likely cause is first (highest confidence)
        let most_likely = hypotheses.first().cloned();

        // Diagnostic confidence based on hypothesis consistency
        let diagnostic_confidence = if hypotheses.len() > 0 {
            hypotheses.first().map(|h| h.confidence).unwrap_or(0.0)
        } else {
            0.0
        };

        Some(RootCauseAnalysis {
            failure_event_idx,
            failure_event_type: format!("{:?}", failure_event),
            hypotheses,
            most_likely_cause: most_likely,
            diagnostic_confidence,
            stats,
        })
    }

    /// Detect failure modes in mission
    pub fn detect_failure_modes(&mut self) {
        // Simple pattern-based detection
        let mut i = 0;
        while i < self.events.len() {
            match &self.events[i] {
                // Navigation deadlock: repeated identical positions
                MissionEvent::RobotPose { pose, .. } => {
                    if i + 2 < self.events.len() {
                        if let MissionEvent::RobotPose { pose: pose2, .. } = &self.events[i + 1] {
                            if let MissionEvent::RobotPose { pose: pose3, .. } = &self.events[i + 2]
                            {
                                let dist1 = (pose.x - pose2.x).powi(2)
                                    + (pose.y - pose2.y).powi(2)
                                    + (pose.z - pose2.z).powi(2);
                                let dist2 = (pose2.x - pose3.x).powi(2)
                                    + (pose2.y - pose3.y).powi(2)
                                    + (pose2.z - pose3.z).powi(2);

                                if dist1 < 0.01 && dist2 < 0.01 {
                                    self.failure_modes.push(FailureMode {
                                        failure_type: "navigation_deadlock".to_string(),
                                        severity: "high".to_string(),
                                        first_event_idx: i,
                                        event_chain: vec![i, i + 1, i + 2],
                                        confidence: 0.9,
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Get failure modes
    pub fn failure_modes(&self) -> &[FailureMode] {
        &self.failure_modes
    }

    fn _trace_root_causes(&self, failure_idx: usize, graph: &CausalGraph) -> Vec<RootCauseHypothesis> {
        let mut hypotheses = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // BFS backward through causal graph to find all root paths
        let mut queue: VecDeque<(usize, Vec<usize>, f32)> = VecDeque::new();
        queue.push_back((failure_idx, vec![failure_idx], 1.0));

        while let Some((current_idx, chain, cumulative_conf)) = queue.pop_front() {
            if visited.contains(&current_idx) {
                continue;
            }
            visited.insert(current_idx);

            // Get causes of current event
            let causes = graph.get_direct_causes(current_idx);
            if !causes.is_empty() {
                for cause_link in causes {
                    let cause_idx = cause_link.source_event_idx;
                    let mut new_chain = chain.clone();
                    new_chain.insert(0, cause_idx);

                    let new_confidence = cumulative_conf * cause_link.confidence;

                    // If confidence is high enough, explore further
                    if new_confidence > 0.3 {
                        queue.push_back((cause_idx, new_chain.clone(), new_confidence));
                    }

                    // If this is a leaf node (no more causes), create hypothesis
                    let has_causes = !graph.get_direct_causes(cause_idx).is_empty();
                    if !has_causes {
                        let hypothesis = self._create_hypothesis(
                            cause_idx,
                            failure_idx,
                            &new_chain,
                            new_confidence,
                        );
                        hypotheses.push(hypothesis);
                    }
                }
            }
        }

        // Sort by confidence descending
        hypotheses.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // Keep top hypotheses
        hypotheses.truncate(10);
        hypotheses
    }

    fn _create_hypothesis(
        &self,
        root_idx: usize,
        _failure_idx: usize,
        chain: &[usize],
        confidence: f32,
    ) -> RootCauseHypothesis {
        let root_event = self.events.get(root_idx);
        let root_type = root_event.map(|e| format!("{:?}", e)).unwrap_or_default();

        let explanation = self._explain_causality(chain);
        let criticality = self._calculate_criticality(chain);
        let alternatives = self._find_alternatives(root_idx);

        RootCauseHypothesis {
            id: format!("root_cause_{}", root_idx),
            root_event_idx: root_idx,
            root_event_type: root_type,
            causal_chain: chain.to_vec(),
            chain_length: chain.len(),
            confidence,
            explanation,
            alternative_causes: alternatives,
            criticality,
        }
    }

    fn _explain_causality(&self, chain: &[usize]) -> String {
        let events: Vec<String> = chain
            .iter()
            .filter_map(|idx| {
                self.events.get(*idx).map(|e| match e {
                    MissionEvent::ObstacleDetected { .. } => "Obstacle Detected".to_string(),
                    MissionEvent::NavigationDecision { .. } => "Navigation Decision".to_string(),
                    MissionEvent::RobotPose { .. } => "Robot Pose Update".to_string(),
                    MissionEvent::OdometryUpdate { .. } => "Odometry Update".to_string(),
                    _ => format!("{:?}", e),
                })
            })
            .collect();

        if events.is_empty() {
            return "Unknown cause".to_string();
        }

        format!("{} → {}", events.join(" → "), "Failure")
    }

    fn _calculate_criticality(&self, chain: &[usize]) -> f32 {
        // Criticality based on chain length: shorter chains = more critical
        if chain.len() <= 1 {
            1.0
        } else if chain.len() <= 3 {
            0.8
        } else if chain.len() <= 5 {
            0.6
        } else {
            0.4
        }
    }

    fn _find_alternatives(&self, root_idx: usize) -> Vec<String> {
        // Simplified: return common alternatives
        match self.events.get(root_idx) {
            Some(MissionEvent::ObstacleDetected { .. }) => {
                vec!["Sensor malfunction".to_string(), "False positive detection".to_string()]
            }
            Some(MissionEvent::NavigationDecision { .. }) => {
                vec!["Suboptimal path planning".to_string(), "State estimation error".to_string()]
            }
            _ => vec!["Environmental change".to_string()],
        }
    }

    fn _compute_stats(&self, hypotheses: &[RootCauseHypothesis]) -> DiagnosticStats {
        let avg_chain_length = hypotheses
            .iter()
            .map(|h| h.chain_length as f32)
            .sum::<f32>()
            / hypotheses.len().max(1) as f32;

        let max_chain_length = hypotheses.iter().map(|h| h.chain_length).max().unwrap_or(0);

        let high_conf = hypotheses.iter().filter(|h| h.confidence > 0.8).count();

        // Consensus: how similar are top hypotheses?
        let consensus = if hypotheses.len() > 1 {
            let top1 = hypotheses.first().map(|h| h.confidence).unwrap_or(0.0);
            let top2 = hypotheses.get(1).map(|h| h.confidence).unwrap_or(0.0);
            if top1 > 0.0 {
                1.0 - (top1 - top2).abs().min(1.0)
            } else {
                0.0
            }
        } else {
            1.0
        };

        DiagnosticStats {
            total_hypotheses: hypotheses.len(),
            high_confidence_hypotheses: high_conf,
            avg_chain_length,
            max_chain_length,
            consensus_score: consensus,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let events = vec![];
        let analyzer = RootCauseAnalyzer::new(events);
        assert_eq!(analyzer.failure_modes().len(), 0);
    }

    #[test]
    fn test_add_failure_mode() {
        let mut analyzer = RootCauseAnalyzer::new(vec![]);
        let mode = FailureMode {
            failure_type: "collision".to_string(),
            severity: "critical".to_string(),
            first_event_idx: 0,
            event_chain: vec![0, 1],
            confidence: 0.95,
        };

        analyzer.add_failure_mode(mode);
        assert_eq!(analyzer.failure_modes().len(), 1);
    }

    #[test]
    fn test_hypothesis_creation() {
        let analyzer = RootCauseAnalyzer::new(vec![]);
        let chain = vec![0, 1, 2];
        let hyp = analyzer._create_hypothesis(0, 2, &chain, 0.85);

        assert_eq!(hyp.chain_length, 3);
        assert_eq!(hyp.confidence, 0.85);
        assert!(hyp.criticality > 0.0);
    }

    #[test]
    fn test_calculate_criticality() {
        let analyzer = RootCauseAnalyzer::new(vec![]);

        assert_eq!(analyzer._calculate_criticality(&[0]), 1.0);
        assert!(analyzer._calculate_criticality(&[0, 1]) > 0.7);
        assert!(analyzer._calculate_criticality(&[0, 1, 2, 3, 4, 5, 6]) < 0.5);
    }

    #[test]
    fn test_explain_causality() {
        let analyzer = RootCauseAnalyzer::new(vec![]);
        let explanation = analyzer._explain_causality(&[]);
        assert_eq!(explanation, "Unknown cause");
    }

    #[test]
    fn test_compute_stats() {
        let hyp1 = RootCauseHypothesis {
            id: "h1".to_string(),
            root_event_idx: 0,
            root_event_type: "obstacle".to_string(),
            causal_chain: vec![0, 1],
            chain_length: 2,
            confidence: 0.95,
            explanation: "test".to_string(),
            alternative_causes: vec![],
            criticality: 0.8,
        };

        let hyp2 = RootCauseHypothesis {
            id: "h2".to_string(),
            root_event_idx: 1,
            root_event_type: "sensor".to_string(),
            causal_chain: vec![1, 2, 3],
            chain_length: 3,
            confidence: 0.70,
            explanation: "test".to_string(),
            alternative_causes: vec![],
            criticality: 0.7,
        };

        let analyzer = RootCauseAnalyzer::new(vec![]);
        let stats = analyzer._compute_stats(&vec![hyp1, hyp2]);

        assert_eq!(stats.total_hypotheses, 2);
        assert_eq!(stats.high_confidence_hypotheses, 1);
        assert!(stats.avg_chain_length > 0.0);
    }

    #[test]
    fn test_detect_failure_modes() {
        let events = vec![];
        let mut analyzer = RootCauseAnalyzer::new(events);
        analyzer.detect_failure_modes();
        // Should not crash
    }

    #[test]
    fn test_find_alternatives() {
        let analyzer = RootCauseAnalyzer::new(vec![]);
        let alts = analyzer._find_alternatives(999); // Non-existent event
        assert!(!alts.is_empty());
    }
}
