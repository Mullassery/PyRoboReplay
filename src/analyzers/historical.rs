//! Historical Findings Database
//!
//! Tracks gap findings across fleet to enable learning and improve scoring.

use crate::analyzers::{RealityGapFinding, MissionAnalysisData};
use std::collections::HashMap;

/// Historical findings database
pub struct HistoricalDatabase {
    // In-memory store for tracking (real implementation would use SQLite)
    findings: Vec<FindingRecord>,
    missions: Vec<MissionRecord>,
}

/// A recorded finding
#[derive(Debug, Clone)]
pub struct FindingRecord {
    pub id: usize,
    pub mission_id: String,
    pub robot_id: String,
    pub timestamp: f32,
    pub finding_type: String,
    pub category: String,
    pub reality_gap_score: f32,
    pub confidence: f32,
    pub severity: String,
    pub verified: bool,
    pub actual_root_cause: Option<String>,
}

/// A recorded mission
#[derive(Debug, Clone)]
pub struct MissionRecord {
    pub mission_id: String,
    pub robot_id: String,
    pub robot_type: String,
    pub timestamp: f32,
    pub duration_seconds: f32,
    pub success: bool,
}

impl HistoricalDatabase {
    /// Create a new historical database
    pub fn new() -> Self {
        HistoricalDatabase {
            findings: Vec::new(),
            missions: Vec::new(),
        }
    }

    /// Record a finding from a mission
    pub fn record_finding(
        &mut self,
        finding: &RealityGapFinding,
        mission: &MissionAnalysisData,
    ) -> usize {
        let id = self.findings.len();

        self.findings.push(FindingRecord {
            id,
            mission_id: mission.mission_id.clone(),
            robot_id: "unknown".to_string(), // Would extract from mission metadata
            timestamp: finding.detection_time_sec.unwrap_or(0.0),
            finding_type: finding.finding_type.clone(),
            category: finding.category.clone(),
            reality_gap_score: finding.reality_gap_score,
            confidence: finding.confidence,
            severity: format!("{:?}", finding.severity),
            verified: false,
            actual_root_cause: None,
        });

        id
    }

    /// Record a mission
    pub fn record_mission(&mut self, mission: &MissionAnalysisData, success: bool) -> String {
        let mission_id = mission.mission_id.clone();

        self.missions.push(MissionRecord {
            mission_id: mission_id.clone(),
            robot_id: "unknown".to_string(),
            robot_type: mission.robot_type.clone(),
            timestamp: 0.0, // Would extract from mission
            duration_seconds: mission.duration_sec,
            success,
        });

        mission_id
    }

    /// Get gap frequency: what % of missions have this gap?
    pub fn gap_frequency(&self, category: &str, robot_type: &str) -> f32 {
        let robot_missions = self
            .missions
            .iter()
            .filter(|m| m.robot_type == robot_type)
            .count();

        if robot_missions == 0 {
            return 0.0;
        }

        let missions_with_gap = self
            .missions
            .iter()
            .filter(|m| {
                m.robot_type == robot_type
                    && self
                        .findings
                        .iter()
                        .any(|f| f.mission_id == m.mission_id && f.category == category)
            })
            .count();

        missions_with_gap as f32 / robot_missions as f32
    }

    /// Check if this gap is known for this robot type
    pub fn is_known_gap_for_robot_type(&self, category: &str, robot_type: &str) -> bool {
        self.gap_frequency(category, robot_type) > 0.3
    }

    /// Get supporting cases for a gap category
    pub fn supporting_cases(&self, category: &str) -> usize {
        self.findings
            .iter()
            .filter(|f| f.category == category && f.verified)
            .count()
    }

    /// Calculate score accuracy: what % of high-confidence scores were correct?
    pub fn score_accuracy(&self) -> f32 {
        let high_confidence = self
            .findings
            .iter()
            .filter(|f| f.confidence > 0.7 && f.verified)
            .count();

        if high_confidence == 0 {
            return 0.0;
        }

        let correct = self
            .findings
            .iter()
            .filter(|f| {
                f.confidence > 0.7 && f.verified && f.actual_root_cause.is_some()
            })
            .count();

        correct as f32 / high_confidence as f32
    }

