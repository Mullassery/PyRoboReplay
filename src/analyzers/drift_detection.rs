//! Drift-Driven Detection Integration
//!
//! Integrates statistical drift detection into gap scoring pipeline.
//! High drift → boost gap_score; correlates with sim-to-real phenomena.

use std::collections::HashMap;

/// Drift statistics from statistical analysis
#[derive(Debug, Clone)]
pub struct DriftStats {
    /// Drift magnitude in standard deviations (2.0 = 2σ)
    pub drift_sigma: f32,

    /// Drift direction: positive (increasing) or negative (decreasing)
    pub drift_direction: f32, // -1.0 to 1.0

    /// Drift detection confidence (0.0-1.0)
    pub confidence: f32,

    /// Metric that drifted (e.g., "response_time")
    pub metric: String,

    /// Drift type: "trend", "jump", "oscillation"
    pub drift_type: String,
}

/// Drift detector for time-series signals
pub struct DriftDetector;

impl DriftDetector {
    /// Detect drift in a signal using sliding window comparison
    pub fn detect_drift(
        signal: &[f32],
        window_size: usize,
    ) -> Option<DriftStats> {
        if signal.len() < window_size * 2 {
            return None; // Not enough data
        }

        let n = signal.len();
        let split = n / 2;

        // First half vs second half
        let first_half = &signal[0..split];
        let second_half = &signal[split..];

        let mean_first = first_half.iter().sum::<f32>() / first_half.len() as f32;
        let mean_second = second_half.iter().sum::<f32>() / second_half.len() as f32;

        // Variance (simplified)
        let var_first = first_half
            .iter()
            .map(|x| (x - mean_first).powi(2))
            .sum::<f32>()
            / first_half.len() as f32;
        let var_second = second_half
            .iter()
            .map(|x| (x - mean_second).powi(2))
            .sum::<f32>()
            / second_half.len() as f32;

        // Pooled standard deviation
        let pooled_var = (var_first + var_second) / 2.0;
        let pooled_std = pooled_var.sqrt().max(0.01); // Avoid division by zero

        // Drift in standard deviations
        let drift_sigma = (mean_second - mean_first) / pooled_std;

        // Direction
        let drift_direction = if drift_sigma > 0.0 { 1.0 } else { -1.0 };

        // Confidence based on magnitude and stability
        let magnitude_confidence = (drift_sigma.abs() / 3.0).min(1.0); // 3σ = max confidence
        let stability_confidence = (1.0 - (var_second / (var_first + 0.01)).abs().min(2.0) / 2.0)
            .max(0.0); // High if variances similar
        let overall_confidence = (magnitude_confidence * 0.6 + stability_confidence * 0.4)
            .clamp(0.0, 1.0);

        // Classify drift type
        let drift_type = if drift_sigma.abs() > 2.0 {
            "jump".to_string() // Sudden shift
        } else if drift_sigma.abs() > 1.0 {
            "trend".to_string() // Gradual trend
        } else {
            "oscillation".to_string() // Noise-like
        };

        Some(DriftStats {
            drift_sigma: drift_sigma.abs(),
            drift_direction,
            confidence: overall_confidence,
            metric: "unknown".to_string(),
            drift_type,
        })
    }

