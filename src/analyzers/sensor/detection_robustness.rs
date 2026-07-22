//! Detection Robustness Analyzer
//!
//! Analyzes why object detection fails: environmental factors vs algorithm issues.
//! Correlates detection confidence with image quality to classify root cause.

use crate::analyzers::{
    GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding, Severity, Evidence,
};
use std::collections::HashMap;

pub struct DetectionRobustnessAnalyzer;

impl DetectionRobustnessAnalyzer {
    pub fn new() -> Self {
        DetectionRobustnessAnalyzer
    }

    /// Analyze detection confidence trends over mission
    pub fn analyze_detection_confidence(
        &self,
        detection_results: &[crate::analyzers::DetectionResult],
        camera_frames: &[crate::analyzers::CameraFrame],
    ) -> Option<RealityGapFinding> {
        if detection_results.len() < 20 {
            return None;
        }

        // Group detections by frame (approximate by time)
        let frame_confidences = self.compute_frame_confidences(detection_results);

        if frame_confidences.len() < 10 {
            return None;
        }

        // Compute confidence trend
        let (initial_conf, final_conf, trend_slope) =
            self.compute_confidence_trend(&frame_confidences);

        // Check if confidence is declining
        let confidence_decline_pct = (initial_conf - final_conf) / initial_conf.max(0.01) * 100.0;

        if confidence_decline_pct < 10.0 {
            return None; // Insufficient decline
        }

        // Extract image quality trend if available
        let image_quality = self.extract_image_quality_trend(camera_frames);

        // Correlate: is confidence decline correlated with image quality?
        let correlation = if !image_quality.is_empty() && !frame_confidences.is_empty() {
            self.compute_correlation(&frame_confidences, &image_quality)
        } else {
            0.0
        };

        let mut metrics = HashMap::new();
        metrics.insert("confidence_decline_pct".to_string(), confidence_decline_pct);
        metrics.insert("initial_confidence".to_string(), initial_conf);
        metrics.insert("final_confidence".to_string(), final_conf);
        metrics.insert("trend_slope_per_hour".to_string(), trend_slope);
        metrics.insert(
            "image_quality_correlation".to_string(),
            correlation,
        );

        // Classify root cause based on correlation
        let (finding_type, description, gap_score, confidence, severity) = if correlation > 0.6 {
            // Strong correlation: environmental issue (sim gap)
            (
                "Detection Confidence Degradation (Environmental)".to_string(),
                format!(
                    "Object detection confidence declined {:.1}% over mission. \
                     Strong correlation with image quality (r={:.2}). \
                     Likely environmental factors: lighting, shadows, weather.",
                    confidence_decline_pct,
                    correlation
                ),
                0.80,
                0.78,
                Severity::Medium,
            )
        } else if correlation > 0.3 {
            // Moderate correlation: mixed causes
            (
                "Detection Confidence Degradation (Mixed)".to_string(),
                format!(
                    "Object detection confidence declined {:.1}%. \
                     Moderate correlation with image quality (r={:.2}). \
                     Could be environmental or algorithmic.",
                    confidence_decline_pct,
                    correlation
                ),
                0.60,
                0.65,
                Severity::Medium,
            )
        } else {
            // Low correlation: algorithmic issue (code bug)
            (
                "Detection Confidence Degradation (Algorithmic)".to_string(),
                format!(
                    "Object detection confidence declined {:.1}% but image quality stable. \
                     Low correlation (r={:.2}). \
                     Likely algorithmic issue: model drift, distribution shift, or resource contention.",
                    confidence_decline_pct,
                    correlation
                ),
                0.40,
                0.72,
                Severity::High,
            )
        };

        Some(RealityGapFinding {
            domain: RealityDomain::Sensor,
            category: "Detection Robustness".to_string(),
            finding_type,
            severity,
            confidence,
            reality_gap_score: gap_score,
            description,
            evidence: vec![
                Evidence {
                    signal: "detection_confidence".to_string(),
                    value: final_conf,
                    timestamp: frame_confidences.last().map(|(t, _)| *t).unwrap_or(0.0),
                    confidence: 0.85,
                },
                Evidence {
                    signal: "confidence_trend".to_string(),
                    value: trend_slope,
                    timestamp: frame_confidences.last().map(|(t, _)| *t).unwrap_or(0.0),
                    confidence: 0.80,
                },
                Evidence {
                    signal: "quality_correlation".to_string(),
                    value: correlation,
                    timestamp: frame_confidences.last().map(|(t, _)| *t).unwrap_or(0.0),
                    confidence: 0.75,
                },
            ],
            metrics,
            sim_recreation_suggestion:
                if correlation > 0.6 {
                    "Model dynamic lighting: sky dome sun position(time), glare effects, shadows. \
                     Run sim with time-varying lighting to reproduce degradation."
                        .to_string()
                } else {
                    "Model distribution shift: real-world data differs from training data. \
                     Augment training with challenging lighting/weather conditions."
                        .to_string()
                },
            remediation:
                if correlation > 0.6 {
                    "1. Ensure adequate robot lighting (LED ring). \
                     2. Add lens hood to reduce glare. \
                     3. Use polarizing filter for reflections. \
                     4. Train model with lighting augmentation."
                        .to_string()
                } else {
                    "1. Re-train detection model with real deployment data. \
                     2. Implement online learning / active learning. \
                     3. Add model monitoring / drift detection. \
                     4. Consider domain adaptation techniques."
                        .to_string()
                },
            detection_time_sec: frame_confidences.last().map(|(t, _)| *t),
        })
    }

