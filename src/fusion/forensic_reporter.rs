//! Forensic Report Generation
//!
//! Produces comprehensive forensic investigation reports combining:
//! - RGB-Thermal fusion analysis
//! - Invisible person detection
//! - Missed detection analysis
//! - Root cause analysis
//! - Safety critical incident reports

use crate::fusion::rgb_thermal_fusion::FusionStatistics;
use crate::fusion::invisible_person_detector::InvisiblePersonSummary;
use serde::{Deserialize, Serialize};

/// Root cause analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    /// What happened
    pub observation: String,
    /// When it happened
    pub timestamp: f32,
    /// Immediate cause
    pub immediate_cause: String,
    /// Contributing factors
    pub contributing_factors: Vec<String>,
    /// Sensor failures (if any)
    pub sensor_failures: Vec<String>,
    /// Model failures (if any)
    pub model_failures: Vec<String>,
    /// Environmental factors
    pub environmental_factors: Vec<String>,
    /// Could fusion have prevented this?
    pub fusion_prevention_potential: bool,
}

/// Sensor disagreement finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorDisagreementFinding {
    /// Location of disagreement
    pub location: (f32, f32),
    /// RGB assessment
    pub rgb_assessment: String,
    /// Thermal assessment
    pub thermal_assessment: String,
    /// Resolution (which was correct)
    pub resolution: String,
}

/// Missed detection incident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissedDetectionIncident {
    /// What was missed
    pub object_type: String,
    /// When it should have been detected (RGB)
    pub rgb_miss_timestamp: f32,
    /// When it was detectable (thermal)
    pub thermal_detect_timestamp: f32,
    /// Location
    pub location: (f32, f32),
    /// Detection latency (RGB vs thermal)
    pub detection_latency_sec: f32,
    /// Why RGB failed
    pub rgb_failure_reason: String,
    /// Thermal evidence
    pub thermal_evidence: String,
    /// Safety criticality (0-1)
    pub safety_criticality: f32,
}

/// Comprehensive forensic report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicReport {
    /// Mission ID
    pub mission_id: String,
    /// Analysis timestamp
    pub analysis_timestamp: f32,
    /// Mission start time
    pub mission_start: f32,
    /// Mission end time
    pub mission_end: f32,
    /// Fusion statistics
    pub fusion_stats: Option<FusionStatistics>,
    /// Invisible person summary
    pub invisible_persons: Option<InvisiblePersonSummary>,
    /// Root causes
    pub root_causes: Vec<RootCauseAnalysis>,
    /// Missed detections
    pub missed_detections: Vec<MissedDetectionIncident>,
    /// Sensor disagreements
    pub sensor_disagreements: Vec<SensorDisagreementFinding>,
    /// Safety-critical findings
    pub safety_critical_findings: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Key insights
    pub insights: Vec<String>,
}

/// Forensic report generator
pub struct ForensicReporter {
    /// Current report being built
    pub report: ForensicReport,
}

impl ForensicReporter {
    /// Create new reporter for mission
    pub fn new(mission_id: &str, mission_start: f32, mission_end: f32) -> Self {
        ForensicReporter {
            report: ForensicReport {
                mission_id: mission_id.to_string(),
                analysis_timestamp: mission_end,
                mission_start,
                mission_end,
                fusion_stats: None,
                invisible_persons: None,
                root_causes: Vec::new(),
                missed_detections: Vec::new(),
                sensor_disagreements: Vec::new(),
                safety_critical_findings: Vec::new(),
                recommendations: Vec::new(),
                insights: Vec::new(),
            },
        }
    }

    /// Add fusion statistics
    pub fn add_fusion_stats(&mut self, stats: FusionStatistics) {
        self.report.fusion_stats = Some(stats);
    }

    /// Add invisible person findings
    pub fn add_invisible_persons(&mut self, summary: InvisiblePersonSummary) {
        self.report.invisible_persons = Some(summary);
    }

    /// Add root cause analysis
    pub fn add_root_cause(&mut self, rca: RootCauseAnalysis) {
        self.report.root_causes.push(rca);
    }

    /// Add missed detection
    pub fn add_missed_detection(&mut self, incident: MissedDetectionIncident) {
        if incident.safety_criticality > 0.7 {
            self.report
                .safety_critical_findings
                .push(format!(
                    "{} missed at ({:.0}, {:.0}), {:.1}s latency",
                    incident.object_type, incident.location.0, incident.location.1, incident.detection_latency_sec
                ));
        }
        self.report.missed_detections.push(incident);
    }

    /// Add sensor disagreement
    pub fn add_disagreement(&mut self, disagreement: SensorDisagreementFinding) {
        self.report.sensor_disagreements.push(disagreement);
    }