    /// Detect drifts in multiple metrics
    pub fn detect_multi_metric_drift(
        signals: &HashMap<String, Vec<f32>>,
        window_size: usize,
    ) -> Vec<DriftStats> {
        signals
            .iter()
            .filter_map(|(metric, signal)| {
                if let Some(mut drift) = Self::detect_drift(signal, window_size) {
                    drift.metric = metric.clone();
                    Some(drift)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get drift severity multiplier for gap scoring
    /// Returns: (multiplier, explanation)
    pub fn drift_severity_multiplier(drift: &DriftStats) -> (f32, String) {
        let base_multiplier = match drift.drift_type.as_str() {
            "jump" => 1.5, // Sudden jumps are concerning
            "trend" => 1.2, // Gradual trends are moderate
            _ => 1.0,       // Noise-like oscillations are normal
        };

        // Scale by sigma magnitude
        let sigma_scale = (drift.drift_sigma / 2.0).min(1.5); // 2σ = 1x, 4σ = 1.5x
        let multiplier = (base_multiplier * sigma_scale).min(2.0); // Cap at 2x

        let explanation = format!(
            "Drift: {} ({:.1}σ), type: {}",
            drift.metric, drift.drift_sigma, drift.drift_type
        );

        (multiplier, explanation)
    }

    /// Check if drift is significant (anomalous)
    pub fn is_significant(drift: &DriftStats) -> bool {
        // Significant if: >1σ drift AND confidence >0.6
        drift.drift_sigma > 1.0 && drift.confidence > 0.6
    }
}

/// Drift-aware gap scorer
pub struct DriftAwareScorer;

impl DriftAwareScorer {
    /// Boost gap score based on detected drift
    pub fn boost_gap_score(
        base_gap_score: f32,
        drifts: &[DriftStats],
    ) -> (f32, String) {
        if drifts.is_empty() {
            return (base_gap_score, "No drift detected".to_string());
        }

        // Find most significant drift
        let most_significant = drifts
            .iter()
            .max_by(|a, b| {
                a.drift_sigma
                    .partial_cmp(&b.drift_sigma)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(drift) = most_significant {
            if DriftDetector::is_significant(drift) {
                let (multiplier, explanation) = DriftDetector::drift_severity_multiplier(drift);
                let boosted = (base_gap_score * multiplier).min(1.0);

                return (boosted, format!("{} [boost: {:.2}x]", explanation, multiplier));
            }
        }

        (base_gap_score, "Drift present but not significant".to_string())
    }

    /// Compute drift-aware confidence
    pub fn drift_aware_confidence(
        base_confidence: f32,
        drifts: &[DriftStats],
    ) -> f32 {
        if drifts.is_empty() {
            return base_confidence;
        }

        // High drift → higher confidence (drift corroborates finding)
        let avg_drift_sigma = drifts.iter().map(|d| d.drift_sigma).sum::<f32>()
            / drifts.len() as f32;

        // Boost confidence if drift is significant
        let drift_boost = if avg_drift_sigma > 2.0 { 0.15 } else if avg_drift_sigma > 1.0 { 0.10 } else { 0.0 };

        (base_confidence + drift_boost).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_no_drift() {
        let signal = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let drift = DriftDetector::detect_drift(&signal, 2);

        assert!(drift.is_some());
        let drift = drift.unwrap();
        assert!(drift.drift_sigma < 0.1); // Nearly zero drift
    }

    #[test]
    fn test_detect_upward_drift() {
        let mut signal = vec![];
        for i in 0..10 {
            signal.push(i as f32); // 0, 1, 2, ..., 9
        }

        let drift = DriftDetector::detect_drift(&signal, 2);
        assert!(drift.is_some());

        let drift = drift.unwrap();
        assert!(drift.drift_sigma > 0.0); // Positive drift
        assert_eq!(drift.drift_direction, 1.0); // Upward
    }

    #[test]
    fn test_detect_downward_drift() {
        let mut signal = vec![];
        for i in (0..10).rev() {
            signal.push(i as f32); // 9, 8, 7, ..., 0
        }

        let drift = DriftDetector::detect_drift(&signal, 2);
        assert!(drift.is_some());

        let drift = drift.unwrap();
        assert!(drift.drift_direction < 0.0); // Downward
    }

    #[test]
    fn test_drift_type_classification() {
        // Small drift
        let small_drift = DriftStats {
            drift_sigma: 0.5,
            drift_direction: 1.0,
            confidence: 0.7,
            metric: "test".to_string(),
            drift_type: "oscillation".to_string(),
        };

        assert_eq!(small_drift.drift_type, "oscillation");

        // Large drift
        let large_drift = DriftStats {
            drift_sigma: 3.0,
            drift_direction: 1.0,
            confidence: 0.8,
            metric: "test".to_string(),
            drift_type: "jump".to_string(),
        };

        assert_eq!(large_drift.drift_type, "jump");
    }

    #[test]
    fn test_is_significant() {
        let significant = DriftStats {
            drift_sigma: 2.5,
            drift_direction: 1.0,
            confidence: 0.8,
            metric: "test".to_string(),
            drift_type: "jump".to_string(),
        };

        assert!(DriftDetector::is_significant(&significant));

        let insignificant = DriftStats {
            drift_sigma: 0.5,
            drift_direction: 1.0,
            confidence: 0.9,
            metric: "test".to_string(),
            drift_type: "oscillation".to_string(),
        };

        assert!(!DriftDetector::is_significant(&insignificant));
    }

    #[test]
    fn test_drift_severity_multiplier() {
        let jump_drift = DriftStats {
            drift_sigma: 2.0,
            drift_direction: 1.0,
            confidence: 0.8,
            metric: "response_time".to_string(),
            drift_type: "jump".to_string(),
        };

        let (multiplier, _) = DriftDetector::drift_severity_multiplier(&jump_drift);
        assert!(multiplier > 1.0);
        assert!(multiplier < 2.0);
    }

    #[test]
    fn test_boost_gap_score() {
        let drift = DriftStats {
            drift_sigma: 2.5,
            drift_direction: 1.0,
            confidence: 0.85,
            metric: "response_time".to_string(),
            drift_type: "jump".to_string(),
        };

        let base_score = 0.6;
        let (boosted, _) = DriftAwareScorer::boost_gap_score(base_score, &[drift]);

        assert!(boosted > base_score); // Should be boosted
        assert!(boosted <= 1.0);
    }

    #[test]
    fn test_drift_aware_confidence() {
        let drift = DriftStats {
            drift_sigma: 2.5,
            drift_direction: 1.0,
            confidence: 0.85,
            metric: "test".to_string(),
            drift_type: "jump".to_string(),
        };

        let base_conf = 0.7;
        let boosted = DriftAwareScorer::drift_aware_confidence(base_conf, &[drift]);

        assert!(boosted > base_conf); // Should boost confidence
        assert!(boosted <= 1.0);
    }

    #[test]
    fn test_multi_metric_drift() {
        let mut signals = HashMap::new();
        signals.insert(
            "response_time".to_string(),
            vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0],
        );
        signals.insert(
            "temperature".to_string(),
            vec![25.0, 25.0, 25.0, 40.0, 40.0, 40.0],
        );

        let drifts = DriftDetector::detect_multi_metric_drift(&signals, 2);
        assert_eq!(drifts.len(), 2);
    }
}
