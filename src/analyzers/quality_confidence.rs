//! Quality-Aware Confidence Scoring
//!
//! Embeds data quality metadata into confidence calculations.
//! High-quality data → higher confidence; low-quality → lower confidence.

use std::collections::HashMap;

/// Quality metrics for a data source
#[derive(Debug, Clone)]
pub struct QualityMetadata {
    /// Data completeness (0.0-1.0): what % of expected data points present?
    pub completeness: f32,

    /// Signal-to-noise ratio (0.0-1.0): how clean is the signal?
    pub signal_to_noise: f32,

    /// Sensor health score (0.0-1.0): is the sensor functioning properly?
    pub sensor_health: f32,

    /// Calibration status (0.0-1.0): is the sensor calibrated?
    pub calibration_status: f32,

    /// Temporal consistency (0.0-1.0): are timestamps consistent?
    pub temporal_consistency: f32,

    /// Overall quality score (0.0-1.0)
    pub overall_quality: f32,
}

impl QualityMetadata {
    /// Create new quality metadata
    pub fn new() -> Self {
        QualityMetadata {
            completeness: 1.0,
            signal_to_noise: 1.0,
            sensor_health: 1.0,
            calibration_status: 1.0,
            temporal_consistency: 1.0,
            overall_quality: 1.0,
        }
    }

    /// Compute overall quality from components
    pub fn compute_overall_quality(&mut self) {
        // Weighted average: prioritize completeness and calibration
        self.overall_quality = (self.completeness * 0.3
            + self.signal_to_noise * 0.2
            + self.sensor_health * 0.2
            + self.calibration_status * 0.2
            + self.temporal_consistency * 0.1)
            .clamp(0.0, 1.0);
    }

    /// Mark sensor as degraded (e.g., water on lens, thermal throttle)
    pub fn mark_degraded(&mut self, degradation_level: f32) {
        self.sensor_health *= 1.0 - (degradation_level * 0.3); // Up to 30% reduction
        self.compute_overall_quality();
    }

    /// Mark data as incomplete (e.g., message drops)
    pub fn mark_incomplete(&mut self, missing_fraction: f32) {
        self.completeness *= 1.0 - missing_fraction;
        self.compute_overall_quality();
    }

    /// Report calibration drift
    pub fn mark_uncalibrated(&mut self, drift_ppm: f32) {
        // 1000 ppm drift = 50% confidence loss
        let calibration_loss = (drift_ppm / 2000.0).min(1.0);
        self.calibration_status *= 1.0 - calibration_loss;
        self.compute_overall_quality();
    }
}

impl Default for QualityMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Quality-aware confidence calculator
pub struct QualityAwareConfidence;

impl QualityAwareConfidence {
    /// Compute quality-adjusted confidence
    /// Formula: adjusted = base_confidence × quality_weight + overall_quality × (1 - quality_weight)
    pub fn adjust_confidence(
        base_confidence: f32,
        quality: &QualityMetadata,
    ) -> (f32, String) {
        // Quality weight: how much should quality influence confidence?
        // Higher quality data → weight increases confidence calculation
        let quality_weight = 0.6; // 60% base confidence, 40% quality score

        let adjusted = base_confidence * quality_weight + quality.overall_quality * (1.0 - quality_weight);

        let explanation = format!(
            "Quality adj: {:.0}% (completeness: {:.0}%, SNR: {:.0}%, health: {:.0}%, cal: {:.0}%)",
            adjusted * 100.0,
            quality.completeness * 100.0,
            quality.signal_to_noise * 100.0,
            quality.sensor_health * 100.0,
            quality.calibration_status * 100.0
        );

        (adjusted.clamp(0.0, 1.0), explanation)
    }

    /// Compute confidence from evidence + quality
    /// When quality is high, evidence is trusted more
    /// When quality is low, evidence confidence is reduced
    pub fn evidence_quality_confidence(
        evidence_confidence: f32,
        quality: &QualityMetadata,
    ) -> f32 {
        // Evidence × Quality interaction
        let quality_boost = if quality.overall_quality > 0.8 {
            0.1 // High quality → boost by 10%
        } else if quality.overall_quality < 0.5 {
            -0.2 // Low quality → reduce by 20%
        } else {
            0.0
        };

        (evidence_confidence + quality_boost).clamp(0.0, 1.0)
    }

