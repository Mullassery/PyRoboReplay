use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Proximity zone type classification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProximityZoneType {
    SafetyZone,
    WarningZone,
    ProtectiveZone,
    WorkZone,
}

/// Proximity zone event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProximityZoneEvent {
    pub robot_id: String,
    pub timestamp: DateTime<Utc>,
    pub zone_id: String,
    pub zone_type: ProximityZoneType,
    pub distance_m: f32,
    pub action_taken: String,
}

/// Emergency stop event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyStopEvent {
    pub robot_id: String,
    pub timestamp: DateTime<Utc>,
    pub cause: String,
    pub stop_distance_m: f32,
    pub recovery_time_ms: Option<u64>,
}

/// Speed compliance event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedComplianceEvent {
    pub robot_id: String,
    pub timestamp: DateTime<Utc>,
    pub actual_speed_mps: f32,
    pub limit_mps: f32,
    pub compliant: bool,
}

/// Operator presence event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorPresenceEvent {
    pub robot_id: String,
    pub timestamp: DateTime<Utc>,
    pub operator_id: Option<String>,
    pub present: bool,
}

/// Compliance event wrapper enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceEvent {
    ProximityZone(ProximityZoneEvent),
    EmergencyStop(EmergencyStopEvent),
    SpeedCompliance(SpeedComplianceEvent),
    OperatorPresence(OperatorPresenceEvent),
}

impl ComplianceEvent {
    /// Extract timestamp from compliance event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            ComplianceEvent::ProximityZone(e) => e.timestamp,
            ComplianceEvent::EmergencyStop(e) => e.timestamp,
            ComplianceEvent::SpeedCompliance(e) => e.timestamp,
            ComplianceEvent::OperatorPresence(e) => e.timestamp,
        }
    }

    /// Extract robot ID from compliance event
    pub fn robot_id(&self) -> &str {
        match self {
            ComplianceEvent::ProximityZone(e) => &e.robot_id,
            ComplianceEvent::EmergencyStop(e) => &e.robot_id,
            ComplianceEvent::SpeedCompliance(e) => &e.robot_id,
            ComplianceEvent::OperatorPresence(e) => &e.robot_id,
        }
    }
}

/// Compliance violation type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum ViolationType {
    ProximityViolation,
    UnacknowledgedEmergencyStop,
    SpeedLimitExceeded,
    OperatorAbsenceDuringMotion,
}

impl std::fmt::Display for ViolationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationType::ProximityViolation => write!(f, "ProximityViolation"),
            ViolationType::UnacknowledgedEmergencyStop => write!(f, "UnacknowledgedEmergencyStop"),
            ViolationType::SpeedLimitExceeded => write!(f, "SpeedLimitExceeded"),
            ViolationType::OperatorAbsenceDuringMotion => write!(f, "OperatorAbsenceDuringMotion"),
        }
    }
}

/// Compliance violation severity
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, PartialOrd, Ord, Eq)]
pub enum ViolationSeverity {
    Minor,
    Major,
    Critical,
}

/// A compliance violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub violation_id: String,
    pub violation_type: ViolationType,
    pub robot_id: String,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub severity: ViolationSeverity,
}

/// Compliance configuration with thresholds
#[derive(Debug, Clone)]
pub struct ComplianceConfig {
    pub min_proximity_distance_m: f32,
    pub emergency_stop_recovery_timeout_ms: u64,
    pub max_speed_mps: f32,
    pub operator_motion_tolerance_ms: u64,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        ComplianceConfig {
            min_proximity_distance_m: 0.5,
            emergency_stop_recovery_timeout_ms: 30_000,
            max_speed_mps: 2.0,
            operator_motion_tolerance_ms: 1_000,
        }
    }
}

/// Compliance report for a mission (ISO 3691-4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub report_id: Uuid,
    pub mission_id: String,
    pub standard: String,
    pub report_timestamp: DateTime<Utc>,
    pub violations: Vec<ComplianceViolation>,
    pub overall_compliant: bool,
    pub summary: String,
    pub violation_count_by_type: HashMap<String, usize>,
}

