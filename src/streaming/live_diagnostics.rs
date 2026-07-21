use crate::streaming::channel::StreamEvent;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveAlert {
    pub alert_id: String,
    pub mission_id: String,
    pub severity: AlertSeverity,
    pub event_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub suggested_action: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct DiagnosticsConfig {
    pub alert_window_ms: u64,
    pub max_buffered_events: usize,
    pub enable_root_cause: bool,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        DiagnosticsConfig {
            alert_window_ms: 5000,
            max_buffered_events: 500,
            enable_root_cause: false,
        }
    }
}

pub struct LiveDiagnostics {
    config: DiagnosticsConfig,
    recent_events: Arc<Mutex<VecDeque<StreamEvent>>>,
    alerts: Arc<Mutex<Vec<LiveAlert>>>,
}

impl LiveDiagnostics {
    pub fn new(config: DiagnosticsConfig) -> Self {
        LiveDiagnostics {
            config,
            recent_events: Arc::new(Mutex::new(VecDeque::new())),
            alerts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn process_event(&self, event: &StreamEvent) -> Option<LiveAlert> {
        {
            let mut recent = self.recent_events.lock().unwrap();
            if recent.len() >= self.config.max_buffered_events {
                recent.pop_front();
            }
            recent.push_back(event.clone());
        }

        let recent = self.recent_events.lock().unwrap();
        let events_ref = recent.iter().collect::<Vec<_>>();

        let alert = if let Some(alert) = self.detect_navigation_deadlock(&events_ref) {
            Some(alert)
        } else if let Some(alert) = self.detect_obstacle_storm(&events_ref) {
            Some(alert)
        } else if let Some(alert) = self.detect_sensor_dropout(&events_ref) {
            Some(alert)
        } else {
            None
        };

        drop(recent);

        if let Some(ref alert) = alert {
            self.alerts.lock().unwrap().push(alert.clone());
        }

        alert
    }

    pub fn get_alerts(&self) -> Vec<LiveAlert> {
        self.alerts.lock().unwrap().clone()
    }

    pub fn get_recent_alerts(&self, since: DateTime<Utc>) -> Vec<LiveAlert> {
        self.alerts
            .lock()
            .unwrap()
            .iter()
            .filter(|a| a.timestamp >= since)
            .cloned()
            .collect()
    }

    pub fn clear_alerts(&self) {
        self.alerts.lock().unwrap().clear();
    }

    pub fn event_count_in_window(&self) -> usize {
        self.recent_events.lock().unwrap().len()
    }

    fn detect_navigation_deadlock(&self, events: &[&StreamEvent]) -> Option<LiveAlert> {
        let window_duration = Duration::milliseconds(self.config.alert_window_ms as i64);
        let now = Utc::now();
        let window_start = now - window_duration;

        let nav_decisions: Vec<_> = events
            .iter()
            .filter(|e| {
                e.event_type == "navigation_decision" && e.timestamp >= window_start
            })
            .collect();

        if nav_decisions.len() >= 3 {
            let mut robot_poses: Vec<_> = events
                .iter()
                .filter(|e| {
                    e.event_type == "robot_pose" && e.timestamp >= window_start
                })
                .collect();

            robot_poses.sort_by_key(|e| e.timestamp);

            let pose_stasis = robot_poses.len() <= 1;

            if pose_stasis {
                return Some(LiveAlert {
                    alert_id: Uuid::new_v4().to_string(),
                    mission_id: events[0].mission_id.clone(),
                    severity: AlertSeverity::High,
                    event_type: "navigation_deadlock".to_string(),
                    description: format!(
                        "Navigation deadlock detected: {} navigation decisions in last {}ms with no pose change",
                        nav_decisions.len(),
                        self.config.alert_window_ms
                    ),
                    timestamp: now,
                    suggested_action: Some("Review obstacle configuration and navigation planner state".to_string()),
                    confidence: 0.85,
                });
            }
        }

        None
    }

    fn detect_obstacle_storm(&self, events: &[&StreamEvent]) -> Option<LiveAlert> {
        let window_duration = Duration::milliseconds(self.config.alert_window_ms as i64);
        let now = Utc::now();
        let window_start = now - window_duration;

        let obstacle_events: Vec<_> = events
            .iter()
            .filter(|e| {
                e.event_type == "obstacle_detected" && e.timestamp >= window_start
            })
            .collect();

        if obstacle_events.len() >= 5 {
            return Some(LiveAlert {
                alert_id: Uuid::new_v4().to_string(),
                mission_id: events[0].mission_id.clone(),
                severity: AlertSeverity::Medium,
                event_type: "obstacle_storm".to_string(),
                description: format!(
                    "Obstacle detection storm: {} detections in last {}ms",
                    obstacle_events.len(),
                    self.config.alert_window_ms
                ),
                timestamp: now,
                suggested_action: Some(
                    "Verify sensor calibration and environmental conditions".to_string(),
                ),
                confidence: 0.75,
            });
        }

        None
    }

    fn detect_sensor_dropout(&self, events: &[&StreamEvent]) -> Option<LiveAlert> {
        if events.len() < 2 {
            return None;
        }

        let window_duration = Duration::milliseconds(self.config.alert_window_ms as i64);
        let now = Utc::now();
        let window_start = now - window_duration;

        let sensor_events: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(
                    e.event_type.as_str(),
                    "lidar_scan" | "camera_frame" | "imu_data"
                ) && e.timestamp >= window_start
            })
            .collect();

        if sensor_events.is_empty() {
            return Some(LiveAlert {
                alert_id: Uuid::new_v4().to_string(),
                mission_id: events[0].mission_id.clone(),
                severity: AlertSeverity::Critical,
                event_type: "sensor_dropout".to_string(),
                description: format!(
                    "Sensor dropout detected: no sensor data in last {}ms",
                    self.config.alert_window_ms
                ),
                timestamp: now,
                suggested_action: Some("Check sensor connections and power supply".to_string()),
                confidence: 0.95,
            });
        }

        let mut last_sensor_time = sensor_events[0].timestamp;
        for event in &sensor_events[1..] {
            let gap = event.timestamp - last_sensor_time;
            if gap > Duration::milliseconds(6000) {
                return Some(LiveAlert {
                    alert_id: Uuid::new_v4().to_string(),
                    mission_id: events[0].mission_id.clone(),
                    severity: AlertSeverity::High,
                    event_type: "sensor_dropout".to_string(),
                    description: format!(
                        "Sensor dropout detected: {}ms gap in sensor data",
                        gap.num_milliseconds()
                    ),
                    timestamp: now,
                    suggested_action: Some("Investigate sensor performance and network connectivity".to_string()),
                    confidence: 0.80,
                });
            }
            last_sensor_time = event.timestamp;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_diagnostics_creation() {
        let config = DiagnosticsConfig::default();
        let diagnostics = LiveDiagnostics::new(config);
        assert_eq!(diagnostics.event_count_in_window(), 0);
        assert_eq!(diagnostics.get_alerts().len(), 0);
    }

    #[test]
    fn test_process_normal_event_no_alert() {
        let config = DiagnosticsConfig::default();
        let diagnostics = LiveDiagnostics::new(config);

        let event = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "lidar_scan".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 0,
        };

        let alert = diagnostics.process_event(&event);
        assert!(alert.is_none());
    }

    #[test]
    fn test_detect_navigation_deadlock() {
        let config = DiagnosticsConfig::default();
        let diagnostics = LiveDiagnostics::new(config);

        let now = Utc::now();

        // Add sensor data to avoid sensor dropout alert
        let sensor_event = StreamEvent {
            event_id: "sensor_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "lidar_scan".to_string(),
            timestamp: now,
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 0,
        };
        diagnostics.process_event(&sensor_event);

        // Add navigation decisions (deadlock detection)
        for i in 0..3 {
            let event = StreamEvent {
                event_id: format!("event_{}", i),
                mission_id: "mission_1".to_string(),
                event_type: "navigation_decision".to_string(),
                timestamp: now + Duration::milliseconds(i as i64 * 1000),
                robot_id: Some("robot_1".to_string()),
                payload: serde_json::json!({}),
                sequence_number: (i + 1) as u64,
            };
            diagnostics.process_event(&event);
        }

        let alerts = diagnostics.get_alerts();
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].event_type, "navigation_deadlock");
        assert_eq!(alerts[0].severity, AlertSeverity::High);
    }

    #[test]
    fn test_detect_obstacle_storm() {
        let config = DiagnosticsConfig::default();
        let diagnostics = LiveDiagnostics::new(config);

        let now = Utc::now();

        // Add sensor data to avoid sensor dropout alert
        let sensor_event = StreamEvent {
            event_id: "sensor_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "lidar_scan".to_string(),
            timestamp: now,
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 0,
        };
        diagnostics.process_event(&sensor_event);

        // Add obstacles (storm detection)
        for i in 0..6 {
            let event = StreamEvent {
                event_id: format!("event_{}", i),
                mission_id: "mission_1".to_string(),
                event_type: "obstacle_detected".to_string(),
                timestamp: now + Duration::milliseconds((i as i64) * 100 + 1),
                robot_id: Some("robot_1".to_string()),
                payload: serde_json::json!({}),
                sequence_number: (i + 1) as u64,
            };
            diagnostics.process_event(&event);
        }

        let alerts = diagnostics.get_alerts();
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].event_type, "obstacle_storm");
        assert_eq!(alerts[0].severity, AlertSeverity::Medium);
    }

    #[test]
    fn test_detect_sensor_dropout() {
        let config = DiagnosticsConfig {
            alert_window_ms: 10000,
            ..Default::default()
        };
        let diagnostics = LiveDiagnostics::new(config);

        let now = Utc::now();

        // Add a sensor event at t=0
        let first_sensor = StreamEvent {
            event_id: "sensor_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "lidar_scan".to_string(),
            timestamp: now - Duration::milliseconds(8000),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 0,
        };
        diagnostics.process_event(&first_sensor);

        // Add non-sensor events in between to avoid triggering navigation deadlock
        let non_sensor = StreamEvent {
            event_id: "other_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "robot_pose".to_string(),
            timestamp: now - Duration::milliseconds(5000),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 1,
        };
        diagnostics.process_event(&non_sensor);

        // Now add a sensor event with a 7-second gap (exceeds 6s threshold)
        let second_sensor = StreamEvent {
            event_id: "sensor_2".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "imu_data".to_string(),
            timestamp: now - Duration::milliseconds(1000),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 2,
        };
        let alert = diagnostics.process_event(&second_sensor);

        assert!(alert.is_some());
        let alert_unwrapped = alert.unwrap();
        assert_eq!(alert_unwrapped.event_type, "sensor_dropout");
        assert_eq!(alert_unwrapped.severity, AlertSeverity::High);
    }

    #[test]
    fn test_get_recent_alerts_filtered_by_time() {
        let config = DiagnosticsConfig::default();
        let diagnostics = LiveDiagnostics::new(config);

        let now = Utc::now();
        for i in 0..3 {
            let event = StreamEvent {
                event_id: format!("event_{}", i),
                mission_id: "mission_1".to_string(),
                event_type: "navigation_decision".to_string(),
                timestamp: now + Duration::milliseconds(i as i64 * 1000),
                robot_id: Some("robot_1".to_string()),
                payload: serde_json::json!({}),
                sequence_number: i as u64,
            };
            diagnostics.process_event(&event);
        }

        let recent = diagnostics.get_recent_alerts(now);
        assert!(!recent.is_empty());
    }
}
