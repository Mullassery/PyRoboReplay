//! Severity Classification for Reality Gap Findings
//!
//! Multi-factor decision tree for CRITICAL/HIGH/MEDIUM/LOW classification.

use crate::analyzers::{RealityGapFinding, Severity, MissionAnalysisData};
use std::collections::HashMap;

/// Classify gap finding by severity
pub struct SeverityClassifier;

impl SeverityClassifier {
    /// Classify severity based on multiple factors
    pub fn classify(
        finding: &RealityGapFinding,
        mission: &MissionAnalysisData,
    ) -> Severity {
        // Factor 1: Direct performance impact
        let performance_impact = Self::compute_performance_impact(finding);

        // Factor 2: Mission criticality
        let mission_criticality = Self::mission_criticality_score(&mission.robot_type);

        // Factor 3: Safety implications
        let safety_risk = Self::assess_safety_risk(finding);

        // Factor 4: Frequency (is this recurring?)
        let recurrence_factor = Self::estimate_recurrence(finding);

        // Decision tree
        if safety_risk > 0.7 || performance_impact > 0.8 {
            return Severity::Critical;
        }

        if performance_impact > 0.6 || (mission_criticality > 0.8 && performance_impact > 0.3) {
            return Severity::High;
        }

        if performance_impact > 0.3 || recurrence_factor > 0.6 {
            return Severity::Medium;
        }

        Severity::Low
    }

    /// Compute performance impact (0.0-1.0)
    /// Maps finding metrics to degradation severity
    fn compute_performance_impact(finding: &RealityGapFinding) -> f32 {
        let impact = match finding.category.as_str() {
            "Mechanical Degradation" => {
                // Response time increase as percentage
                if let Some(slope) = finding.metrics.get("trend_slope_ms_per_hour") {
                    (*slope / 0.1).min(1.0) // Normalize: expect ~0.1 ms/hour
                } else {
                    0.5
                }
            }

            "Thermal Effects" => {
                // Efficiency decline percentage
                if let Some(decline) = finding.metrics.get("efficiency_decline_pct") {
                    (*decline / 20.0).min(1.0) // Normalize to 20% decline threshold
                } else {
                    0.5
                }
            }

            "Detection Robustness" => {
                // Confidence decline percentage
                if let Some(decline) = finding.metrics.get("confidence_decline_pct") {
                    (*decline / 50.0).min(1.0) // Normalize to 50% decline
                } else {
                    0.5
                }
            }

            "Optical Contamination" => {
                // Sharpness decline percentage
                if let Some(decline) = finding.metrics.get("sharpness_decline_pct") {
                    (*decline / 30.0).min(1.0) // Normalize to 30% decline
                } else {
                    0.5
                }
            }

            "Clock Drift" => {
                // PPM drift rate
                if let Some(drift) = finding.metrics.get("lidar_drift_ppm") {
                    (*drift / 2000.0).min(1.0) // Normalize: 2000 ppm is critical
                } else {
                    0.5
                }
            }

            _ => 0.5,
        };

        impact
    }

    /// Mission criticality (0.0-1.0)
    /// How critical is this mission for operations?
    fn mission_criticality_score(robot_type: &str) -> f32 {
        match robot_type {
            "warehouse_robot" | "delivery_robot" | "autonomous_vehicle" => 0.9,
            "industrial_robot" | "inspection_robot" => 0.7,
            "research_robot" | "exploration_robot" => 0.3,
            "testing_robot" | "simulator" => 0.1,
            _ => 0.5,
        }
    }

    /// Safety risk assessment (0.0-1.0)
    /// Does this gap create a safety hazard?
    fn assess_safety_risk(finding: &RealityGapFinding) -> f32 {
        if finding
            .finding_type
            .contains("Obstacle")
            || finding.finding_type.contains("Collision")
            || finding.finding_type.contains("Emergency Stop")
            || finding.finding_type.contains("Timestamp Reversal")
        {
            0.8 // High safety risk
        } else if finding
            .category
            .contains("Detection")
            || finding.category.contains("Mechanical")
        {
            0.3 // Medium safety risk
        } else {
            0.1 // Low safety risk
        }
    }

    /// Recurrence estimation (0.0-1.0)
    /// Is this a systematic issue that will happen again?
    fn estimate_recurrence(finding: &RealityGapFinding) -> f32 {
        // If we see a trend or pattern, it's likely to recur
        if let Some(trend) = finding.metrics.get("trend_slope_ms_per_hour") {
            if *trend > 0.01 {
                return 0.8; // Systematic trend = recurring
            }
        }

        if let Some(correlation) = finding.metrics.get("quality_confidence_correlation") {
            if *correlation > 0.6 {
                return 0.7; // Environmental factor = recurring
            }
        }

        if finding.evidence.len() > 2 {
            return 0.6; // Multiple evidence sources = systematic
        }

        0.2 // One-off event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classifier_creation() {
        let _classifier = SeverityClassifier;
    }

    #[test]
    fn test_mission_criticality() {
        let warehouse = SeverityClassifier::mission_criticality_score("warehouse_robot");
        let research = SeverityClassifier::mission_criticality_score("research_robot");
        assert!(warehouse > research); // Warehouse more critical
    }

    #[test]
    fn test_safety_risk_assessment() {
        // Mock finding with collision risk
        let collision_risk = SeverityClassifier::assess_safety_risk(&RealityGapFinding {
            domain: crate::analyzers::RealityDomain::Sensor,
            category: "Detection".to_string(),
            finding_type: "Collision Risk".to_string(),
            severity: Severity::High,
            confidence: 0.8,
            reality_gap_score: 0.7,
            description: "test".to_string(),
            evidence: vec![],
            metrics: HashMap::new(),
            sim_recreation_suggestion: "test".to_string(),
            remediation: "test".to_string(),
            detection_time_sec: None,
        });

        assert!(collision_risk > 0.3); // Should have safety risk
    }

    #[test]
    fn test_performance_impact_normalization() {
        let mut metrics = HashMap::new();
        metrics.insert("trend_slope_ms_per_hour".to_string(), 0.05);

        let finding = RealityGapFinding {
            domain: crate::analyzers::RealityDomain::Physical,
            category: "Mechanical Degradation".to_string(),
            finding_type: "test".to_string(),
            severity: Severity::Medium,
            confidence: 0.8,
            reality_gap_score: 0.7,
            description: "test".to_string(),
            evidence: vec![],
            metrics,
            sim_recreation_suggestion: "test".to_string(),
            remediation: "test".to_string(),
            detection_time_sec: None,
        };

        let impact = SeverityClassifier::compute_performance_impact(&finding);
        assert!(impact > 0.4); // 0.05 / 0.1 = 0.5
    }
}
