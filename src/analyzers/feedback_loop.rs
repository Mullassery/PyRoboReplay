//! Feedback Loop for Gap Learning
//!
//! Records detected gaps, collects human feedback, and learns from verification.

use crate::analyzers::{MissionAnalysisData, RealityGapFinding};
use crate::analyzers::aggregation::ConsolidatedFinding;
use crate::analyzers::historical::{HistoricalDatabase, FindingRecord};
use std::collections::HashMap;

/// Feedback event from human verification
#[derive(Debug, Clone)]
pub enum FeedbackEvent {
    /// Verified correct: this was the actual root cause
    VerifiedCorrect(String), // root_cause
    /// Partially correct: contributing factor but not sole cause
    PartiallyCorrect(String), // actual_primary_cause
    /// Incorrect: wrong diagnosis
    Incorrect(String), // actual_root_cause
    /// Inconclusive: unable to determine
    Inconclusive,
}

/// Feedback recording for a single finding
#[derive(Debug, Clone)]
pub struct FindingFeedback {
    pub finding_id: usize,
    pub mission_id: String,
    pub feedback_event: FeedbackEvent,
    pub feedback_timestamp: f32,
    pub additional_notes: String,
}

/// Feedback loop manager: record findings and learn from feedback
pub struct FeedbackLoopManager {
    database: HistoricalDatabase,
    pending_feedback: Vec<FindingFeedback>,
}

impl FeedbackLoopManager {
    /// Create new feedback loop manager
    pub fn new() -> Self {
        FeedbackLoopManager {
            database: HistoricalDatabase::new(),
            pending_feedback: Vec::new(),
        }
    }

    /// Record raw findings from detector analysis
    pub fn record_findings(
        &mut self,
        findings: &[RealityGapFinding],
        mission: &MissionAnalysisData,
    ) -> Vec<usize> {
        let mut finding_ids = Vec::new();

        for finding in findings {
            let id = self.database.record_finding(finding, mission);
            finding_ids.push(id);
        }

        finding_ids
    }

    /// Record consolidated findings (fused from multiple detectors)
    pub fn record_consolidated_findings(
        &mut self,
        consolidated: &[ConsolidatedFinding],
        mission: &MissionAnalysisData,
    ) -> Vec<usize> {
        let mut finding_ids = Vec::new();

        for c in consolidated {
            // Use primary finding for recording (highest confidence component)
            if let Some(primary) = c
                .component_findings
                .iter()
                .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            {
                let id = self.database.record_finding(primary, mission);
                finding_ids.push(id);
            }
        }

        finding_ids
    }

    /// Record mission result
    pub fn record_mission(&mut self, mission: &MissionAnalysisData, success: bool) -> String {
        self.database.record_mission(mission, success)
    }

    /// Submit feedback for a specific finding
    pub fn submit_feedback(&mut self, feedback: FindingFeedback) {
        let root_cause = match &feedback.feedback_event {
            FeedbackEvent::VerifiedCorrect(rc) => rc.clone(),
            FeedbackEvent::PartiallyCorrect(rc) => rc.clone(),
            FeedbackEvent::Incorrect(rc) => rc.clone(),
            FeedbackEvent::Inconclusive => "Unknown".to_string(),
        };

        self.database
            .verify_finding(feedback.finding_id, root_cause);
        self.pending_feedback.push(feedback);
    }

    /// Get feedback summary by gap category
    pub fn feedback_summary(&self) -> HashMap<String, FeedbackStats> {
        let mut stats: HashMap<String, FeedbackStats> = HashMap::new();

        for feedback in &self.pending_feedback {
            let key = format!("{:?}", feedback.feedback_event);
            let entry = stats.entry(key).or_insert_with(FeedbackStats::default);
            entry.count += 1;

            match &feedback.feedback_event {
                FeedbackEvent::VerifiedCorrect(_) => entry.correct += 1,
                FeedbackEvent::PartiallyCorrect(_) => entry.partial += 1,
                FeedbackEvent::Incorrect(_) => entry.incorrect += 1,
                FeedbackEvent::Inconclusive => entry.inconclusive += 1,
            }
        }

        stats
    }

    /// Get accuracy metric: (correct + partial*0.5) / total
    pub fn feedback_accuracy(&self) -> f32 {
        if self.pending_feedback.is_empty() {
            return 0.0;
        }

        let correct = self
            .pending_feedback
            .iter()
            .filter(|f| matches!(f.feedback_event, FeedbackEvent::VerifiedCorrect(_)))
            .count() as f32;

        let partial = self
            .pending_feedback
            .iter()
            .filter(|f| matches!(f.feedback_event, FeedbackEvent::PartiallyCorrect(_)))
            .count() as f32;

        (correct + partial * 0.5) / self.pending_feedback.len() as f32
    }

    /// Get category accuracy: how accurate are predictions for each gap type?
    pub fn category_accuracy(&self) -> HashMap<String, f32> {
        let mut by_category: HashMap<String, (usize, usize)> = HashMap::new(); // (total, correct)

        for feedback in &self.pending_feedback {
            let cat = feedback.mission_id.clone(); // Would use category from finding in real impl
            let entry = by_category.entry(cat).or_insert((0, 0));
            entry.0 += 1;

            if matches!(
                feedback.feedback_event,
                FeedbackEvent::VerifiedCorrect(_) | FeedbackEvent::PartiallyCorrect(_)
            ) {
                entry.1 += 1;
            }
        }

        by_category
            .into_iter()
            .map(|(cat, (total, correct))| (cat, correct as f32 / total as f32))
            .collect()
    }

    /// Access to historical database for queries
    pub fn database(&self) -> &HistoricalDatabase {
        &self.database
    }

