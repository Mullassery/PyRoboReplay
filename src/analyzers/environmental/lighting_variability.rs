//! Lighting Variability Detector
//!
//! Simulation typically renders scenes under constant, uniform lighting.
//! Real-world missions cross shadows, glare, indoor/outdoor transitions, and
//! time-of-day changes. Without raw pixel access, this detector uses frame
//! quality_entropy (image entropy — depressed by under/over-exposure, which
//! lighting swings cause) as a proxy signal: real missions with genuine
//! lighting variability show measurably higher entropy variance over time
//! than a constantly-lit scene would.

use crate::analyzers::{CameraFrame, Evidence, RealityDomain, RealityGapFinding, Severity};
use std::collections::HashMap;

const MIN_FRAMES: usize = 20;
/// Std-dev of entropy above which we call it "variable lighting" rather than
/// normal frame-to-frame noise. Entropy is roughly on a 0-8 bits/pixel scale
/// for 8-bit imagery; a stddev above ~0.5 indicates real swings, not noise.
const ENTROPY_STDDEV_THRESHOLD: f32 = 0.5;

pub struct LightingVariabilityDetector;

impl LightingVariabilityDetector {
    pub fn new() -> Self {
        LightingVariabilityDetector
    }

    pub fn analyze(&self, camera_frames: &[CameraFrame]) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();

        // Group by camera — different cameras (e.g. front vs. downward-facing)
        // can have very different lighting exposure to the environment.
        let mut by_camera: HashMap<String, Vec<&CameraFrame>> = HashMap::new();
        for frame in camera_frames {
            by_camera.entry(frame.camera_id.clone()).or_default().push(frame);
        }

        for (camera_id, frames) in by_camera {
            let entropies: Vec<f32> = frames.iter().filter_map(|f| f.quality_entropy).collect();
            if entropies.len() < MIN_FRAMES {
                continue;
            }

            let mean = entropies.iter().sum::<f32>() / entropies.len() as f32;
            let variance =
                entropies.iter().map(|e| (e - mean).powi(2)).sum::<f32>() / entropies.len() as f32;
            let stddev = variance.sqrt();

            if stddev > ENTROPY_STDDEV_THRESHOLD {
                let min = entropies.iter().cloned().fold(f32::INFINITY, f32::min);
                let max = entropies.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

                let severity = if stddev > ENTROPY_STDDEV_THRESHOLD * 3.0 {
                    Severity::High
                } else if stddev > ENTROPY_STDDEV_THRESHOLD * 1.5 {
                    Severity::Medium
                } else {
                    Severity::Low
                };

                let mut metrics = HashMap::new();
                metrics.insert(format!("{camera_id}_entropy_mean"), mean);
                metrics.insert(format!("{camera_id}_entropy_stddev"), stddev);
                metrics.insert(format!("{camera_id}_entropy_range"), max - min);

                // Pick the frame furthest from the mean as representative evidence.
                let evidence_frame = frames
                    .iter()
                    .zip(frames.iter().filter_map(|f| f.quality_entropy).map(Some))
                    .filter_map(|(f, e)| e.map(|e| (f, e)))
                    .max_by(|(_, a), (_, b)| (a - mean).abs().partial_cmp(&(b - mean).abs()).unwrap())
                    .map(|(f, e)| Evidence {
                        signal: format!("{camera_id}_quality_entropy"),
                        value: e,
                        timestamp: f.timestamp,
                        confidence: 0.6, // proxy signal, not a direct lux measurement — moderate confidence
                    });

                findings.push(RealityGapFinding {
                    domain: RealityDomain::Environmental,
                    category: "Dynamic Lighting".to_string(),
                    finding_type: format!("{camera_id} Exposure Variability"),
                    severity,
                    confidence: 0.6,
                    reality_gap_score: 0.75,
                    description: format!(
                        "Camera '{camera_id}' frame entropy varied by {stddev:.2} (std dev) across \
                         the mission (range {min:.2}-{max:.2}), consistent with real lighting changes \
                         (shadows, glare, indoor/outdoor transitions) that a constantly-lit simulation \
                         wouldn't reproduce. This is inferred from frame-quality entropy, not a direct \
                         lux measurement, so treat as a signal to investigate rather than a certainty."
                    ),
                    evidence: evidence_frame.into_iter().collect(),
                    metrics,
                    sim_recreation_suggestion:
                        "Add dynamic lighting (time-of-day cycle, directional shadows, or randomized \
                         exposure) to the simulated environment instead of constant uniform lighting."
                            .to_string(),
                    remediation:
                        "If perception models were trained only on simulated (constant-lit) data, \
                         validate their robustness to real exposure swings; consider exposure-augmentation \
                         during training.".to_string(),
                    detection_time_sec: None,
                });
            }
        }

        findings
    }
}

impl Default for LightingVariabilityDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(camera_id: &str, timestamp: f32, entropy: f32) -> CameraFrame {
        CameraFrame {
            timestamp,
            camera_id: camera_id.to_string(),
            width: 1920,
            height: 1080,
            frame_index: (timestamp * 10.0) as usize,
            quality_sharpness: None,
            quality_entropy: Some(entropy),
        }
    }

    #[test]
    fn stable_entropy_produces_no_finding() {
        let frames: Vec<CameraFrame> =
            (0..30).map(|i| frame("front", i as f32 * 0.1, 6.0 + (i % 2) as f32 * 0.05)).collect();
        let detector = LightingVariabilityDetector::new();
        assert!(detector.analyze(&frames).is_empty());
    }

    #[test]
    fn swinging_entropy_produces_a_finding_with_correct_camera_id() {
        // Alternates between well-exposed (7.0) and poorly-exposed (3.0) —
        // a clear real-lighting-variation signature.
        let frames: Vec<CameraFrame> = (0..30)
            .map(|i| {
                let entropy = if i % 4 < 2 { 7.0 } else { 3.0 };
                frame("front", i as f32 * 0.1, entropy)
            })
            .collect();
        let detector = LightingVariabilityDetector::new();
        let findings = detector.analyze(&frames);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].domain, RealityDomain::Environmental);
        assert!(findings[0].finding_type.contains("front"));
    }

    #[test]
    fn too_few_frames_is_skipped_not_falsely_flagged() {
        let frames: Vec<CameraFrame> = (0..5).map(|i| frame("front", i as f32 * 0.1, if i % 2 == 0 { 8.0 } else { 1.0 })).collect();
        let detector = LightingVariabilityDetector::new();
        assert!(detector.analyze(&frames).is_empty());
    }

    #[test]
    fn cameras_are_analyzed_independently() {
        let mut frames: Vec<CameraFrame> =
            (0..30).map(|i| frame("stable_cam", i as f32 * 0.1, 6.0)).collect();
        frames.extend((0..30).map(|i| {
            let entropy = if i % 2 == 0 { 7.5 } else { 2.5 };
            frame("variable_cam", i as f32 * 0.1, entropy)
        }));
        let detector = LightingVariabilityDetector::new();
        let findings = detector.analyze(&frames);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].finding_type.contains("variable_cam"));
    }
}
