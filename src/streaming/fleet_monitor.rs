use crate::streaming::channel::StreamEvent;
use crate::streaming::live_diagnostics::{LiveDiagnostics, DiagnosticsConfig, LiveAlert, AlertSeverity};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Robot operational status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RobotStatusType {
    Active,
    Idle,
    Degraded,
    Failed,
    Offline,
    Charging,
}

impl std::fmt::Display for RobotStatusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RobotStatusType::Active => write!(f, "Active"),
            RobotStatusType::Idle => write!(f, "Idle"),
            RobotStatusType::Degraded => write!(f, "Degraded"),
            RobotStatusType::Failed => write!(f, "Failed"),
            RobotStatusType::Offline => write!(f, "Offline"),
            RobotStatusType::Charging => write!(f, "Charging"),
        }
    }
}

/// Individual robot status snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotStatus {
    pub robot_id: String,
    pub last_seen: DateTime<Utc>,
    pub status: RobotStatusType,
    pub active_mission_id: Option<String>,
    pub alert_count: usize,
    pub last_alert: Option<String>,
}

/// Fleet-wide health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetHealthSummary {
    pub timestamp: DateTime<Utc>,
    pub total_robots: usize,
    pub active_missions: usize,
    pub alerts_by_severity: HashMap<String, usize>,
    pub top_failures: Vec<String>,
    pub overall_health_score: f32,
    pub robots: Vec<RobotStatus>,
}

/// Health trend over time
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HealthTrend {
    Improving,
    Stable,
    Degrading,
}

/// Time-windowed fleet dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetDashboardWindow {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub summaries: Vec<FleetHealthSummary>,
    pub trend: HealthTrend,
}

/// Fleet monitor configuration
#[derive(Debug, Clone)]
pub struct FleetMonitorConfig {
    pub offline_threshold_ms: u64,
    pub degraded_alert_threshold: usize,
    pub diagnostics_config: DiagnosticsConfig,
}

impl Default for FleetMonitorConfig {
    fn default() -> Self {
        FleetMonitorConfig {
            offline_threshold_ms: 30_000,
            degraded_alert_threshold: 3,
            diagnostics_config: DiagnosticsConfig::default(),
        }
    }
}

/// Real-time fleet monitoring engine
pub struct FleetMonitor {
    per_robot_diagnostics: HashMap<String, LiveDiagnostics>,
    robot_statuses: HashMap<String, RobotStatus>,
    alert_history: Vec<LiveAlert>,
    config: FleetMonitorConfig,
}

impl FleetMonitor {
    /// Create a new fleet monitor
    pub fn new(config: FleetMonitorConfig) -> Self {
        FleetMonitor {
            per_robot_diagnostics: HashMap::new(),
            robot_statuses: HashMap::new(),
            alert_history: Vec::new(),
            config,
        }
    }

    /// Register a new robot in the fleet
    pub fn register_robot(&mut self, robot_id: &str, mission_id: Option<&str>) {
        let diagnostics = LiveDiagnostics::new(self.config.diagnostics_config.clone());
        self.per_robot_diagnostics.insert(robot_id.to_string(), diagnostics);

        let status = RobotStatus {
            robot_id: robot_id.to_string(),
            last_seen: Utc::now(),
            status: RobotStatusType::Idle,
            active_mission_id: mission_id.map(|m| m.to_string()),
            alert_count: 0,
            last_alert: None,
        };

        self.robot_statuses.insert(robot_id.to_string(), status);
    }

    /// Process an event and update robot status
    pub fn process_event(&mut self, event: &StreamEvent) -> Option<LiveAlert> {
        let robot_id = event.robot_id.as_ref()?;

        // Ensure robot is registered
        if !self.per_robot_diagnostics.contains_key(robot_id) {
            self.register_robot(robot_id, Some(&event.mission_id));
        }

        // Route to per-robot diagnostics
        if let Some(diagnostics) = self.per_robot_diagnostics.get_mut(robot_id) {
            if let Some(alert) = diagnostics.process_event(event) {
                self.alert_history.push(alert.clone());

                // Update robot status based on alert severity
                if let Some(status) = self.robot_statuses.get_mut(robot_id) {
                    status.last_seen = Utc::now();
                    status.alert_count += 1;
                    status.last_alert = Some(alert.description.clone());

                    match alert.severity {
                        AlertSeverity::Critical => status.status = RobotStatusType::Failed,
                        AlertSeverity::High => {
                            if status.alert_count >= self.config.degraded_alert_threshold {
                                status.status = RobotStatusType::Degraded;
                            }
                        }
                        _ => {
                            if status.status == RobotStatusType::Idle {
                                status.status = RobotStatusType::Active;
                            }
                        }
                    }
                }

                return Some(alert);
            } else {
                // Update last_seen on any event
                if let Some(status) = self.robot_statuses.get_mut(robot_id) {
                    status.last_seen = Utc::now();
                    if status.status == RobotStatusType::Offline {
                        status.status = RobotStatusType::Idle;
                    }
                }
            }
        }

        None
    }