/// Generates compliance reports for missions
pub struct ComplianceReportGenerator {
    config: ComplianceConfig,
}

impl ComplianceReportGenerator {
    /// Create a new compliance report generator
    pub fn new(config: ComplianceConfig) -> Self {
        ComplianceReportGenerator { config }
    }

    /// Generate a compliance report for a mission
    pub fn generate_report(
        &self,
        mission_id: &str,
        events: &[ComplianceEvent],
    ) -> ComplianceReport {
        let mut violations = Vec::new();

        violations.extend(self.check_proximity_violations(events));
        violations.extend(self.check_emergency_stop_violations(events));
        violations.extend(self.check_speed_violations(events));
        violations.extend(self.check_operator_presence_violations(events));

        let overall_compliant = violations.is_empty();
        let summary = if overall_compliant {
            "Mission completed with no compliance violations".to_string()
        } else {
            format!(
                "Mission completed with {} compliance violation(s)",
                violations.len()
            )
        };

        let mut violation_count_by_type = HashMap::new();
        for violation in &violations {
            *violation_count_by_type
                .entry(violation.violation_type.to_string())
                .or_insert(0) += 1;
        }

        ComplianceReport {
            report_id: Uuid::new_v4(),
            mission_id: mission_id.to_string(),
            standard: "ISO 3691-4".to_string(),
            report_timestamp: Utc::now(),
            violations,
            overall_compliant,
            summary,
            violation_count_by_type,
        }
    }

    /// Check proximity zone violations
    fn check_proximity_violations(&self, events: &[ComplianceEvent]) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();

        for event in events {
            if let ComplianceEvent::ProximityZone(pz_event) = event {
                if pz_event.distance_m < self.config.min_proximity_distance_m {
                    violations.push(ComplianceViolation {
                        violation_id: format!("prox_{}", Uuid::new_v4()),
                        violation_type: ViolationType::ProximityViolation,
                        robot_id: pz_event.robot_id.clone(),
                        timestamp: pz_event.timestamp,
                        description: format!(
                            "Proximity distance {:.2}m below minimum {:.2}m in zone {}",
                            pz_event.distance_m, self.config.min_proximity_distance_m, pz_event.zone_id
                        ),
                        severity: ViolationSeverity::Critical,
                    });
                }
            }
        }