    /// Analyze false positive rate and spatial patterns
    pub fn analyze_false_positives(
        &self,
        detection_results: &[crate::analyzers::DetectionResult],
    ) -> Option<RealityGapFinding> {
        if detection_results.len() < 50 {
            return None;
        }

        // Heuristic: consider detections with confidence < 0.3 as likely false positives
        let low_confidence_count = detection_results.iter().filter(|d| d.confidence < 0.3).count();
        let fp_rate = low_confidence_count as f32 / detection_results.len() as f32;

        if fp_rate < 0.05 {
            return None; // Below threshold
        }

        // Analyze spatial distribution of FPs
        let fp_locations: Vec<(f32, f32)> = detection_results
            .iter()
            .filter(|d| d.confidence < 0.3)
            .map(|d| (d.x, d.y))
            .collect();

        // Check if FPs cluster in specific regions (edge, bright areas, etc.)
        let clustering_score = self.compute_spatial_clustering(&fp_locations);

        let mut metrics = HashMap::new();
        metrics.insert("false_positive_rate".to_string(), fp_rate * 100.0);
        metrics.insert("low_confidence_detections".to_string(), low_confidence_count as f32);
        metrics.insert("spatial_clustering_score".to_string(), clustering_score);

        let (description, gap_score, remediation) = if clustering_score > 0.7 {
            (
                format!(
                    "False positives cluster in specific image regions (clustering: {:.2}). \
                     Likely caused by lighting artifacts or image borders.",
                    clustering_score
                ),
                0.82,
                "1. Check for lens artifacts at image edges. \
                 2. Investigate bright spots (glare, reflections). \
                 3. Tune detection ROI to exclude problematic regions. \
                 4. Add spatial masking in post-processing."
                    .to_string(),
            )
        } else {
            (
                format!(
                    "False positives scattered throughout image ({:.1}% rate). \
                     Likely model confidence calibration issue.",
                    fp_rate * 100.0
                ),
                0.65,
                "1. Re-calibrate model confidence thresholds. \
                 2. Apply temperature scaling to confidence scores. \
                 3. Retrain with false positive examples. \
                 4. Implement NMS (non-maximum suppression) tuning."
                    .to_string(),
            )
        };

        Some(RealityGapFinding {
            domain: RealityDomain::Sensor,
            category: "Detection Robustness".to_string(),
            finding_type: "False Positive Rate Elevated".to_string(),
            severity: Severity::Medium,
            confidence: 0.72,
            reality_gap_score: gap_score,
            description,
            evidence: vec![
                Evidence {
                    signal: "false_positive_rate".to_string(),
                    value: fp_rate,
                    timestamp: detection_results.last().map(|d| d.timestamp).unwrap_or(0.0),
                    confidence: 0.85,
                },
                Evidence {
                    signal: "spatial_clustering".to_string(),
                    value: clustering_score,
                    timestamp: detection_results.last().map(|d| d.timestamp).unwrap_or(0.0),
                    confidence: 0.70,
                },
            ],
            metrics,
            sim_recreation_suggestion:
                if clustering_score > 0.7 {
                    "Add lens artifacts to simulation: vignetting, flare, edge distortion."
                } else {
                    "Train detection model with out-of-distribution examples."
                }
                .to_string(),
            remediation,
            detection_time_sec: detection_results.last().map(|d| d.timestamp),
        })
    }

