//! Thermal Effects Detector
//!
//! Detects temperature-related performance degradation.

use crate::analyzers::{
    GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding, Severity, Evidence,
};
use std::collections::HashMap;

pub struct ThermalEffectsDetector;

impl ThermalEffectsDetector {
    pub fn new() -> Self {
        ThermalEffectsDetector
    }

    /// Analyze motor efficiency degradation with temperature
    pub fn analyze_motor_efficiency(
        &self,
        motor_currents: &[crate::analyzers::MotorCurrent],
        joint_states: &[crate::analyzers::JointState],
        thermal_readings: &[crate::analyzers::ThermalReading],
    ) -> Option<RealityGapFinding> {
        if motor_currents.is_empty()
            || joint_states.is_empty()
            || thermal_readings.is_empty()
        {
            return None;
        }

        // Compute initial and final efficiency
        let initial_efficiency = self.compute_efficiency(&motor_currents[0..10.min(motor_currents.len())],
            &joint_states);
        let final_efficiency = self.compute_efficiency(
            &motor_currents[motor_currents.len().saturating_sub(10)..],
            &joint_states,
        );

        let efficiency_decline = (initial_efficiency - final_efficiency) / initial_efficiency.max(0.01);

        // Find peak temperature
        let peak_temp = thermal_readings
            .iter()
            .map(|t| t.temperature_c)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(25.0);

        // Threshold: 5% efficiency decline
        if efficiency_decline > 0.05 && peak_temp > 50.0 {
            let mut metrics = HashMap::new();
            metrics.insert("efficiency_decline_pct".to_string(), efficiency_decline * 100.0);
            metrics.insert("peak_temperature_c".to_string(), peak_temp);
            metrics.insert("initial_efficiency".to_string(), initial_efficiency);
            metrics.insert("final_efficiency".to_string(), final_efficiency);

            return Some(RealityGapFinding {
                domain: RealityDomain::Physical,
                category: "Thermal Effects".to_string(),
                finding_type: "Motor Efficiency Degradation".to_string(),
                severity: Severity::Medium,
                confidence: 0.78,
                reality_gap_score: 0.70,
                description: format!(
                    "Motor efficiency declined {:.1}% as temperature rose to {:.1}°C. \
                     Likely cause: thermal throttling or increased electrical resistance.",
                    efficiency_decline * 100.0,
                    peak_temp
                ),
                evidence: vec![
                    Evidence {
                        signal: "motor_efficiency".to_string(),
                        value: final_efficiency,
                        timestamp: motor_currents.last().map(|m| m.timestamp).unwrap_or(0.0),
                        confidence: 0.80,
                    },
                    Evidence {
                        signal: "temperature_c".to_string(),
                        value: peak_temp,
                        timestamp: thermal_readings.last().map(|t| t.timestamp).unwrap_or(0.0),
                        confidence: 0.90,
                    },
                ],
                metrics,
                sim_recreation_suggestion:
                    "Model temperature-dependent efficiency: η(T) = η₀ * (1 - 0.005 * (T - 25)). \
                     Run Gazebo with ambient temperature increasing over mission."
                        .to_string(),
                remediation:
                    "1. Ensure adequate ventilation/cooling of motor drivers. \
                     2. Reduce continuous motor load during mission. \
                     3. Consider more efficient gearing or motor selection."
                        .to_string(),
                detection_time_sec: thermal_readings.last().map(|t| t.timestamp),
            });
        }

        None
    }

    fn compute_efficiency(
        &self,
        motor_currents: &[crate::analyzers::MotorCurrent],
        _joint_states: &[crate::analyzers::JointState],
    ) -> f32 {
        // Simple efficiency: low current = high efficiency
        if motor_currents.is_empty() {
            return 0.5;
        }

        let avg_current = motor_currents.iter().map(|m| m.current_amps).sum::<f32>()
            / motor_currents.len() as f32;

        // Normalize to 0-1 scale (lower current = higher efficiency)
        (1.0 - (avg_current / 50.0).min(1.0)).max(0.0)
    }
}

impl GapDetector for ThermalEffectsDetector {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();

        if let Some(finding) = self.analyze_motor_efficiency(
            &mission_data.motor_currents,
            &mission_data.joint_states,
            &mission_data.thermal_readings,
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
        let _detector = ThermalEffectsDetector::new();
    }
}
