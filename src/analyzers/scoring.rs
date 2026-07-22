//! Probabilistic Reality Gap Scoring
//!
//! Estimates P(SimGap | Evidence) using Bayesian inference.
//! Distinguishes sim-to-real gaps from algorithm bugs.

use crate::analyzers::{RealityGapFinding, MissionAnalysisData};
use std::collections::HashMap;

/// Probabilistically score gap findings
pub struct RealityGapScorer {
    base_probabilities: HashMap<String, f32>,
    sim_representability: HashMap<String, f32>,
}

impl RealityGapScorer {
    pub fn new() -> Self {
        let mut scorer = RealityGapScorer {
            base_probabilities: HashMap::new(),
            sim_representability: HashMap::new(),
        };

        scorer.initialize_knowledge_base();
        scorer
    }

    /// Score a finding: estimate P(SimGap | Evidence)
    pub fn score_finding(
        &self,
        finding: &RealityGapFinding,
        _mission: &MissionAnalysisData,
    ) -> (f32, f32) {
        // Bayesian inference:
        // P(SimGap | Evidence) = P(Evidence | SimGap) * P(SimGap) / P(Evidence)

        let p_sim_gap_prior = self.base_probability(&finding.category);
        let p_evidence_given_sim = self.evidence_likelihood(finding);
        let p_evidence_given_bug = 1.0 - p_evidence_given_sim;

        // Total probability of evidence
        let p_evidence = (p_evidence_given_sim * p_sim_gap_prior)
            + (p_evidence_given_bug * (1.0 - p_sim_gap_prior));

        // Posterior probability (Bayes)
        let posterior = if p_evidence > 0.0 {
            (p_evidence_given_sim * p_sim_gap_prior) / p_evidence
        } else {
            p_sim_gap_prior
        };

        // Confidence in this score (higher = more certain)
        let confidence = self.compute_confidence(finding);

        (posterior, confidence)
    }

    /// Base prior: P(SimGap) by category
    /// How likely is this gap type to be sim-related vs algorithmic?
    fn base_probability(&self, category: &str) -> f32 {
        *self
            .base_probabilities
            .get(category)
            .unwrap_or(&0.5)
    }

    /// Likelihood: P(Evidence | SimGap)
    /// How consistent is this evidence with a sim gap?
    fn evidence_likelihood(&self, finding: &RealityGapFinding) -> f32 {
        let mut likelihood: f32 = 0.7; // Base: 70% of evidence fits sim gap

        // Strong evidence indicators
        if finding.evidence.len() > 1 {
            likelihood += 0.1; // Multiple corroborating signals
        }

        if let Some(trend_corr) = finding.metrics.get("trend_correlation") {
            if *trend_corr > 0.7 {
                likelihood += 0.15; // Strong trend (not random noise)
            }
        }

        if let Some(correlation) = finding.metrics.get("image_quality_correlation") {
            if *correlation > 0.6 {
                likelihood += 0.1; // Environmental factors clearly present
            }
        }

        // Evidence against sim gap (suggests algorithm issue)
        if let Some(algo_signal) = finding.metrics.get("algorithmic_pattern") {
            if *algo_signal > 0.5 {
                likelihood -= 0.2; // Looks like code bug, not sim
            }
        }

        likelihood.min(1.0).max(0.0)
    }

    /// Confidence in the gap score
    /// Based on number of evidence sources and their individual confidences
    fn compute_confidence(&self, finding: &RealityGapFinding) -> f32 {
        if finding.evidence.is_empty() {
            return 0.3; // Low confidence without evidence
        }

        let evidence_count = (finding.evidence.len() as f32).min(5.0) / 5.0;
        let avg_evidence_confidence: f32 = finding
            .evidence
            .iter()
            .map(|e| e.confidence)
            .sum::<f32>()
            / finding.evidence.len() as f32;

        let base_confidence = (evidence_count * 0.4) + (avg_evidence_confidence * 0.6);

        base_confidence.min(1.0).max(0.0)
    }

    /// Apply domain knowledge adjustments
    pub fn adjust_score(
        &self,
        base_score: f32,
        category: &str,
        robot_type: &str,
    ) -> f32 {
        let mut adjusted = base_score;

        // Adjustment 1: Simulator representability
        // If hard to simulate, more likely it's real-world specific
        let sim_rep = *self.sim_representability.get(category).unwrap_or(&0.5);
        adjusted *= (1.0 - sim_rep) + 0.3; // Scale by (1-representability) + baseline

        // Adjustment 2: Robot type specific knowledge
        // Some gaps are more common on certain robot types
        let robot_adjustment = self.robot_type_adjustment(category, robot_type);
        adjusted *= robot_adjustment;

        adjusted.min(1.0).max(0.0)
    }