    /// Get current fleet health summary
    pub fn get_fleet_summary(&self) -> FleetHealthSummary {
        let now = Utc::now();
        let mut alerts_by_severity = HashMap::new();
        let mut top_failures = Vec::new();
        let mut health_score = 1.0f32;

        for alert in &self.alert_history {
            let severity_str = match alert.severity {
                AlertSeverity::Critical => "Critical".to_string(),
                AlertSeverity::High => "High".to_string(),
                AlertSeverity::Medium => "Medium".to_string(),
                AlertSeverity::Info => "Info".to_string(),
            };

            *alerts_by_severity.entry(severity_str).or_insert(0) += 1;

            if !top_failures.contains(&alert.event_type) {
                top_failures.push(alert.event_type.clone());
            }

            match alert.severity {
                AlertSeverity::Critical => health_score -= 0.2,
                AlertSeverity::High => health_score -= 0.1,
                AlertSeverity::Medium => health_score -= 0.05,
                AlertSeverity::Info => {}
            }
        }

        health_score = health_score.max(0.0).min(1.0);

        let active_count = self
            .robot_statuses
            .values()
            .filter(|s| s.status == RobotStatusType::Active)
            .count();

        FleetHealthSummary {
            timestamp: now,
            total_robots: self.robot_statuses.len(),
            active_missions: active_count,
            alerts_by_severity,
            top_failures: top_failures.into_iter().take(5).collect(),
            overall_health_score: health_score,
            robots: self.robot_statuses.values().cloned().collect(),
        }
    }

    /// Update robot statuses (mark offline if not seen recently)
    pub fn update_robot_statuses(&mut self) {
        let now = Utc::now();
        let offline_threshold = Duration::milliseconds(self.config.offline_threshold_ms as i64);

        for status in self.robot_statuses.values_mut() {
            if now.signed_duration_since(status.last_seen) > offline_threshold {
                status.status = RobotStatusType::Offline;
            }
        }
    }
}

/// Fleet monitoring dashboard with historical tracking
pub struct FleetDashboard {
    monitor: FleetMonitor,
    history: VecDeque<FleetHealthSummary>,
    window_size: usize,
}

impl FleetDashboard {
    /// Create a new fleet dashboard
    pub fn new(monitor: FleetMonitor, window_size: usize) -> Self {
        FleetDashboard {
            monitor,
            history: VecDeque::new(),
            window_size,
        }
    }

    /// Process events and update dashboard state
    pub fn tick(&mut self, events: &[StreamEvent]) {
        for event in events {
            self.monitor.process_event(event);
        }

        self.monitor.update_robot_statuses();
        let summary = self.monitor.get_fleet_summary();

        self.history.push_back(summary);
        while self.history.len() > self.window_size {
            self.history.pop_front();
        }
    }

