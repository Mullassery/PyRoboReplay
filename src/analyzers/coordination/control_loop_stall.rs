//! Control Loop Stall Detector
//!
//! Detects gaps in the control message stream where the robot's own control
//! loop appears to have stalled (no commands issued for far longer than the
//! established cadence) mid-mission. In simulation, control loops rarely
//! stall — real deployments can, from scheduler contention, driver hangs, or
//! (in a multi-robot context, once that data is available) coordination
//! backpressure. This is a real, checkable signal from data that actually
//! exists in `MissionAnalysisData` today.

use crate::analyzers::{ControlMessage, Evidence, RealityDomain, RealityGapFinding, Severity};
use std::collections::HashMap;

const MIN_MESSAGES: usize = 20;
/// A gap this many times the median inter-command interval is treated as a
/// stall rather than normal jitter.
const STALL_MULTIPLIER: f32 = 10.0;

pub struct ControlLoopStallDetector;

impl ControlLoopStallDetector {
    pub fn new() -> Self {
        ControlLoopStallDetector
    }

    pub fn analyze(&self, control_messages: &[ControlMessage], mission_duration_sec: f32) -> Vec<RealityGapFinding> {
        if control_messages.len() < MIN_MESSAGES {
            return Vec::new();
        }

        let mut timestamps: Vec<f32> = control_messages.iter().map(|m| m.timestamp).collect();
        timestamps.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut intervals: Vec<f32> = timestamps.windows(2).map(|w| w[1] - w[0]).filter(|d| *d > 0.0).collect();
        if intervals.len() < MIN_MESSAGES - 1 {
            return Vec::new();
        }
        intervals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_interval = intervals[intervals.len() / 2];
        if median_interval <= 0.0 {
            return Vec::new();
        }

        let stall_threshold = median_interval * STALL_MULTIPLIER;
        let mut stalls: Vec<(f32, f32)> = Vec::new(); // (start_time, duration)
        for w in timestamps.windows(2) {
            let gap = w[1] - w[0];
            if gap >= stall_threshold {
                stalls.push((w[0], gap));
            }
        }

        if stalls.is_empty() {
            return Vec::new();
        }

        let longest = stalls.iter().cloned().fold((0.0, 0.0), |acc, s| if s.1 > acc.1 { s } else { acc });
        let total_stall_time: f32 = stalls.iter().map(|(_, d)| d).sum();

        let severity = if longest.1 > mission_duration_sec * 0.1 {
            Severity::High
        } else if stalls.len() > 3 {
            Severity::Medium
        } else {
            Severity::Low
        };

        let mut metrics = HashMap::new();
        metrics.insert("stall_count".to_string(), stalls.len() as f32);
        metrics.insert("longest_stall_sec".to_string(), longest.1);
        metrics.insert("total_stall_sec".to_string(), total_stall_time);
        metrics.insert("median_interval_sec".to_string(), median_interval);

        vec![RealityGapFinding {
            domain: RealityDomain::Coordination,
            category: "Control Loop Stability".to_string(),
            finding_type: "Control Loop Stall".to_string(),
            severity,
            confidence: 0.7,
            reality_gap_score: 0.55, // could be a real scheduling/driver issue rather than a pure sim gap
            description: format!(
                "Control command stream stalled {} time(s) during the mission; the longest gap was \
                 {:.2}s vs a typical inter-command interval of {:.3}s ({:.0}x). Simulated control loops \
                 rarely stall this way — likely causes include scheduler contention, driver hangs, or \
                 (in multi-robot deployments) coordination backpressure that this single-robot data \
                 can't distinguish further.",
                stalls.len(), longest.1, median_interval, longest.1 / median_interval
            ),
            evidence: vec![Evidence {
                signal: "control_message_gap".to_string(),
                value: longest.1,
                timestamp: longest.0,
                confidence: 0.7,
            }],
            metrics,
            sim_recreation_suggestion:
                "Inject occasional control-loop scheduling jitter/stalls into the simulated control \
                 stack rather than assuming a perfectly regular command cadence.".to_string(),
            remediation:
                "Correlate stall timestamps with system logs (CPU load, driver errors) to determine \
                 root cause; if this recurs across missions, consider control-loop watchdog/recovery \
                 logic.".to_string(),
            detection_time_sec: Some(longest.0),
        }]
    }
}

impl Default for ControlLoopStallDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(timestamp: f32) -> ControlMessage {
        ControlMessage { timestamp, joint_id: "j1".to_string(), command_type: "position".to_string(), value: 0.0 }
    }

    #[test]
    fn regular_cadence_produces_no_finding() {
        let messages: Vec<ControlMessage> = (0..50).map(|i| msg(i as f32 * 0.02)).collect();
        let detector = ControlLoopStallDetector::new();
        assert!(detector.analyze(&messages, 1.0).is_empty());
    }

    #[test]
    fn a_large_gap_is_detected_as_a_stall() {
        let mut messages: Vec<ControlMessage> = (0..30).map(|i| msg(i as f32 * 0.02)).collect();
        // Jump: last regular message at 0.58s, then a 2s gap before resuming.
        messages.extend((0..20).map(|i| msg(2.6 + i as f32 * 0.02)));

        let detector = ControlLoopStallDetector::new();
        let findings = detector.analyze(&messages, 3.0);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].domain, RealityDomain::Coordination);
        assert!(findings[0].metrics["longest_stall_sec"] > 1.5);
    }

    #[test]
    fn too_few_messages_is_skipped() {
        let messages: Vec<ControlMessage> = (0..5).map(|i| msg(i as f32)).collect();
        let detector = ControlLoopStallDetector::new();
        assert!(detector.analyze(&messages, 10.0).is_empty());
    }

    #[test]
    fn severity_scales_with_stall_duration_relative_to_mission() {
        let mut messages: Vec<ControlMessage> = (0..30).map(|i| msg(i as f32 * 0.02)).collect();
        // Gap of ~1.4s (2.0 - 0.58) is >10% of a 5s mission -> High severity.
        messages.extend((0..20).map(|i| msg(2.0 + i as f32 * 0.02)));
        let detector = ControlLoopStallDetector::new();
        let findings = detector.analyze(&messages, 5.0);
        assert_eq!(findings[0].severity, Severity::High);
    }
}
