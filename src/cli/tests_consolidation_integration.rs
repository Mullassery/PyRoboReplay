//! Integration tests for Evidence Aggregation + CLI Output
//!
//! Verifies that aggregated findings can be properly formatted and output.

#[cfg(test)]
mod tests {
    use crate::analyzers::{
        RealityGapFinding, Severity, RealityDomain, Evidence, MissionAnalysisData,
    };
    use crate::analyzers::aggregation::EvidenceAggregator;
    use crate::cli::consolidated_output::ConsolidatedFormatter;
    use std::collections::HashMap;

    fn create_test_finding(category: &str, confidence: f32, gap_score: f32) -> RealityGapFinding {
        RealityGapFinding {
            domain: RealityDomain::Physical,
            category: category.to_string(),
            finding_type: format!("Test {}", category),
            severity: Severity::Medium,
            confidence,
            reality_gap_score: gap_score,
            description: "Test description".to_string(),
            evidence: vec![Evidence {
                signal: "test_signal".to_string(),
                value: 0.5,
                timestamp: 100.0,
                confidence: 0.85,
            }],
            metrics: HashMap::new(),
            sim_recreation_suggestion: "Simulate wear".to_string(),
            remediation: "Replace component".to_string(),
            detection_time_sec: None,
        }
    }

    #[test]
    fn test_end_to_end_aggregation_to_text_output() {
        // Create findings for same root cause
        let findings = vec![
            create_test_finding("Mechanical Degradation", 0.8, 0.7),
            create_test_finding("Structural Dynamics", 0.75, 0.75),
        ];

        // Aggregate
        let consolidated = EvidenceAggregator::aggregate(findings);
        assert_eq!(consolidated.len(), 1);
        assert_eq!(consolidated[0].detector_count, 2);

        // Format to text
        let formatter = ConsolidatedFormatter::new(true);
        let text = formatter.format_text(&consolidated);

        // Verify output
        assert!(text.contains("Consolidated Reality Gap Analysis"));
        assert!(text.contains("Mechanical Degradation")); // Root cause
        assert!(text.contains("2")); // Detector count
        assert!(text.contains("Supporting Detectors"));
    }

    #[test]
    fn test_multiple_root_causes_independent() {
        // Create findings for different root causes
        let findings = vec![
            create_test_finding("Mechanical Degradation", 0.8, 0.7),
            create_test_finding("Optical Contamination", 0.7, 0.6),
            create_test_finding("Thermal Effects", 0.75, 0.8),
        ];

        // Aggregate
        let consolidated = EvidenceAggregator::aggregate(findings);
        assert_eq!(consolidated.len(), 3); // Three different root causes

        // Format to JSON
        let formatter = ConsolidatedFormatter::new(false);
        let json = formatter.format_json(&consolidated);

        // Verify JSON structure
        assert_eq!(json["summary"]["total_consolidated"], 3);
        assert_eq!(json["summary"]["total_raw_findings"], 3);

        // Each finding should be single detector
        for finding in consolidated {
            assert_eq!(finding.detector_count, 1);
        }
    }

    #[test]
    fn test_confidence_boosting_in_output() {
        // Create 3 findings that will consolidate
        let findings = vec![
            create_test_finding("Mechanical Degradation", 0.70, 0.7),
            create_test_finding("Structural Dynamics", 0.65, 0.75),
            create_test_finding("Mechanical Degradation", 0.75, 0.65),
        ];

        // Aggregate (should produce 1 consolidated)
        let consolidated = EvidenceAggregator::aggregate(findings);
        assert_eq!(consolidated.len(), 1);
        assert_eq!(consolidated[0].detector_count, 3);

        // Average confidence: (0.70 + 0.65 + 0.75) / 3 = 0.7
        // With 2 extra detectors: 0.7 + 0.2 (capped) = 0.9
        let base_avg: f32 = (0.70 + 0.65 + 0.75) / 3.0;
        let expected_confidence: f32 = (base_avg + 0.2).min(1.0);
        assert!((consolidated[0].consolidated_confidence - expected_confidence).abs() < 0.01);

        // Verify it's boosted from base average
        assert!(consolidated[0].consolidated_confidence > base_avg);

        // Format and verify confidence boost is visible
        let formatter = ConsolidatedFormatter::new(true);
        let text = formatter.format_text(&consolidated);
        assert!(text.contains(&format!("{:.0}%", consolidated[0].consolidated_confidence * 100.0)));
    }

    #[test]
    fn test_html_output_with_detector_badges() {
        let findings = vec![
            create_test_finding("Thermal Effects", 0.8, 0.7),
            create_test_finding("Motor Current", 0.75, 0.75),
        ];

        let consolidated = EvidenceAggregator::aggregate(findings);
        let formatter = ConsolidatedFormatter::new(false);
        let html = formatter.format_html(&consolidated, "test_mission_001");

        // Verify HTML structure
        assert!(html.contains("<!DOCTYPE html"));
        assert!(html.contains("test_mission_001"));
        assert!(html.contains("detector-badge"));
        assert!(html.contains("confidence-fill"));
        assert!(html.contains("Consolidated Reality Gap Analysis"));
    }

    #[test]
    fn test_redundancy_calculation() {
        let findings = vec![
            create_test_finding("Mechanical Degradation", 0.8, 0.7),
            create_test_finding("Mechanical Degradation", 0.75, 0.75),
            create_test_finding("Thermal Effects", 0.7, 0.6),
        ];

        let consolidated = EvidenceAggregator::aggregate(findings);

        // Should have 2 consolidated findings
        // First has 2 detectors, second has 1
        // Redundancy = (2 + 1) / 2 = 1.5x
        assert_eq!(consolidated.len(), 2);

        let formatter = ConsolidatedFormatter::new(false);
        let json = formatter.format_json(&consolidated);

        assert_eq!(json["summary"]["total_consolidated"], 2);
        assert_eq!(json["summary"]["total_raw_findings"], 3);

        // Redundancy factor is total detectors / consolidated findings
        // (2 + 1) / 2 = 1.5
        let redundancy = json["summary"]["redundancy_factor"].as_f64().unwrap_or(1.0) as f32;
        assert!((redundancy - 1.5).abs() < 0.1, "Expected 1.5, got {}", redundancy);
    }

    #[test]
    fn test_empty_findings_output() {
        let findings: Vec<RealityGapFinding> = vec![];
        let consolidated = EvidenceAggregator::aggregate(findings);

        let formatter = ConsolidatedFormatter::new(false);

        // Text output
        let text = formatter.format_text(&consolidated);
        assert!(text.contains("No reality gaps detected"));

        // JSON output
        let json = formatter.format_json(&consolidated);
        assert_eq!(json["summary"]["total_consolidated"], 0);

        // HTML output
        let html = formatter.format_html(&consolidated, "clean_mission");
        assert!(html.contains("clean_mission"));
    }
}
