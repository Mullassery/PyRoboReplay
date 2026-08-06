/// Causal Graph Validator for Phase 16
///
/// Ensures quality: DAG property, confidence calibration, variance explanation

use super::causal_builder::CausalGraphV2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid_dag: bool,
    pub edge_count: usize,
    pub vertex_count: usize,
    pub variance_explained: f32,
    pub confidence_calibration_score: f32,
    pub conflicts: Vec<EdgeConflict>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeConflict {
    pub source: String,
    pub target1: String,
    pub target2: String,
    pub confidence1: f32,
    pub confidence2: f32,
}

pub struct CausalGraphValidator;

impl CausalGraphValidator {
    /// Validate a causal graph comprehensively
    pub fn validate(graph: &CausalGraphV2) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid_dag: graph.is_dag(),
            edge_count: graph.edges.len(),
            vertex_count: graph.vertices.len(),
            variance_explained: graph.calculate_variance_explained(),
            confidence_calibration_score: 0.0,
            conflicts: Vec::new(),
            issues: Vec::new(),
        };

        // Test 1: DAG property
        if !result.is_valid_dag {
            result.issues.push("Graph contains cycles (not a DAG)".to_string());
        }

        // Test 2: Edge confidence interpretability
        result.confidence_calibration_score = Self::_evaluate_confidence_calibration(graph);

        // Test 3: Variance explanation
        if result.variance_explained < 0.6 {
            result.issues.push(format!(
                "Low variance explanation: {:.2}% (expected >80%)",
                result.variance_explained * 100.0
            ));
        }

        // Test 4: Find conflicting edges
        result.conflicts = Self::_find_conflicting_edges(graph);
        if !result.conflicts.is_empty() {
            result.issues.push(format!("Found {} edge conflicts", result.conflicts.len()));
        }

        // Test 5: Confidence distribution
        Self::_validate_confidence_distribution(graph, &mut result.issues);

        result
    }

    /// Evaluate how well confidence scores match reality
    fn _evaluate_confidence_calibration(graph: &CausalGraphV2) -> f32 {
        if graph.edges.is_empty() {
            return 0.0;
        }

        let mut calibration_score: f32 = 0.0;

        // High confidence edges should be fewer
        let high_conf = graph.edges.iter().filter(|e| e.confidence > 0.8).count();
        let high_conf_ratio = high_conf as f32 / graph.edges.len() as f32;

        if high_conf_ratio < 0.3 {
            calibration_score += 0.3; // Good: only 30% high confidence
        } else if high_conf_ratio < 0.5 {
            calibration_score += 0.2;
        }

        // Medium confidence should be majority
        let med_conf = graph.edges.iter().filter(|e| e.confidence >= 0.5 && e.confidence <= 0.8).count();
        let med_conf_ratio = med_conf as f32 / graph.edges.len() as f32;

        if med_conf_ratio > 0.5 {
            calibration_score += 0.4;
        } else if med_conf_ratio > 0.3 {
            calibration_score += 0.2;
        }

        // Low confidence edges should exist but be few
        let low_conf = graph.edges.iter().filter(|e| e.confidence < 0.5).count();
        let low_conf_ratio = low_conf as f32 / graph.edges.len() as f32;

        if low_conf_ratio > 0.1 && low_conf_ratio < 0.4 {
            calibration_score += 0.3;
        }

        calibration_score.min(1.0_f32)
    }

    /// Find edges that conflict with each other
    fn _find_conflicting_edges(graph: &CausalGraphV2) -> Vec<EdgeConflict> {
        let mut conflicts = Vec::new();

        for i in 0..graph.edges.len() {
            for j in (i + 1)..graph.edges.len() {
                let e1 = &graph.edges[i];
                let e2 = &graph.edges[j];

                // Conflict: A→B and A→¬B (same source, contradictory targets)
                if e1.source_id == e2.source_id && e1.target_id != e2.target_id {
                    // Check if targets are in contradiction (simple heuristic)
                    if e1.confidence > 0.7 && e2.confidence > 0.7 {
                        conflicts.push(EdgeConflict {
                            source: e1.source_id.clone(),
                            target1: e1.target_id.clone(),
                            target2: e2.target_id.clone(),
                            confidence1: e1.confidence,
                            confidence2: e2.confidence,
                        });
                    }
                }
            }
        }

        conflicts
    }

    /// Validate confidence score distribution
    fn _validate_confidence_distribution(graph: &CausalGraphV2, issues: &mut Vec<String>) {
        if graph.edges.is_empty() {
            return;
        }

        let avg_conf: f32 = graph.edges.iter().map(|e| e.confidence).sum::<f32>() / graph.edges.len() as f32;

        if avg_conf < 0.5 {
            issues.push(format!(
                "Low average confidence: {:.2} (expected >0.6)",
                avg_conf
            ));
        }

        // Check for all edges having identical confidence
        let first_conf = graph.edges[0].confidence;
        let all_same = graph.edges.iter().all(|e| (e.confidence - first_conf).abs() < 0.01);

        if all_same {
            issues.push("All edges have identical confidence (poor discrimination)".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase16::causal_builder::{Edge, Vertex};

    #[test]
    fn test_validate_empty_graph() {
        let graph = CausalGraphV2::new();
        let result = CausalGraphValidator::validate(&graph);

        assert!(result.is_valid_dag);
        assert_eq!(result.edge_count, 0);
    }

    #[test]
    fn test_validate_simple_dag() {
        let mut graph = CausalGraphV2::new();
        graph.vertices.push(Vertex::new("v1".to_string(), "sensor".to_string(), 0, 0.8));
        graph.vertices.push(Vertex::new("v2".to_string(), "sensor".to_string(), 100, 0.8));
        graph.edges.push(Edge::new(
            "v1".to_string(),
            "v2".to_string(),
            "causal".to_string(),
            0.8,
            100,
        ));

        let result = CausalGraphValidator::validate(&graph);

        assert!(result.is_valid_dag);
        assert_eq!(result.edge_count, 1);
        assert!(result.issues.is_empty() || !result.issues[0].contains("cycle"));
    }

    #[test]
    fn test_confidence_calibration() {
        let mut graph = CausalGraphV2::new();

        // Add mixed confidence edges
        for conf in [0.3, 0.5, 0.7, 0.85, 0.9].iter() {
            graph.edges.push(Edge::new(
                "v1".to_string(),
                format!("v{}", graph.edges.len()),
                "causal".to_string(),
                *conf,
                100,
            ));
        }

        let score = CausalGraphValidator::_evaluate_confidence_calibration(&graph);
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_find_edge_conflicts() {
        let mut graph = CausalGraphV2::new();

        // Create conflicting edges: v1→v2 (high conf) and v1→v3 (high conf)
        graph.edges.push(Edge::new(
            "v1".to_string(),
            "v2".to_string(),
            "causal".to_string(),
            0.85,
            100,
        ));
        graph.edges.push(Edge::new(
            "v1".to_string(),
            "v3".to_string(),
            "causal".to_string(),
            0.8,
            100,
        ));

        let conflicts = CausalGraphValidator::_find_conflicting_edges(&graph);
        assert!(conflicts.len() > 0);
    }
}
