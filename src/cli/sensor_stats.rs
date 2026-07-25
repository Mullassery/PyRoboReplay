/// Sensor statistics panel for terminal replay
/// Displays real-time metrics for each sensor type

use crate::core::event::{MissionRecord, MissionEvent};
use std::collections::HashMap;

/// Statistics for a single sensor
#[derive(Debug, Clone)]
pub struct SensorStats {
    pub sensor_name: String,
    pub sensor_type: String,
    pub frame_count: usize,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub duration_seconds: f64,
    pub avg_fps: f32,
    pub data_quality: f32, // 0.0-1.0, gaps reduce quality
    pub encoding: String,  // for camera
    pub resolution: Option<(u32, u32)>, // width, height for camera
}

impl SensorStats {
    /// Create empty stats
    pub fn new(sensor_name: String, sensor_type: String) -> Self {
        Self {
            sensor_name,
            sensor_type,
            frame_count: 0,
            first_timestamp: None,
            last_timestamp: None,
            duration_seconds: 0.0,
            avg_fps: 0.0,
            data_quality: 1.0,
            encoding: String::new(),
            resolution: None,
        }
    }

    /// Calculate quality score based on gaps
    pub fn calculate_quality(&mut self, total_frames: usize) {
        if self.frame_count == 0 || total_frames == 0 {
            self.data_quality = 0.0;
            return;
        }

        // Quality = how many expected frames we got
        // Assumes frames should be evenly distributed
        self.data_quality = (self.frame_count as f32 / total_frames as f32).min(1.0);
    }

    /// Get quality indicator (█ ▓ ▒ ░)
    pub fn quality_bar(&self) -> String {
        let blocks = (self.data_quality * 10.0) as usize;
        let filled = "█".repeat(blocks);
        let empty = "░".repeat(10 - blocks);
        format!("{}{}", filled, empty)
    }

    /// Get quality emoji
    pub fn quality_emoji(&self) -> &'static str {
        match (self.data_quality * 4.0) as usize {
            4 => "✅", // Excellent
            3 => "🟢", // Good
            2 => "🟡", // Fair
            1 => "🟠", // Poor
            _ => "🔴", // Bad
        }
    }
}

/// Sensor metadata panel with real-time statistics
#[derive(Debug, Clone)]
pub struct SensorMetadataPanel {
    stats: HashMap<String, SensorStats>,
    all_sensors: Vec<String>,
}

impl SensorMetadataPanel {
    /// Create new metadata panel from mission
    pub fn from_mission(mission: &MissionRecord) -> Self {
        let mut panel = SensorMetadataPanel {
            stats: HashMap::new(),
            all_sensors: Vec::new(),
        };

        // Extract unique sensors
        let mut sensor_map: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, event) in mission.events.iter().enumerate() {
            match event {
                MissionEvent::LidarScan { data, .. } => {
                    let key = format!("Lidar ({})", data.frame_id);
                    sensor_map.entry(key).or_insert_with(Vec::new).push(idx);
                }
                MissionEvent::CameraFrame { data, .. } => {
                    let key = format!("Camera ({})", data.sensor_id);
                    sensor_map.entry(key).or_insert_with(Vec::new).push(idx);
                }
                MissionEvent::IMUData { .. } => {
                    sensor_map
                        .entry("IMU".to_string())
                        .or_insert_with(Vec::new)
                        .push(idx);
                }
                MissionEvent::OdometryUpdate { .. } => {
                    sensor_map
                        .entry("Odometry".to_string())
                        .or_insert_with(Vec::new)
                        .push(idx);
                }
                MissionEvent::CostmapUpdate { .. } => {
                    sensor_map
                        .entry("Costmap".to_string())
                        .or_insert_with(Vec::new)
                        .push(idx);
                }
                _ => {}
            }
        }

