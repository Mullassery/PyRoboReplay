use crate::core::event::MissionEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A detected anomaly or failure in mission execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Failure {
    /// Unique failure ID
    pub id: String,
    /// Failure type (e.g., "near_collision", "localization_loss")
    pub failure_type: String,
    /// Timestamp of failure detection
    pub timestamp: DateTime<Utc>,
    /// Timestamp as Unix timestamp for Python
    pub timestamp_seconds: f64,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Severity level
    pub severity: String,  // "critical", "high", "medium", "low"
    /// Human-readable description
    pub description: String,
    /// Systems affected by this failure
    pub affected_systems: Vec<String>,
    /// Evidence that triggered this detection
    pub evidence: HashMap<String, String>,
}

impl Failure {
    pub fn new(
        failure_type: String,
        timestamp: DateTime<Utc>,
        confidence: f32,
        severity: String,
        description: String,
    ) -> Self {
        let timestamp_seconds = timestamp.timestamp() as f64 + timestamp.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
        let id = format!("{}_{}", failure_type, uuid::Uuid::new_v4());
        Failure {
            id,
            failure_type,
            timestamp,
            timestamp_seconds,
            confidence,
            severity,
            description,
            affected_systems: Vec::new(),
            evidence: HashMap::new(),
        }
    }

    pub fn with_system(mut self, system: String) -> Self {
        self.affected_systems.push(system);
        self
    }

    pub fn with_evidence(mut self, key: String, value: String) -> Self {
        self.evidence.insert(key, value);
        self
    }
}

/// Detects anomalies and failures across 8 categories
pub struct AnomalyDetector {
    events: Vec<MissionEvent>,
    /// Minimum LiDAR range threshold (meters)
    lidar_threshold: f32,
    /// Perception confidence threshold
    perception_confidence_threshold: f32,
}

impl AnomalyDetector {
    pub fn new(events: Vec<MissionEvent>) -> Self {
        AnomalyDetector {
            events,
            lidar_threshold: 0.5,  // 50cm collision warning
            perception_confidence_threshold: 0.5,
        }
    }

    /// Detect all types of failures/anomalies
    pub fn detect_all(&self) -> Vec<Failure> {
        let mut failures = Vec::new();
        failures.extend(self.detect_near_collision());
        failures.extend(self.detect_perception_failure());
        failures.extend(self.detect_sensor_dropout());
        failures.extend(self.detect_communication_loss());
        failures.extend(self.detect_navigation_deadlock());
        failures.extend(self.detect_localization_loss());
        failures.extend(self.detect_oscillation());
        failures.extend(self.detect_costmap_anomaly());
        failures
    }