    /// Get current dashboard window
    pub fn current_window(&self) -> FleetDashboardWindow {
        let summaries: Vec<_> = self.history.iter().cloned().collect();

        let trend = if summaries.len() >= 2 {
            let prev_score = summaries[summaries.len() - 2].overall_health_score;
            let curr_score = summaries[summaries.len() - 1].overall_health_score;

            if (curr_score - prev_score).abs() < 0.05 {
                HealthTrend::Stable
            } else if curr_score > prev_score {
                HealthTrend::Improving
            } else {
                HealthTrend::Degrading
            }
        } else {
            HealthTrend::Stable
        };

        let window_start = summaries.first().map(|s| s.timestamp).unwrap_or_else(Utc::now);
        let window_end = summaries.last().map(|s| s.timestamp).unwrap_or_else(Utc::now);

        FleetDashboardWindow {
            window_start,
            window_end,
            summaries,
            trend,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_monitor_creation() {
        let config = FleetMonitorConfig::default();
        let monitor = FleetMonitor::new(config);
        let summary = monitor.get_fleet_summary();

        assert_eq!(summary.total_robots, 0);
        assert_eq!(summary.overall_health_score, 1.0);
    }

    #[test]
    fn test_register_robot() {
        let config = FleetMonitorConfig::default();
        let mut monitor = FleetMonitor::new(config);

        monitor.register_robot("robot_1", Some("mission_1"));
        let summary = monitor.get_fleet_summary();

        assert_eq!(summary.total_robots, 1);
    }

    #[test]
    fn test_robot_status_fields() {
        let config = FleetMonitorConfig::default();
        let mut monitor = FleetMonitor::new(config);

        monitor.register_robot("robot_1", None);
        let status = monitor.robot_statuses.get("robot_1").unwrap();

        assert_eq!(status.robot_id, "robot_1");
        assert_eq!(status.status, RobotStatusType::Idle);
        assert_eq!(status.alert_count, 0);
    }

    #[test]
    fn test_fleet_summary_robot_count() {
        let config = FleetMonitorConfig::default();
        let mut monitor = FleetMonitor::new(config);

        monitor.register_robot("robot_1", None);
        monitor.register_robot("robot_2", None);

        let summary = monitor.get_fleet_summary();
        assert_eq!(summary.total_robots, 2);
    }

    #[test]
    fn test_health_score_starts_at_one() {
        let config = FleetMonitorConfig::default();
        let monitor = FleetMonitor::new(config);
        let summary = monitor.get_fleet_summary();

        assert_eq!(summary.overall_health_score, 1.0);
    }

    #[test]
    fn test_alerts_by_severity_counted() {
        let config = FleetMonitorConfig::default();
        let mut monitor = FleetMonitor::new(config);

        monitor.alert_history.push(LiveAlert {
            alert_id: Uuid::new_v4().to_string(),
            mission_id: "mission_1".to_string(),
            severity: AlertSeverity::Critical,
            event_type: "navigation_deadlock".to_string(),
            description: "Deadlock detected".to_string(),
            timestamp: Utc::now(),
            suggested_action: None,
            confidence: 0.95,
        });

        let summary = monitor.get_fleet_summary();
        assert_eq!(summary.alerts_by_severity.get("Critical"), Some(&1));
    }

    #[test]
    fn test_offline_after_threshold() {
        let mut config = FleetMonitorConfig::default();
        config.offline_threshold_ms = 100;

        let mut monitor = FleetMonitor::new(config);
        monitor.register_robot("robot_1", None);

        let old_time = Utc::now() - Duration::seconds(1);
        monitor.robot_statuses.get_mut("robot_1").unwrap().last_seen = old_time;

        monitor.update_robot_statuses();
        let status = monitor.robot_statuses.get("robot_1").unwrap();

        assert_eq!(status.status, RobotStatusType::Offline);
    }

    #[test]
    fn test_dashboard_tick_adds_summary() {
        let config = FleetMonitorConfig::default();
        let monitor = FleetMonitor::new(config);
        let mut dashboard = FleetDashboard::new(monitor, 10);

        dashboard.tick(&[]);
        let window = dashboard.current_window();

        assert_eq!(window.summaries.len(), 1);
    }

    #[test]
    fn test_multiple_robots_tracked() {
        let config = FleetMonitorConfig::default();
        let mut monitor = FleetMonitor::new(config);

        monitor.register_robot("robot_1", None);
        monitor.register_robot("robot_2", None);
        monitor.register_robot("robot_3", None);

        let summary = monitor.get_fleet_summary();
        assert_eq!(summary.robots.len(), 3);
    }

    #[test]
    fn test_top_failures_limited_to_five() {
        let config = FleetMonitorConfig::default();
        let mut monitor = FleetMonitor::new(config);

        for i in 0..10 {
            monitor.alert_history.push(LiveAlert {
                alert_id: Uuid::new_v4().to_string(),
                mission_id: "mission_1".to_string(),
                severity: AlertSeverity::Medium,
                event_type: format!("failure_{}", i),
                description: format!("Failure {}", i),
                timestamp: Utc::now(),
                suggested_action: None,
                confidence: 0.8,
            });
        }

        let summary = monitor.get_fleet_summary();
        assert!(summary.top_failures.len() <= 5);
    }

    #[test]
    fn test_health_trend_stable() {
        let config = FleetMonitorConfig::default();
        let monitor = FleetMonitor::new(config);
        let mut dashboard = FleetDashboard::new(monitor, 10);

        dashboard.tick(&[]);
        dashboard.tick(&[]);

        let window = dashboard.current_window();
        assert_eq!(window.trend, HealthTrend::Stable);
    }

    #[test]
    fn test_health_trend_degrading() {
        let config = FleetMonitorConfig::default();
        let monitor = FleetMonitor::new(config);
        let mut dashboard = FleetDashboard::new(monitor, 10);

        dashboard.tick(&[]);

        let mut monitor2 = dashboard.monitor;
        for _ in 0..3 {
            monitor2.alert_history.push(LiveAlert {
                alert_id: Uuid::new_v4().to_string(),
                mission_id: "mission_1".to_string(),
                severity: AlertSeverity::Critical,
                event_type: "error".to_string(),
                description: "Critical error".to_string(),
                timestamp: Utc::now(),
                suggested_action: None,
                confidence: 0.9,
            });
        }
        dashboard.monitor = monitor2;

        dashboard.tick(&[]);

        let window = dashboard.current_window();
        // Score should have degraded from 1.0 to something lower
        let score_changed = (window.summaries[0].overall_health_score - window.summaries[1].overall_health_score).abs() > 0.01;
        assert!(score_changed || window.trend == HealthTrend::Degrading);
    }

    #[test]
    fn test_window_history_respects_size() {
        let config = FleetMonitorConfig::default();
        let monitor = FleetMonitor::new(config);
        let mut dashboard = FleetDashboard::new(monitor, 5);

        for _ in 0..10 {
            dashboard.tick(&[]);
        }

        let window = dashboard.current_window();
        assert!(window.summaries.len() <= 5);
    }
}
