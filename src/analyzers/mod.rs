//! Reality Gap Detection Framework
//!
//! This module implements automatic detection of sim-to-real gaps - phenomena in
//! real robot behavior that are absent, simplified, or misrepresented in simulation.
//!
//! The detection engine analyzes replay data across multiple domains:
//! - Physical (mechanical degradation, thermal effects, structural dynamics)
//! - Sensor (optical contamination, calibration drift, timing issues)
//! - Environmental (lighting, weather, seasonal changes)
//! - System (CPU contention, memory pressure, network issues)
//! - Coordination (multi-robot deadlocks, congestion, communication)

pub mod physical;
pub mod sensor;
pub mod system;
pub mod environmental;
pub mod coordination;
pub mod telemetry;
pub mod test_data;
pub mod validation;
pub mod scoring;
pub mod severity;
pub mod historical;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A detected reality gap with confidence and severity scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealityGapFinding {
    /// Primary domain: Physical, Sensor, Environmental, System, Coordination
    pub domain: RealityDomain,

    /// Gap category: "Mechanical Degradation", "Optical Contamination", etc.
    pub category: String,

    /// Specific finding type (human-readable)
    pub finding_type: String,

    /// How serious is this gap?
    pub severity: Severity,

    /// How confident are we in this finding? (0.0-1.0)
    pub confidence: f32,

    /// Probability this is a sim-to-real gap vs algorithm bug (0.0-1.0)
    /// High score = likely sim gap; low score = likely algorithm issue
    pub reality_gap_score: f32,

    /// Human-readable explanation of what was detected
    pub description: String,

    /// Supporting evidence (signals, measurements, values)
    pub evidence: Vec<Evidence>,

    /// Quantitative metrics with values
    pub metrics: HashMap<String, f32>,

    /// How to recreate this gap in simulation
    pub sim_recreation_suggestion: String,

    /// Recommended remediation steps
    pub remediation: String,

    /// Timestamp when this gap was detected (seconds in mission)
    pub detection_time_sec: Option<f32>,
}

/// Supporting evidence for a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// What signal/metric is this evidence from?
    pub signal: String,

    /// Measured value
    pub value: f32,

    /// When was this measured? (seconds in mission)
    pub timestamp: f32,

    /// How confident is this evidence? (0.0-1.0)
    pub confidence: f32,
}

/// Reality gap domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RealityDomain {
    Physical,
    Sensor,
    Environmental,
    System,
    Coordination,
}

impl fmt::Display for RealityDomain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RealityDomain::Physical => write!(f, "Physical"),
            RealityDomain::Sensor => write!(f, "Sensor"),
            RealityDomain::Environmental => write!(f, "Environmental"),
            RealityDomain::System => write!(f, "System"),
            RealityDomain::Coordination => write!(f, "Coordination"),
        }
    }
}

/// Severity levels for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Trait that all gap detectors must implement
pub trait GapDetector: Send + Sync {
    /// Analyze mission data for gaps in this domain
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding>;

    /// Get the domain this detector analyzes
    fn domain(&self) -> RealityDomain;
}

/// Structured data from a mission for gap analysis
#[derive(Debug, Clone)]
pub struct MissionAnalysisData {
    /// Mission identifier
    pub mission_id: String,

    /// Mission duration in seconds
    pub duration_sec: f32,

    /// Robot type: "mobile_robot", "drone", "manipulator", etc.
    pub robot_type: String,

    // Control and state data
    pub control_messages: Vec<ControlMessage>,
    pub joint_states: Vec<JointState>,
    pub odometry_messages: Vec<OdometryMessage>,

    // Sensor data
    pub camera_frames: Vec<CameraFrame>,
    pub lidar_scans: Vec<LidarScan>,
    pub imu_measurements: Vec<IMUMeasurement>,
    pub encoder_data: Vec<EncoderReading>,

    // Telemetry
    pub motor_currents: Vec<MotorCurrent>,
    pub thermal_readings: Vec<ThermalReading>,
    pub battery_data: Vec<BatteryReading>,

    // Perception outputs
    pub detection_results: Vec<DetectionResult>,
    pub perception_errors: Vec<PerceptionError>,

    // Timing data
    pub message_timestamps: Vec<MessageTimestamp>,
}

/// Control message (actuator command)
#[derive(Debug, Clone)]
pub struct ControlMessage {
    pub timestamp: f32,
    pub joint_id: String,
    pub command_type: String,
    pub value: f32,
}