    fn compute_frame_confidences(
        &self,
        detections: &[crate::analyzers::DetectionResult],
    ) -> Vec<(f32, f32)> {
        let mut frame_conf: HashMap<usize, Vec<f32>> = HashMap::new();

        for detection in detections {
            frame_conf
                .entry(detection.frame_index)
                .or_insert_with(Vec::new)
                .push(detection.confidence);
        }

        let mut result: Vec<(f32, f32)> = frame_conf
            .into_iter()
            .map(|(frame_idx, confs)| {
                let avg_conf = confs.iter().sum::<f32>() / confs.len() as f32;
                (frame_idx as f32, avg_conf)
            })
            .collect();

        result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    fn compute_confidence_trend(&self, data: &[(f32, f32)]) -> (f32, f32, f32) {
        if data.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sample_size = (data.len() / 10).max(1);
        let initial: f32 = data
            .iter()
            .take(sample_size)
            .map(|(_, conf)| conf)
            .sum::<f32>()
            / sample_size as f32;

        let final_val: f32 = data
            .iter()
            .rev()
            .take(sample_size)
            .map(|(_, conf)| conf)
            .sum::<f32>()
            / sample_size as f32;

        let duration = data.last().map(|(f, _)| *f).unwrap_or(0.0)
            - data.first().map(|(f, _)| *f).unwrap_or(0.0);

        let slope = if duration > 0.0 {
            (final_val - initial) / duration * 3600.0 // Per hour
        } else {
            0.0
        };

        (initial, final_val, slope)
    }

    fn extract_image_quality_trend(
        &self,
        frames: &[crate::analyzers::CameraFrame],
    ) -> Vec<(f32, f32)> {
        frames
            .iter()
            .filter_map(|f| {
                f.quality_sharpness.map(|s| (f.frame_index as f32, s))
            })
            .collect()
    }

    fn compute_correlation(&self, conf: &[(f32, f32)], quality: &[(f32, f32)]) -> f32 {
        if conf.is_empty() || quality.is_empty() {
            return 0.0;
        }

        // Align by frame index
        let mut conf_map: HashMap<u32, f32> = HashMap::new();
        for (frame, c) in conf {
            conf_map.insert(*frame as u32, *c);
        }

        let mut pairs = Vec::new();
        for (frame, q) in quality {
            if let Some(c) = conf_map.get(&(*frame as u32)) {
                pairs.push((*c, *q));
            }
        }

        if pairs.len() < 5 {
            return 0.0;
        }

        // Pearson correlation
        let n = pairs.len() as f32;
        let mean_conf = pairs.iter().map(|(c, _)| *c).sum::<f32>() / n;
        let mean_qual = pairs.iter().map(|(_, q)| *q).sum::<f32>() / n;

        let mut numerator = 0.0;
        let mut denom_conf = 0.0;
        let mut denom_qual = 0.0;

        for (c, q) in pairs {
            numerator += (c - mean_conf) * (q - mean_qual);
            denom_conf += (c - mean_conf).powi(2);
            denom_qual += (q - mean_qual).powi(2);
        }

        let denom = (denom_conf * denom_qual).sqrt();
        if denom > 0.0 {
            (numerator / denom).abs()
        } else {
            0.0
        }
    }

    fn compute_spatial_clustering(&self, locations: &[(f32, f32)]) -> f32 {
        if locations.len() < 3 {
            return 0.0;
        }

        // Simple clustering: measure how concentrated FPs are
        // High clustering = most FPs in small area (likely artifact)
        // Low clustering = spread throughout (likely model issue)

        let center_x = locations.iter().map(|(x, _)| x).sum::<f32>() / locations.len() as f32;
        let center_y = locations.iter().map(|(_, y)| y).sum::<f32>() / locations.len() as f32;

        let avg_distance = locations
            .iter()
            .map(|(x, y)| ((x - center_x).powi(2) + (y - center_y).powi(2)).sqrt())
            .sum::<f32>()
            / locations.len() as f32;

        // Normalize to 0-1 based on typical image size (assume ~600x800)
        let max_distance = (600_f32.powi(2) + 800_f32.powi(2)).sqrt();
        (1.0 - (avg_distance / max_distance)).max(0.0)
    }
}

impl GapDetector for DetectionRobustnessAnalyzer {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();

        if let Some(finding) = self.analyze_detection_confidence(
            &mission_data.detection_results,
            &mission_data.camera_frames,
        ) {
            findings.push(finding);
        }

        if let Some(finding) = self.analyze_false_positives(&mission_data.detection_results) {
            findings.push(finding);
        }

        findings
    }

    fn domain(&self) -> RealityDomain {
        RealityDomain::Sensor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let _detector = DetectionRobustnessAnalyzer::new();
    }

    #[test]
    fn test_frame_confidences_empty() {
        let detector = DetectionRobustnessAnalyzer::new();
        let result = detector.compute_frame_confidences(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_confidence_trend_empty() {
        let detector = DetectionRobustnessAnalyzer::new();
        let (init, final_val, slope) = detector.compute_confidence_trend(&[]);
        assert_eq!(init, 0.0);
        assert_eq!(final_val, 0.0);
        assert_eq!(slope, 0.0);
    }

    #[test]
    fn test_spatial_clustering() {
        let detector = DetectionRobustnessAnalyzer::new();
        let tight_cluster = vec![(100.0, 100.0), (101.0, 101.0), (102.0, 102.0)];
        let tight_score = detector.compute_spatial_clustering(&tight_cluster);
        assert!(tight_score > 0.8); // High clustering

        let spread = vec![(0.0, 0.0), (300.0, 400.0), (600.0, 800.0)];
        let spread_score = detector.compute_spatial_clustering(&spread);
        assert!(spread_score < tight_score); // Lower clustering
    }
}
