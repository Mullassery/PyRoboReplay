use crate::streaming::live_diagnostics::{LiveAlert, AlertSeverity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// SLA violation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlaViolationType {
    NavigationDeadlockExceeded,
    SensorDropoutExceeded,
    CoverageNotMet,
    EmergencyStopLimitExceeded,
    SpeedViolationExceeded,
}

/// SLA violation severity
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, PartialOrd, Eq, Ord)]
pub enum SlaViolationSeverity {
    Medium,
    High,
    Critical,
}

/// SLA contract for a mission type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaContract {
    pub contract_id: Uuid,
    pub mission_type: String,
    pub max_navigation_deadlock_duration_ms: u64,
    pub max_sensor_dropout_duration_ms: u64,
    pub min_coverage_pct: f32,
    pub max_emergency_stops: usize,
    pub max_speed_violations: usize,
}

impl SlaContract {
    /// Create contract with defaults for mission type
    pub fn new(mission_type: &str) -> Self {
        SlaContract {
            contract_id: Uuid::new_v4(),
            mission_type: mission_type.to_string(),
            max_navigation_deadlock_duration_ms: 60_000,
            max_sensor_dropout_duration_ms: 30_000,
            min_coverage_pct: 80.0,
            max_emergency_stops: 5,
            max_speed_violations: 10,
        }
    }
}

/// Individual SLA violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaViolation {
    pub violation_id: Uuid,
    pub contract_id: Uuid,
    pub mission_id: String,
    pub violation_type: SlaViolationType,
    pub occurred_at: DateTime<Utc>,
    pub description: String,
    pub severity: SlaViolationSeverity,
}

/// SLA enforcement report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaEnforcementReport {
    pub report_id: Uuid,
    pub mission_id: String,
    pub contract_id: Uuid,
    pub violations: Vec<SlaViolation>,
    pub overall_compliant: bool,
    pub compliance_score: f32,
    pub generated_at: DateTime<Utc>,
}

/// SLA error types
#[derive(Debug, Error)]
pub enum SlaError {
    #[error("Contract not found: {0}")]
    ContractNotFound(String),
    #[error("Mission already registered: {0}")]
    MissionAlreadyRegistered(String),
    #[error("Mission not found: {0}")]
    MissionNotFound(String),
}

/// Per-mission SLA state tracking
#[derive(Debug, Clone)]
struct MissionSlaState {
    contract: SlaContract,
    mission_id: String,
    started_at: DateTime<Utc>,
    navigation_deadlock_start: Option<DateTime<Utc>>,
    sensor_dropout_start: Option<DateTime<Utc>>,
    emergency_stop_count: usize,
    speed_violation_count: usize,
    violations: Vec<SlaViolation>,
}

/// SLA monitor for tracking mission compliance
pub struct SlaMonitor {
    active_missions: HashMap<String, MissionSlaState>,
    contracts: HashMap<Uuid, SlaContract>,
}

impl SlaMonitor {
    /// Create new SLA monitor
    pub fn new() -> Self {
        SlaMonitor {
            active_missions: HashMap::new(),
            contracts: HashMap::new(),
        }
    }

    /// Register a contract
    pub fn register_contract(&mut self, contract: SlaContract) {
        self.contracts.insert(contract.contract_id, contract);
    }

    /// Start monitoring a mission
    pub fn start_mission(&mut self, mission_id: &str, contract_id: &Uuid) -> Result<(), SlaError> {
        if self.active_missions.contains_key(mission_id) {
            return Err(SlaError::MissionAlreadyRegistered(mission_id.to_string()));
        }

        let contract = self
            .contracts
            .get(contract_id)
            .cloned()
            .ok_or_else(|| SlaError::ContractNotFound(contract_id.to_string()))?;

        let state = MissionSlaState {
            contract,
            mission_id: mission_id.to_string(),
            started_at: Utc::now(),
            navigation_deadlock_start: None,
            sensor_dropout_start: None,
            emergency_stop_count: 0,
            speed_violation_count: 0,
            violations: Vec::new(),
        };

        self.active_missions.insert(mission_id.to_string(), state);
        Ok(())
    }

