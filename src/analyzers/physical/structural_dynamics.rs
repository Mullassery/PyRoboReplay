//! Structural Dynamics Detector
//!
//! Detects vibration, oscillation, and resonance effects.

use crate::analyzers::{
    GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding, Severity, Evidence,
};
use std::collections::HashMap;

pub struct StructuralDynamicsDetector;

impl StructuralDynamicsDetector {
    pub fn new() -> Self {
        StructuralDynamicsDetector
    }

    /// Detect post-motion oscillation indicating structural flex
    pub fn analyze_oscillation(&self, joint_states: &[crate::analyzers::JointState]) -> Option<RealityGapFinding> {
        if joint_states.len() < 20 {
            return None;
        }

        // Find periods where velocity crosses zero (motion stopped)
        // Then check if position continues to oscillate

        let mut stop_indices = Vec::new();
        for i in 1..joint_states.len() {
            let prev_vel = joint_states[i - 1].velocity;
            let curr_vel = joint_states[i].velocity;

            // Zero crossing: velocity changes sign
            if (prev_vel > 0.01 && curr_vel <= 0.01) || (prev_vel < -0.01 && curr_vel >= -0.01) {
                stop_indices.push(i);
            }
        }

        if stop_indices.len() < 3 {
            return None;
        }

        // Analyze oscillation after each stop
        let mut oscillation_detected = false;
        let mut max_oscillation_amplitude: f32 = 0.0;

        for &stop_idx in stop_indices.iter().take(3) {
            if stop_idx + 20 < joint_states.len() {
                // Look at 20 samples after motion stopped
                let post_motion = &joint_states[stop_idx..stop_idx + 20];

                // Compute position variance (oscillation indicator)
                let mean_pos = post_motion.iter().map(|s| s.position).sum::<f32>() / post_motion.len() as f32;
                let variance = post_motion
                    .iter()
                    .map(|s| (s.position - mean_pos).powi(2))
                    .sum::<f32>()
                    / post_motion.len() as f32;

                let std_dev = variance.sqrt();

                if std_dev > 0.01 {
                    oscillation_detected = true;
                    max_oscillation_amplitude = max_oscillation_amplitude.max(std_dev);
                }
            }
        }

        if oscillation_detected {
            let mut metrics = HashMap::new();
            metrics.insert("max_oscillation_amplitude_rad".to_string(), max_oscillation_amplitude);
            metrics.insert("oscillation_detections".to_string(), oscillation_detected as i32 as f32);

            return Some(RealityGapFinding {
                domain: RealityDomain::Physical,
                category: "Structural Dynamics".to_string(),
                finding_type: "Post-Motion Oscillation".to_string(),
                severity: Severity::Low,
                confidence: 0.65,
                reality_gap_score: 0.80,
                description: format!(
                    "Joint oscillates after commanded stop with amplitude {:.3} rad. \
                     Indicates structural compliance or damping issues.",
                    max_oscillation_amplitude
                ),
                evidence: vec![Evidence {
                    signal: "post_motion_oscillation".to_string(),
                    value: max_oscillation_amplitude,
                    timestamp: joint_states.last().map(|s| s.timestamp).unwrap_or(0.0),
                    confidence: 0.70,
                }],
                metrics,
                sim_recreation_suggestion:
                    "Add joint compliance and damping: τ + 2ζω₀τ̇ + ω₀²τ = command. \
                     Use ζ ≈ 0.3-0.5, ω₀ ≈ 2π*5 Hz (typical structural mode)."
                        .to_string(),
                remediation:
                    "1. Increase joint controller damping (PD gains). \
                     2. Add series damper if available. \
                     3. Reduce motion speed commands to limit excitation."
                        .to_string(),
                detection_time_sec: joint_states.last().map(|s| s.timestamp),
            });
        }

        None
    }
}

impl GapDetector for StructuralDynamicsDetector {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();

        if let Some(finding) = self.analyze_oscillation(&mission_data.joint_states) {
            findings.push(finding);
        }

        findings
    }

    fn domain(&self) -> RealityDomain {
        RealityDomain::Physical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let _detector = StructuralDynamicsDetector::new();
    }
}