    /// Generate executive summary
    pub fn generate_executive_summary(&self) -> String {
        let mut summary = String::from("FORENSIC INVESTIGATION EXECUTIVE SUMMARY\n");
        summary.push_str("==========================================\n\n");

        summary.push_str(&format!(
            "Mission: {}\n",
            self.report.mission_id
        ));
        summary.push_str(&format!(
            "Duration: {:.1}s ({:.1}s to {:.1}s)\n\n",
            self.report.mission_end - self.report.mission_start,
            self.report.mission_start,
            self.report.mission_end
        ));

        if let Some(stats) = &self.report.fusion_stats {
            summary.push_str("SENSOR PERFORMANCE:\n");
            summary.push_str(&format!(
                "  RGB Detections: {}\n",
                stats.rgb_detections
            ));
            summary.push_str(&format!(
                "  Thermal-Only Detections: {}\n",
                stats.thermal_only_detections
            ));
            summary.push_str(&format!(
                "  RGB Miss Rate: {:.1}%\n",
                stats.rgb_miss_rate * 100.0
            ));
            summary.push_str(&format!(
                "  Confidence Improvement (Fusion): +{:.1}%\n\n",
                stats.confidence_improvement * 100.0
            ));
        }

        if let Some(inv_persons) = &self.report.invisible_persons {
            summary.push_str("INVISIBLE PERSONS:\n");
            summary.push_str(&format!(
                "  Total Detected: {}\n",
                inv_persons.total_invisible_persons
            ));
            summary.push_str(&format!(
                "  High Confidence: {}\n",
                inv_persons.high_confidence_detections
            ));
            summary.push_str(&format!(
                "  Fusion Could Have Improved: {}\n\n",
                inv_persons.fusion_improvement_potential
            ));
        }

        summary.push_str(&format!(
            "MISSED DETECTIONS: {}\n",
            self.report.missed_detections.len()
        ));
        summary.push_str(&format!(
            "SAFETY-CRITICAL FINDINGS: {}\n\n",
            self.report.safety_critical_findings.len()
        ));

        if !self.report.safety_critical_findings.is_empty() {
            summary.push_str("CRITICAL ISSUES:\n");
            for finding in &self.report.safety_critical_findings {
                summary.push_str(&format!("  ⚠ {}\n", finding));
            }
            summary.push_str("\n");
        }

        if !self.report.recommendations.is_empty() {
            summary.push_str("TOP RECOMMENDATIONS:\n");
            for (idx, rec) in self.report.recommendations.iter().take(3).enumerate() {
                summary.push_str(&format!("  {}. {}\n", idx + 1, rec));
            }
        }

        summary
    }

    /// Generate detailed forensic report
    pub fn generate_detailed_report(&self) -> String {
        let mut report = self.generate_executive_summary();
        report.push_str("\n\n");
        report.push_str("DETAILED FORENSIC ANALYSIS\n");
        report.push_str("==========================\n\n");

        // Missed detections section
        if !self.report.missed_detections.is_empty() {
            report.push_str("MISSED DETECTION ANALYSIS:\n\n");
            for incident in &self.report.missed_detections {
                report.push_str(&format!(
                    "{}:\n",
                    incident.object_type
                ));
                report.push_str(&format!(
                    "  Location: ({:.0}, {:.0})\n",
                    incident.location.0, incident.location.1
                ));
                report.push_str(&format!(
                    "  Detection Latency: RGB miss at {:.1}s, Thermal at {:.1}s ({:.1}s gap)\n",
                    incident.rgb_miss_timestamp, incident.thermal_detect_timestamp, incident.detection_latency_sec
                ));
                report.push_str(&format!(
                    "  RGB Failure: {}\n",
                    incident.rgb_failure_reason
                ));
                report.push_str(&format!(
                    "  Thermal Evidence: {}\n",
                    incident.thermal_evidence
                ));
                report.push_str(&format!(
                    "  Safety Criticality: {:.0}%\n\n",
                    incident.safety_criticality * 100.0
                ));
            }
        }

        // Root cause section
        if !self.report.root_causes.is_empty() {
            report.push_str("ROOT CAUSE ANALYSIS:\n\n");
            for rca in &self.report.root_causes {
                report.push_str(&format!(
                    "Event: {}\n",
                    rca.observation
                ));
                report.push_str(&format!(
                    "  Immediate Cause: {}\n",
                    rca.immediate_cause
                ));
                if !rca.contributing_factors.is_empty() {
                    report.push_str("  Contributing Factors:\n");
                    for factor in &rca.contributing_factors {
                        report.push_str(&format!("    • {}\n", factor));
                    }
                }
                if rca.fusion_prevention_potential {
                    report.push_str("  → Fusion could have prevented this\n");
                }
                report.push_str("\n");
            }
        }

        // Recommendations
        if !self.report.recommendations.is_empty() {
            report.push_str("RECOMMENDATIONS FOR FUTURE SYSTEMS:\n\n");
            for (idx, rec) in self.report.recommendations.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", idx + 1, rec));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forensic_reporter_creation() {
        let reporter = ForensicReporter::new("mission_001", 100.0, 200.0);
        assert_eq!(reporter.report.mission_id, "mission_001");
    }

    #[test]
    fn test_add_root_cause() {
        let mut reporter = ForensicReporter::new("mission_001", 100.0, 200.0);
        let rca = RootCauseAnalysis {
            observation: "Robot collision".to_string(),
            timestamp: 150.0,
            immediate_cause: "Person not detected".to_string(),
            contributing_factors: vec!["low_light".to_string()],
            sensor_failures: vec![],
            model_failures: vec![],
            environmental_factors: vec!["darkness".to_string()],
            fusion_prevention_potential: true,
        };
        reporter.add_root_cause(rca);
        assert_eq!(reporter.report.root_causes.len(), 1);
    }

    #[test]
    fn test_executive_summary_generation() {
        let reporter = ForensicReporter::new("mission_001", 100.0, 200.0);
        let summary = reporter.generate_executive_summary();
        assert!(summary.contains("FORENSIC INVESTIGATION"));
    }

    #[test]
    fn test_detailed_report_generation() {
        let reporter = ForensicReporter::new("mission_001", 100.0, 200.0);
        let report = reporter.generate_detailed_report();
        assert!(report.contains("DETAILED FORENSIC ANALYSIS"));
    }
}