    /// Check alert for SLA violations
    pub fn check_alert(&mut self, mission_id: &str, alert: &LiveAlert) -> Option<SlaViolation> {
        let state = self.active_missions.get_mut(mission_id)?;

        match alert.event_type.as_str() {
            "navigation_deadlock" => {
                if state.navigation_deadlock_start.is_none() {
                    state.navigation_deadlock_start = Some(Utc::now());
                }

                if let Some(start) = state.navigation_deadlock_start {
                    let duration_ms = (Utc::now() - start).num_milliseconds() as u64;
                    if duration_ms > state.contract.max_navigation_deadlock_duration_ms {
                        let violation = SlaViolation {
                            violation_id: Uuid::new_v4(),
                            contract_id: state.contract.contract_id,
                            mission_id: mission_id.to_string(),
                            violation_type: SlaViolationType::NavigationDeadlockExceeded,
                            occurred_at: Utc::now(),
                            description: format!(
                                "Navigation deadlock exceeded {} ms limit",
                                state.contract.max_navigation_deadlock_duration_ms
                            ),
                            severity: SlaViolationSeverity::Critical,
                        };

                        state.violations.push(violation.clone());
                        return Some(violation);
                    }
                }
            }
            "sensor_dropout" => {
                if state.sensor_dropout_start.is_none() {
                    state.sensor_dropout_start = Some(Utc::now());
                }

                if let Some(start) = state.sensor_dropout_start {
                    let duration_ms = (Utc::now() - start).num_milliseconds() as u64;
                    if duration_ms > state.contract.max_sensor_dropout_duration_ms {
                        let violation = SlaViolation {
                            violation_id: Uuid::new_v4(),
                            contract_id: state.contract.contract_id,
                            mission_id: mission_id.to_string(),
                            violation_type: SlaViolationType::SensorDropoutExceeded,
                            occurred_at: Utc::now(),
                            description: format!(
                                "Sensor dropout exceeded {} ms limit",
                                state.contract.max_sensor_dropout_duration_ms
                            ),
                            severity: SlaViolationSeverity::High,
                        };

                        state.violations.push(violation.clone());
                        return Some(violation);
                    }
                }
            }
            _ => {}
        }

        None
    }

    /// End mission and generate report
    pub fn end_mission(&mut self, mission_id: &str) -> Option<SlaEnforcementReport> {
        let state = self.active_missions.remove(mission_id)?;

        let overall_compliant = state.violations.is_empty();
        let compliance_score = if overall_compliant {
            1.0
        } else {
            let penalty: f32 = state.violations.iter().map(|v| match v.severity {
                SlaViolationSeverity::Critical => 0.3,
                SlaViolationSeverity::High => 0.2,
                SlaViolationSeverity::Medium => 0.1,
            }).sum();
            (1.0 - penalty).max(0.0)
        };

        Some(SlaEnforcementReport {
            report_id: Uuid::new_v4(),
            mission_id: mission_id.to_string(),
            contract_id: state.contract.contract_id,
            violations: state.violations,
            overall_compliant,
            compliance_score,
            generated_at: Utc::now(),
        })
    }
}