    /// Determine if data quality is sufficient for trust
    pub fn is_high_quality(quality: &QualityMetadata) -> bool {
        quality.overall_quality > 0.75
            && quality.completeness > 0.7
            && quality.sensor_health > 0.7
    }

    /// Determine if data quality is sufficient for any inference
    pub fn is_acceptable_quality(quality: &QualityMetadata) -> bool {
        quality.overall_quality > 0.5
    }

    /// Get quality assessment text
    pub fn quality_assessment(quality: &QualityMetadata) -> String {
        if Self::is_high_quality(quality) {
            "High quality data - high confidence".to_string()
        } else if Self::is_acceptable_quality(quality) {
            "Acceptable quality data - moderate confidence".to_string()
        } else {
            "Low quality data - low confidence, caution advised".to_string()
        }
    }
}

/// Quality metric aggregator for multi-sensor systems
pub struct QualityAggregator;

impl QualityAggregator {
    /// Compute fleet-wide data quality
    pub fn aggregate_quality(
        qualities: &HashMap<String, QualityMetadata>,
    ) -> QualityMetadata {
        if qualities.is_empty() {
            return QualityMetadata::new();
        }

        let mut aggregate = QualityMetadata::new();

        aggregate.completeness = qualities.values().map(|q| q.completeness).sum::<f32>()
            / qualities.len() as f32;
        aggregate.signal_to_noise = qualities.values().map(|q| q.signal_to_noise).sum::<f32>()
            / qualities.len() as f32;
        aggregate.sensor_health = qualities.values().map(|q| q.sensor_health).sum::<f32>()
            / qualities.len() as f32;
        aggregate.calibration_status =
            qualities.values().map(|q| q.calibration_status).sum::<f32>()
                / qualities.len() as f32;
        aggregate.temporal_consistency =
            qualities.values().map(|q| q.temporal_consistency).sum::<f32>()
                / qualities.len() as f32;

        aggregate.compute_overall_quality();
        aggregate
    }

    /// Find worst-quality sensor
    pub fn worst_quality(
        qualities: &HashMap<String, QualityMetadata>,
    ) -> Option<(String, f32)> {
        qualities
            .iter()
            .min_by(|a, b| {
                a.1.overall_quality
                    .partial_cmp(&b.1.overall_quality)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, quality)| (name.clone(), quality.overall_quality))
    }