    /// Mutable access for learning algorithms
    pub fn database_mut(&mut self) -> &mut HistoricalDatabase {
        &mut self.database
    }

    /// Get pending feedback count
    pub fn pending_feedback_count(&self) -> usize {
        self.pending_feedback.len()
    }

    /// Clear pending feedback (e.g., after processing by learning algorithm)
    pub fn clear_pending_feedback(&mut self) {
        self.pending_feedback.clear();
    }
}

impl Default for FeedbackLoopManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for feedback events
#[derive(Debug, Clone, Default)]
pub struct FeedbackStats {
    pub count: usize,
    pub correct: usize,
    pub partial: usize,
    pub incorrect: usize,
    pub inconclusive: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzers::{RealityDomain, Severity};
    use std::collections::HashMap;

    fn create_test_mission() -> MissionAnalysisData {
        MissionAnalysisData {
            mission_id: "test_mission".to_string(),
            duration_sec: 600.0,
            robot_type: "mobile_robot".to_string(),
            control_messages: vec![],
            joint_states: vec![],
            odometry_messages: vec![],
            camera_frames: vec![],
            lidar_scans: vec![],
            imu_measurements: vec![],
            encoder_data: vec![],
            motor_currents: vec![],
            thermal_readings: vec![],
            battery_data: vec![],
            detection_results: vec![],
            perception_errors: vec![],
            message_timestamps: vec![],
        }
    }

    fn create_test_finding() -> RealityGapFinding {
        RealityGapFinding {
            domain: RealityDomain::Physical,
            category: "Mechanical Degradation".to_string(),
            finding_type: "Response Time Degradation".to_string(),
            severity: Severity::Medium,
            confidence: 0.8,
            reality_gap_score: 0.75,
            description: "Test finding".to_string(),
            evidence: vec![],
            metrics: HashMap::new(),
            sim_recreation_suggestion: "Add wear simulation".to_string(),
            remediation: "Replace bearings".to_string(),
            detection_time_sec: None,
        }
    }

    #[test]
    fn test_manager_creation() {
        let _manager = FeedbackLoopManager::new();
    }

    #[test]
    fn test_record_findings() {
        let mut manager = FeedbackLoopManager::new();
        let mission = create_test_mission();
        let findings = vec![create_test_finding()];

        let ids = manager.record_findings(&findings, &mission);
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], 0); // First finding is ID 0
    }

    #[test]
    fn test_record_mission() {
        let mut manager = FeedbackLoopManager::new();
        let mission = create_test_mission();

        let mission_id = manager.record_mission(&mission, true);
        assert_eq!(mission_id, "test_mission");
    }

    #[test]
    fn test_submit_feedback() {
        let mut manager = FeedbackLoopManager::new();
        let mission = create_test_mission();
        let findings = vec![create_test_finding()];

        let ids = manager.record_findings(&findings, &mission);

        let feedback = FindingFeedback {
            finding_id: ids[0],
            mission_id: "test_mission".to_string(),
            feedback_event: FeedbackEvent::VerifiedCorrect("Bearing Wear".to_string()),
            feedback_timestamp: 100.0,
            additional_notes: "Confirmed by visual inspection".to_string(),
        };

        manager.submit_feedback(feedback);
        assert_eq!(manager.pending_feedback_count(), 1);
    }

    #[test]
    fn test_feedback_accuracy() {
        let mut manager = FeedbackLoopManager::new();

        // Add 2 correct, 1 partial, 1 incorrect
        manager.pending_feedback.push(FindingFeedback {
            finding_id: 0,
            mission_id: "m1".to_string(),
            feedback_event: FeedbackEvent::VerifiedCorrect("Root".to_string()),
            feedback_timestamp: 0.0,
            additional_notes: String::new(),
        });

        manager.pending_feedback.push(FindingFeedback {
            finding_id: 1,
            mission_id: "m2".to_string(),
            feedback_event: FeedbackEvent::VerifiedCorrect("Root".to_string()),
            feedback_timestamp: 0.0,
            additional_notes: String::new(),
        });

        manager.pending_feedback.push(FindingFeedback {
            finding_id: 2,
            mission_id: "m3".to_string(),
            feedback_event: FeedbackEvent::PartiallyCorrect("Root".to_string()),
            feedback_timestamp: 0.0,
            additional_notes: String::new(),
        });

        manager.pending_feedback.push(FindingFeedback {
            finding_id: 3,
            mission_id: "m4".to_string(),
            feedback_event: FeedbackEvent::Incorrect("Root".to_string()),
            feedback_timestamp: 0.0,
            additional_notes: String::new(),
        });

        // Accuracy: (2 + 0.5) / 4 = 0.625
        let accuracy = manager.feedback_accuracy();
        assert!((accuracy - 0.625).abs() < 0.01);
    }

    #[test]
    fn test_feedback_summary() {
        let mut manager = FeedbackLoopManager::new();

        manager.pending_feedback.push(FindingFeedback {
            finding_id: 0,
            mission_id: "m1".to_string(),
            feedback_event: FeedbackEvent::VerifiedCorrect("Root".to_string()),
            feedback_timestamp: 0.0,
            additional_notes: String::new(),
        });

        let summary = manager.feedback_summary();
        assert!(summary.len() > 0);
    }

    #[test]
    fn test_clear_pending_feedback() {
        let mut manager = FeedbackLoopManager::new();

        manager.pending_feedback.push(FindingFeedback {
            finding_id: 0,
            mission_id: "m1".to_string(),
            feedback_event: FeedbackEvent::VerifiedCorrect("Root".to_string()),
            feedback_timestamp: 0.0,
            additional_notes: String::new(),
        });

        assert_eq!(manager.pending_feedback_count(), 1);
        manager.clear_pending_feedback();
        assert_eq!(manager.pending_feedback_count(), 0);
    }
}
