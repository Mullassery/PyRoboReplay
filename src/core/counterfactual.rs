use crate::core::causality::CausalGraph;
use crate::core::event::MissionEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Counterfactual scenario: simulate mission outcome with modified events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualScenario {
    /// Scenario ID
    pub id: String,
    /// Description of the modification
    pub description: String,
    /// Event indices to remove (simulate not happening)
    pub removed_events: Vec<usize>,
    /// Event indices to add (simulate happening)
    pub added_events: Vec<usize>,
    /// Predicted outcome (success, failure, unknown)
    pub predicted_outcome: String,
    /// Confidence in prediction (0.0-1.0)
    pub confidence: f32,
    /// Impact analysis
    pub impact: ScenarioImpact,
}

/// Impact of counterfactual scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioImpact {
    /// Events affected by the change
    pub affected_events: Vec<usize>,
    /// Number of events downstream affected
    pub cascade_size: usize,
    /// Whether the failure would still occur
    pub failure_prevented: bool,
    /// New potential failure modes introduced
    pub new_risks: Vec<String>,
    /// Confidence that this outcome would occur (0.0-1.0)
    pub outcome_confidence: f32,
}

/// Critical causal link: removing it would change mission outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalCausalLink {
    /// Source event index
    pub source_event_idx: usize,
    /// Target event index
    pub target_event_idx: usize,
    /// Criticality score (0.0-1.0, higher = more critical)
    pub criticality: f32,
    /// Why this link is critical
    pub reason: String,
    /// Number of downstream events affected if removed
    pub cascade_size: usize,
    /// Alternative paths if this link breaks
    pub alternative_paths: usize,
}

/// Counterfactual analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualAnalysis {
    /// Original failure event
    pub failure_event_idx: usize,
    /// Critical causal links (sorted by criticality)
    pub critical_links: Vec<CriticalCausalLink>,
    /// Counterfactual scenarios (what-if analysis)
    pub scenarios: Vec<CounterfactualScenario>,
    /// Most effective intervention (best way to prevent failure)
    pub best_intervention: Option<CounterfactualScenario>,
    /// Statistics
    pub stats: CounterfactualStats,
}

/// Statistics for counterfactual analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualStats {
    /// Total scenarios analyzed
    pub scenarios_analyzed: usize,
    /// Scenarios where failure is prevented
    pub failure_preventable: usize,
    /// Average cascade size
    pub avg_cascade_size: f32,
    /// Most critical link identified
    pub most_critical_score: f32,
    /// Intervention feasibility (0.0-1.0)
    pub intervention_feasibility: f32,
}

/// Counterfactual analyzer
pub struct CounterfactualAnalyzer {
    /// Mission events
    events: Vec<MissionEvent>,
    /// Causal graph
    causal_graph: Option<CausalGraph>,
    /// Analyzed scenarios cache
    scenarios: Vec<CounterfactualScenario>,
}

impl CounterfactualAnalyzer {
    /// Create new counterfactual analyzer
    pub fn new(events: Vec<MissionEvent>) -> Self {
        CounterfactualAnalyzer {
            events,
            causal_graph: None,
            scenarios: Vec::new(),
        }
    }

    /// Set causal graph for analysis
    pub fn with_causal_graph(mut self, graph: CausalGraph) -> Self {
        self.causal_graph = Some(graph);
        self
    }

    /// Analyze counterfactual: what if event didn't happen?
    pub fn scenario_remove_event(
        &mut self,
        event_idx: usize,
        failure_idx: usize,
    ) -> Option<CounterfactualScenario> {
        let graph = self.causal_graph.as_ref()?;

        // Find all events that depend on this one
        let affected = self._find_downstream_events(event_idx, graph);

        // Estimate if failure would still occur
        let failure_prevented = self._estimate_failure_prevention(event_idx, failure_idx, &affected, graph);

        // Identify new risks
        let new_risks = self._identify_new_risks(event_idx);

        // Calculate cascade size
        let cascade_size = affected.len();

        // Confidence based on cascade size and link strengths
        let confidence = if cascade_size > 5 {
            0.95 // High confidence if large cascade
        } else if cascade_size > 2 {
            0.80
        } else {
            0.60 // Low confidence if few downstream effects
        };

        let outcome_confidence = if failure_prevented { 0.85 } else { 0.70 };

        let scenario = CounterfactualScenario {
            id: format!("scenario_remove_{}", event_idx),
            description: format!("Remove Event {}", event_idx),
            removed_events: vec![event_idx],
            added_events: vec![],
            predicted_outcome: if failure_prevented {
                "failure_prevented".to_string()
            } else {
                "failure_still_occurs".to_string()
            },
            confidence,
            impact: ScenarioImpact {
                affected_events: affected.clone(),
                cascade_size,
                failure_prevented,
                new_risks: new_risks.clone(),
                outcome_confidence,
            },
        };

        self.scenarios.push(scenario.clone());
        Some(scenario)
    }

