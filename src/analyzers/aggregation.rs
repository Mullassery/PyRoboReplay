//! Evidence Aggregation
//!
//! Fuses multiple findings into consolidated gaps with boosted confidence.

use crate::analyzers::{RealityGapFinding, Severity};
use std::collections::HashMap;

/// Aggregated finding from multiple detectors
#[derive(Debug, Clone)]
pub struct ConsolidatedFinding {
    pub root_cause: String,
    pub component_findings: Vec<RealityGapFinding>,
    pub consolidated_gap_score: f32,
    pub consolidated_confidence: f32,
    pub detector_count: usize,
    pub explanation: String,
}

/// Aggregate multiple findings into consolidated findings
pub struct EvidenceAggregator;

impl EvidenceAggregator {
    /// Fuse findings by root cause, boosting confidence for agreement
    pub fn aggregate(findings: Vec<RealityGapFinding>) -> Vec<ConsolidatedFinding> {
        if findings.is_empty() {
            return Vec::new();
        }

        // Group findings by inferred root cause
        let mut groups: HashMap<String, Vec<RealityGapFinding>> = HashMap::new();

        for finding in findings {
            let root_cause = Self::infer_root_cause(&finding);
            groups.entry(root_cause).or_insert_with(Vec::new).push(finding);
        }

        // Consolidate each group
        let mut consolidated = Vec::new();
        for (root_cause, group) in groups {
            consolidated.push(Self::consolidate_group(root_cause, group));
        }

        // Sort by consolidated confidence
        consolidated.sort_by(|a, b| {
            b.consolidated_confidence
                .partial_cmp(&a.consolidated_confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        consolidated
    }

    /// Map individual finding to underlying root cause
    fn infer_root_cause(finding: &RealityGapFinding) -> String {
        match finding.category.as_str() {
            // Physical domain
            "Mechanical Degradation" | "Structural Dynamics" => "Mechanical Degradation".to_string(),

            // Thermal-related
            "Thermal Effects" => "Thermal Degradation".to_string(),

            // Optical/detection-related
            "Optical Contamination" | "Detection Robustness" => {
                if finding.finding_type.contains("Environmental") {
                    "Environmental Perception Failure".to_string()
                } else {
                    "Sensor Quality Degradation".to_string()
                }
            }

            // Temporal/system-related
            "Temporal Synchronization" | "Clock Drift" => "Timing Synchronization".to_string(),

            // Default: use category as root cause
            _ => finding.category.clone(),
        }
    }

    /// Consolidate a group of findings with confidence boosting
    fn consolidate_group(
        root_cause: String,
        findings: Vec<RealityGapFinding>,
    ) -> ConsolidatedFinding {
        let detector_count = findings.len();

        // Compute average gap score
        let avg_gap_score = if !findings.is_empty() {
            findings
                .iter()
                .map(|f| f.reality_gap_score)
                .sum::<f32>()
                / findings.len() as f32
        } else {
            0.5
        };

        // Compute average confidence
        let avg_confidence = if !findings.is_empty() {
            findings
                .iter()
                .map(|f| f.confidence)
                .sum::<f32>()
                / findings.len() as f32
        } else {
            0.0
        };

        // Agreement bonus: +10% per additional detector (up to 30% total)
        let agreement_bonus = if detector_count > 1 {
            (0.1 * ((detector_count - 1) as f32)).min(0.3)
        } else {
            0.0
        };

        let consolidated_confidence = (avg_confidence + agreement_bonus).min(1.0);

        // Generate explanation from component detectors
        let detectors: Vec<&str> = findings
            .iter()
            .map(|f| f.category.as_str())
            .collect();
        let explanation = Self::generate_explanation(&detectors);

        ConsolidatedFinding {
            root_cause,
            component_findings: findings,
            consolidated_gap_score: avg_gap_score,
            consolidated_confidence,
            detector_count,
            explanation,
        }
    }

    /// Generate natural explanation from detector types
    fn generate_explanation(detectors: &[&str]) -> String {
        if detectors.is_empty() {
            return "Unknown issue".to_string();
        }

        let detector_str = detectors.join(", ");

        match detectors.len() {
            1 => format!("Detected by: {}", detector_str),
            _ => format!(
                "Multiple indicators ({} detectors): {}",
                detectors.len(),
                detector_str
            ),
        }
    }

    /// Get highest-confidence findings after aggregation
    pub fn top_findings(consolidated: &[ConsolidatedFinding], count: usize) -> Vec<&ConsolidatedFinding> {
        consolidated.iter().take(count).collect()
    }

    /// Compute redundancy: how many findings map to same root cause?
    pub fn redundancy_factor(consolidated: &[ConsolidatedFinding]) -> f32 {
        if consolidated.is_empty() {
            return 1.0;
        }

        let total_components: usize = consolidated.iter().map(|c| c.detector_count).sum();
        let root_cause_count = consolidated.len();

        (total_components as f32) / (root_cause_count as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregator_empty() {
        let result = EvidenceAggregator::aggregate(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_root_cause_inference() {
        let finding = RealityGapFinding {
            domain: crate::analyzers::RealityDomain::Physical,
            category: "Mechanical Degradation".to_string(),
            finding_type: "test".to_string(),
            severity: Severity::Medium,
            confidence: 0.8,
            reality_gap_score: 0.7,
            description: "test".to_string(),
            evidence: vec![],
            metrics: HashMap::new(),
            sim_recreation_suggestion: "test".to_string(),
            remediation: "test".to_string(),
            detection_time_sec: None,
        };

        let root_cause = EvidenceAggregator::infer_root_cause(&finding);
        assert_eq!(root_cause, "Mechanical Degradation");
    }

    #[test]
    fn test_confidence_boosting() {
        let findings = vec![
            RealityGapFinding {
                domain: crate::analyzers::RealityDomain::Physical,
                category: "Mechanical Degradation".to_string(),
                finding_type: "test".to_string(),
                severity: Severity::Medium,
                confidence: 0.8,
                reality_gap_score: 0.7,
                description: "test".to_string(),
                evidence: vec![],
                metrics: HashMap::new(),
                sim_recreation_suggestion: "test".to_string(),
                remediation: "test".to_string(),
                detection_time_sec: None,
            },
            RealityGapFinding {
                domain: crate::analyzers::RealityDomain::Physical,
                category: "Structural Dynamics".to_string(),
                finding_type: "test".to_string(),
                severity: Severity::Medium,
                confidence: 0.75,
                reality_gap_score: 0.75,
                description: "test".to_string(),
                evidence: vec![],
                metrics: HashMap::new(),
                sim_recreation_suggestion: "test".to_string(),
                remediation: "test".to_string(),
                detection_time_sec: None,
            },
        ];

        let consolidated = EvidenceAggregator::aggregate(findings);
        assert_eq!(consolidated.len(), 1);
        assert_eq!(consolidated[0].detector_count, 2);

        // Confidence should be boosted: (0.8 + 0.75) / 2 + 0.1 = 0.775 + 0.1 = 0.875
        let expected_confidence = 0.775 + 0.1;
        assert!((consolidated[0].consolidated_confidence - expected_confidence).abs() < 0.01);
    }

    #[test]
    fn test_redundancy_factor() {
        let consolidated = vec![ConsolidatedFinding {
            root_cause: "test".to_string(),
            component_findings: vec![],
            consolidated_gap_score: 0.7,
            consolidated_confidence: 0.8,
            detector_count: 3,
            explanation: "test".to_string(),
        }];

        let redundancy = EvidenceAggregator::redundancy_factor(&consolidated);
        assert_eq!(redundancy, 3.0); // 3 detectors / 1 root cause
    }

    #[test]
    fn test_explanation_generation() {
        let detectors = vec!["Mechanical", "Thermal"];
        let explanation = EvidenceAggregator::generate_explanation(&detectors);
        assert!(explanation.contains("2 detectors"));
        assert!(explanation.contains("Mechanical"));
    }
}