        // Build stats for each sensor
        for (sensor_key, indices) in sensor_map.iter() {
            let sensor_type = if sensor_key.contains("Lidar") {
                "LidarScan"
            } else if sensor_key.contains("Camera") {
                "CameraFrame"
            } else if sensor_key.contains("IMU") {
                "IMUData"
            } else if sensor_key.contains("Odometry") {
                "OdometryUpdate"
            } else if sensor_key.contains("Costmap") {
                "CostmapUpdate"
            } else {
                "Unknown"
            };

            let mut stats = SensorStats::new(sensor_key.clone(), sensor_type.to_string());
            stats.frame_count = indices.len();

            // Get first and last timestamps
            if let Some(&first_idx) = indices.first() {
                if let Some(ts) = mission.events[first_idx].timestamp_str() {
                    stats.first_timestamp = Some(ts);
                }
            }

            if let Some(&last_idx) = indices.last() {
                if let Some(ts) = mission.events[last_idx].timestamp_str() {
                    stats.last_timestamp = Some(ts);
                }
            }

            // Calculate duration and FPS
            if let (Some(first_ts), Some(last_ts)) = (&stats.first_timestamp, &stats.last_timestamp) {
                if let Ok(first) = chrono::DateTime::parse_from_rfc3339(first_ts) {
                    if let Ok(last) = chrono::DateTime::parse_from_rfc3339(last_ts) {
                        let duration = (last - first).num_milliseconds() as f64 / 1000.0;
                        stats.duration_seconds = duration;
                        if duration > 0.0 {
                            stats.avg_fps = (stats.frame_count as f32 / duration as f32).max(0.0);
                        }
                    }
                }
            }

            // Get encoding and resolution for cameras
            if sensor_key.contains("Camera") {
                if let Some(&idx) = indices.first() {
                    if let MissionEvent::CameraFrame { data, .. } = &mission.events[idx] {
                        stats.encoding = data.encoding.clone();
                        stats.resolution = Some((data.width, data.height));
                    }
                }
            }

            // Calculate quality
            stats.calculate_quality(mission.events.len());

            panel.all_sensors.push(sensor_key.clone());
            panel.stats.insert(sensor_key.clone(), stats);
        }

        // Sort sensors for consistent display
        panel.all_sensors.sort();

        panel
    }

    /// Get stats for a specific sensor
    pub fn get_stats(&self, sensor_name: &str) -> Option<&SensorStats> {
        self.stats.get(sensor_name)
    }

    /// Get all sensor names
    pub fn sensor_names(&self) -> &[String] {
        &self.all_sensors
    }

    /// Render full metadata panel
    pub fn render(&self) -> String {
        let mut output = String::new();

        output.push_str("📊 Sensor Metadata Panel\n");
        output.push_str("════════════════════════════════════════════════════════════════\n");

        for sensor_name in &self.all_sensors {
            if let Some(stats) = self.get_stats(sensor_name) {
                output.push_str(&self.render_sensor_stats(stats));
                output.push('\n');
            }
        }

        output
    }

    /// Render stats for a single sensor
    fn render_sensor_stats(&self, stats: &SensorStats) -> String {
        let mut output = String::new();

        // Header with sensor name and quality
        output.push_str(&format!(
            "├─ {} {} [{}]\n",
            stats.sensor_name,
            stats.quality_emoji(),
            stats.quality_bar()
        ));

        // Frame count
        output.push_str(&format!("│  Frames: {}", stats.frame_count));
        if stats.avg_fps > 0.0 {
            output.push_str(&format!(" ({:.1} fps avg)", stats.avg_fps));
        }
        output.push('\n');

        // Duration
        if stats.duration_seconds > 0.0 {
            output.push_str(&format!(
                "│  Duration: {:.1}s\n",
                stats.duration_seconds
            ));
        }

        // Timestamps
        if let Some(first_ts) = &stats.first_timestamp {
            output.push_str(&format!("│  First: {}\n", first_ts));
        }
        if let Some(last_ts) = &stats.last_timestamp {
            output.push_str(&format!("│  Last:  {}\n", last_ts));
        }

        // Sensor-specific info
        if !stats.encoding.is_empty() {
            output.push_str(&format!("│  Encoding: {}", stats.encoding));
            if let Some((w, h)) = stats.resolution {
                output.push_str(&format!(" ({}×{})", w, h));
            }
            output.push('\n');
        }

        // Quality details
        output.push_str(&format!(
            "│  Quality: {:.0}% complete\n",
            stats.data_quality * 100.0
        ));

        output
    }

    /// Render compact single-line summary for each sensor
    pub fn render_compact(&self) -> String {
        let mut output = String::new();

        output.push_str("Sensors: ");
        for (i, sensor_name) in self.all_sensors.iter().enumerate() {
            if i > 0 {
                output.push_str(" | ");
            }
            if let Some(stats) = self.get_stats(sensor_name) {
                output.push_str(&format!(
                    "{} {} {:.1}fps ({} frames)",
                    stats.quality_emoji(),
                    stats.sensor_name,
                    stats.avg_fps,
                    stats.frame_count
                ));
            }
        }
        output.push('\n');

        output
    }

    /// Get summary statistics
    pub fn summary(&self) -> String {
        let total_frames: usize = self.stats.values().map(|s| s.frame_count).sum();
        let avg_quality =
            self.stats.values().map(|s| s.data_quality).sum::<f32>() / self.stats.len().max(1) as f32;

        format!(
            "Total: {} events across {} sensors | Avg Quality: {:.0}%",
            total_frames,
            self.stats.len(),
            avg_quality * 100.0
        )
    }
}

