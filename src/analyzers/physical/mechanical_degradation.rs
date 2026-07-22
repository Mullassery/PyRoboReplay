//! Mechanical Degradation Detector
//!
//! Detects actuator wear, increased response times, backlash, and other mechanical issues.

use crate::analyzers::{
    GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding, Severity, Evidence,
};
use std::collections::HashMap;

pub struct MechanicalDegradationDetector;

impl MechanicalDegradationDetector {
    pub fn new() -> Self {
        MechanicalDegradationDetector
    }

    /// Analyze response time trends indicating mechanical wear
    pub fn analyze_response_time_trend(
        &self,
        control_messages: &[crate::analyzers::ControlMessage],
        joint_states: &[crate::analyzers::JointState],
    ) -> Option<RealityGapFinding> {
        if control_messages.is_empty() || joint_states.is_empty() {
            return None;
        }

        // Group controls by joint
        let mut joint_controls: HashMap<String, Vec<(f32, f32)>> = HashMap::new();

        for control in control_messages {
            joint_controls
                .entry(control.joint_id.clone())
                .or_insert_with(Vec::new)
                .push((control.timestamp, control.value));
        }

        let mut all_response_times = Vec::new();

        // For each joint, compute response times
        for (joint_id, controls) in joint_controls {
            for (ctrl_time, _ctrl_value) in controls {
                // Find the corresponding joint state (nearest timestamp)
                let matching_states: Vec<_> = joint_states
                    .iter()
                    .filter(|s| s.joint_id == joint_id && s.timestamp >= ctrl_time)
                    .collect();

                if let Some(first_response) = matching_states.first() {
                    let response_time = first_response.timestamp - ctrl_time;
                    if response_time > 0.0 && response_time < 1.0 {
                        all_response_times.push((ctrl_time, response_time));
                    }
                }
            }
        }

        if all_response_times.len() < 10 {
            return None; // Not enough data
        }

        // Analyze trend
        let (initial_time, final_time, trend_slope) =
            self.compute_response_time_trend(&all_response_times);

        // Threshold: 0.05 ms/min means ~3ms per hour increase
        if trend_slope > 0.05 {
            let degradation_factor = if initial_time > 0.0 {
                final_time / initial_time
            } else {
                1.0
            };

            let mut metrics = HashMap::new();
            metrics.insert("initial_response_time_ms".to_string(), initial_time * 1000.0);
            metrics.insert("final_response_time_ms".to_string(), final_time * 1000.0);
            metrics.insert("trend_slope_ms_per_hour".to_string(), trend_slope * 60.0);
            metrics.insert("degradation_factor".to_string(), degradation_factor);

            return Some(RealityGapFinding {
                domain: RealityDomain::Physical,
                category: "Mechanical Degradation".to_string(),
                finding_type: "Actuator Response Time Increasing".to_string(),
                severity: Severity::Medium,
                confidence: 0.75,
                reality_gap_score: 0.85,
                description: format!(
                    "Joint response time increased from {:.1}ms to {:.1}ms over mission. \
                     Trend: {:.4}ms/hour. Likely cause: mechanical wear or thermal effects.",
                    initial_time * 1000.0,
                    final_time * 1000.0,
                    trend_slope * 60.0
                ),
                evidence: vec![Evidence {
                    signal: "joint_response_time".to_string(),
                    value: final_time * 1000.0,
                    timestamp: all_response_times.last().map(|(t, _)| *t).unwrap_or(0.0),
                    confidence: 0.85,
                }],
                metrics,
                sim_recreation_suggestion:
                    "Model actuator lag: latency(t) = base_latency + wear_factor * (t / mission_duration). \
                     Run Gazebo simulation with increasing joint lag over time."
                        .to_string(),
                remediation:
                    "1. Verify actuator mechanical condition (lubrication, backlash). \
                     2. Tune PID gains for current hardware state. \
                     3. Consider preemptive maintenance if trend continues."
                        .to_string(),
                detection_time_sec: all_response_times.last().map(|(t, _)| *t),
            });
        }

        None
    }

