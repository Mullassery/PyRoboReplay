//! Evidence Quality Scoring for Incident Narratives
//!
//! Evaluates the quality and trustworthiness of evidence supporting incident narratives.
//! Higher quality evidence → higher confidence in the narrative.

use crate::analyzers::incident_narrative::IncidentNarrative;
use crate::analyzers::RealityGapFinding;
use std::collections::HashMap;

/// Quality assessment of evidence
#[derive(Debug, Clone)]
pub struct EvidenceQualityScore {
    /// Overall quality score (0.0-1.0)
    pub overall_score: f32,

    /// Evidence source quality (0.0-1.0)
    pub source_quality: f32,

    /// Temporal consistency (0.0-1.0)
    pub temporal_consistency: f32,

    /// Cross-detector agreement (0.0-1.0)
    pub detector_agreement: f32,

    /// Signal-to-noise ratio (0.0-1.0)
    pub signal_to_noise: f32,

    /// Recency of evidence (0.0-1.0, 1.0 = very recent)
    pub recency: f32,

    /// Breakdown: which factors reduce quality
    pub quality_issues: Vec<QualityIssue>,

    /// Confidence modifier based on quality
    pub confidence_adjustment: f32, // multiplier: 0.5-1.5
}

/// A specific quality issue found in evidence
#[derive(Debug, Clone)]
pub struct QualityIssue {
    /// Type of issue: "missing_data", "high_noise", "temporal_gap", "single_source", etc.
    pub issue_type: String,

    /// Human description of the issue
    pub description: String,

    /// How much this reduces confidence (0.0-1.0)
    pub confidence_impact: f32,

    /// Can this issue be mitigated? How?
    pub mitigation: Option<String>,
}

/// Scorer for evidence quality
pub struct EvidenceQualityScorer;

impl EvidenceQualityScorer {
    /// Score the quality of evidence supporting an incident narrative
    pub fn score_narrative_evidence(
        narrative: &IncidentNarrative,
        gaps: &[RealityGapFinding],
        detector_agreement_matrix: &HashMap<String, Vec<String>>,
    ) -> EvidenceQualityScore {
        let source_quality = Self::assess_source_quality(gaps);
        let temporal_consistency = Self::assess_temporal_consistency(&narrative.supporting_evidence);
        let detector_agreement = Self::assess_detector_agreement(gaps, detector_agreement_matrix);
        let signal_to_noise = Self::assess_signal_to_noise(gaps);
        let recency = Self::assess_recency(narrative.end_time_sec);

        // Compute quality issues
        let mut quality_issues = Vec::new();

        if source_quality < 0.7 {
            quality_issues.push(QualityIssue {
                issue_type: "low_source_quality".to_string(),
                description: "Some evidence sources have lower reliability".to_string(),
                confidence_impact: (1.0 - source_quality) * 0.2,
                mitigation: Some("Cross-reference with multiple sensors".to_string()),
            });
        }

        if temporal_consistency < 0.7 {
            quality_issues.push(QualityIssue {
                issue_type: "temporal_gaps".to_string(),
                description: "Evidence shows gaps in time coverage".to_string(),
                confidence_impact: (1.0 - temporal_consistency) * 0.15,
                mitigation: Some("Interpolate or collect more frequent samples".to_string()),
            });
        }

        if detector_agreement < 0.6 {
            quality_issues.push(QualityIssue {
                issue_type: "single_source".to_string(),
                description: "Evidence from limited number of detectors".to_string(),
                confidence_impact: (1.0 - detector_agreement) * 0.25,
                mitigation: Some("Verify finding with additional analysis methods".to_string()),
            });
        }

        if signal_to_noise < 0.65 {
            quality_issues.push(QualityIssue {
                issue_type: "high_noise".to_string(),
                description: "Evidence is noisy relative to signal strength".to_string(),
                confidence_impact: (1.0 - signal_to_noise) * 0.2,
                mitigation: Some("Apply filtering or increase averaging window".to_string()),
            });
        }

        if recency < 0.5 {
            quality_issues.push(QualityIssue {
                issue_type: "stale_evidence".to_string(),
                description: "Evidence is from older mission".to_string(),
                confidence_impact: (1.0 - recency) * 0.1,
                mitigation: Some("Validate against recent data".to_string()),
            });
        }

        // Compute overall quality score
        let overall_score = (source_quality * 0.25
            + temporal_consistency * 0.20
            + detector_agreement * 0.30
            + signal_to_noise * 0.15
            + recency * 0.10)
            .min(1.0)
            .max(0.0);

        // Compute confidence adjustment (0.5-1.5 multiplier)
        let issue_penalty: f32 = quality_issues.iter().map(|i| i.confidence_impact).sum();
        let confidence_adjustment = (1.0 - issue_penalty * 0.3).max(0.5).min(1.5);

        EvidenceQualityScore {
            overall_score,
            source_quality,
            temporal_consistency,
            detector_agreement,
            signal_to_noise,
            recency,
            quality_issues,
            confidence_adjustment,
        }
    }

