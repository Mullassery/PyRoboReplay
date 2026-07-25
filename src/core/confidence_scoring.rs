use crate::core::event::MissionEvent;
use crate::core::failure_detection::{DetectedFailure, FailureSeverity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceTier {
    Fact,        // 1.0 - directly observed/logged
    HighInference, // 0.6-0.8 - strong pattern match
    Hypothesis,  // 0.4-0.6 - reasonable inference
    Speculative, // <0.4 - possible but weak evidence
}

impl ConfidenceTier {
    pub fn range(&self) -> (f32, f32) {
        match self {
            ConfidenceTier::Fact => (0.95, 1.0),
            ConfidenceTier::HighInference => (0.6, 0.8),
            ConfidenceTier::Hypothesis => (0.4, 0.6),
            ConfidenceTier::Speculative => (0.0, 0.4),
        }
    }

    pub fn classify(confidence: f32) -> Self {
        match confidence {
            c if c >= 0.95 => ConfidenceTier::Fact,
            c if c >= 0.6 => ConfidenceTier::HighInference,
            c if c >= 0.4 => ConfidenceTier::Hypothesis,
            _ => ConfidenceTier::Speculative,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub description: String,
    pub evidence_strength: f32, // 0.0-1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceChain {
    pub failure_id: String,
    pub base_confidence: f32,
    pub adjusted_confidence: f32,
    pub confidence_tier: ConfidenceTier,
    pub evidence_items: Vec<EvidenceItem>,
    pub corroborating_factors: Vec<String>,
    pub contradicting_factors: Vec<String>,
    pub aggregation_method: String, // "consensus", "weighted", "bayesian", etc.
}

impl ConfidenceChain {
    pub fn new(failure_id: String, base_confidence: f32) -> Self {
        let confidence_tier = ConfidenceTier::classify(base_confidence);
        Self {
            failure_id,
            base_confidence,
            adjusted_confidence: base_confidence,
            confidence_tier,
            evidence_items: Vec::new(),
            corroborating_factors: Vec::new(),
            contradicting_factors: Vec::new(),
            aggregation_method: "consensus".to_string(),
        }
    }

    pub fn add_evidence(&mut self, evidence: EvidenceItem) {
        self.evidence_items.push(evidence);
    }

    pub fn add_corroborating_factor(&mut self, factor: String) {
        self.corroborating_factors.push(factor);
    }

    pub fn add_contradicting_factor(&mut self, factor: String) {
        self.contradicting_factors.push(factor);
    }

    pub fn strength_summary(&self) -> String {
        format!(
            "{} (base: {:.0}%, adjusted: {:.0}%)",
            format!("{:?}", self.confidence_tier),
            self.base_confidence * 100.0,
            self.adjusted_confidence * 100.0
        )
    }
}

pub struct ConfidenceScoringEngine {
    failure_scores: HashMap<String, ConfidenceChain>,
    all_events: Vec<MissionEvent>,
}

impl ConfidenceScoringEngine {
    pub fn new(events: Vec<MissionEvent>) -> Self {
        Self {
            failure_scores: HashMap::new(),
            all_events: events,
        }
    }

    pub fn score_failure(&mut self, failure: &DetectedFailure) -> ConfidenceChain {
        let mut chain = ConfidenceChain::new(failure.id.clone(), failure.confidence);

        // Add base evidence from the failure itself
        for event_id in &failure.event_ids {
            chain.add_evidence(EvidenceItem {
                event_id: event_id.clone(),
                event_type: failure.failure_type.clone(),
                timestamp: failure.timestamp.to_rfc3339(),
                description: failure.description.clone(),
                evidence_strength: failure.confidence,
            });
        }

        // Apply corroboration boost based on severity and multiple evidence sources
        let corroboration_boost = self.calculate_corroboration_boost(&failure.event_ids, &chain);
        if !corroboration_boost.is_empty() {
            chain.corroborating_factors.extend(corroboration_boost.clone());
            chain.adjusted_confidence = (chain.base_confidence + 0.05).min(1.0);
        }

        // Check for contradicting evidence
        let contradictions = self.find_contradicting_evidence(failure);
        if !contradictions.is_empty() {
            chain.contradicting_factors.extend(contradictions);
            chain.adjusted_confidence = (chain.adjusted_confidence - 0.1).max(0.0);
        }

        // Update confidence tier based on adjusted confidence
        chain.confidence_tier = ConfidenceTier::classify(chain.adjusted_confidence);

        self.failure_scores
            .insert(failure.id.clone(), chain.clone());
        chain
    }

    pub fn score_failures(&mut self, failures: &[DetectedFailure]) -> Vec<ConfidenceChain> {
        failures.iter().map(|f| self.score_failure(f)).collect()
    }

    pub fn aggregate_confidence(&self, failure_ids: &[String]) -> f32 {
        if failure_ids.is_empty() {
            return 0.0;
        }

        let scores: Vec<f32> = failure_ids
            .iter()
            .filter_map(|id| {
                self.failure_scores
                    .get(id)
                    .map(|chain| chain.adjusted_confidence)
            })
            .collect();

        if scores.is_empty() {
            return 0.0;
        }

        // Weighted average: prioritize high-confidence failures
        let weighted_sum: f32 = scores.iter().map(|&s| s * s).sum();
        let weight_sum: f32 = scores.iter().map(|&s| s).sum();

        if weight_sum > 0.0 {
            weighted_sum / weight_sum
        } else {
            0.0
        }
    }

    pub fn get_confidence_chain(&self, failure_id: &str) -> Option<&ConfidenceChain> {
        self.failure_scores.get(failure_id)
    }

    pub fn get_all_chains(&self) -> Vec<&ConfidenceChain> {
        self.failure_scores.values().collect()
    }

    fn calculate_corroboration_boost(&self, event_ids: &[String], _chain: &ConfidenceChain) -> Vec<String> {
        let mut factors = Vec::new();

        // Multiple event sources corroborate the diagnosis
        if event_ids.len() > 1 {
            factors.push(format!("Multiple evidence sources ({} events)", event_ids.len()));
        }

        // Cross-layer corroboration (events from different layers)
        let layer_types: std::collections::HashSet<&str> = event_ids
            .iter()
            .filter_map(|id| {
                // Extract layer hint from event ID (crude but effective)
                if id.contains("layer1") || id.contains("ros") {
                    Some("layer1")
                } else if id.contains("layer2") || id.contains("kernel") {
                    Some("layer2")
                } else if id.contains("layer3") || id.contains("metric") {
                    Some("layer3")
                } else if id.contains("layer4") || id.contains("config") {
                    Some("layer4")
                } else {
                    None
                }
            })
            .collect();

        if layer_types.len() > 1 {
            factors.push(format!("Cross-layer confirmation ({} layers)", layer_types.len()));
        }

        factors
    }

    fn find_contradicting_evidence(&self, failure: &DetectedFailure) -> Vec<String> {
        let mut contradictions = Vec::new();

        // If severity is Critical but base confidence <0.80, that's contradictory
        if failure.severity == FailureSeverity::Critical && failure.confidence < 0.80 {
            contradictions.push("High severity but moderate confidence".to_string());
        }

        // If many events suggest the failure but detection confidence is low
        if failure.event_ids.len() > 3 && failure.confidence < 0.60 {
            contradictions
                .push("Multiple evidence sources but low confidence score".to_string());
        }

        contradictions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_confidence_tier_classification() {
        assert_eq!(
            ConfidenceTier::classify(1.0),
            ConfidenceTier::Fact
        );
        assert_eq!(
            ConfidenceTier::classify(0.95),
            ConfidenceTier::Fact
        );
        assert_eq!(
            ConfidenceTier::classify(0.75),
            ConfidenceTier::HighInference
        );
        assert_eq!(
            ConfidenceTier::classify(0.50),
            ConfidenceTier::Hypothesis
        );
        assert_eq!(
            ConfidenceTier::classify(0.30),
            ConfidenceTier::Speculative
        );
    }

    #[test]
    fn test_confidence_tier_ranges() {
        let (min, max) = ConfidenceTier::Fact.range();
        assert!(min >= 0.95);
        assert_eq!(max, 1.0);

        let (min, max) = ConfidenceTier::HighInference.range();
        assert!(min >= 0.6 && min <= 0.8);
        assert!(max >= 0.6 && max <= 0.8);
    }

    #[test]
    fn test_confidence_chain_creation() {
        let chain = ConfidenceChain::new("failure_1".to_string(), 0.85);
        assert_eq!(chain.failure_id, "failure_1");
        assert_eq!(chain.base_confidence, 0.85);
        assert_eq!(chain.adjusted_confidence, 0.85);
        assert_eq!(chain.confidence_tier, ConfidenceTier::HighInference);
    }

    #[test]
    fn test_confidence_chain_evidence_addition() {
        let mut chain = ConfidenceChain::new("failure_1".to_string(), 0.85);
        chain.add_evidence(EvidenceItem {
            event_id: "evt_1".to_string(),
            event_type: "planner_timeout".to_string(),
            timestamp: "2024-07-25T14:22:15Z".to_string(),
            description: "Navigation planner exceeded 5s timeout".to_string(),
            evidence_strength: 1.0,
        });

        assert_eq!(chain.evidence_items.len(), 1);
    }

    #[test]
    fn test_confidence_chain_corroboration() {
        let mut chain = ConfidenceChain::new("failure_1".to_string(), 0.70);
        chain.add_corroborating_factor("Multiple sources confirm".to_string());

        assert_eq!(chain.corroborating_factors.len(), 1);
    }

    #[test]
    fn test_confidence_scoring_engine_creation() {
        let engine = ConfidenceScoringEngine::new(Vec::new());
        assert_eq!(engine.all_events.len(), 0);
    }

    #[test]
    fn test_aggregate_confidence_empty() {
        let engine = ConfidenceScoringEngine::new(Vec::new());
        let agg = engine.aggregate_confidence(&[]);
        assert_eq!(agg, 0.0);
    }

    #[test]
    fn test_strength_summary() {
        let chain = ConfidenceChain::new("failure_1".to_string(), 0.85);
        let summary = chain.strength_summary();
        assert!(summary.contains("HighInference"));
        assert!(summary.contains("85%"));
    }
}