    /// Detect near-collision events (LiDAR minimum range threshold)
    pub fn detect_near_collision(&self) -> Vec<Failure> {
        let mut failures = Vec::new();

        for event in &self.events {
            if let MissionEvent::LidarScan { timestamp, data, .. } = event {
                if let Some(&min_range) = data.ranges.iter().min_by(|a, b| {
                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    if min_range < self.lidar_threshold && min_range > 0.0 {
                        let confidence = 1.0 - (min_range / self.lidar_threshold);
                        let severity = match min_range {
                            r if r < 0.25 => "critical",
                            r if r < 0.5 => "high",
                            _ => "medium",
                        }
                        .to_string();

                        let mut failure = Failure::new(
                            "near_collision".to_string(),
                            *timestamp,
                            confidence.min(1.0) as f32,
                            severity,
                            format!(
                                "LiDAR detected obstacle at {:.2}m (threshold: {:.2}m)",
                                min_range, self.lidar_threshold
                            ),
                        )
                        .with_system("lidar".to_string())
                        .with_system("planner".to_string());

                        failure
                            .evidence
                            .insert("min_range_m".to_string(), format!("{:.2}", min_range));
                        failure
                            .evidence
                            .insert("threshold_m".to_string(), format!("{:.2}", self.lidar_threshold));
                        failures.push(failure);
                    }
                }
            }
        }

        failures
    }

    /// Detect perception failures (low detection confidence)
    pub fn detect_perception_failure(&self) -> Vec<Failure> {
        let mut failures = Vec::new();
        let mut low_confidence_count = 0;
        let mut total_frames = 0;

        for event in &self.events {
            if let MissionEvent::CameraFrame { timestamp, .. } = event {
                total_frames += 1;
                // Placeholder for detection logic
                // In a real implementation, we'd parse detections from camera data
                // For now, we track that cameras are present
            }
        }

        // Only report if we have actual low-confidence data
        if low_confidence_count > 0 && total_frames > 10 {
            if let Some(last_event) = self.events.last() {
                if let MissionEvent::CameraFrame { timestamp, .. } = last_event {
                    let ratio = low_confidence_count as f32 / total_frames as f32;
                    if ratio > 0.3 {
                        let mut failure = Failure::new(
                            "perception_failure".to_string(),
                            *timestamp,
                            (ratio * 0.9) as f32,
                            "medium".to_string(),
                            format!(
                                "{:.0}% of detections below confidence threshold ({:.0}%)",
                                ratio * 100.0,
                                self.perception_confidence_threshold * 100.0
                            ),
                        )
                        .with_system("camera".to_string())
                        .with_system("perception".to_string());

                        failure
                            .evidence
                            .insert("low_confidence_count".to_string(), low_confidence_count.to_string());
                        failure
                            .evidence
                            .insert("total_frames".to_string(), total_frames.to_string());
                        failures.push(failure);
                    }
                }
            }
        }

        failures
    }

    /// Detect sensor dropout (sudden stop in data stream)
    pub fn detect_sensor_dropout(&self) -> Vec<Failure> {
        let mut failures = Vec::new();
        let mut sensor_last_seen: HashMap<String, DateTime<Utc>> = HashMap::new();

        for event in &self.events {
            let (sensor_type, timestamp) = match event {
                MissionEvent::LidarScan { timestamp, .. } => ("lidar", *timestamp),
                MissionEvent::CameraFrame { timestamp, .. } => ("camera", *timestamp),
                MissionEvent::IMUData { timestamp, .. } => ("imu", *timestamp),
                MissionEvent::OdometryUpdate { timestamp, .. } => ("odometry", *timestamp),
                _ => continue,
            };

            sensor_last_seen.insert(sensor_type.to_string(), timestamp);
        }

        // Check for sensors that stopped reporting
        let last_timestamp = sensor_last_seen
            .values()
            .max()
            .cloned();

        if let Some(last_ts) = last_timestamp {
            for (sensor, last_seen) in &sensor_last_seen {
                let gap_seconds = (last_ts - *last_seen).num_seconds() as f64;
                if gap_seconds > 1.0 {
                    let mut failure = Failure::new(
                        "sensor_dropout".to_string(),
                        last_ts,
                        0.80,
                        "high".to_string(),
                        format!(
                            "{} sensor stopped reporting {:.2}s ago",
                            sensor, gap_seconds
                        ),
                    )
                    .with_system(sensor.clone());

                    failure
                        .evidence
                        .insert("sensor".to_string(), sensor.clone());
                    failure
                        .evidence
                        .insert("gap_seconds".to_string(), format!("{:.2}", gap_seconds));
                    failures.push(failure);
                }
            }
        }

        failures
    }

    /// Detect communication loss (message dropout)
    pub fn detect_communication_loss(&self) -> Vec<Failure> {
        let mut failures = Vec::new();
        let mut message_timestamps = Vec::new();

        for event in &self.events {
            match event {
                MissionEvent::LidarScan { timestamp, .. }
                | MissionEvent::CameraFrame { timestamp, .. }
                | MissionEvent::OdometryUpdate { timestamp, .. } => {
                    message_timestamps.push(*timestamp);
                }
                _ => {}
            }
        }

        if message_timestamps.len() > 10 {
            message_timestamps.sort_by_key(|ts| ts.timestamp());

            let mut gaps = Vec::new();
            for window in message_timestamps.windows(2) {
                let gap_ms = (window[1] - window[0]).num_milliseconds();
                if gap_ms >= 0 {
                    gaps.push(gap_ms as f64 / 1000.0);
                }
            }

            let avg_gap = gaps.iter().sum::<f64>() / gaps.len() as f64;
            let max_gap = gaps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            if max_gap > avg_gap * 3.0 {
                let mut failure = Failure::new(
                    "communication_loss".to_string(),
                    message_timestamps[message_timestamps.len() - 1],
                    0.75,
                    "high".to_string(),
                    format!(
                        "Communication dropout detected: max gap {:.3}s (avg: {:.3}s)",
                        max_gap, avg_gap
                    ),
                )
                .with_system("communication".to_string())
                .with_system("network".to_string());

                failure.evidence.insert("max_gap_s".to_string(), format!("{:.3}", max_gap));
                failure.evidence.insert("avg_gap_s".to_string(), format!("{:.3}", avg_gap));
                failures.push(failure);
            }
        }

        failures
    }

    /// Detect navigation deadlock (no progress despite replanning)
    pub fn detect_navigation_deadlock(&self) -> Vec<Failure> {
        let mut failures = Vec::new();
        let mut decision_count = 0;

        for event in &self.events {
            if let MissionEvent::NavigationDecision { timestamp, .. } = event {
                decision_count += 1;

                // If too many replans with short duration, likely deadlock
                if decision_count > 20 {
                    let mut failure = Failure::new(
                        "navigation_deadlock".to_string(),
                        *timestamp,
                        0.75,
                        "high".to_string(),
                        format!("Navigation deadlock: {} replans detected", decision_count),
                    )
                    .with_system("planner".to_string())
                    .with_system("navigation".to_string());

                    failure
                        .evidence
                        .insert("replan_count".to_string(), decision_count.to_string());
                    failures.push(failure);
                    break;
                }
            }
        }

        failures
    }

    /// Detect localization loss (low pose confidence)
    pub fn detect_localization_loss(&self) -> Vec<Failure> {
        let mut failures = Vec::new();

        for event in &self.events {
            if let MissionEvent::RobotPose {
                timestamp,
                confidence: Some(conf),
                ..
            } = event
            {
                if *conf < 0.5 {
                    let mut failure = Failure::new(
                        "localization_loss".to_string(),
                        *timestamp,
                        0.80,
                        "high".to_string(),
                        format!("Localization confidence low: {:.1}%", conf * 100.0),
                    )
                    .with_system("localization".to_string())
                    .with_system("odometry".to_string());

                    failure
                        .evidence
                        .insert("confidence".to_string(), format!("{:.2}", conf));
                    failures.push(failure);
                }
            }
        }

        failures
    }

    /// Detect oscillation (robot moving back and forth)
    pub fn detect_oscillation(&self) -> Vec<Failure> {
        let mut failures = Vec::new();
        let mut positions = Vec::new();

        for event in &self.events {
            if let MissionEvent::OdometryUpdate { timestamp, data, .. } = event {
                positions.push((timestamp, data.pose.x, data.pose.y));
            }
        }

        if positions.len() > 20 {
            let start = positions[0];
            let end = positions[positions.len() - 1];
            let time_elapsed = (*end.0 - *start.0).num_seconds() as f64;
            let distance_traveled = ((end.1 - start.1).powi(2) + (end.2 - start.2).powi(2)).sqrt();

            // Check for oscillation pattern
            let mut direction_changes = 0;
            for window in positions.windows(3) {
                let dx1 = window[1].1 - window[0].1;
                let dy1 = window[1].2 - window[0].2;
                let dx2 = window[2].1 - window[1].1;
                let dy2 = window[2].2 - window[1].2;

                let dot_product = dx1 * dx2 + dy1 * dy2;
                if dot_product < 0.0 {
                    direction_changes += 1;
                }
            }

            if direction_changes > positions.len() / 3 && time_elapsed > 5.0 {
                let velocity = distance_traveled / time_elapsed.max(0.1);
                let mut failure = Failure::new(
                    "oscillation".to_string(),
                    *end.0,
                    0.70,
                    "medium".to_string(),
                    format!(
                        "Oscillating movement: {} direction changes, velocity {:.2} m/s",
                        direction_changes, velocity
                    ),
                )
                .with_system("navigation".to_string())
                .with_system("odometry".to_string());

                failure
                    .evidence
                    .insert("direction_changes".to_string(), direction_changes.to_string());
                failure
                    .evidence
                    .insert("velocity_m_s".to_string(), format!("{:.2}", velocity));
                failures.push(failure);
            }
        }

        failures
    }

    /// Detect costmap anomalies (sudden changes)
    pub fn detect_costmap_anomaly(&self) -> Vec<Failure> {
        let mut failures = Vec::new();
        let mut prev_obstacle_count = 0;

        for event in &self.events {
            if let MissionEvent::CostmapUpdate { timestamp, data, .. } = event {
                let obstacle_count = data.data.iter().filter(|&&x| x > 128).count();

                if prev_obstacle_count > 0 {
                    let change_ratio = (obstacle_count as i32 - prev_obstacle_count as i32).abs()
                        as f32
                        / prev_obstacle_count as f32;

                    if change_ratio > 0.8 {
                        let mut failure = Failure::new(
                            "costmap_anomaly".to_string(),
                            *timestamp,
                            0.65,
                            "low".to_string(),
                            format!(
                                "Costmap sudden change: {} obstacles (was {}), {:.0}% change",
                                obstacle_count,
                                prev_obstacle_count,
                                change_ratio * 100.0
                            ),
                        )
                        .with_system("costmap".to_string())
                        .with_system("perception".to_string());

                        failure.evidence.insert(
                            "current_obstacles".to_string(),
                            obstacle_count.to_string(),
                        );
                        failure.evidence.insert(
                            "previous_obstacles".to_string(),
                            prev_obstacle_count.to_string(),
                        );
                        failures.push(failure);
                    }
                }

                prev_obstacle_count = obstacle_count;
            }
        }

        failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_events() {
        let detector = AnomalyDetector::new(vec![]);
        let failures = detector.detect_all();
        assert_eq!(failures.len(), 0);
    }

    #[test]
    fn test_detector_creation() {
        let detector = AnomalyDetector::new(vec![]);
        assert_eq!(detector.lidar_threshold, 0.5);
    }
}