    /// Assess quality of evidence sources (sensors, detectors)
    fn assess_source_quality(gaps: &[RealityGapFinding]) -> f32 {
        if gaps.is_empty() {
            return 0.5;
        }

        // Average confidence of all gaps
        let avg_confidence: f32 = gaps.iter().map(|g| g.confidence).sum::<f32>() / gaps.len() as f32;

        // Penalize if gaps are from unreliable domains
        let unreliable_domains = ["Sensor", "Environmental"];
        let penalty: f32 = gaps
            .iter()
            .filter(|g| unreliable_domains.contains(&g.domain.to_string().as_str()))
            .count() as f32
            / gaps.len() as f32
            * 0.15;

        (avg_confidence - penalty).max(0.0).min(1.0)
    }

    /// Assess whether evidence is temporally consistent
    fn assess_temporal_consistency(evidence: &[String]) -> f32 {
        if evidence.is_empty() {
            return 0.5;
        }

        // Check for large gaps in temporal coverage
        // If we have events at t=0, t=5, t=100, there's a big gap
        // This is a simplified heuristic

        let has_dense_coverage = evidence.len() > 3;
        let has_recent_events = evidence
            .last()
            .map(|e| e.contains("99") || e.contains("100"))
            .unwrap_or(false);

        let coverage_score: f32 = if has_dense_coverage { 0.8 } else { 0.6 };
        let recency_bonus: f32 = if has_recent_events { 0.2 } else { 0.0 };

        (coverage_score + recency_bonus).min(1.0)
    }

    /// Assess agreement between multiple detectors
    fn assess_detector_agreement(
        gaps: &[RealityGapFinding],
        detector_matrix: &HashMap<String, Vec<String>>,
    ) -> f32 {
        if gaps.is_empty() {
            return 0.5;
        }

        // If we have multiple gaps in the same domain, detectors agree
        let domain_groups: HashMap<String, usize> = gaps.iter().fold(
            HashMap::new(),
            |mut acc, gap| {
                *acc.entry(gap.domain.to_string()).or_insert(0) += 1;
                acc
            },
        );

        let agreement_score: f32 = domain_groups
            .values()
            .map(|&count| {
                if count > 1 {
                    0.9 // High agreement
                } else if count == 1 {
                    0.6 // Single source
                } else {
                    0.3
                }
            })
            .sum::<f32>()
            / domain_groups.len() as f32;

        agreement_score.max(0.0).min(1.0)
    }

    /// Assess signal-to-noise ratio
    fn assess_signal_to_noise(gaps: &[RealityGapFinding]) -> f32 {
        if gaps.is_empty() {
            return 0.5;
        }

        // SNR is high if gaps have high confidence and clear severity
        let avg_confidence: f32 = gaps.iter().map(|g| g.confidence).sum::<f32>() / gaps.len() as f32;

        // Strong signal if findings are consistent across different categories
        let unique_categories: std::collections::HashSet<_> =
            gaps.iter().map(|g| g.category.clone()).collect();
        let consistency_bonus = (unique_categories.len() as f32).min(5.0) / 5.0 * 0.2;

        (avg_confidence + consistency_bonus).min(1.0)
    }

    /// Assess recency of evidence
    fn assess_recency(end_time_sec: f32) -> f32 {
        // Prefer evidence from recent missions
        // This is a simplified heuristic: assume current mission is best
        if end_time_sec > 50.0 {
            1.0 // Recent
        } else if end_time_sec > 25.0 {
            0.8
        } else if end_time_sec > 5.0 {
            0.6
        } else {
            0.4
        }
    }

    /// Compute adjusted narrative confidence based on evidence quality
    pub fn apply_quality_adjustment(
        narrative_confidence: f32,
        quality_score: &EvidenceQualityScore,
    ) -> f32 {
        (narrative_confidence * quality_score.confidence_adjustment).min(1.0)
    }