    /// Get statistics by category
    pub fn category_statistics(&self, category: &str) -> CategoryStats {
        let findings: Vec<_> = self
            .findings
            .iter()
            .filter(|f| f.category == category)
            .collect();

        if findings.is_empty() {
            return CategoryStats::default();
        }

        let avg_gap_score: f32 = findings.iter().map(|f| f.reality_gap_score).sum::<f32>()
            / findings.len() as f32;
        let avg_confidence: f32 = findings.iter().map(|f| f.confidence).sum::<f32>()
            / findings.len() as f32;
        let verified_count = findings.iter().filter(|f| f.verified).count();

        CategoryStats {
            total_findings: findings.len(),
            avg_gap_score,
            avg_confidence,
            verified_count,
            frequency: self.gap_frequency(category, "all"),
        }
    }

    /// Get all findings for a mission
    pub fn mission_findings(&self, mission_id: &str) -> Vec<FindingRecord> {
        self.findings
            .iter()
            .filter(|f| f.mission_id == mission_id)
            .cloned()
            .collect()
    }

    /// Mark a finding as verified with actual root cause
    pub fn verify_finding(&mut self, finding_id: usize, root_cause: String) {
        if let Some(finding) = self.findings.get_mut(finding_id) {
            finding.verified = true;
            finding.actual_root_cause = Some(root_cause);
        }
    }

    /// Get most common gap for a robot type
    pub fn most_common_gap(&self, robot_type: &str) -> Option<(String, usize)> {
        let mut gap_counts: HashMap<String, usize> = HashMap::new();

        for finding in self.findings.iter().filter(|f| {
            self.missions
                .iter()
                .find(|m| m.mission_id == f.mission_id)
                .map(|m| m.robot_type == robot_type)
                .unwrap_or(false)
        }) {
            *gap_counts.entry(finding.category.clone()).or_insert(0) += 1;
        }

        gap_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(cat, count)| (cat, count))
    }

    /// Get trending gaps (increasing frequency)
    pub fn trending_gaps(&self, robot_type: &str, time_window_hours: f32) -> Vec<(String, f32)> {
        // In real implementation, would slice findings by time window
        // For now, return all unique categories with their frequency
        let mut categories: HashMap<String, usize> = HashMap::new();

        for finding in &self.findings {
            *categories.entry(finding.category.clone()).or_insert(0) += 1;
        }

        let total = categories.values().sum::<usize>().max(1);

        categories
            .into_iter()
            .map(|(cat, count)| (cat, count as f32 / total as f32))
            .collect()
    }
}

impl Default for HistoricalDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a gap category
#[derive(Debug, Clone, Default)]
pub struct CategoryStats {
    pub total_findings: usize,
    pub avg_gap_score: f32,
    pub avg_confidence: f32,
    pub verified_count: usize,
    pub frequency: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = HistoricalDatabase::new();
        assert_eq!(db.findings.len(), 0);
        assert_eq!(db.missions.len(), 0);
    }

    #[test]
    fn test_record_mission() {
        let mut db = HistoricalDatabase::new();
        let mission = MissionAnalysisData {
            mission_id: "test_1".to_string(),
            duration_sec: 600.0,
            robot_type: "wheel_robot".to_string(),
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
        };

        db.record_mission(&mission, true);
        assert_eq!(db.missions.len(), 1);
        assert_eq!(db.missions[0].mission_id, "test_1");
    }

    #[test]
    fn test_gap_frequency() {
        let mut db = HistoricalDatabase::new();

        // Record 10 wheel_robot missions
        for i in 0..10 {
            let mission = MissionAnalysisData {
                mission_id: format!("mission_{}", i),
                duration_sec: 600.0,
                robot_type: "wheel_robot".to_string(),
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
            };
            db.record_mission(&mission, true);
        }

        // In real test, would add findings for some missions
        let frequency = db.gap_frequency("Mechanical Degradation", "wheel_robot");
        assert_eq!(frequency, 0.0); // No findings recorded yet
    }

    #[test]
    fn test_category_statistics() {
        let db = HistoricalDatabase::new();
        let stats = db.category_statistics("Mechanical Degradation");
        assert_eq!(stats.total_findings, 0);
    }

    #[test]
    fn test_most_common_gap() {
        let db = HistoricalDatabase::new();
        let gap = db.most_common_gap("wheel_robot");
        assert!(gap.is_none()); // No findings
    }
}