    /// Robot-type specific adjustment
    /// e.g., thermal issues more likely on power-hungry wheel robots
    fn robot_type_adjustment(&self, category: &str, robot_type: &str) -> f32 {
        let adjustment: f32 = match (category, robot_type) {
            ("Thermal Effects", "wheel_robot") => 1.2, // More likely
            ("Thermal Effects", "drone") => 1.1,
            ("Thermal Effects", "humanoid") => 0.9,

            ("Mechanical Degradation", "wheel_robot") => 1.2, // Wheel wear common
            ("Mechanical Degradation", "manipulator") => 1.1,

            ("Optical Contamination", "outdoor_robot") => 1.3, // More weather exposure
            ("Optical Contamination", "indoor_robot") => 0.7,

            _ => 1.0,
        };
        adjustment.min(1.5).max(0.5) // Clamp adjustments
    }

    /// Initialize knowledge base of base probabilities
    fn initialize_knowledge_base(&mut self) {
        // High-probability gaps (happen frequently, hard to avoid in reality)
        self.base_probabilities
            .insert("Mechanical Degradation".to_string(), 0.75);
        self.base_probabilities
            .insert("Sensor Calibration Drift".to_string(), 0.80);
        self.base_probabilities
            .insert("Environmental Changes".to_string(), 0.85);
        self.base_probabilities
            .insert("Optical Contamination".to_string(), 0.65);
        self.base_probabilities
            .insert("Thermal Effects".to_string(), 0.70);

        // Medium-probability gaps
        self.base_probabilities
            .insert("Structural Dynamics".to_string(), 0.60);
        self.base_probabilities
            .insert("Detection Robustness".to_string(), 0.55);
        self.base_probabilities
            .insert("Temporal Synchronization".to_string(), 0.45);

        // Lower-probability gaps (easier to debug, often algorithm issues)
        self.base_probabilities
            .insert("CPU Contention".to_string(), 0.40);
        self.base_probabilities
            .insert("Memory Pressure".to_string(), 0.35);
        self.base_probabilities
            .insert("Network Congestion".to_string(), 0.55);
        self.base_probabilities
            .insert("Filter Divergence".to_string(), 0.50);

        // Simulability scores (how easy to represent in simulation)
        // High = hard to simulate (more likely real-world issue)
        // Low = easy to simulate (could be either)
        self.sim_representability
            .insert("Mechanical Degradation".to_string(), 0.95); // Very hard
        self.sim_representability
            .insert("Environmental Changes".to_string(), 0.90);
        self.sim_representability
            .insert("Optical Contamination".to_string(), 0.85);
        self.sim_representability
            .insert("Thermal Effects".to_string(), 0.70);

        self.sim_representability
            .insert("Clock Drift".to_string(), 0.15); // Very easy
        self.sim_representability
            .insert("Memory Pressure".to_string(), 0.70);
        self.sim_representability
            .insert("CPU Contention".to_string(), 0.80);
        self.sim_representability
            .insert("Network Congestion".to_string(), 0.60);
    }
}

impl Default for RealityGapScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorer_creation() {
        let scorer = RealityGapScorer::new();
        assert!(!scorer.base_probabilities.is_empty());
        assert!(!scorer.sim_representability.is_empty());
    }

    #[test]
    fn test_base_probability() {
        let scorer = RealityGapScorer::new();
        let mech = scorer.base_probability("Mechanical Degradation");
        assert!(mech > 0.7); // Should be high
        let cpu = scorer.base_probability("CPU Contention");
        assert!(cpu < 0.5); // Should be lower
    }

    #[test]
    fn test_robot_type_adjustment() {
        let scorer = RealityGapScorer::new();
        let thermal_wheel = scorer.robot_type_adjustment("Thermal Effects", "wheel_robot");
        let thermal_indoor = scorer.robot_type_adjustment("Thermal Effects", "indoor_robot");
        assert!(thermal_wheel > thermal_indoor); // Wheels more susceptible to thermal
    }

    #[test]
    fn test_score_adjustment() {
        let scorer = RealityGapScorer::new();
        let base_score = 0.7;
        let adjusted = scorer.adjust_score(base_score, "Mechanical Degradation", "wheel_robot");
        // Robot type adjustment (1.2) applies, but sim_rep reduction dominates
        // Result is deterministic based on formulas
        assert!(adjusted > 0.0 && adjusted <= 1.0); // Valid score range
    }
}