    /// Analyze counterfactual: replace event with alternative
    pub fn scenario_replace_event(
        &mut self,
        event_idx: usize,
        alternative_type: &str,
        _failure_idx: usize,
    ) -> Option<CounterfactualScenario> {
        let graph = self.causal_graph.as_ref()?;

        // Find affected events
        let affected = self._find_downstream_events(event_idx, graph);

        // Estimate impact of alternative
        let confidence = match alternative_type {
            "optimal" => 0.90,
            "conservative" => 0.75,
            "risky" => 0.55,
            _ => 0.65,
        };

        let failure_prevented = match alternative_type {
            "optimal" => true,
            "conservative" => {
                // Conservative may or may not prevent failure
                affected.len() < 3
            }
            "risky" => {
                // Risky likely doesn't prevent
                false
            }
            _ => false,
        };

        let scenario = CounterfactualScenario {
            id: format!("scenario_replace_{}_with_{}", event_idx, alternative_type),
            description: format!(
                "Replace Event {} with {} alternative",
                event_idx, alternative_type
            ),
            removed_events: vec![event_idx],
            added_events: vec![], // In reality would add the alternative
            predicted_outcome: if failure_prevented {
                "failure_prevented".to_string()
            } else {
                "failure_still_occurs".to_string()
            },
            confidence,
            impact: ScenarioImpact {
                affected_events: affected.clone(),
                cascade_size: affected.len(),
                failure_prevented,
                new_risks: vec![],
                outcome_confidence: if failure_prevented { 0.80 } else { 0.65 },
            },
        };

        self.scenarios.push(scenario.clone());
        Some(scenario)
    }

    /// Identify critical causal links that would prevent failure if broken
    pub fn identify_critical_links(
        &self,
        failure_idx: usize,
    ) -> Option<Vec<CriticalCausalLink>> {
        let graph = self.causal_graph.as_ref()?;

        let mut critical_links = Vec::new();

        // Analyze each link in the graph
        for link in graph.links() {
            // Only consider links that lead toward the failure
            let downstream = self._find_downstream_events(link.target_event_idx, graph);

            if downstream.contains(&failure_idx) {
                let cascade_size = downstream.len();
                let alternative_paths = self._count_alternative_paths(
                    link.source_event_idx,
                    link.target_event_idx,
                    graph,
                );

                // Criticality: high confidence + many downstream events + few alternatives
                let criticality =
                    link.confidence as f32 * (cascade_size as f32 / 10.0) / (alternative_paths.max(1) as f32);

                critical_links.push(CriticalCausalLink {
                    source_event_idx: link.source_event_idx,
                    target_event_idx: link.target_event_idx,
                    criticality: criticality.min(1.0),
                    reason: format!(
                        "Breaking this link eliminates {} downstream events",
                        cascade_size
                    ),
                    cascade_size,
                    alternative_paths,
                });
            }
        }

        // Sort by criticality
        critical_links.sort_by(|a, b| b.criticality.partial_cmp(&a.criticality).unwrap_or(std::cmp::Ordering::Equal));

        if critical_links.is_empty() {
            None
        } else {
            Some(critical_links)
        }
    }

    /// Perform full counterfactual analysis
    pub fn analyze(&mut self, failure_idx: usize) -> Option<CounterfactualAnalysis> {
        let critical_links = self.identify_critical_links(failure_idx)?;

        // Generate scenarios for top critical links
        let mut best_scenario: Option<CounterfactualScenario> = None;
        let mut best_score = 0.0;

        for link in critical_links.iter().take(3) {
            if let Some(scenario) = self.scenario_remove_event(link.source_event_idx, failure_idx) {
                let score = if scenario.impact.failure_prevented {
                    scenario.confidence * 0.95
                } else {
                    scenario.confidence * 0.5
                };

                if score > best_score {
                    best_score = score;
                    best_scenario = Some(scenario);
                }
            }
        }

        // Calculate statistics
        let stats = self._compute_stats(&critical_links);

        Some(CounterfactualAnalysis {
            failure_event_idx: failure_idx,
            critical_links,
            scenarios: self.scenarios.clone(),
            best_intervention: best_scenario,
            stats,
        })
    }

    fn _find_downstream_events(&self, event_idx: usize, graph: &CausalGraph) -> Vec<usize> {
        let mut downstream = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![event_idx];

        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            // Find all events this one causes
            for link in graph.links() {
                if link.source_event_idx == current && !visited.contains(&link.target_event_idx) {
                    downstream.push(link.target_event_idx);
                    queue.push(link.target_event_idx);
                }
            }
        }