impl Default for SlaMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sla_contract_creation() {
        let contract = SlaContract::new("warehouse_delivery");
        assert_eq!(contract.mission_type, "warehouse_delivery");
        assert_eq!(contract.max_emergency_stops, 5);
    }

    #[test]
    fn test_sla_monitor_creation() {
        let monitor = SlaMonitor::new();
        assert_eq!(monitor.active_missions.len(), 0);
    }

    #[test]
    fn test_register_contract() {
        let mut monitor = SlaMonitor::new();
        let contract = SlaContract::new("delivery");
        let contract_id = contract.contract_id;

        monitor.register_contract(contract);
        assert!(monitor.contracts.contains_key(&contract_id));
    }

    #[test]
    fn test_start_mission() {
        let mut monitor = SlaMonitor::new();
        let contract = SlaContract::new("delivery");
        let contract_id = contract.contract_id;

        monitor.register_contract(contract);
        let result = monitor.start_mission("mission_1", &contract_id);

        assert!(result.is_ok());
        assert!(monitor.active_missions.contains_key("mission_1"));
    }

    #[test]
    fn test_deadlock_within_limit() {
        let mut monitor = SlaMonitor::new();
        let contract = SlaContract::new("delivery");
        let contract_id = contract.contract_id;

        monitor.register_contract(contract);
        monitor.start_mission("mission_1", &contract_id).unwrap();

        let alert = LiveAlert {
            alert_id: Uuid::new_v4().to_string(),
            mission_id: "mission_1".to_string(),
            severity: AlertSeverity::Critical,
            event_type: "navigation_deadlock".to_string(),
            description: "Deadlock detected".to_string(),
            timestamp: Utc::now(),
            suggested_action: None,
            confidence: 0.95,
        };

        let violation = monitor.check_alert("mission_1", &alert);
        assert!(violation.is_none()); // Within limit
    }

    #[test]
    fn test_deadlock_exceeds_limit() {
        let mut monitor = SlaMonitor::new();
        let mut contract = SlaContract::new("delivery");
        contract.max_navigation_deadlock_duration_ms = 100; // Very short limit
        let contract_id = contract.contract_id;

        monitor.register_contract(contract);
        monitor.start_mission("mission_1", &contract_id).unwrap();

        // Simulate old deadlock start time
        let state = monitor.active_missions.get_mut("mission_1").unwrap();
        state.navigation_deadlock_start = Some(Utc::now() - chrono::Duration::seconds(1));

        let alert = LiveAlert {
            alert_id: Uuid::new_v4().to_string(),
            mission_id: "mission_1".to_string(),
            severity: AlertSeverity::Critical,
            event_type: "navigation_deadlock".to_string(),
            description: "Deadlock detected".to_string(),
            timestamp: Utc::now(),
            suggested_action: None,
            confidence: 0.95,
        };

        let violation = monitor.check_alert("mission_1", &alert);
        assert!(violation.is_some());
    }

    #[test]
    fn test_sensor_dropout_within_limit() {
        let mut monitor = SlaMonitor::new();
        let contract = SlaContract::new("delivery");
        let contract_id = contract.contract_id;

        monitor.register_contract(contract);
        monitor.start_mission("mission_1", &contract_id).unwrap();

        let alert = LiveAlert {
            alert_id: Uuid::new_v4().to_string(),
            mission_id: "mission_1".to_string(),
            severity: AlertSeverity::High,
            event_type: "sensor_dropout".to_string(),
            description: "Sensor dropout".to_string(),
            timestamp: Utc::now(),
            suggested_action: None,
            confidence: 0.9,
        };

        let violation = monitor.check_alert("mission_1", &alert);
        assert!(violation.is_none());
    }

    #[test]
    fn test_sensor_dropout_exceeds_limit() {
        let mut monitor = SlaMonitor::new();
        let mut contract = SlaContract::new("delivery");
        contract.max_sensor_dropout_duration_ms = 100;
        let contract_id = contract.contract_id;

        monitor.register_contract(contract);
        monitor.start_mission("mission_1", &contract_id).unwrap();

        let state = monitor.active_missions.get_mut("mission_1").unwrap();
        state.sensor_dropout_start = Some(Utc::now() - chrono::Duration::seconds(1));

        let alert = LiveAlert {
            alert_id: Uuid::new_v4().to_string(),
            mission_id: "mission_1".to_string(),
            severity: AlertSeverity::High,
            event_type: "sensor_dropout".to_string(),
            description: "Sensor dropout".to_string(),
            timestamp: Utc::now(),
            suggested_action: None,
            confidence: 0.9,
        };

        let violation = monitor.check_alert("mission_1", &alert);
        assert!(violation.is_some());
    }

    #[test]
    fn test_end_mission_compliant() {
        let mut monitor = SlaMonitor::new();
        let contract = SlaContract::new("delivery");
        let contract_id = contract.contract_id;

        monitor.register_contract(contract);
        monitor.start_mission("mission_1", &contract_id).unwrap();

        let report = monitor.end_mission("mission_1");
        assert!(report.is_some());
        let r = report.unwrap();
        assert!(r.overall_compliant);
        assert_eq!(r.compliance_score, 1.0);
    }

    #[test]
    fn test_end_mission_non_compliant() {
        let mut monitor = SlaMonitor::new();
        let contract = SlaContract::new("delivery");
        let contract_id = contract.contract_id;

        monitor.register_contract(contract);
        monitor.start_mission("mission_1", &contract_id).unwrap();

        let state = monitor.active_missions.get_mut("mission_1").unwrap();
        state.violations.push(SlaViolation {
            violation_id: Uuid::new_v4(),
            contract_id,
            mission_id: "mission_1".to_string(),
            violation_type: SlaViolationType::SpeedViolationExceeded,
            occurred_at: Utc::now(),
            description: "Speed exceeded".to_string(),
            severity: SlaViolationSeverity::Medium,
        });

        let report = monitor.end_mission("mission_1");
        assert!(report.is_some());
        let r = report.unwrap();
        assert!(!r.overall_compliant);
        assert!(r.compliance_score < 1.0);
    }

    #[test]
    fn test_compliance_score_calculation() {
        let mut monitor = SlaMonitor::new();
        let contract = SlaContract::new("delivery");
        let contract_id = contract.contract_id;

        monitor.register_contract(contract);
        monitor.start_mission("mission_1", &contract_id).unwrap();

        let state = monitor.active_missions.get_mut("mission_1").unwrap();
        state.violations.push(SlaViolation {
            violation_id: Uuid::new_v4(),
            contract_id,
            mission_id: "mission_1".to_string(),
            violation_type: SlaViolationType::SpeedViolationExceeded,
            occurred_at: Utc::now(),
            description: "Speed exceeded".to_string(),
            severity: SlaViolationSeverity::Critical,
        });

        let report = monitor.end_mission("mission_1").unwrap();
        assert_eq!(report.compliance_score, 0.7); // Critical penalty is 0.3, so 1.0 - 0.3 = 0.7
    }

    #[test]
    fn test_start_mission_already_exists() {
        let mut monitor = SlaMonitor::new();
        let contract = SlaContract::new("delivery");
        let contract_id = contract.contract_id;

        monitor.register_contract(contract.clone());
        monitor.start_mission("mission_1", &contract_id).unwrap();

        let result = monitor.start_mission("mission_1", &contract_id);
        assert!(matches!(result, Err(SlaError::MissionAlreadyRegistered(_))));
    }

    #[test]
    fn test_start_mission_contract_not_found() {
        let mut monitor = SlaMonitor::new();
        let fake_contract_id = Uuid::new_v4();

        let result = monitor.start_mission("mission_1", &fake_contract_id);
        assert!(matches!(result, Err(SlaError::ContractNotFound(_))));
    }

    #[test]
    fn test_end_mission_not_found() {
        let mut monitor = SlaMonitor::new();
        let report = monitor.end_mission("nonexistent");

        assert!(report.is_none());
    }

    #[test]
    fn test_sla_violation_severity_ordering() {
        assert!(SlaViolationSeverity::Critical > SlaViolationSeverity::High);
        assert!(SlaViolationSeverity::High > SlaViolationSeverity::Medium);
    }
}
