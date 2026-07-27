//! Localization failure analysis
//!
//! Detects and categorizes localization issues:
//! - AMCL particle divergence
//! - Odometry drift (wheel slip)
//! - Sensor degradation (rain, dirt, low light)
//! - TF tree corruption or latency
//! - Feature-sparse environments

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalizationCause {
    /// Particles spread excessively (covariance too high)
    ParticleDivergence,
    /// Odometry accumulating systematic error
    OdometryDrift,
    /// Sensor unable to provide sufficient features (rain, low light, texture-less)
    FeatureStarvation,
    /// Sensor noise or degradation increasing
    SensorDegradation,
    /// TF tree delays or inconsistencies
    TFTreeCorruption,
    /// Insufficient feature density in environment
    LowFeatureDensity,
    /// Map matches become ambiguous
    MapAmbiguity,
}

impl std::fmt::Display for LocalizationCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalizationCause::ParticleDivergence => write!(f, "Particle Divergence"),
            LocalizationCause::OdometryDrift => write!(f, "Odometry Drift"),
            LocalizationCause::FeatureStarvation => write!(f, "Feature Starvation"),
            LocalizationCause::SensorDegradation => write!(f, "Sensor Degradation"),
            LocalizationCause::TFTreeCorruption => write!(f, "TF Tree Corruption"),
            LocalizationCause::LowFeatureDensity => write!(f, "Low Feature Density"),
            LocalizationCause::MapAmbiguity => write!(f, "Map Ambiguity"),
        }
    }
}

/// Localization issue with diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizationIssue {
    pub cause: LocalizationCause,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Analyzes localization failures
pub struct LocalizationAnalyzer;

impl LocalizationAnalyzer {
    /// Analyze AMCL particle spread
    pub fn analyze_particle_divergence(
        particle_spread_before: f32,
        particle_spread_after: f32,
    ) -> Option<LocalizationIssue> {
        if particle_spread_after > particle_spread_before * 2.0 && particle_spread_after > 1.0 {
            return Some(LocalizationIssue {
                cause: LocalizationCause::ParticleDivergence,
                confidence: 0.88,
                evidence: vec![
                    format!("Particle spread increased from {:.2}m to {:.2}m",
                            particle_spread_before, particle_spread_after),
                    "Particles no longer concentrated around true pose".to_string(),
                ],
                recommendations: vec![
                    "Increase AMCL recovery behavior frequency".to_string(),
                    "Tune initial covariance parameters".to_string(),
                    "Add loop closure constraints if available".to_string(),
                ],
            });
        }
        None
    }

    /// Analyze odometry drift
    pub fn analyze_odometry_drift(
        odometry_error_rate: f32,  // meters per meter traveled
        distance_traveled: f32,
        timestamp: i64,
    ) -> Option<LocalizationIssue> {
        if odometry_error_rate > 0.01 && distance_traveled > 10.0 {
            let estimated_total_error = distance_traveled * odometry_error_rate;
            return Some(LocalizationIssue {
                cause: LocalizationCause::OdometryDrift,
                confidence: 0.82,
                evidence: vec![
                    format!("Odometry error rate: {:.2}% per meter", odometry_error_rate * 100.0),
                    format!("Total estimated error after {:.1}m: {:.2}m",
                            distance_traveled, estimated_total_error),
                ],
                recommendations: vec![
                    "Calibrate wheel encoders (systematic offset)".to_string(),
                    "Check for tire wear or pressure inconsistency".to_string(),
                    "Enable IMU fused odometry if available".to_string(),
                    "Deploy visual odometry (VO/VIO) for correction".to_string(),
                ],
            });
        }
        None
    }

