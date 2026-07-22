//! Clock Drift Detector
//!
//! Detects sensor clock synchronization issues and timing problems.

use crate::analyzers::{
    GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding, Severity, Evidence,
};
use std::collections::HashMap;

pub struct ClockDriftDetector;

impl ClockDriftDetector {
    pub fn new() -> Self {
        ClockDriftDetector
    }

    /// Analyze sensor timing for clock drift
    pub fn analyze_sensor_timing(
        &self,
        message_timestamps: &[crate::analyzers::MessageTimestamp],
    ) -> Option<RealityGapFinding> {
        if message_timestamps.len() < 20 {
            return None;
        }

        // Group by sensor
        let mut sensor_messages: HashMap<String, Vec<f32>> = HashMap::new();
        for msg in message_timestamps {
            sensor_messages
                .entry(msg.sensor_id.clone())
                .or_insert_with(Vec::new)
                .push(msg.timestamp);
        }

        // Analyze each sensor's timing
        let mut findings = None;
        let mut highest_drift = 0.0;

        for (sensor_id, timestamps) in sensor_messages {
            if timestamps.len() < 20 {
                continue;
            }

            // Compute inter-message intervals
            let mut intervals = Vec::new();
            for i in 1..timestamps.len() {
                let interval = timestamps[i] - timestamps[i - 1];
                if interval > 0.0 && interval < 1.0 {
                    intervals.push(interval);
                }
            }

            if intervals.len() < 10 {
                continue;
            }

            // Compute drift: slope of interval vs message index
            let (slope, _r_squared) = self.linear_regression(&intervals);

            // Expected interval: mean interval
            let expected_interval = intervals.iter().sum::<f32>() / intervals.len() as f32;

            // Drift in ppm: (actual_rate - expected_rate) / expected_rate * 1_000_000
            let drift_ppm = if expected_interval > 0.0 {
                (slope - expected_interval) / expected_interval * 1_000_000.0
            } else {
                0.0
            };

            // Threshold: 500+ ppm (0.05%) is significant
            if drift_ppm.abs() > 500.0 {
                let hours_to_1s_skew = 3600.0 / (drift_ppm.abs() / 1_000_000.0 * expected_interval);

                if drift_ppm.abs() > highest_drift {
                    highest_drift = drift_ppm.abs();

                    let mut metrics = HashMap::new();
                    metrics.insert(format!("{}_drift_ppm", sensor_id), drift_ppm);
                    metrics.insert(format!("{}_expected_interval_ms", sensor_id), expected_interval * 1000.0);
                    metrics.insert(format!("{}_hours_to_1s_skew", sensor_id), hours_to_1s_skew);

                    findings = Some(RealityGapFinding {
                        domain: RealityDomain::System,
                        category: "Temporal Synchronization".to_string(),
                        finding_type: format!("{} Clock Drift", sensor_id),
                        severity: Severity::High,
                        confidence: 0.90,
                        reality_gap_score: 0.92,
                        description: format!(
                            "Sensor {} clock running {:.0} ppm {}. \
                             After {} hours, will be ~1.0s out of sync.",
                            sensor_id,
                            drift_ppm.abs(),
                            if drift_ppm > 0.0 { "fast" } else { "slow" },
                            hours_to_1s_skew
                        ),
                        evidence: vec![Evidence {
                            signal: "message_interval_drift".to_string(),
                            value: drift_ppm,
                            timestamp: timestamps.last().copied().unwrap_or(0.0),
                            confidence: 0.92,
                        }],
                        metrics,
                        sim_recreation_suggestion: format!(
                            "Per-sensor clock model: time_{}(t) = t * (1 + {:.6})",
                            sensor_id,
                            drift_ppm / 1_000_000.0
                        ),
                        remediation:
                            "1. Verify NTP (Network Time Protocol) is synchronized and working. \
                             2. Check sensor hardware real-time clock (RTC) accuracy. \
                             3. Increase time synchronization frequency (e.g., NTP update rate). \
                             4. Consider using GPS-disciplined clock if available."
                                .to_string(),
                        detection_time_sec: timestamps.last().copied(),
                    });
                }
            }
        }

        findings
    }

    /// Detect timestamp reversals (time going backwards)
    pub fn analyze_timestamp_reversals(
        &self,
        message_timestamps: &[crate::analyzers::MessageTimestamp],
    ) -> Option<RealityGapFinding> {
        if message_timestamps.len() < 2 {
            return None;
        }

        let reversals: Vec<usize> = message_timestamps
            .windows(2)
            .enumerate()
            .filter_map(|(i, w)| {
                if w[1].timestamp < w[0].timestamp {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        if !reversals.is_empty() {
            let mut metrics = HashMap::new();
            metrics.insert("reversal_count".to_string(), reversals.len() as f32);

            return Some(RealityGapFinding {
                domain: RealityDomain::System,
                category: "Temporal Synchronization".to_string(),
                finding_type: "Timestamp Reversal".to_string(),
                severity: Severity::Critical,
                confidence: 0.99,
                reality_gap_score: 0.85,
                description: format!(
                    "Timestamp reversals detected ({} occurrences). \
                     Indicates ROS bag corruption, clock reset, or time jump event.",
                    reversals.len()
                ),
                evidence: vec![Evidence {
                    signal: "timestamp_reversals".to_string(),
                    value: reversals.len() as f32,
                    timestamp: message_timestamps.last().map(|m| m.timestamp).unwrap_or(0.0),
                    confidence: 0.99,
                }],
                metrics,
                sim_recreation_suggestion:
                    "Inject timestamp reversals in replay: randomly skip backward in time at specific indices."
                        .to_string(),
                remediation:
                    "1. Regenerate ROS bag from source (may be corrupted). \
                     2. Check for clock resets during bag recording. \
                     3. Use bag filter/repair tools to fix timestamps. \
                     4. Review system logs during mission for NTP issues."
                        .to_string(),
                detection_time_sec: message_timestamps.get(reversals[0]).map(|m| m.timestamp),
            });
        }

        None
    }

    fn linear_regression(&self, data: &[f32]) -> (f32, f32) {
        if data.is_empty() {
            return (0.0, 0.0);
        }

        let n = data.len() as f32;
        let x_mean = (data.len() as f32) / 2.0; // Index: 0, 1, 2, ..., n-1
        let y_mean = data.iter().sum::<f32>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, y) in data.iter().enumerate() {
            let x = i as f32;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }

        let slope = if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        };

        // R-squared: measure of fit quality
        let mut ss_res = 0.0;
        let mut ss_tot = 0.0;
        for (i, y) in data.iter().enumerate() {
            let x = i as f32;
            let y_pred = slope * (x - x_mean) + y_mean;
            ss_res += (y - y_pred).powi(2);
            ss_tot += (y - y_mean).powi(2);
        }

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        (slope, r_squared)
    }
}

impl GapDetector for ClockDriftDetector {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();

        if let Some(finding) = self.analyze_sensor_timing(&mission_data.message_timestamps) {
            findings.push(finding);
        }

        if let Some(finding) = self.analyze_timestamp_reversals(&mission_data.message_timestamps) {
            findings.push(finding);
        }

        findings
    }

    fn domain(&self) -> RealityDomain {
        RealityDomain::System
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let _detector = ClockDriftDetector::new();
    }

    #[test]
    fn test_linear_regression() {
        let detector = ClockDriftDetector::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (slope, r_squared) = detector.linear_regression(&data);
        assert!(slope > 0.0); // Should be positive
        assert!(r_squared > 0.7); // Reasonable fit for linear data
    }
}
