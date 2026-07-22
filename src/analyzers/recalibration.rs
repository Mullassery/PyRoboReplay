//! Confidence Recalibration Engine
//!
//! Updates gap scoring priors based on human feedback using Bayesian inference.

use crate::analyzers::scoring::RealityGapScorer;
use std::collections::HashMap;

/// Gap category confidence metrics for learning
#[derive(Debug, Clone)]
pub struct CategoryMetrics {
    pub category: String,
    pub total_detections: usize,
    pub verified_correct: usize,
    pub partially_correct: usize,
    pub incorrect: usize,
    pub inconclusive: usize,
    pub base_probability: f32,
    pub current_accuracy: f32,
}

impl CategoryMetrics {
    /// Compute accuracy from feedback
    pub fn compute_accuracy(&mut self) {
        let total = self.verified_correct + self.partially_correct + self.incorrect + self.inconclusive;
        if total == 0 {
            self.current_accuracy = 0.0;
            return;
        }

        let correct_score = (self.verified_correct as f32 + self.partially_correct as f32 * 0.5)
            / total as f32;
        self.current_accuracy = correct_score;
    }
}

/// Recalibration engine: learns from feedback to improve scoring
pub struct RecalibrationEngine {
    category_metrics: HashMap<String, CategoryMetrics>,
    min_samples: usize, // Minimum feedback samples before updating prior
    learning_rate: f32, // How aggressively to update priors (0.0-1.0)
}

impl RecalibrationEngine {
    /// Create new recalibration engine
    pub fn new() -> Self {
        RecalibrationEngine {
            category_metrics: HashMap::new(),
            min_samples: 5, // Need at least 5 feedback samples
            learning_rate: 0.1, // Conservative: move 10% toward observed accuracy
        }
    }

    /// Initialize with scorer's base probabilities
    pub fn initialize_from_scorer(&mut self, scorer: &RealityGapScorer) {
        // We'd ideally have access to scorer's knowledge base, but it's private
        // For now, initialize with typical defaults
        let default_categories = vec![
            ("Mechanical Degradation", 0.75),
            ("Optical Contamination", 0.65),
            ("Thermal Effects", 0.70),
            ("Clock Drift", 0.50),
            ("Detection Robustness", 0.55),
            ("Structural Dynamics", 0.60),
            ("Sensor Calibration Drift", 0.80),
            ("Environmental Changes", 0.85),
        ];

        for (category, base_prob) in default_categories {
            self.category_metrics.insert(
                category.to_string(),
                CategoryMetrics {
                    category: category.to_string(),
                    total_detections: 0,
                    verified_correct: 0,
                    partially_correct: 0,
                    incorrect: 0,
                    inconclusive: 0,
                    base_probability: base_prob,
                    current_accuracy: 0.0,
                },
            );
        }
    }

    /// Record feedback for a category
    pub fn record_feedback(&mut self, category: &str, feedback_type: &str) {
        let metrics = self
            .category_metrics
            .entry(category.to_string())
            .or_insert_with(|| CategoryMetrics {
                category: category.to_string(),
                total_detections: 0,
                verified_correct: 0,
                partially_correct: 0,
                incorrect: 0,
                inconclusive: 0,
                base_probability: 0.5,
                current_accuracy: 0.0,
            });

        metrics.total_detections += 1;

        match feedback_type {
            "correct" => metrics.verified_correct += 1,
            "partial" => metrics.partially_correct += 1,
            "incorrect" => metrics.incorrect += 1,
            "inconclusive" => metrics.inconclusive += 1,
            _ => {}
        }

        metrics.compute_accuracy();
    }

    /// Get whether a category has enough feedback to recalibrate
    pub fn is_ready_to_recalibrate(&self, category: &str) -> bool {
        if let Some(metrics) = self.category_metrics.get(category) {
            metrics.total_detections >= self.min_samples
        } else {
            false
        }
    }