    /// Analyze feature starvation
    pub fn analyze_feature_starvation(
        detected_features: u32,
        features_threshold: u32,
        lighting_conditions: Option<&str>,
    ) -> Option<LocalizationIssue> {
        if detected_features < features_threshold {
            return Some(LocalizationIssue {
                cause: LocalizationCause::FeatureStarvation,
                confidence: 0.80,
                evidence: vec![
                    format!("Only {} features detected (threshold: {})",
                            detected_features, features_threshold),
                    lighting_conditions
                        .map(|c| format!("Lighting: {}", c))
                        .unwrap_or_else(|| "Unknown lighting conditions".to_string()),
                ],
                recommendations: vec![
                    "Deploy fiducial markers (AprilTags, ArUco)".to_string(),
                    "Add LED beacons or reflectors in low-texture areas".to_string(),
                    "Install supplemental lighting".to_string(),
                    "Upgrade to multi-spectral camera (NIR-sensitive)".to_string(),
                ],
            });
        }
        None
    }

    /// Analyze sensor degradation
    pub fn analyze_sensor_degradation(
        noise_before: f32,
        noise_after: f32,
        outlier_rate_before: f32,
        outlier_rate_after: f32,
    ) -> Option<LocalizationIssue> {
        let noise_increase = noise_after - noise_before;
        let outlier_increase = outlier_rate_after - outlier_rate_before;

        if noise_increase > noise_before * 0.5 || outlier_increase > 0.05 {
            return Some(LocalizationIssue {
                cause: LocalizationCause::SensorDegradation,
                confidence: 0.85,
                evidence: vec![
                    format!("Sensor noise increased by {:.0}%",
                            (noise_increase / noise_before) * 100.0),
                    format!("Outlier rate increased from {:.2}% to {:.2}%",
                            outlier_rate_before * 100.0, outlier_rate_after * 100.0),
                ],
                recommendations: vec![
                    "Inspect camera/LiDAR optics (rain, dirt, dust)".to_string(),
                    "Clean sensor lenses and filters".to_string(),
                    "Check for thermal drift or misalignment".to_string(),
                    "Increase outlier rejection thresholds in AMCL".to_string(),
                ],
            });
        }
        None
    }

    /// Summarize localization health
    pub fn summarize_localization(
        localization_confidence: f32,
        estimated_error: f32,
        map_matches_per_scan: u32,
    ) -> String {
        let mut summary = String::new();

        if localization_confidence < 0.5 {
            summary.push_str("❌ CRITICAL: Localization confidence very low\n");
        } else if localization_confidence < 0.7 {
            summary.push_str("⚠️  WARNING: Localization degraded\n");
        } else {
            summary.push_str("✅ Localization nominal\n");
        }

        if estimated_error > 0.5 {
            summary.push_str(&format!("   Estimated error: {:.2}m (high)\n", estimated_error));
        } else {
            summary.push_str(&format!("   Estimated error: {:.2}m (acceptable)\n", estimated_error));
        }

        if map_matches_per_scan < 10 {
            summary.push_str("   ⚠️  Few map matches per scan (low feature density?)\n");
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_divergence_detection() {
        let issue = LocalizationAnalyzer::analyze_particle_divergence(0.3, 1.5);
        assert!(issue.is_some());
        let i = issue.unwrap();
        assert_eq!(i.cause, LocalizationCause::ParticleDivergence);
        assert!(i.confidence > 0.8);
    }

    #[test]
    fn test_odometry_drift_detection() {
        let issue = LocalizationAnalyzer::analyze_odometry_drift(0.02, 20.0, 1000);
        assert!(issue.is_some());
    }

    #[test]
    fn test_feature_starvation_detection() {
        let issue = LocalizationAnalyzer::analyze_feature_starvation(3, 50, Some("low light (0.15 lux)"));
        assert!(issue.is_some());
        let i = issue.unwrap();
        assert_eq!(i.cause, LocalizationCause::FeatureStarvation);
    }

    #[test]
    fn test_sensor_degradation_detection() {
        let issue = LocalizationAnalyzer::analyze_sensor_degradation(0.05, 0.12, 0.01, 0.08);
        assert!(issue.is_some());
    }

    #[test]
    fn test_summarize_localization_critical() {
        let summary = LocalizationAnalyzer::summarize_localization(0.3, 1.5, 2);
        assert!(summary.contains("CRITICAL"));
    }
}
