//! Thermal Drift Detector
//!
//! Ambient/environmental temperature swings during a mission (as distinct
//! from motor/component heating under load, which is the Physical domain's
//! concern) are a weather/time-of-day signal simulation usually holds
//! constant. Flags locations whose readings drift significantly over the
//! mission timeline in a way that's hard to explain by load alone (i.e. a
//! sustained trend, not just noise).

use crate::analyzers::{Evidence, RealityDomain, RealityGapFinding, Severity, ThermalReading};
use std::collections::HashMap;

const MIN_READINGS: usize = 10;
/// A sustained ambient drift of >= 5°C over a mission is a real environmental
/// signal (weather, sun exposure, HVAC cycling) rather than measurement noise.
const DRIFT_THRESHOLD_C: f32 = 5.0;

pub struct ThermalDriftDetector;

impl ThermalDriftDetector {
    pub fn new() -> Self {
        ThermalDriftDetector
    }

    pub fn analyze(&self, thermal_readings: &[ThermalReading]) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();

        let mut by_location: HashMap<String, Vec<&ThermalReading>> = HashMap::new();
        for reading in thermal_readings {
            by_location.entry(reading.location.clone()).or_default().push(reading);
        }

        for (location, mut readings) in by_location {
            if readings.len() < MIN_READINGS {
                continue;
            }
            readings.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

            let (slope, _) = linear_regression(
                &readings.iter().map(|r| r.timestamp).collect::<Vec<_>>(),
                &readings.iter().map(|r| r.temperature_c).collect::<Vec<_>>(),
            );

            let duration = readings.last().unwrap().timestamp - readings.first().unwrap().timestamp;
            if duration <= 0.0 {
                continue;
            }
            let total_drift = slope * duration;

            if total_drift.abs() >= DRIFT_THRESHOLD_C {
                let severity = if total_drift.abs() >= DRIFT_THRESHOLD_C * 2.0 {
                    Severity::Medium
                } else {
                    Severity::Low
                };

                let mut metrics = HashMap::new();
                metrics.insert(format!("{location}_drift_c"), total_drift);
                metrics.insert(format!("{location}_drift_c_per_min"), slope * 60.0);

                findings.push(RealityGapFinding {
                    domain: RealityDomain::Environmental,
                    category: "Ambient Temperature Drift".to_string(),
                    finding_type: format!("{location} Thermal Drift"),
                    severity,
                    confidence: 0.65,
                    reality_gap_score: 0.6,
                    description: format!(
                        "Ambient temperature at '{location}' drifted {total_drift:+.1}°C over the \
                         {duration:.0}s mission ({:.2}°C/min sustained trend) — consistent with real \
                         weather, sun exposure, or HVAC cycling that a fixed-temperature simulation \
                         wouldn't reproduce.",
                        slope * 60.0
                    ),
                    evidence: vec![Evidence {
                        signal: format!("{location}_temperature_c"),
                        value: total_drift,
                        timestamp: readings.last().unwrap().timestamp,
                        confidence: 0.65,
                    }],
                    metrics,
                    sim_recreation_suggestion:
                        "Model ambient temperature as a slowly-varying signal (e.g. a mission-length \
                         ramp or diurnal curve) instead of a fixed constant.".to_string(),
                    remediation:
                        "If thermal-sensitive behavior (battery derating, sensor calibration) was \
                         tuned only against constant-temperature sim data, validate against the \
                         observed real drift range.".to_string(),
                    detection_time_sec: None,
                });
            }
        }

        findings
    }
}

impl Default for ThermalDriftDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple ordinary-least-squares slope/intercept.
fn linear_regression(x: &[f32], y: &[f32]) -> (f32, f32) {
    let n = x.len() as f32;
    let mean_x = x.iter().sum::<f32>() / n;
    let mean_y = y.iter().sum::<f32>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..x.len() {
        num += (x[i] - mean_x) * (y[i] - mean_y);
        den += (x[i] - mean_x).powi(2);
    }
    let slope = if den != 0.0 { num / den } else { 0.0 };
    let intercept = mean_y - slope * mean_x;
    (slope, intercept)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(location: &str, timestamp: f32, temp: f32) -> ThermalReading {
        ThermalReading { timestamp, location: location.to_string(), temperature_c: temp }
    }

    #[test]
    fn stable_temperature_produces_no_finding() {
        let readings: Vec<ThermalReading> =
            (0..20).map(|i| reading("ambient", i as f32 * 10.0, 22.0 + (i % 2) as f32 * 0.1)).collect();
        let detector = ThermalDriftDetector::new();
        assert!(detector.analyze(&readings).is_empty());
    }

    #[test]
    fn sustained_drift_produces_a_finding() {
        // 20 readings over 200s, ramping from 15C to 25C — a clear 10C drift.
        let readings: Vec<ThermalReading> =
            (0..20).map(|i| reading("ambient", i as f32 * 10.0, 15.0 + i as f32 * 0.5)).collect();
        let detector = ThermalDriftDetector::new();
        let findings = detector.analyze(&readings);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].finding_type.contains("ambient"));
        assert!(findings[0].metrics[&"ambient_drift_c".to_string()] > 9.0);
    }

    #[test]
    fn linear_regression_recovers_known_slope() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![1.0, 3.0, 5.0, 7.0, 9.0]; // y = 2x + 1
        let (slope, intercept) = linear_regression(&x, &y);
        assert!((slope - 2.0).abs() < 1e-4);
        assert!((intercept - 1.0).abs() < 1e-4);
    }

    #[test]
    fn locations_are_analyzed_independently() {
        let mut readings: Vec<ThermalReading> =
            (0..20).map(|i| reading("stable_loc", i as f32 * 10.0, 20.0)).collect();
        readings.extend((0..20).map(|i| reading("drifting_loc", i as f32 * 10.0, 10.0 + i as f32 * 0.4)));
        let detector = ThermalDriftDetector::new();
        let findings = detector.analyze(&readings);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].finding_type.contains("drifting_loc"));
    }
}
