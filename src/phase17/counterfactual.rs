/// Counterfactual Analysis Engine - Generate "what if?" alternative histories

use crate::phase16::causal_builder::CausalGraphV2;
use crate::core::event::MissionEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryType {
    RemoveNode,                          // "What if this decision didn't happen?"
    ReplaceNode,                         // "What if this decision was different?"
    ModifyEdgeWeight,                    // "What if factor A was stronger?"
    RemoveEdge,                          // "What if this causal link didn't exist?"
    ParallelDecisions,                   // "What if we chose alternative B instead of A?"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualQuery {
    pub query_type: String,
    pub target_node_id: String,
    pub parameter: Option<String>,  // e.g., new confidence, alternative action
    pub timestamp: i64,
}

impl CounterfactualQuery {
    pub fn remove_node(node_id: String) -> Self {
        CounterfactualQuery {
            query_type: "remove_node".to_string(),
            target_node_id: node_id,
            parameter: None,
            timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        }
    }

    pub fn replace_node(node_id: String, new_value: String) -> Self {
        CounterfactualQuery {
            query_type: "replace_node".to_string(),
            target_node_id: node_id,
            parameter: Some(new_value),
            timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        }
    }

    pub fn modify_edge_weight(source_id: String, target_id: String, new_confidence: f32) -> Self {
        CounterfactualQuery {
            query_type: "modify_edge_weight".to_string(),
            target_node_id: format!("{}→{}", source_id, target_id),
            parameter: Some(new_confidence.to_string()),
            timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    pub query: CounterfactualQuery,
    pub original_outcome: String,
    pub alternative_outcome: String,
    pub divergence_point: usize,      // Event index where timeline diverges
    pub outcome_change_magnitude: f32, // 0-1 scale, how different?
    pub confidence: f32,               // How certain about this counterfactual?
    pub affected_events: Vec<usize>,   // Which events would differ
    pub summary: String,               // Human-readable explanation
}

pub struct CounterfactualAnalyzer {
    graph: CausalGraphV2,
    baseline_outcome: String,
    events: Vec<MissionEvent>,
}

impl CounterfactualAnalyzer {
    pub fn new(graph: CausalGraphV2, baseline_outcome: String, events: Vec<MissionEvent>) -> Self {
        CounterfactualAnalyzer {
            graph,
            baseline_outcome,
            events,
        }
    }

    /// Execute a counterfactual query: "What if X hadn't happened?"
    pub fn analyze_counterfactual(&self, query: CounterfactualQuery) -> CounterfactualResult {
        let affected = self._identify_affected_events(&query);
        let alternative_outcome = self._simulate_alternative(&query, &affected);

        let divergence = affected.first().copied().unwrap_or(0);
        let magnitude = self._calculate_outcome_change(&self.baseline_outcome, &alternative_outcome);

        CounterfactualResult {
            query: query.clone(),
            original_outcome: self.baseline_outcome.clone(),
            alternative_outcome: alternative_outcome.clone(),
            divergence_point: divergence,
            outcome_change_magnitude: magnitude,
            confidence: self._calculate_confidence(&query, &affected),
            affected_events: affected,
            summary: self._generate_summary(&query, &alternative_outcome, magnitude),
        }
    }

    fn _identify_affected_events(&self, query: &CounterfactualQuery) -> Vec<usize> {
        let mut affected = Vec::new();

        // Find events that depend on the target node
        for (idx, edge) in self.graph.edges.iter().enumerate() {
            if edge.source_id == query.target_node_id {
                // This edge emanates from the target node
                if let Ok(target_idx) = edge.target_id.strip_prefix("event_").unwrap_or("").parse::<usize>() {
                    affected.push(target_idx);
                }
            }
        }

        affected.sort_unstable();
        affected
    }

    fn _simulate_alternative(&self, query: &CounterfactualQuery, affected: &[usize]) -> String {
        match query.query_type.as_str() {
            "remove_node" => {
                if affected.is_empty() {
                    format!("Mission outcome unchanged (isolated event)")
                } else {
                    format!("Mission delayed by ~{} events", affected.len())
                }
            }
            "replace_node" => {
                format!("Alternative path taken: {}", query.parameter.as_ref().unwrap_or(&"unknown".to_string()))
            }
            "modify_edge_weight" => {
                let new_conf: f32 = query.parameter.as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.5);

                if new_conf > 0.8 {
                    "Edge is now high confidence - cascading effects more likely".to_string()
                } else if new_conf < 0.3 {
                    "Edge is now low confidence - causal chain may break".to_string()
                } else {
                    "Edge confidence moderate - outcome uncertain".to_string()
                }
            }
            _ => "Unknown query type".to_string(),
        }
    }

    fn _calculate_outcome_change(&self, original: &str, alternative: &str) -> f32 {
        if original == alternative {
            0.0  // No change
        } else if original.contains("success") && alternative.contains("failure") {
            1.0  // Complete reversal
        } else if original.contains("failure") && alternative.contains("success") {
            1.0  // Complete reversal
        } else {
            0.5  // Partial change
        }
    }

    fn _calculate_confidence(&self, query: &CounterfactualQuery, affected: &[usize]) -> f32 {
        // Confidence decreases with number of affected events
        let base_confidence = 0.85;
        let penalty = (affected.len() as f32) * 0.02;
        (base_confidence - penalty).max(0.3)
    }

    fn _generate_summary(&self, query: &CounterfactualQuery, outcome: &str, magnitude: f32) -> String {
        let impact = match magnitude {
            m if m > 0.8 => "drastically",
            m if m > 0.5 => "significantly",
            m if m > 0.2 => "moderately",
            _ => "slightly",
        };

        match query.query_type.as_str() {
            "remove_node" => {
                format!(
                    "Removing decision {} would {} change the outcome. Result: {}",
                    query.target_node_id, impact, outcome
                )
            }
            "replace_node" => {
                format!(
                    "Choosing alternative {} instead would {} alter the mission. Result: {}",
                    query.parameter.as_ref().unwrap_or(&"unknown".to_string()),
                    impact,
                    outcome
                )
            }
            _ => format!("Counterfactual scenario would {} affect outcome: {}", impact, outcome),
        }
    }

    /// Compare two potential decisions at a divergence point
    pub fn compare_decision_paths(&self, decision_a: &str, decision_b: &str) -> HashMap<String, f32> {
        let mut comparison = HashMap::new();

        // Placeholder metrics
        comparison.insert("path_a_success_rate".to_string(), 0.75);
        comparison.insert("path_b_success_rate".to_string(), 0.68);
        comparison.insert("path_a_avg_cost".to_string(), 100.5);
        comparison.insert("path_b_avg_cost".to_string(), 95.2);
        comparison.insert("path_a_avg_latency_ms".to_string(), 5000.0);
        comparison.insert("path_b_avg_latency_ms".to_string(), 5800.0);

        comparison
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counterfactual_query_creation() {
        let query = CounterfactualQuery::remove_node("decision_1".to_string());
        assert_eq!(query.query_type, "remove_node");
        assert_eq!(query.target_node_id, "decision_1");
    }

    #[test]
    fn test_replace_node_query() {
        let query = CounterfactualQuery::replace_node(
            "decision_1".to_string(),
            "alternative_B".to_string(),
        );
        assert_eq!(query.query_type, "replace_node");
        assert_eq!(query.parameter, Some("alternative_B".to_string()));
    }

    #[test]
    fn test_outcome_change_calculation() {
        let analyzer = CounterfactualAnalyzer::new(
            CausalGraphV2::new(),
            "Mission success".to_string(),
            Vec::new(),
        );

        // No change
        assert_eq!(analyzer._calculate_outcome_change("success", "success"), 0.0);

        // Complete reversal
        assert_eq!(analyzer._calculate_outcome_change("success", "failure"), 1.0);

        // Partial change
        let change = analyzer._calculate_outcome_change("success with delay", "partial success");
        assert!(change > 0.0 && change < 1.0);
    }
}
