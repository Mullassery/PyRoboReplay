//! Optical Contamination Detector
//!
//! Detects image quality degradation due to lens dirt, water droplets, condensation.

use crate::analyzers::{
    GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding, Severity, Evidence,
};
use std::collections::HashMap;

pub struct OpticalContaminationDetector;

impl OpticalContaminationDetector {
    pub fn new() -> Self {
        OpticalContaminationDetector
    }

    /// Analyze image quality degradation over mission time
    pub fn analyze_image_quality(
        &self,
        camera_frames: &[crate::analyzers::CameraFrame],
        detection_results: &[crate::analyzers::DetectionResult],
    ) -> Option<RealityGapFinding> {
        if camera_frames.len() < 10 {
            return None;
        }

        // Extract sharpness values (provided in CameraFrame if available)
        let sharpness_values: Vec<(f32, f32)> = camera_frames
            .iter()
            .filter_map(|f| f.quality_sharpness.map(|s| (f.timestamp, s)))
            .collect();

        if sharpness_values.len() < 10 {
            // No quality data available
            return None;
        }

        // Analyze sharpness trend
        let (initial_sharpness, final_sharpness, trend_slope) =
            self.compute_sharpness_trend(&sharpness_values);

        // Threshold: >15% decline in sharpness
        let sharpness_decline_pct = (initial_sharpness - final_sharpness) / initial_sharpness * 100.0;

        if sharpness_decline_pct > 15.0 {
            // Cross-reference with detection confidence
            let detection_confidence_decline =
                self.analyze_detection_confidence_trend(detection_results);

            let mut metrics = HashMap::new();
            metrics.insert("sharpness_decline_pct".to_string(), sharpness_decline_pct);
            metrics.insert("initial_sharpness".to_string(), initial_sharpness);
            metrics.insert("final_sharpness".to_string(), final_sharpness);
            metrics.insert(
                "trend_slope_pct_per_min".to_string(),
                trend_slope * 6000.0, // Convert from per-100ms to per-minute
            );

            if let Some(conf_decline) = detection_confidence_decline {
                metrics.insert("detection_confidence_decline_pct".to_string(), conf_decline);
            }

            let confidence = if detection_confidence_decline.is_some() {
                0.82 // Higher confidence if corroborated by detection data
            } else {
                0.75
            };

            return Some(RealityGapFinding {
                domain: RealityDomain::Sensor,
                category: "Optical Contamination".to_string(),
                finding_type: "Camera Lens Degradation".to_string(),
                severity: Severity::Medium,
                confidence,
                reality_gap_score: 0.78,
                description: format!(
                    "Camera image sharpness declined {:.1}% over mission. \
                     Likely cause: lens dirt, water droplets, or thermal focus shift.",
                    sharpness_decline_pct
                ),
                evidence: vec![
                    Evidence {
                        signal: "image_sharpness".to_string(),
                        value: final_sharpness,
                        timestamp: sharpness_values.last().map(|(t, _)| *t).unwrap_or(0.0),
                        confidence: 0.80,
                    },
                    Evidence {
                        signal: "sharpness_trend".to_string(),
                        value: trend_slope,
                        timestamp: sharpness_values.last().map(|(t, _)| *t).unwrap_or(0.0),
                        confidence: 0.85,
                    },
                ],
                metrics,
                sim_recreation_suggestion:
                    "Add procedural lens blur: output_image = GaussianBlur(input, kernel_size=f(time)). \
                     Model water droplets as localized Gaussian blurs with 10-20px radius."
                        .to_string(),
                remediation:
                    "1. Clean camera lens with appropriate lens cleaner. \
                     2. Check for water ingress (condensation on lens). \
                     3. Inspect lens for scratches or permanent damage. \
                     4. Consider protective lens cover or rain shield."
                        .to_string(),
                detection_time_sec: sharpness_values.last().map(|(t, _)| *t),
            });
        }

        None
    }

    fn compute_sharpness_trend(&self, data: &[(f32, f32)]) -> (f32, f32, f32) {
        if data.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        // Use first 10% and last 10% of data to compute trend
        let sample_size = (data.len() / 10).max(1);
        let initial: f32 = data
            .iter()
            .take(sample_size)
            .map(|(_, sharp)| sharp)
            .sum::<f32>()
            / sample_size as f32;

        let final_val: f32 = data
            .iter()
            .rev()
            .take(sample_size)
            .map(|(_, sharp)| sharp)
            .sum::<f32>()
            / sample_size as f32;

        let duration = data.last().map(|(t, _)| *t).unwrap_or(0.0)
            - data.first().map(|(t, _)| *t).unwrap_or(0.0);

        let slope = if duration > 0.0 {
            (final_val - initial) / duration / 100.0 // Per 100ms
        } else {
            0.0
        };

        (initial, final_val, slope)
    }

    /// Analyze if detection confidence declined alongside image quality
    fn analyze_detection_confidence_trend(
        &self,
        detections: &[crate::analyzers::DetectionResult],
    ) -> Option<f32> {
        if detections.len() < 10 {
            return None;
        }

        // Group detections by frame (timestamp)
        let mut frame_confidence: Vec<(f32, f32)> = Vec::new();
        let mut current_frame_confidences: Vec<f32> = Vec::new();
        let mut last_timestamp = detections[0].timestamp;

        for detection in detections {
            if (detection.timestamp - last_timestamp).abs() < 0.01 {
                current_frame_confidences.push(detection.confidence);
            } else {
                if !current_frame_confidences.is_empty() {
                    let avg_conf = current_frame_confidences.iter().sum::<f32>()
                        / current_frame_confidences.len() as f32;
                    frame_confidence.push((last_timestamp, avg_conf));
                }
                current_frame_confidences.clear();
                current_frame_confidences.push(detection.confidence);
                last_timestamp = detection.timestamp;
            }
        }

        if frame_confidence.len() < 10 {
            return None;
        }

        // Compute trend
        let sample_size = (frame_confidence.len() / 10).max(1);
        let initial: f32 = frame_confidence
            .iter()
            .take(sample_size)
            .map(|(_, conf)| conf)
            .sum::<f32>()
            / sample_size as f32;

        let final_val: f32 = frame_confidence
            .iter()
            .rev()
            .take(sample_size)
            .map(|(_, conf)| conf)
            .sum::<f32>()
            / sample_size as f32;

        let decline_pct = (initial - final_val) / initial * 100.0;

        if decline_pct > 5.0 {
            Some(decline_pct)
        } else {
            None
        }
    }
}

impl GapDetector for OpticalContaminationDetector {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();

        if let Some(finding) = self.analyze_image_quality(
            &mission_data.camera_frames,
            &mission_data.detection_results,
        ) {
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
        let _detector = OpticalContaminationDetector::new();
    }

    #[test]
    fn test_sharpness_trend_empty() {
        let detector = OpticalContaminationDetector::new();
        let (initial, final_val, slope) = detector.compute_sharpness_trend(&[]);
        assert_eq!(initial, 0.0);
        assert_eq!(final_val, 0.0);
        assert_eq!(slope, 0.0);
    }
}
