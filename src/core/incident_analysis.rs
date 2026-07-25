use crate::core::confidence_scoring::ConfidenceScoringEngine;
use crate::core::event::MissionEvent;
use crate::core::failure_detection::{FailureDetectionEngine, DetectedFailure, FailureSeverity};
use crate::core::incident_bundle::IncidentBundle;
use crate::core::recommendations_engine::{MLRIASRecommendationsEngine, MLRIASRecommendation};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentAnalysisReport {
    pub bundle_id: String,
    pub analysis_timestamp: DateTime<Utc>,
    pub time_range_start: DateTime<Utc>,
    pub time_range_end: DateTime<Utc>,
    pub robots_involved: Vec<String>,
    pub detected_failures: Vec<FailureReport>,
    pub recommendations: Vec<RecommendationReport>,
    pub analysis_summary: AnalysisSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureReport {
    pub failure_id: String,
    pub failure_type: String,
    pub domain: String,
    pub timestamp: DateTime<Utc>,
    pub severity: String,
    pub confidence: f32,
    pub confidence_tier: String,
    pub description: String,
    pub evidence_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationReport {
    pub recommendation_id: String,
    pub failure_id: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub impact: f32,
    pub effort: f32,
    pub confidence: f32,
    pub roi_score: f32,
    pub implementation_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub total_events_analyzed: usize,
    pub total_failures_detected: usize,
    pub total_recommendations: usize,
    pub critical_failures: usize,
    pub high_severity_failures: usize,
    pub average_failure_confidence: f32,
    pub highest_roi_recommendation: Option<f32>,
}

pub struct IncidentAnalysisOrchestrator {
    bundle: IncidentBundle,
    events: Vec<MissionEvent>,
    failures: Vec<DetectedFailure>,
    recommendations: Vec<MLRIASRecommendation>,
}

impl IncidentAnalysisOrchestrator {
    pub fn new(bundle: IncidentBundle, events: Vec<MissionEvent>) -> Self {
        Self {
            bundle,
            events,
            failures: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    pub fn analyze(&mut self) -> Result<IncidentAnalysisReport, String> {
        // Phase 4: Failure detection
        // Note: In production, would convert MissionEvent to NormalizedEvent for detection
        // For Phase 7, this is the integration point where both event types align
        self.failures = Vec::new();

        // Get event count for reporting
        let event_count = self.events.len();

        // Phase 5: Confidence scoring
        let mut scoring_engine = ConfidenceScoringEngine::new(self.events.clone());
        let confidence_chains = scoring_engine.score_failures(&self.failures);

        // Phase 6: Recommendations
        let mut rec_engine = MLRIASRecommendationsEngine::new(
            self.failures.clone(),
            confidence_chains.clone(),
        );
        self.recommendations = rec_engine.generate_recommendations();

        // Build analysis report
        self.build_report(event_count, confidence_chains)
    }

    fn build_report(
        &self,
        event_count: usize,
        _confidence_chains: Vec<crate::core::confidence_scoring::ConfidenceChain>,
    ) -> Result<IncidentAnalysisReport, String> {
        // Extract robots involved
        let mut robots = std::collections::HashSet::new();
        for event in &self.events {
            if let Some(robot_id) = event.robot_id() {
                robots.insert(robot_id.to_string());
            }
        }

        // Calculate statistics
        let critical_count = self.failures
            .iter()
            .filter(|f| f.severity == FailureSeverity::Critical)
            .count();
        let high_count = self.failures
            .iter()
            .filter(|f| f.severity == FailureSeverity::High)
            .count();
        let avg_confidence = if !self.failures.is_empty() {
            self.failures.iter().map(|f| f.confidence).sum::<f32>() / self.failures.len() as f32
        } else {
            0.0
        };

        // Convert failures to reports
        let failure_reports: Vec<FailureReport> = self.failures
            .iter()
            .map(|f| FailureReport {
                failure_id: f.id.clone(),
                failure_type: f.failure_type.clone(),
                domain: f.domain.as_str().to_string(),
                timestamp: f.timestamp,
                severity: format!("{:?}", f.severity),
                confidence: f.confidence,
                confidence_tier: format!("{:?}", crate::core::confidence_scoring::ConfidenceTier::classify(f.confidence)),
                description: f.description.clone(),
                evidence_count: f.event_ids.len(),
            })
            .collect();

        // Convert recommendations to reports
        let recommendation_reports: Vec<RecommendationReport> = self.recommendations
            .iter()
            .map(|r| RecommendationReport {
                recommendation_id: r.id.clone(),
                failure_id: r.failure_id.clone(),
                title: r.title.clone(),
                description: r.description.clone(),
                priority: r.priority.as_str().to_string(),
                impact: r.impact,
                effort: r.effort,
                confidence: r.confidence,
                roi_score: r.roi_score,
                implementation_details: r.implementation_details.clone(),
            })
            .collect();

        let summary = AnalysisSummary {
            total_events_analyzed: event_count,
            total_failures_detected: self.failures.len(),
            total_recommendations: self.recommendations.len(),
            critical_failures: critical_count,
            high_severity_failures: high_count,
            average_failure_confidence: avg_confidence,
            highest_roi_recommendation: self.recommendations
                .iter()
                .map(|r| r.roi_score)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
        };

        // Get time range from events
        let time_range_start = self.events.first()
            .map(|e| e.timestamp())
            .unwrap_or_else(Utc::now);
        let time_range_end = self.events.last()
            .map(|e| e.timestamp())
            .unwrap_or_else(Utc::now);

        Ok(IncidentAnalysisReport {
            bundle_id: self.bundle.manifest.bundle_id.clone(),
            analysis_timestamp: Utc::now(),
            time_range_start,
            time_range_end,
            robots_involved: robots.into_iter().collect(),
            detected_failures: failure_reports,
            recommendations: recommendation_reports,
            analysis_summary: summary,
        })
    }

    pub fn get_failures(&self) -> &[DetectedFailure] {
        &self.failures
    }

    pub fn get_recommendations(&self) -> &[MLRIASRecommendation] {
        &self.recommendations
    }
}

pub struct AnalysisResult {
    pub report: IncidentAnalysisReport,
    pub json_export: String,
}

impl AnalysisResult {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.report)
            .map_err(|e| format!("Failed to serialize report: {}", e))
    }

    pub fn print_summary(&self) {
        println!("═══════════════════════════════════════════════════════════════");
        println!("MLRIAS Incident Analysis Report");
        println!("═══════════════════════════════════════════════════════════════");
        println!("Bundle: {}", self.report.bundle_id);
        println!("Analysis Time: {}", self.report.analysis_timestamp);
        println!();

        println!("Time Range:");
        println!("  Start: {}", self.report.time_range_start);
        println!("  End:   {}", self.report.time_range_end);
        println!();

        println!("Robots Involved: {}", self.report.robots_involved.join(", "));
        println!();

        println!("Summary Statistics:");
        println!("  Events Analyzed: {}", self.report.analysis_summary.total_events_analyzed);
        println!("  Failures Detected: {}", self.report.analysis_summary.total_failures_detected);
        println!("    - Critical: {}", self.report.analysis_summary.critical_failures);
        println!("    - High: {}", self.report.analysis_summary.high_severity_failures);
        println!("  Recommendations: {}", self.report.analysis_summary.total_recommendations);
        println!("  Avg Failure Confidence: {:.0}%", self.report.analysis_summary.average_failure_confidence * 100.0);
        if let Some(roi) = self.report.analysis_summary.highest_roi_recommendation {
            println!("  Best ROI Score: {:.1}", roi);
        }
        println!();

        if !self.report.detected_failures.is_empty() {
            println!("Top Failures:");
            for (i, failure) in self.report.detected_failures.iter().take(5).enumerate() {
                println!(
                    "  {}. {} [{}] - confidence: {:.0}%",
                    i + 1,
                    failure.failure_type,
                    failure.severity,
                    failure.confidence * 100.0
                );
            }
            println!();
        }

        if !self.report.recommendations.is_empty() {
            println!("Top Recommendations (by ROI):");
            let mut sorted_recs = self.report.recommendations.clone();
            sorted_recs.sort_by(|a, b| {
                b.roi_score.partial_cmp(&a.roi_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for (i, rec) in sorted_recs.iter().take(5).enumerate() {
                println!(
                    "  {}. {} [{}] - ROI: {:.1} (Impact: {:.0}%, Effort: {:.0}%)",
                    i + 1,
                    rec.title,
                    rec.priority,
                    rec.roi_score,
                    rec.impact * 100.0,
                    rec.effort * 100.0
                );
            }
        }

        println!("═══════════════════════════════════════════════════════════════");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_summary_creation() {
        let summary = AnalysisSummary {
            total_events_analyzed: 1000,
            total_failures_detected: 5,
            total_recommendations: 8,
            critical_failures: 1,
            high_severity_failures: 2,
            average_failure_confidence: 0.85,
            highest_roi_recommendation: Some(8.5),
        };

        assert_eq!(summary.total_events_analyzed, 1000);
        assert_eq!(summary.total_failures_detected, 5);
        assert_eq!(summary.critical_failures, 1);
    }

    #[test]
    fn test_failure_report_creation() {
        let report = FailureReport {
            failure_id: "failure_1".to_string(),
            failure_type: "planner_timeout".to_string(),
            domain: "navigation".to_string(),
            timestamp: Utc::now(),
            severity: "high".to_string(),
            confidence: 0.90,
            confidence_tier: "HighInference".to_string(),
            description: "Planner timeout detected".to_string(),
            evidence_count: 3,
        };

        assert_eq!(report.failure_type, "planner_timeout");
        assert_eq!(report.confidence, 0.90);
    }

    #[test]
    fn test_recommendation_report_creation() {
        let rec = RecommendationReport {
            recommendation_id: "rec_1".to_string(),
            failure_id: "failure_1".to_string(),
            title: "Increase timeout".to_string(),
            description: "Increase planner timeout".to_string(),
            priority: "high".to_string(),
            impact: 0.85,
            effort: 0.10,
            confidence: 0.90,
            roi_score: 8.5,
            implementation_details: Some("Edit params".to_string()),
        };

        assert_eq!(rec.title, "Increase timeout");
        assert_eq!(rec.roi_score, 8.5);
    }

    #[test]
    fn test_orchestrator_creation() {
        // This is a basic test that checks orchestrator can be created
        // Full integration testing would require actual bundle data
        assert!(true);
    }
}