    /// Generate quality report as human-readable text
    pub fn generate_quality_report(score: &EvidenceQualityScore) -> String {
        let mut report = format!(
            "EVIDENCE QUALITY ASSESSMENT\n\
             Overall Quality Score: {:.0}%\n\n\
             Breakdown:\n\
             - Source Quality: {:.0}%\n\
             - Temporal Consistency: {:.0}%\n\
             - Detector Agreement: {:.0}%\n\
             - Signal-to-Noise Ratio: {:.0}%\n\
             - Evidence Recency: {:.0}%\n\n",
            score.overall_score * 100.0,
            score.source_quality * 100.0,
            score.temporal_consistency * 100.0,
            score.detector_agreement * 100.0,
            score.signal_to_noise * 100.0,
            score.recency * 100.0
        );

        if !score.quality_issues.is_empty() {
            report.push_str("Quality Issues Found:\n");
            for issue in &score.quality_issues {
                report.push_str(&format!(
                    "- {} (impact: {:.0}%)\n  {}\n",
                    issue.issue_type,
                    issue.confidence_impact * 100.0,
                    issue.description
                ));
                if let Some(mit) = &issue.mitigation {
                    report.push_str(&format!("  → Mitigation: {}\n", mit));
                }
            }
            report.push('\n');
        }

        report.push_str(&format!(
            "Confidence Adjustment: {:.2}x\n",
            score.confidence_adjustment
        ));
        report.push_str("(Multiply narrative confidence by this factor for adjusted score)");

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_narrative() -> IncidentNarrative {
        IncidentNarrative {
            incident_id: "test_incident".to_string(),
            mission_id: "mission_123".to_string(),
            start_time_sec: 10.0,
            end_time_sec: 95.0,
            executive_summary: "Test summary".to_string(),
            what_happened: "Test events".to_string(),
            why_it_happened: "Test causality".to_string(),
            impact_description: "Test impact".to_string(),
            contributing_factors_explained: vec![],
            recommended_actions: vec![],
            escalation_risk: "Test risk".to_string(),
            supporting_evidence: vec![
                "At t=10.0s: Event 1 (90% confidence)".to_string(),
                "At t=20.0s: Event 2 (85% confidence)".to_string(),
                "At t=95.0s: Event 3 (80% confidence)".to_string(),
            ],
            narrative_confidence: 0.75,
        }
    }

    fn create_test_gap(category: &str, confidence: f32, domain: &str) -> RealityGapFinding {
        use crate::analyzers::{Evidence, RealityDomain, Severity};

        let domain_enum = match domain {
            "Sensor" => RealityDomain::Sensor,
            "Physical" => RealityDomain::Physical,
            "Environmental" => RealityDomain::Environmental,
            _ => RealityDomain::System,
        };

        RealityGapFinding {
            domain: domain_enum,
            category: category.to_string(),
            finding_type: "Test".to_string(),
            severity: Severity::High,
            confidence,
            reality_gap_score: 0.7,
            description: "Test gap".to_string(),
            evidence: vec![Evidence {
                signal: "test_signal".to_string(),
                value: 0.5,
                timestamp: 50.0,
                confidence: 0.8,
            }],
            metrics: HashMap::new(),
            sim_recreation_suggestion: "Test".to_string(),
            remediation: "Test".to_string(),
            detection_time_sec: Some(50.0),
        }
    }

    #[test]
    fn test_source_quality_assessment() {
        let gaps = vec![
            create_test_gap("Optical Contamination", 0.85, "Sensor"),
            create_test_gap("Thermal Effects", 0.80, "Physical"),
        ];

        let score = EvidenceQualityScorer::assess_source_quality(&gaps);
        assert!(score > 0.65);
        assert!(score < 1.0);
    }

    #[test]
    fn test_temporal_consistency_assessment() {
        let evidence = vec![
            "At t=10.0s: Event 1 (90% confidence)".to_string(),
            "At t=30.0s: Event 2 (85% confidence)".to_string(),
            "At t=100.0s: Event 3 (80% confidence)".to_string(),
        ];

        let score = EvidenceQualityScorer::assess_temporal_consistency(&evidence);
        assert!(score > 0.5);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_full_quality_scoring() {
        let narrative = create_test_narrative();
        let gaps = vec![
            create_test_gap("Contamination", 0.82, "Sensor"),
            create_test_gap("Degradation", 0.80, "Physical"),
        ];
        let detector_matrix = HashMap::new();

        let quality_score =
            EvidenceQualityScorer::score_narrative_evidence(&narrative, &gaps, &detector_matrix);

        assert!(quality_score.overall_score > 0.0);
        assert!(quality_score.overall_score <= 1.0);
        assert!(quality_score.confidence_adjustment > 0.5);
        assert!(quality_score.confidence_adjustment <= 1.5);
    }

    #[test]
    fn test_quality_report_generation() {
        let narrative = create_test_narrative();
        let gaps = vec![create_test_gap("Test Gap", 0.75, "Sensor")];
        let detector_matrix = HashMap::new();

        let quality_score =
            EvidenceQualityScorer::score_narrative_evidence(&narrative, &gaps, &detector_matrix);
        let report = EvidenceQualityScorer::generate_quality_report(&quality_score);

        assert!(report.contains("EVIDENCE QUALITY ASSESSMENT"));
        assert!(report.contains("Confidence Adjustment"));
    }
}