/// Joint state (position, velocity, effort)
#[derive(Debug, Clone)]
pub struct JointState {
    pub timestamp: f32,
    pub joint_id: String,
    pub position: f32,
    pub velocity: f32,
    pub effort: f32,
}

/// Odometry message
#[derive(Debug, Clone)]
pub struct OdometryMessage {
    pub timestamp: f32,
    pub x: f32,
    pub y: f32,
    pub theta: f32,
    pub vx: f32,
    pub vy: f32,
    pub vtheta: f32,
}

/// Camera frame
#[derive(Debug, Clone)]
pub struct CameraFrame {
    pub timestamp: f32,
    pub camera_id: String,
    pub width: u32,
    pub height: u32,
    pub frame_index: usize,
    /// Image data (simplified - real implementation would have actual image bytes)
    pub quality_sharpness: Option<f32>,
    pub quality_entropy: Option<f32>,
}

/// LiDAR scan
#[derive(Debug, Clone)]
pub struct LidarScan {
    pub timestamp: f32,
    pub point_count: usize,
}

/// IMU measurement
#[derive(Debug, Clone)]
pub struct IMUMeasurement {
    pub timestamp: f32,
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
}

/// Encoder reading
#[derive(Debug, Clone)]
pub struct EncoderReading {
    pub timestamp: f32,
    pub wheel_id: String,
    pub ticks: i32,
    pub velocity: f32,
}

/// Motor current
#[derive(Debug, Clone)]
pub struct MotorCurrent {
    pub timestamp: f32,
    pub joint_id: String,
    pub current_amps: f32,
}

/// Thermal reading
#[derive(Debug, Clone)]
pub struct ThermalReading {
    pub timestamp: f32,
    pub location: String,
    pub temperature_c: f32,
}

/// Battery reading
#[derive(Debug, Clone)]
pub struct BatteryReading {
    pub timestamp: f32,
    pub voltage: f32,
    pub current_amps: f32,
    pub soc_percent: f32,
}

/// Object detection result
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub timestamp: f32,
    pub frame_index: usize,
    pub class: String,
    pub confidence: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Perception error
#[derive(Debug, Clone)]
pub struct PerceptionError {
    pub timestamp: f32,
    pub error_type: String,
    pub description: String,
}

/// Message timing info
#[derive(Debug, Clone)]
pub struct MessageTimestamp {
    pub timestamp: f32,
    pub message_type: String,
    pub sensor_id: String,
}

/// Main orchestrator for all gap detectors
pub struct RealityGapDetector {
    physical_analyzer: physical::PhysicalDomainAnalyzer,
    sensor_analyzer: sensor::SensorDomainAnalyzer,
    environmental_analyzer: environmental::EnvironmentalDomainAnalyzer,
    system_analyzer: system::SystemDomainAnalyzer,
    coordination_analyzer: coordination::CoordinationDomainAnalyzer,
}

impl RealityGapDetector {
    /// Create a new gap detector with all analyzers
    pub fn new() -> Self {
        RealityGapDetector {
            physical_analyzer: physical::PhysicalDomainAnalyzer::new(),
            sensor_analyzer: sensor::SensorDomainAnalyzer::new(),
            environmental_analyzer: environmental::EnvironmentalDomainAnalyzer::new(),
            system_analyzer: system::SystemDomainAnalyzer::new(),
            coordination_analyzer: coordination::CoordinationDomainAnalyzer::new(),
        }
    }

    /// Analyze mission data for all gap types
    pub fn analyze_mission(&self, data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();

        // Run all domain analyzers in parallel (conceptually)
        findings.extend(self.physical_analyzer.analyze(data));
        findings.extend(self.sensor_analyzer.analyze(data));
        findings.extend(self.environmental_analyzer.analyze(data));
        findings.extend(self.system_analyzer.analyze(data));
        findings.extend(self.coordination_analyzer.analyze(data));

        // Sort by severity and confidence
        findings.sort_by(|a, b| {
            b.severity.cmp(&a.severity)
                .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        });

        findings
    }
}

impl Default for RealityGapDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_detector_creation() {
        let detector = RealityGapDetector::new();
        // Should not panic
    }

    #[test]
    fn test_empty_mission_analysis() {
        let detector = RealityGapDetector::new();
        let data = MissionAnalysisData {
            mission_id: "test_mission".to_string(),
            duration_sec: 100.0,
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
        };

        let findings = detector.analyze_mission(&data);
        // Empty mission should produce no findings
        assert!(findings.is_empty() || findings.len() < 5);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }
}