    /// Find best-quality sensor
    pub fn best_quality(
        qualities: &HashMap<String, QualityMetadata>,
    ) -> Option<(String, f32)> {
        qualities
            .iter()
            .max_by(|a, b| {
                a.1.overall_quality
                    .partial_cmp(&b.1.overall_quality)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, quality)| (name.clone(), quality.overall_quality))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_metadata_creation() {
        let quality = QualityMetadata::new();
        assert_eq!(quality.overall_quality, 1.0);
        assert_eq!(quality.completeness, 1.0);
    }

    #[test]
    fn test_quality_computation() {
        let mut quality = QualityMetadata::new();
        quality.completeness = 0.9;
        quality.signal_to_noise = 0.8;
        quality.sensor_health = 0.7;
        quality.calibration_status = 1.0;
        quality.temporal_consistency = 0.95;

        quality.compute_overall_quality();

        // Expected: 0.9*0.3 + 0.8*0.2 + 0.7*0.2 + 1.0*0.2 + 0.95*0.1
        // = 0.27 + 0.16 + 0.14 + 0.2 + 0.095 = 0.875
        assert!((quality.overall_quality - 0.875).abs() < 0.01);
    }

    #[test]
    fn test_mark_degraded() {
        let mut quality = QualityMetadata::new();
        quality.mark_degraded(0.5);

        assert!(quality.sensor_health < 1.0);
        assert!(quality.overall_quality < 1.0);
    }

    #[test]
    fn test_mark_incomplete() {
        let mut quality = QualityMetadata::new();
        quality.mark_incomplete(0.2); // 20% missing

        assert!(quality.completeness < 1.0);
        assert_eq!(quality.completeness, 0.8);
    }

    #[test]
    fn test_mark_uncalibrated() {
        let mut quality = QualityMetadata::new();
        quality.mark_uncalibrated(500.0); // 500 ppm drift

        assert!(quality.calibration_status < 1.0);
    }

    #[test]
    fn test_adjust_confidence() {
        let quality = QualityMetadata::new();
        let (adjusted, _) = QualityAwareConfidence::adjust_confidence(0.8, &quality);

        // With perfect quality, should be boosted slightly
        assert!(adjusted > 0.79);
    }

    #[test]
    fn test_adjust_confidence_low_quality() {
        let mut quality = QualityMetadata::new();
        quality.overall_quality = 0.5;

        let (adjusted, _) = QualityAwareConfidence::adjust_confidence(0.8, &quality);

        // With low quality, should be reduced
        assert!(adjusted < 0.8);
    }

    #[test]
    fn test_is_high_quality() {
        let mut quality = QualityMetadata::new();
        assert!(QualityAwareConfidence::is_high_quality(&quality));

        quality.overall_quality = 0.6;
        assert!(!QualityAwareConfidence::is_high_quality(&quality));
    }

    #[test]
    fn test_is_acceptable_quality() {
        let mut quality = QualityMetadata::new();
        assert!(QualityAwareConfidence::is_acceptable_quality(&quality));

        quality.overall_quality = 0.3;
        assert!(!QualityAwareConfidence::is_acceptable_quality(&quality));
    }

    #[test]
    fn test_quality_assessment() {
        let mut quality = QualityMetadata::new();
        quality.overall_quality = 0.9;
        quality.completeness = 0.8;
        quality.sensor_health = 0.8;

        let assessment = QualityAwareConfidence::quality_assessment(&quality);
        assert!(assessment.contains("High quality"));
    }

    #[test]
    fn test_aggregate_quality() {
        let mut qualities = HashMap::new();

        let mut q1 = QualityMetadata::new();
        q1.completeness = 0.95;
        q1.signal_to_noise = 0.90;
        q1.sensor_health = 0.90;
        q1.calibration_status = 0.95;
        q1.temporal_consistency = 0.95;
        q1.compute_overall_quality();

        let mut q2 = QualityMetadata::new();
        q2.completeness = 0.70;
        q2.signal_to_noise = 0.70;
        q2.sensor_health = 0.60;
        q2.calibration_status = 0.70;
        q2.temporal_consistency = 0.70;
        q2.compute_overall_quality();

        qualities.insert("sensor1".to_string(), q1.clone());
        qualities.insert("sensor2".to_string(), q2.clone());

        let aggregate = QualityAggregator::aggregate_quality(&qualities);
        // Average of q1 and q2 overall qualities
        let expected = (q1.overall_quality + q2.overall_quality) / 2.0;
        assert!((aggregate.overall_quality - expected).abs() < 0.01);
    }

    #[test]
    fn test_worst_quality() {
        let mut qualities = HashMap::new();

        let mut q1 = QualityMetadata::new();
        q1.overall_quality = 0.9;

        let mut q2 = QualityMetadata::new();
        q2.overall_quality = 0.6;

        qualities.insert("sensor1".to_string(), q1);
        qualities.insert("sensor2".to_string(), q2);

        let (worst_name, worst_quality) = QualityAggregator::worst_quality(&qualities).unwrap();
        assert_eq!(worst_name, "sensor2");
        assert_eq!(worst_quality, 0.6);
    }

    #[test]
    fn test_best_quality() {
        let mut qualities = HashMap::new();

        let mut q1 = QualityMetadata::new();
        q1.overall_quality = 0.9;

        let mut q2 = QualityMetadata::new();
        q2.overall_quality = 0.6;

        qualities.insert("sensor1".to_string(), q1);
        qualities.insert("sensor2".to_string(), q2);

        let (best_name, best_quality) = QualityAggregator::best_quality(&qualities).unwrap();
        assert_eq!(best_name, "sensor1");
        assert_eq!(best_quality, 0.9);
    }
}