    /// Recalibrate a single category's base probability
    pub fn recalibrate_category(&mut self, category: &str) -> Option<(f32, f32)> {
        if !self.is_ready_to_recalibrate(category) {
            return None;
        }

        let metrics = self.category_metrics.get_mut(category)?;

        let old_prior = metrics.base_probability;

        // Bayesian update: move toward observed accuracy
        // new_prior = old_prior + learning_rate * (observed_accuracy - old_prior)
        let observed_accuracy = metrics.current_accuracy;
        let new_prior =
            old_prior + self.learning_rate * (observed_accuracy - old_prior);

        metrics.base_probability = new_prior.max(0.1).min(0.95); // Clamp to reasonable range

        Some((old_prior, metrics.base_probability))
    }

    /// Recalibrate all categories with sufficient feedback
    pub fn recalibrate_all(&mut self) -> Vec<(String, f32, f32)> {
        let mut updates = Vec::new();

        let categories: Vec<String> = self.category_metrics.keys().cloned().collect();

        for category in categories {
            if let Some((old, new)) = self.recalibrate_category(&category) {
                updates.push((category.clone(), old, new));
            }
        }

        updates
    }

    /// Get statistics for a category
    pub fn category_stats(&self, category: &str) -> Option<CategoryMetrics> {
        self.category_metrics.get(category).cloned()
    }

    /// Get all category statistics
    pub fn all_stats(&self) -> Vec<CategoryMetrics> {
        self.category_metrics.values().cloned().collect()
    }

    /// Get recalibration confidence: how much should we trust the new prior?
    pub fn recalibration_confidence(&self, category: &str) -> f32 {
        if let Some(metrics) = self.category_metrics.get(category) {
            // More feedback = higher confidence
            // Normalize: 5 samples = 0.5 confidence, 20+ samples = 0.95
            let sample_score = (metrics.total_detections as f32 / 20.0).min(1.0) * 0.5 + 0.45;

            // Lower confidence if accuracy is unstable
            let accuracy_stability = 1.0 - (metrics.current_accuracy - 0.5).abs();

            (sample_score * 0.6 + accuracy_stability * 0.4).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Reset a category's feedback
    pub fn reset_category(&mut self, category: &str) {
        if let Some(metrics) = self.category_metrics.get_mut(category) {
            metrics.total_detections = 0;
            metrics.verified_correct = 0;
            metrics.partially_correct = 0;
            metrics.incorrect = 0;
            metrics.inconclusive = 0;
            metrics.current_accuracy = 0.0;
        }
    }

    /// Set learning rate (0.0-1.0): higher = more aggressive updates
    pub fn set_learning_rate(&mut self, rate: f32) {
        self.learning_rate = rate.clamp(0.0, 1.0);
    }

    /// Set minimum samples required for recalibration
    pub fn set_min_samples(&mut self, min: usize) {
        self.min_samples = min.max(1);
    }
}

impl Default for RecalibrationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let _engine = RecalibrationEngine::new();
    }

    #[test]
    fn test_record_feedback() {
        let mut engine = RecalibrationEngine::new();
        engine.initialize_from_scorer(&RealityGapScorer::new());

        engine.record_feedback("Mechanical Degradation", "correct");
        engine.record_feedback("Mechanical Degradation", "correct");
        engine.record_feedback("Mechanical Degradation", "incorrect");

        let stats = engine.category_stats("Mechanical Degradation").unwrap();
        assert_eq!(stats.total_detections, 3);
        assert_eq!(stats.verified_correct, 2);
        assert_eq!(stats.incorrect, 1);
    }