        downstream
    }

    fn _estimate_failure_prevention(
        &self,
        event_idx: usize,
        failure_idx: usize,
        affected: &[usize],
        graph: &CausalGraph,
    ) -> bool {
        // Failure is prevented if removing this event breaks all paths to failure
        let mut paths_to_failure = 0;
        let mut blocked_paths = 0;

        for link in graph.links() {
            if link.source_event_idx == event_idx {
                // This link is broken by removing the event
                if affected.contains(&failure_idx) {
                    blocked_paths += 1;
                }
            } else if !affected.contains(&link.source_event_idx) {
                // Alternative path exists
                paths_to_failure += 1;
            }
        }

        blocked_paths > 0 && paths_to_failure == 0
    }

    fn _identify_new_risks(&self, _event_idx: usize) -> Vec<String> {
        // Simplified: could analyze what happens if this event is missing
        vec!["Reduced sensing capability".to_string()]
    }

    fn _count_alternative_paths(
        &self,
        source_idx: usize,
        target_idx: usize,
        graph: &CausalGraph,
    ) -> usize {
        // Count alternative causal paths between source and target
        graph
            .links()
            .iter()
            .filter(|link| link.source_event_idx == source_idx && link.target_event_idx != target_idx)
            .count()
    }

    fn _compute_stats(&self, critical_links: &[CriticalCausalLink]) -> CounterfactualStats {
        let failure_preventable = self
            .scenarios
            .iter()
            .filter(|s| s.impact.failure_prevented)
            .count();

        let avg_cascade = if critical_links.is_empty() {
            0.0
        } else {
            critical_links.iter().map(|l| l.cascade_size as f32).sum::<f32>() / critical_links.len() as f32
        };

        let most_critical = critical_links
            .first()
            .map(|l| l.criticality)
            .unwrap_or(0.0);

        // Intervention feasibility: high if we have clear prevention strategies
        let feasibility = if failure_preventable > 0 {
            0.85
        } else if avg_cascade > 5.0 {
            0.70
        } else {
            0.55
        };

        CounterfactualStats {
            scenarios_analyzed: self.scenarios.len(),
            failure_preventable,
            avg_cascade_size: avg_cascade,
            most_critical_score: most_critical,
            intervention_feasibility: feasibility,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = CounterfactualAnalyzer::new(vec![]);
        assert_eq!(analyzer.scenarios.len(), 0);
    }

    #[test]
    fn test_scenario_remove_event() {
        let mut analyzer = CounterfactualAnalyzer::new(vec![]);
        // Without a causal graph, scenario_remove_event returns None
        // This is expected behavior
        let scenario = analyzer.scenario_remove_event(0, 2);
        // Test passes if it handles None gracefully
        assert_eq!(scenario.is_none(), true);
    }

    #[test]
    fn test_scenario_replace_event() {
        let mut analyzer = CounterfactualAnalyzer::new(vec![]);
        let scenario = analyzer.scenario_replace_event(0, "optimal", 2);
        // Replace event requires causal graph, returns None without it
        assert!(scenario.is_none());
    }

    #[test]
    fn test_scenario_types() {
        let mut analyzer = CounterfactualAnalyzer::new(vec![]);

        // Both return None without a causal graph
        let optimal = analyzer.scenario_replace_event(0, "optimal", 2);
        let risky = analyzer.scenario_replace_event(0, "risky", 2);
        assert!(optimal.is_none());
        assert!(risky.is_none());
    }

    #[test]
    fn test_critical_link_structure() {
        let link = CriticalCausalLink {
            source_event_idx: 0,
            target_event_idx: 1,
            criticality: 0.95,
            reason: "test".to_string(),
            cascade_size: 5,
            alternative_paths: 1,
        };

        assert_eq!(link.source_event_idx, 0);
        assert!(link.criticality > 0.9);
    }

    #[test]
    fn test_counterfactual_scenario_structure() {
        let scenario = CounterfactualScenario {
            id: "test".to_string(),
            description: "test scenario".to_string(),
            removed_events: vec![0],
            added_events: vec![],
            predicted_outcome: "failure_prevented".to_string(),
            confidence: 0.85,
            impact: ScenarioImpact {
                affected_events: vec![1, 2, 3],
                cascade_size: 3,
                failure_prevented: true,
                new_risks: vec![],
                outcome_confidence: 0.80,
            },
        };

        assert_eq!(scenario.removed_events.len(), 1);
        assert!(scenario.impact.failure_prevented);
    }

    #[test]
    fn test_scenario_impact_metrics() {
        let impact = ScenarioImpact {
            affected_events: vec![1, 2, 3, 4, 5],
            cascade_size: 5,
            failure_prevented: true,
            new_risks: vec!["Risk A".to_string()],
            outcome_confidence: 0.90,
        };

        assert_eq!(impact.cascade_size, 5);
        assert_eq!(impact.new_risks.len(), 1);
    }

    #[test]
    fn test_counterfactual_stats() {
        let stats = CounterfactualStats {
            scenarios_analyzed: 5,
            failure_preventable: 2,
            avg_cascade_size: 3.5,
            most_critical_score: 0.92,
            intervention_feasibility: 0.80,
        };

        assert_eq!(stats.scenarios_analyzed, 5);
        assert_eq!(stats.failure_preventable, 2);
    }
}