        violations
    }

    /// Check emergency stop violations
    fn check_emergency_stop_violations(&self, events: &[ComplianceEvent]) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();

        for event in events {
            if let ComplianceEvent::EmergencyStop(es_event) = event {
                if let Some(recovery_time) = es_event.recovery_time_ms {
                    if recovery_time > self.config.emergency_stop_recovery_timeout_ms {
                        violations.push(ComplianceViolation {
                            violation_id: format!("estop_{}", Uuid::new_v4()),
                            violation_type: ViolationType::UnacknowledgedEmergencyStop,
                            robot_id: es_event.robot_id.clone(),
                            timestamp: es_event.timestamp,
                            description: format!(
                                "Emergency stop recovery time {}ms exceeds limit {}ms",
                                recovery_time, self.config.emergency_stop_recovery_timeout_ms
                            ),
                            severity: ViolationSeverity::Major,
                        });
                    }
                }
            }
        }

        violations
    }

    /// Check speed violations
    fn check_speed_violations(&self, events: &[ComplianceEvent]) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();

        for event in events {
            if let ComplianceEvent::SpeedCompliance(speed_event) = event {
                if !speed_event.compliant && speed_event.actual_speed_mps > self.config.max_speed_mps {
                    violations.push(ComplianceViolation {
                        violation_id: format!("speed_{}", Uuid::new_v4()),
                        violation_type: ViolationType::SpeedLimitExceeded,
                        robot_id: speed_event.robot_id.clone(),
                        timestamp: speed_event.timestamp,
                        description: format!(
                            "Speed {:.2} m/s exceeds limit {:.2} m/s",
                            speed_event.actual_speed_mps, self.config.max_speed_mps
                        ),
                        severity: ViolationSeverity::Major,
                    });
                }
            }
        }

        violations
    }

    /// Check operator presence violations
    fn check_operator_presence_violations(&self, events: &[ComplianceEvent]) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();
        let mut robot_absence_start: HashMap<String, DateTime<Utc>> = HashMap::new();

        for event in events {
            if let ComplianceEvent::OperatorPresence(op_event) = event {
                if !op_event.present {
                    robot_absence_start.insert(op_event.robot_id.clone(), op_event.timestamp);
                } else if let Some(absence_start) = robot_absence_start.remove(&op_event.robot_id) {
                    let absence_duration = op_event.timestamp - absence_start;
                    if absence_duration.num_milliseconds() > self.config.operator_motion_tolerance_ms as i64 {
                        violations.push(ComplianceViolation {
                            violation_id: format!("op_absence_{}", Uuid::new_v4()),
                            violation_type: ViolationType::OperatorAbsenceDuringMotion,
                            robot_id: op_event.robot_id.clone(),
                            timestamp: absence_start,
                            description: format!(
                                "Robot operated without operator for {}ms",
                                absence_duration.num_milliseconds()
                            ),
                            severity: ViolationSeverity::Critical,
                        });
                    }
                }
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_generator() -> ComplianceReportGenerator {
        ComplianceReportGenerator::new(ComplianceConfig::default())
    }

    #[test]
    fn test_empty_events_no_violations() {
        let generator = create_generator();
        let report = generator.generate_report("mission_1", &[]);

        assert!(report.overall_compliant);
        assert_eq!(report.violations.len(), 0);
        assert_eq!(report.standard, "ISO 3691-4");
    }

    #[test]
    fn test_proximity_violation_below_minimum() {
        let generator = create_generator();
        let now = Utc::now();

        let events = vec![ComplianceEvent::ProximityZone(ProximityZoneEvent {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            zone_id: "zone_1".to_string(),
            zone_type: ProximityZoneType::SafetyZone,
            distance_m: 0.3,
            action_taken: "Alert".to_string(),
        })];

        let report = generator.generate_report("mission_1", &events);

        assert!(!report.overall_compliant);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].violation_type,
            ViolationType::ProximityViolation
        );
        assert_eq!(report.violations[0].severity, ViolationSeverity::Critical);
    }

    #[test]
    fn test_proximity_violation_at_minimum_not_violated() {
        let generator = create_generator();
        let now = Utc::now();

        let events = vec![ComplianceEvent::ProximityZone(ProximityZoneEvent {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            zone_id: "zone_1".to_string(),
            zone_type: ProximityZoneType::SafetyZone,
            distance_m: 0.5,
            action_taken: "Alert".to_string(),
        })];

        let report = generator.generate_report("mission_1", &events);

        assert!(report.overall_compliant);
        assert_eq!(report.violations.len(), 0);
    }

    #[test]
    fn test_emergency_stop_acknowledged_no_violation() {
        let generator = create_generator();
        let now = Utc::now();

        let events = vec![ComplianceEvent::EmergencyStop(EmergencyStopEvent {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            cause: "Manual trigger".to_string(),
            stop_distance_m: 1.5,
            recovery_time_ms: Some(5_000),
        })];

        let report = generator.generate_report("mission_1", &events);

        assert!(report.overall_compliant);
        assert_eq!(report.violations.len(), 0);
    }

    #[test]
    fn test_emergency_stop_unacknowledged_violation() {
        let generator = create_generator();
        let now = Utc::now();

        let events = vec![ComplianceEvent::EmergencyStop(EmergencyStopEvent {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            cause: "Sensor failure".to_string(),
            stop_distance_m: 2.0,
            recovery_time_ms: Some(40_000),
        })];

        let report = generator.generate_report("mission_1", &events);

        assert!(!report.overall_compliant);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].violation_type,
            ViolationType::UnacknowledgedEmergencyStop
        );
    }

    #[test]
    fn test_speed_over_limit_violation() {
        let generator = create_generator();
        let now = Utc::now();

        let events = vec![ComplianceEvent::SpeedCompliance(SpeedComplianceEvent {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            actual_speed_mps: 2.5,
            limit_mps: 2.0,
            compliant: false,
        })];

        let report = generator.generate_report("mission_1", &events);

        assert!(!report.overall_compliant);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].violation_type, ViolationType::SpeedLimitExceeded);
    }

    #[test]
    fn test_speed_compliant_no_violation() {
        let generator = create_generator();
        let now = Utc::now();

        let events = vec![ComplianceEvent::SpeedCompliance(SpeedComplianceEvent {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            actual_speed_mps: 1.8,
            limit_mps: 2.0,
            compliant: true,
        })];

        let report = generator.generate_report("mission_1", &events);

        assert!(report.overall_compliant);
        assert_eq!(report.violations.len(), 0);
    }

    #[test]
    fn test_operator_present_no_violation() {
        let generator = create_generator();
        let now = Utc::now();

        let events = vec![ComplianceEvent::OperatorPresence(OperatorPresenceEvent {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            operator_id: Some("op_1".to_string()),
            present: true,
        })];

        let report = generator.generate_report("mission_1", &events);

        assert!(report.overall_compliant);
        assert_eq!(report.violations.len(), 0);
    }

    #[test]
    fn test_operator_absent_during_motion_violation() {
        let generator = create_generator();
        let start = Utc::now();

        let events = vec![
            ComplianceEvent::OperatorPresence(OperatorPresenceEvent {
                robot_id: "robot_1".to_string(),
                timestamp: start,
                operator_id: Some("op_1".to_string()),
                present: false,
            }),
            ComplianceEvent::OperatorPresence(OperatorPresenceEvent {
                robot_id: "robot_1".to_string(),
                timestamp: start + chrono::Duration::seconds(2),
                operator_id: Some("op_1".to_string()),
                present: true,
            }),
        ];

        let report = generator.generate_report("mission_1", &events);

        assert!(!report.overall_compliant);
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].violation_type,
            ViolationType::OperatorAbsenceDuringMotion
        );
    }

    #[test]
    fn test_multiple_violations_counted_by_type() {
        let generator = create_generator();
        let now = Utc::now();

        let events = vec![
            ComplianceEvent::SpeedCompliance(SpeedComplianceEvent {
                robot_id: "robot_1".to_string(),
                timestamp: now,
                actual_speed_mps: 2.5,
                limit_mps: 2.0,
                compliant: false,
            }),
            ComplianceEvent::SpeedCompliance(SpeedComplianceEvent {
                robot_id: "robot_1".to_string(),
                timestamp: now + chrono::Duration::seconds(1),
                actual_speed_mps: 3.0,
                limit_mps: 2.0,
                compliant: false,
            }),
        ];

        let report = generator.generate_report("mission_1", &events);

        assert_eq!(report.violations.len(), 2);
        assert_eq!(
            report.violation_count_by_type.get("SpeedLimitExceeded"),
            Some(&2)
        );
    }

    #[test]
    fn test_report_summary_compliant() {
        let generator = create_generator();
        let report = generator.generate_report("mission_1", &[]);

        assert!(report.summary.contains("no compliance violations"));
    }

    #[test]
    fn test_report_summary_non_compliant() {
        let generator = create_generator();
        let now = Utc::now();

        let events = vec![ComplianceEvent::ProximityZone(ProximityZoneEvent {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            zone_id: "zone_1".to_string(),
            zone_type: ProximityZoneType::SafetyZone,
            distance_m: 0.2,
            action_taken: "Alert".to_string(),
        })];

        let report = generator.generate_report("mission_1", &events);

        assert!(report.summary.contains("1 compliance violation"));
    }
}