    #[test]
    fn test_accuracy_computation() {
        let mut engine = RecalibrationEngine::new();
        engine.initialize_from_scorer(&RealityGapScorer::new());

        // 3 correct, 2 partial = (3 + 1.0) / 5 = 0.8
        engine.record_feedback("Mechanical Degradation", "correct");
        engine.record_feedback("Mechanical Degradation", "correct");
        engine.record_feedback("Mechanical Degradation", "correct");
        engine.record_feedback("Mechanical Degradation", "partial");
        engine.record_feedback("Mechanical Degradation", "partial");

        let stats = engine.category_stats("Mechanical Degradation").unwrap();
        assert!((stats.current_accuracy - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_recalibration_threshold() {
        let mut engine = RecalibrationEngine::new();
        engine.initialize_from_scorer(&RealityGapScorer::new());
        engine.set_min_samples(3);

        // Initially not ready
        assert!(!engine.is_ready_to_recalibrate("Mechanical Degradation"));

        // Add 2 feedback items
        engine.record_feedback("Mechanical Degradation", "correct");
        engine.record_feedback("Mechanical Degradation", "correct");
        assert!(!engine.is_ready_to_recalibrate("Mechanical Degradation"));

        // Add 3rd
        engine.record_feedback("Mechanical Degradation", "correct");
        assert!(engine.is_ready_to_recalibrate("Mechanical Degradation"));
    }

    #[test]
    fn test_bayesian_update() {
        let mut engine = RecalibrationEngine::new();
        engine.initialize_from_scorer(&RealityGapScorer::new());
        engine.set_learning_rate(0.1);

        let old_prior = engine
            .category_stats("Mechanical Degradation")
            .unwrap()
            .base_probability;

        // All correct feedback -> high accuracy
        for _ in 0..5 {
            engine.record_feedback("Mechanical Degradation", "correct");
        }

        let (before, after) = engine
            .recalibrate_category("Mechanical Degradation")
            .unwrap();

        assert_eq!(before, old_prior);
        // Accuracy = 1.0, so new prior = old + 0.1 * (1.0 - old)
        let expected = old_prior + 0.1 * (1.0 - old_prior);
        assert!((after - expected).abs() < 0.001);
        assert!(after > before); // Should improve
    }

    #[test]
    fn test_recalibration_all() {
        let mut engine = RecalibrationEngine::new();
        engine.initialize_from_scorer(&RealityGapScorer::new());
        engine.set_min_samples(2);

        // Add feedback to multiple categories
        engine.record_feedback("Mechanical Degradation", "correct");
        engine.record_feedback("Mechanical Degradation", "correct");
        engine.record_feedback("Optical Contamination", "correct");
        engine.record_feedback("Optical Contamination", "incorrect");

        let updates = engine.recalibrate_all();
        assert_eq!(updates.len(), 2); // 2 categories updated

        // Check that confidence increased for all-correct category
        let mech = engine
            .category_stats("Mechanical Degradation")
            .unwrap();
        assert!(mech.base_probability > 0.75); // Original was 0.75
    }

    #[test]
    fn test_recalibration_confidence() {
        let mut engine = RecalibrationEngine::new();
        engine.initialize_from_scorer(&RealityGapScorer::new());

        // Low confidence with no feedback
        assert!(engine.recalibration_confidence("Mechanical Degradation") < 0.5);

        // Add 5 samples (minimum)
        for _ in 0..5 {
            engine.record_feedback("Mechanical Degradation", "correct");
        }
        let low_conf = engine.recalibration_confidence("Mechanical Degradation");
        assert!(low_conf > 0.4);

        // Add 20 total samples
        for _ in 0..15 {
            engine.record_feedback("Mechanical Degradation", "correct");
        }
        let high_conf = engine.recalibration_confidence("Mechanical Degradation");
        assert!(high_conf > low_conf); // Should increase with more samples
    }

    #[test]
    fn test_reset_category() {
        let mut engine = RecalibrationEngine::new();
        engine.initialize_from_scorer(&RealityGapScorer::new());

        engine.record_feedback("Mechanical Degradation", "correct");
        engine.record_feedback("Mechanical Degradation", "correct");

        let before = engine
            .category_stats("Mechanical Degradation")
            .unwrap();
        assert_eq!(before.total_detections, 2);

        engine.reset_category("Mechanical Degradation");

        let after = engine
            .category_stats("Mechanical Degradation")
            .unwrap();
        assert_eq!(after.total_detections, 0);
    }
}