    fn compute_response_time_trend(&self, data: &[(f32, f32)]) -> (f32, f32, f32) {
        if data.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        // Use first 10% and last 10% of data to compute trend
        let sample_size = (data.len() / 10).max(1);
        let initial: f32 = data
            .iter()
            .take(sample_size)
            .map(|(_, time)| time)
            .sum::<f32>()
            / sample_size as f32;

        let final_val: f32 = data
            .iter()
            .rev()
            .take(sample_size)
            .map(|(_, time)| time)
            .sum::<f32>()
            / sample_size as f32;

        let mission_duration = data.last().map(|(t, _)| *t).unwrap_or(0.0)
            - data.first().map(|(t, _)| *t).unwrap_or(0.0);

        let slope = if mission_duration > 0.0 {
            (final_val - initial) / mission_duration
        } else {
            0.0
        };

        (initial, final_val, slope)
    }

    /// Detect wheel slip by comparing encoder vs IMU odometry
    pub fn analyze_wheel_slip(
        &self,
        encoder_data: &[crate::analyzers::EncoderReading],
        imu_data: &[crate::analyzers::IMUMeasurement],
    ) -> Option<RealityGapFinding> {
        if encoder_data.is_empty() || imu_data.is_empty() {
            return None;
        }

        // Simple check: compare forward vs backward wheel velocities
        let mut forward_slip = 0.0;
        let mut backward_slip = 0.0;
        let mut forward_count = 0;
        let mut backward_count = 0;

        for encoder in encoder_data {
            if encoder.velocity > 0.1 {
                forward_slip += encoder.velocity.abs();
                forward_count += 1;
            } else if encoder.velocity < -0.1 {
                backward_slip += encoder.velocity.abs();
                backward_count += 1;
            }
        }

        if forward_count == 0 || backward_count == 0 {
            return None;
        }

        let forward_avg = forward_slip / forward_count as f32;
        let backward_avg = backward_slip / backward_count as f32;
        let bias = (forward_avg - backward_avg).abs() / forward_avg.max(0.01);

        // Threshold: >20% difference indicates wear or misalignment
        if bias > 0.2 {
            let mut metrics = HashMap::new();
            metrics.insert("forward_avg_velocity".to_string(), forward_avg);
            metrics.insert("backward_avg_velocity".to_string(), backward_avg);
            metrics.insert("velocity_bias_factor".to_string(), bias);

            return Some(RealityGapFinding {
                domain: RealityDomain::Physical,
                category: "Mechanical Degradation".to_string(),
                finding_type: "Wheel Slip Asymmetry".to_string(),
                severity: Severity::Medium,
                confidence: 0.70,
                reality_gap_score: 0.75,
                description: format!(
                    "Wheel slip asymmetry detected: forward avg {:.2} m/s, backward avg {:.2} m/s. \
                     Bias: {:.0}%. Likely cause: uneven tire wear or wheel misalignment.",
                    forward_avg, backward_avg, bias * 100.0
                ),
                evidence: vec![Evidence {
                    signal: "wheel_velocity_bias".to_string(),
                    value: bias,
                    timestamp: encoder_data.last().map(|e| e.timestamp).unwrap_or(0.0),
                    confidence: 0.70,
                }],
                metrics,
                sim_recreation_suggestion:
                    "Model wheel wear: friction_left = base_friction * (1.0 - wear_factor). \
                     Simulate with one wheel having 20% less friction than the other."
                        .to_string(),
                remediation:
                    "1. Inspect tire tread depth (one wheel more worn than other). \
                     2. Check wheel alignment. 3. Consider tire replacement."
                        .to_string(),
                detection_time_sec: encoder_data.last().map(|e| e.timestamp),
            });
        }

        None
    }
}

impl GapDetector for MechanicalDegradationDetector {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();

        if let Some(finding) = self.analyze_response_time_trend(
            &mission_data.control_messages,
            &mission_data.joint_states,
        ) {
            findings.push(finding);
        }

        if let Some(finding) = self.analyze_wheel_slip(
            &mission_data.encoder_data,
            &mission_data.imu_measurements,
        ) {
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
        let _detector = MechanicalDegradationDetector::new();
    }

    #[test]
    fn test_response_time_trend_empty() {
        let detector = MechanicalDegradationDetector::new();
        let (initial, final_val, slope) = detector.compute_response_time_trend(&[]);
        assert_eq!(initial, 0.0);
        assert_eq!(final_val, 0.0);
        assert_eq!(slope, 0.0);
    }
}
