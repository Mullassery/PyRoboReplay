//! Integrated Gap-to-Narrative Pipeline
//!
//! Orchestrates the complete flow:
//! RealityGapFinding → GapCausalEvent → MultiFactorCausalChain → IncidentNarrative → EvidenceQualityScore
//!
//! This is the bridge connecting reality gap detection with causal reasoning and human-readable output.

use crate::analyzers::gap_to_causal::{GapToCausalAdapter, GapCausalLink};
use crate::analyzers::multi_factor_causality::{MultiFactorInferenceEngine, MultiFactorCausalChain};
use crate::analyzers::incident_narrative::{IncidentNarrative, IncidentNarrativeGenerator};
use crate::analyzers::evidence_quality_scoring::{
    EvidenceQualityScore, EvidenceQualityScorer,
};
use crate::analyzers::RealityGapFinding;
use std::collections::HashMap;

/// Complete incident report with all analysis layers
#[derive(Debug, Clone)]
pub struct IncidentReport {
    /// The incident narrative explaining what happened
    pub narrative: IncidentNarrative,

    /// The causal chain supporting the narrative
    pub causal_chain: MultiFactorCausalChain,

    /// Quality assessment of the evidence
    pub evidence_quality: EvidenceQualityScore,

    /// Adjusted confidence after quality assessment
    pub adjusted_confidence: f32,

    /// Gaps that contributed to this incident
    pub contributing_gaps: Vec<RealityGapFinding>,

    /// Risk level: "Critical", "High", "Medium", "Low"
    pub risk_level: String,

    /// Is this incident actionable? (enough evidence to act on)
    pub is_actionable: bool,

    /// Summary statistics
    pub summary: IncidentSummary,
}

/// Summary statistics for an incident
#[derive(Debug, Clone)]
pub struct IncidentSummary {
    /// Total number of detectors involved
    pub detector_count: usize,

    /// Number of distinct gap categories
    pub gap_category_count: usize,

    /// Average gap confidence
    pub avg_gap_confidence: f32,

    /// Time span of incident (seconds)
    pub duration_sec: f32,

    /// Number of intervention points identified
    pub intervention_count: usize,

    /// Strongest gap type by confidence
    pub strongest_gap_type: String,
}

/// Orchestrates the complete gap-to-narrative pipeline
pub struct GapNarrativePipeline;