impl Default for SensorMetadataPanel {
    fn default() -> Self {
        Self {
            stats: HashMap::new(),
            all_sensors: Vec::new(),
        }
    }
}

// Extension trait for MissionEvent to get timestamp as string
trait EventTimestamp {
    fn timestamp_str(&self) -> Option<String>;
}

impl EventTimestamp for MissionEvent {
    fn timestamp_str(&self) -> Option<String> {
        match self {
            MissionEvent::LidarScan { timestamp, .. }
            | MissionEvent::CameraFrame { timestamp, .. }
            | MissionEvent::IMUData { timestamp, .. }
            | MissionEvent::OdometryUpdate { timestamp, .. }
            | MissionEvent::CostmapUpdate { timestamp, .. }
            | MissionEvent::RobotPose { timestamp, .. }
            | MissionEvent::ObstacleDetected { timestamp, .. }
            | MissionEvent::NavigationDecision { timestamp, .. }
            | MissionEvent::CommunicationEvent { timestamp, .. }
            | MissionEvent::CoordinationEvent { timestamp, .. }
            | MissionEvent::EnvironmentalChange { timestamp, .. }
            | MissionEvent::MissionLifecycle { timestamp, .. }
            | MissionEvent::KernelEvent { timestamp, .. }
            | MissionEvent::LinuxLogEvent { timestamp, .. }
            | MissionEvent::HardwareEvent { timestamp, .. }
            | MissionEvent::ResourceMetric { timestamp, .. }
            | MissionEvent::DDSMetric { timestamp, .. }
            | MissionEvent::NetworkEvent { timestamp, .. }
            | MissionEvent::ConfigurationEvent { timestamp, .. }
            | MissionEvent::ParameterValidationEvent { timestamp, .. } => Some(timestamp.to_rfc3339()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_stats_creation() {
        let stats = SensorStats::new("Lidar".to_string(), "LidarScan".to_string());
        assert_eq!(stats.frame_count, 0);
        assert_eq!(stats.data_quality, 1.0);
    }

    #[test]
    fn test_quality_bar() {
        let mut stats = SensorStats::new("Test".to_string(), "Test".to_string());
        stats.data_quality = 0.5;
        let bar = stats.quality_bar();
        assert!(bar.contains("█"));
        assert!(bar.contains("░"));
    }

    #[test]
    fn test_quality_emoji() {
        let mut stats = SensorStats::new("Test".to_string(), "Test".to_string());

        // (quality * 4.0) as usize determines the match
        stats.data_quality = 1.0;   // 4.0 → 4
        assert_eq!(stats.quality_emoji(), "✅");

        stats.data_quality = 0.75;  // 3.0 → 3
        assert_eq!(stats.quality_emoji(), "🟢");

        stats.data_quality = 0.5;   // 2.0 → 2
        assert_eq!(stats.quality_emoji(), "🟡");

        stats.data_quality = 0.25;  // 1.0 → 1
        assert_eq!(stats.quality_emoji(), "🟠");

        stats.data_quality = 0.1;   // 0.4 → 0
        assert_eq!(stats.quality_emoji(), "🔴");
    }

    #[test]
    fn test_quality_calculation() {
        let mut stats = SensorStats::new("Test".to_string(), "Test".to_string());
        stats.frame_count = 80;

        stats.calculate_quality(100);
        assert_eq!(stats.data_quality, 0.8);
    }

    #[test]
    fn test_metadata_panel_creation() {
        let mission = MissionRecord::new("test");
        let panel = SensorMetadataPanel::from_mission(&mission);
        assert_eq!(panel.sensor_names().len(), 0);
    }

    #[test]
    fn test_compact_render() {
        let panel = SensorMetadataPanel::default();
        let compact = panel.render_compact();
        assert!(compact.contains("Sensors:"));
    }

    #[test]
    fn test_summary() {
        let panel = SensorMetadataPanel::default();
        let summary = panel.summary();
        assert!(summary.contains("Total:"));
        assert!(summary.contains("Quality:"));
    }
}