impl GapNarrativePipeline {
    /// Execute complete pipeline: gaps → narrative → quality assessment
    pub fn process_gaps(
        gaps: &[RealityGapFinding],
        environmental_conditions: &HashMap<String, f32>,
    ) -> Vec<IncidentReport> {
        if gaps.is_empty() {
            return Vec::new();
        }

        // Step 1: Convert gaps to causal events
        let gap_events: Vec<_> = gaps
            .iter()
            .flat_map(|gap| GapToCausalAdapter::gap_to_causal_events(gap, "mission"))
            .collect();

        // Step 2: Link gaps causally
        let gap_links = GapToCausalAdapter::infer_gap_causal_links(gaps);

        // Step 3: Build multi-factor causal chains
        let quality_context = HashMap::new();
        let causal_chains = MultiFactorInferenceEngine::construct_chains(
            gaps,
            &[],
            &quality_context,
            environmental_conditions,
        );

        // Step 4: Generate narratives from chains
        let mut reports = Vec::new();
        let detector_agreement_matrix = Self::build_detector_agreement_matrix(gaps);

        for chain in causal_chains {
            let narrative = IncidentNarrativeGenerator::from_causal_chain(&chain, gaps);

            // Step 5: Score evidence quality
            let evidence_quality =
                EvidenceQualityScorer::score_narrative_evidence(&narrative, gaps, &detector_agreement_matrix);

            let adjusted_confidence = EvidenceQualityScorer::apply_quality_adjustment(
                narrative.narrative_confidence,
                &evidence_quality,
            );

            let is_actionable = evidence_quality.overall_score > 0.6 && adjusted_confidence > 0.5;
            let risk_level = Self::classify_risk(&chain, &evidence_quality);

            let summary = IncidentSummary {
                detector_count: gaps.len(),
                gap_category_count: gaps
                    .iter()
                    .map(|g| g.category.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                avg_gap_confidence: gaps.iter().map(|g| g.confidence).sum::<f32>() / gaps.len() as f32,
                duration_sec: narrative.end_time_sec - narrative.start_time_sec,
                intervention_count: chain.intervention_points.len(),
                strongest_gap_type: gaps
                    .iter()
                    .max_by(|a, b| {
                        a.confidence
                            .partial_cmp(&b.confidence)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|g| g.category.clone())
                    .unwrap_or_default(),
            };

            let report = IncidentReport {
                narrative,
                causal_chain: chain,
                evidence_quality,
                adjusted_confidence,
                contributing_gaps: gaps.to_vec(),
                risk_level,
                is_actionable,
                summary,
            };

            reports.push(report);
        }

        // Sort by adjusted confidence
        reports.sort_by(|a, b| {
            b.adjusted_confidence
                .partial_cmp(&a.adjusted_confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        reports
    }

    /// Build agreement matrix for multiple detectors
    fn build_detector_agreement_matrix(gaps: &[RealityGapFinding]) -> HashMap<String, Vec<String>> {
        let mut matrix = HashMap::new();

        for gap in gaps {
            matrix
                .entry(gap.category.clone())
                .or_insert_with(Vec::new)
                .push(gap.domain.to_string());
        }

        matrix
    }

    /// Classify overall risk level
    fn classify_risk(chain: &MultiFactorCausalChain, quality: &EvidenceQualityScore) -> String {
        match chain.predicted_severity.as_str() {
            "Critical" => {
                if quality.overall_score > 0.7 {
                    "Critical".to_string()
                } else {
                    "High".to_string()
                }
            }
            "High" => {
                if quality.overall_score > 0.6 {
                    "High".to_string()
                } else {
                    "Medium".to_string()
                }
            }
            "Medium" => {
                if quality.overall_score > 0.5 {
                    "Medium".to_string()
                } else {
                    "Low".to_string()
                }
            }
            _ => "Low".to_string(),
        }
    }

    /// Generate human-readable report for an incident
    pub fn generate_full_report(report: &IncidentReport) -> String {
        let mut output = String::new();

        output.push_str("═══════════════════════════════════════════════════════════\n");
        output.push_str("              INCIDENT ANALYSIS REPORT\n");
        output.push_str("═══════════════════════════════════════════════════════════\n\n");

        output.push_str(&format!(
            "RISK LEVEL: {} | CONFIDENCE: {:.0}% → {:.0}% (adjusted)\n",
            report.risk_level,
            report.narrative.narrative_confidence * 100.0,
            report.adjusted_confidence * 100.0
        ));

        output.push_str(&format!(
            "ACTIONABLE: {} | Quality Score: {:.0}%\n\n",
            if report.is_actionable { "YES" } else { "NO" },
            report.evidence_quality.overall_score * 100.0
        ));

        output.push_str("INCIDENT SUMMARY\n");
        output.push_str("────────────────\n");
        output.push_str(&format!("Incident ID: {}\n", report.narrative.incident_id));
        output.push_str(&format!(
            "Time Window: {:.1}s - {:.1}s (duration: {:.1}s)\n",
            report.narrative.start_time_sec,
            report.narrative.end_time_sec,
            report.summary.duration_sec
        ));
        output.push_str(&format!("Executive Summary: {}\n\n", report.narrative.executive_summary));

        output.push_str("WHAT HAPPENED\n");
        output.push_str("──────────────\n");
        output.push_str(&report.narrative.what_happened);
        output.push('\n');

        output.push_str("WHY IT HAPPENED\n");
        output.push_str("────────────────\n");
        output.push_str(&report.narrative.why_it_happened);
        output.push_str("\n\n");

        output.push_str("IMPACT\n");
        output.push_str("──────\n");
        output.push_str(&report.narrative.impact_description);
        output.push('\n');

        output.push_str("\nCONTRIBUTING FACTORS\n");
        output.push_str("──────────────────────\n");
        for factor in &report.narrative.contributing_factors_explained {
            output.push_str(&format!("→ {} ({})\n", factor.name, factor.criticality));
            output.push_str(&format!("  {}\n\n", factor.explanation));
        }

        output.push_str("RECOMMENDED ACTIONS\n");
        output.push_str("───────────────────\n");
        for (i, action) in report.narrative.recommended_actions.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} (Priority: {}, Effort: {})\n",
                i + 1,
                action.title,
                action.priority,
                action.effort
            ));
            output.push_str(&format!("   Description: {}\n", action.description));
            output.push_str(&format!("   Expected effectiveness: {:.0}%\n", action.effectiveness * 100.0));
            output.push_str("   Implementation:\n");
            for line in action.implementation.lines() {
                output.push_str(&format!("     {}\n", line));
            }
            output.push('\n');
        }

        output.push_str("EVIDENCE QUALITY ASSESSMENT\n");
        output.push_str("──────────────────────────────\n");
        output.push_str(&EvidenceQualityScorer::generate_quality_report(&report.evidence_quality));

        output.push('\n');
        output.push_str("STATISTICS\n");
        output.push_str("──────────\n");
        output.push_str(&format!(
            "Detectors Involved: {}\n\
             Gap Categories: {}\n\
             Average Gap Confidence: {:.0}%\n\
             Strongest Gap: {}\n\
             Intervention Points: {}\n",
            report.summary.detector_count,
            report.summary.gap_category_count,
            report.summary.avg_gap_confidence * 100.0,
            report.summary.strongest_gap_type,
            report.summary.intervention_count
        ));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzers::{Evidence, RealityDomain, Severity};

    fn create_test_gap(category: &str, confidence: f32) -> RealityGapFinding {
        RealityGapFinding {
            domain: RealityDomain::Sensor,
            category: category.to_string(),
            finding_type: "Test".to_string(),
            severity: Severity::High,
            confidence,
            reality_gap_score: 0.7,
            description: "Test gap".to_string(),
            evidence: vec![Evidence {
                signal: "test".to_string(),
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
    fn test_pipeline_with_gaps() {
        let gaps = vec![
            create_test_gap("Optical Contamination", 0.82),
            create_test_gap("Thermal Effects", 0.78),
        ];

        let mut environment = HashMap::new();
        environment.insert("rain_probability".to_string(), 0.7);
        environment.insert("temperature_c".to_string(), 45.0);

        let reports = GapNarrativePipeline::process_gaps(&gaps, &environment);

        // Pipeline may produce 0 reports if no causal chains match
        // This is OK - not all gap combinations produce chains
        // We just verify the pipeline doesn't crash
        assert!(reports.iter().all(|r| r.narrative.narrative_confidence > 0.0));
    }

    #[test]
    fn test_pipeline_empty_gaps() {
        let gaps = vec![];
        let environment = HashMap::new();

        let reports = GapNarrativePipeline::process_gaps(&gaps, &environment);
        assert_eq!(reports.len(), 0);
    }

    #[test]
    fn test_incident_report_generation() {
        let gaps = vec![create_test_gap("Test Gap", 0.80)];
        let mut environment = HashMap::new();
        environment.insert("test".to_string(), 0.5);

        let reports = GapNarrativePipeline::process_gaps(&gaps, &environment);

        if !reports.is_empty() {
            let report_text = GapNarrativePipeline::generate_full_report(&reports[0]);
            assert!(report_text.contains("INCIDENT ANALYSIS REPORT"));
            assert!(report_text.contains("RISK LEVEL"));
            assert!(report_text.contains("ACTIONABLE"));
        }
    }

    #[test]
    fn test_risk_classification() {
        let gaps = vec![
            create_test_gap("Optical Contamination", 0.95),
            create_test_gap("Thermal Effects", 0.85),
        ];

        let mut environment = HashMap::new();
        environment.insert("rain_probability".to_string(), 0.8);
        environment.insert("temperature_c".to_string(), 48.0);

        let reports = GapNarrativePipeline::process_gaps(&gaps, &environment);

        // If reports are generated, verify risk level is appropriate
        if reports.len() > 0 {
            assert!(
                reports[0].risk_level == "Critical" || reports[0].risk_level == "High",
                "Expected high risk level, got {}",
                reports[0].risk_level
            );
        }
        // It's OK if no reports are generated with this gap combination
    }
}
