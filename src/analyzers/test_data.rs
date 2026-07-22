//! Test Data Generator for Reality Gap Detection
//!
//! Creates synthetic MissionAnalysisData with controlled gaps for validation.

use crate::analyzers::{
    MissionAnalysisData, ControlMessage, JointState, OdometryMessage, CameraFrame,
    LidarScan, IMUMeasurement, EncoderReading, MotorCurrent, ThermalReading,
    BatteryReading, DetectionResult, PerceptionError, MessageTimestamp,
};

/// Generate synthetic mission data for testing
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate mission with mechanical degradation (response time increases 50%)
    pub fn mechanical_degradation_mission() -> MissionAnalysisData {
        let mut data = MissionAnalysisData {
            mission_id: "test_mechanical_degradation".to_string(),
            duration_sec: 600.0,
            robot_type: "mobile_robot".to_string(),
            control_messages: Vec::new(),
            joint_states: Vec::new(),
            odometry_messages: Vec::new(),
            camera_frames: Vec::new(),
            lidar_scans: Vec::new(),
            imu_measurements: Vec::new(),
            encoder_data: Vec::new(),
            motor_currents: Vec::new(),
            thermal_readings: Vec::new(),
            battery_data: Vec::new(),
            detection_results: Vec::new(),
            perception_errors: Vec::new(),
            message_timestamps: Vec::new(),
        };

        // Generate control-response pairs with degrading response time
        for i in 0..50 {
            let time = (i as f32) * 10.0; // 10 seconds apart
            let progress = (i as f32) / 50.0; // 0.0 to 1.0

            // Response time increases: starts at 100ms, ends at 150ms (50% increase)
            let base_response_time = 0.100 + progress * 0.050;

            data.control_messages.push(ControlMessage {
                timestamp: time,
                joint_id: "joint_1".to_string(),
                command_type: "position".to_string(),
                value: 1.57,
            });

            data.joint_states.push(JointState {
                timestamp: time + base_response_time,
                joint_id: "joint_1".to_string(),
                position: 1.57,
                velocity: 0.0,
                effort: 1.0,
            });
        }

        data
    }

    /// Generate mission with optical contamination (image sharpness declines)
    pub fn optical_contamination_mission() -> MissionAnalysisData {
        let mut data = MissionAnalysisData {
            mission_id: "test_optical_contamination".to_string(),
            duration_sec: 600.0,
            robot_type: "mobile_robot".to_string(),
            control_messages: Vec::new(),
            joint_states: Vec::new(),
            odometry_messages: Vec::new(),
            camera_frames: Vec::new(),
            lidar_scans: Vec::new(),
            imu_measurements: Vec::new(),
            encoder_data: Vec::new(),
            motor_currents: Vec::new(),
            thermal_readings: Vec::new(),
            battery_data: Vec::new(),
            detection_results: Vec::new(),
            perception_errors: Vec::new(),
            message_timestamps: Vec::new(),
        };

        // Generate camera frames with degrading sharpness (clean to dirty)
        for i in 0..100 {
            let time = (i as f32) * 6.0; // 6 seconds apart
            let progress = (i as f32) / 100.0; // 0.0 to 1.0

            // Sharpness decreases: starts at 90, ends at 50 (44% decline)
            let sharpness = 90.0 - (progress * 40.0);

            data.camera_frames.push(CameraFrame {
                timestamp: time,
                camera_id: "camera_0".to_string(),
                width: 640,
                height: 480,
                frame_index: i,
                quality_sharpness: Some(sharpness),
                quality_entropy: Some(7.5 - progress * 2.0),
            });

            // Detection confidence declines with sharpness
            data.detection_results.push(DetectionResult {
                timestamp: time,
                frame_index: i,
                class: "obstacle".to_string(),
                confidence: 0.85 - (progress * 0.25),
                x: 320.0,
                y: 240.0,
                width: 50.0,
                height: 50.0,
            });
        }

        data
    }

    /// Generate mission with thermal effects (motor efficiency declines with heat)
    pub fn thermal_effects_mission() -> MissionAnalysisData {
        let mut data = MissionAnalysisData {
            mission_id: "test_thermal_effects".to_string(),
            duration_sec: 600.0,
            robot_type: "mobile_robot".to_string(),
            control_messages: Vec::new(),
            joint_states: Vec::new(),
            odometry_messages: Vec::new(),
            camera_frames: Vec::new(),
            lidar_scans: Vec::new(),
            imu_measurements: Vec::new(),
            encoder_data: Vec::new(),
            motor_currents: Vec::new(),
            thermal_readings: Vec::new(),
            battery_data: Vec::new(),
            detection_results: Vec::new(),
            perception_errors: Vec::new(),
            message_timestamps: Vec::new(),
        };

        // Generate motor current and thermal readings
        for i in 0..100 {
            let time = (i as f32) * 6.0;
            let progress = (i as f32) / 100.0;

            // Temperature increases from 25°C to 85°C
            let temperature = 25.0 + progress * 60.0;

            // Motor current increases with temperature (efficiency degrades)
            // Starts at 10A, increases to 12A (20% increase)
            let current = 10.0 + progress * 2.0;

            data.motor_currents.push(MotorCurrent {
                timestamp: time,
                joint_id: "motor_1".to_string(),
                current_amps: current,
            });

            data.thermal_readings.push(ThermalReading {
                timestamp: time,
                location: "motor_1".to_string(),
                temperature_c: temperature,
            });

            data.joint_states.push(JointState {
                timestamp: time,
                joint_id: "motor_1".to_string(),
                position: (i as f32) * 0.1,
                velocity: 1.0 - progress * 0.2, // Velocity decreases with heat
                effort: current,
            });
        }

        data
    }

    /// Generate mission with clock drift (sensor timing skew)
    pub fn clock_drift_mission() -> MissionAnalysisData {
        let mut data = MissionAnalysisData {
            mission_id: "test_clock_drift".to_string(),
            duration_sec: 600.0,
            robot_type: "mobile_robot".to_string(),
            control_messages: Vec::new(),
            joint_states: Vec::new(),
            odometry_messages: Vec::new(),
            camera_frames: Vec::new(),
            lidar_scans: Vec::new(),
            imu_measurements: Vec::new(),
            encoder_data: Vec::new(),
            motor_currents: Vec::new(),
            thermal_readings: Vec::new(),
            battery_data: Vec::new(),
            detection_results: Vec::new(),
            perception_errors: Vec::new(),
            message_timestamps: Vec::new(),
        };

        // Generate LiDAR messages with clock running 1000 ppm fast
        let lidar_rate_hz = 10.0;
        let nominal_interval = 1.0 / lidar_rate_hz;
        let drift_factor = 1.001; // 1000 ppm = 0.1% = 1.001x

        for i in 0..100 {
            let nominal_time = (i as f32) * nominal_interval;
            let drifted_time = nominal_time * drift_factor; // Clock running fast

            data.message_timestamps.push(MessageTimestamp {
                timestamp: drifted_time,
                message_type: "lidar_scan".to_string(),
                sensor_id: "lidar_0".to_string(),
            });
        }

        // Also generate camera messages (reference clock, no drift)
        for i in 0..50 {
            let time = (i as f32) * 0.033; // ~30 Hz

            data.message_timestamps.push(MessageTimestamp {
                timestamp: time,
                message_type: "camera_frame".to_string(),
                sensor_id: "camera_0".to_string(),
            });
        }

        data
    }

    /// Generate mission with detection failures (lighting-induced)
    pub fn detection_failure_mission() -> MissionAnalysisData {
        let mut data = MissionAnalysisData {
            mission_id: "test_detection_failure".to_string(),
            duration_sec: 600.0,
            robot_type: "mobile_robot".to_string(),
            control_messages: Vec::new(),
            joint_states: Vec::new(),
            odometry_messages: Vec::new(),
            camera_frames: Vec::new(),
            lidar_scans: Vec::new(),
            imu_measurements: Vec::new(),
            encoder_data: Vec::new(),
            motor_currents: Vec::new(),
            thermal_readings: Vec::new(),
            battery_data: Vec::new(),
            detection_results: Vec::new(),
            perception_errors: Vec::new(),
            message_timestamps: Vec::new(),
        };

        // Generate detection results with confidence declining due to lighting
        for i in 0..100 {
            let time = (i as f32) * 6.0;
            let progress = (i as f32) / 100.0;

            // Image brightness changes (simulates time of day or shadow)
            let brightness = 0.5 + 0.3 * (progress * 4.0 * std::f32::consts::PI).sin();

            // Camera sharpness stable (not optical issue)
            data.camera_frames.push(CameraFrame {
                timestamp: time,
                camera_id: "camera_0".to_string(),
                width: 640,
                height: 480,
                frame_index: i,
                quality_sharpness: Some(85.0), // Stays constant
                quality_entropy: Some(6.0 + brightness),
            });

            // Detection confidence varies with brightness (environmental issue)
            let detection_count = if brightness > 0.6 { 5 } else { 2 };

            for j in 0..detection_count {
                data.detection_results.push(DetectionResult {
                    timestamp: time + (j as f32) * 0.1,
                    frame_index: i,
                    class: "obstacle".to_string(),
                    confidence: 0.85 - (1.0 - brightness) * 0.3,
                    x: 100.0 + (j as f32) * 100.0,
                    y: 240.0,
                    width: 40.0,
                    height: 40.0,
                });
            }

            // Add false positives in dark regions
            if brightness < 0.4 {
                data.detection_results.push(DetectionResult {
                    timestamp: time,
                    frame_index: i,
                    class: "noise".to_string(),
                    confidence: 0.2, // Low confidence false positive
                    x: 50.0,
                    y: 100.0,
                    width: 30.0,
                    height: 30.0,
                });
            }
        }

        data
    }

    /// Generate mission with no gaps (healthy robot behavior)
    pub fn healthy_mission() -> MissionAnalysisData {
        let mut data = MissionAnalysisData {
            mission_id: "test_healthy".to_string(),
            duration_sec: 600.0,
            robot_type: "mobile_robot".to_string(),
            control_messages: Vec::new(),
            joint_states: Vec::new(),
            odometry_messages: Vec::new(),
            camera_frames: Vec::new(),
            lidar_scans: Vec::new(),
            imu_measurements: Vec::new(),
            encoder_data: Vec::new(),
            motor_currents: Vec::new(),
            thermal_readings: Vec::new(),
            battery_data: Vec::new(),
            detection_results: Vec::new(),
            perception_errors: Vec::new(),
            message_timestamps: Vec::new(),
        };

        // Consistent control-response pairs
        for i in 0..50 {
            let time = (i as f32) * 10.0;
            let stable_response_time = 0.100; // Constant 100ms

            data.control_messages.push(ControlMessage {
                timestamp: time,
                joint_id: "joint_1".to_string(),
                command_type: "position".to_string(),
                value: 1.57,
            });

            data.joint_states.push(JointState {
                timestamp: time + stable_response_time,
                joint_id: "joint_1".to_string(),
                position: 1.57,
                velocity: 0.0,
                effort: 1.0,
            });
        }

        // Stable motor current
        for i in 0..100 {
            let time = (i as f32) * 6.0;

            data.motor_currents.push(MotorCurrent {
                timestamp: time,
                joint_id: "motor_1".to_string(),
                current_amps: 10.0, // Stable
            });

            data.thermal_readings.push(ThermalReading {
                timestamp: time,
                location: "motor_1".to_string(),
                temperature_c: 40.0, // Stable
            });
        }

        // Good detection performance
        for i in 0..100 {
            let time = (i as f32) * 6.0;

            data.camera_frames.push(CameraFrame {
                timestamp: time,
                camera_id: "camera_0".to_string(),
                width: 640,
                height: 480,
                frame_index: i,
                quality_sharpness: Some(85.0), // Stable
                quality_entropy: Some(6.5),
            });

            data.detection_results.push(DetectionResult {
                timestamp: time,
                frame_index: i,
                class: "obstacle".to_string(),
                confidence: 0.85, // Stable confidence
                x: 320.0,
                y: 240.0,
                width: 50.0,
                height: 50.0,
            });
        }

        // Stable sensor timing
        for i in 0..100 {
            let time = (i as f32) * 0.1; // 10 Hz

            data.message_timestamps.push(MessageTimestamp {
                timestamp: time,
                message_type: "lidar_scan".to_string(),
                sensor_id: "lidar_0".to_string(),
            });
        }

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mechanical_degradation_generation() {
        let mission = TestDataGenerator::mechanical_degradation_mission();
        assert_eq!(mission.mission_id, "test_mechanical_degradation");
        assert_eq!(mission.control_messages.len(), 50);
        assert_eq!(mission.joint_states.len(), 50);
    }

    #[test]
    fn test_optical_contamination_generation() {
        let mission = TestDataGenerator::optical_contamination_mission();
        assert_eq!(mission.mission_id, "test_optical_contamination");
        assert_eq!(mission.camera_frames.len(), 100);
        assert_eq!(mission.detection_results.len(), 100);
    }

    #[test]
    fn test_thermal_effects_generation() {
        let mission = TestDataGenerator::thermal_effects_mission();
        assert_eq!(mission.mission_id, "test_thermal_effects");
        assert_eq!(mission.motor_currents.len(), 100);
        assert_eq!(mission.thermal_readings.len(), 100);
    }

    #[test]
    fn test_clock_drift_generation() {
        let mission = TestDataGenerator::clock_drift_mission();
        assert_eq!(mission.mission_id, "test_clock_drift");
        // Should have both lidar and camera messages
        let lidar_msgs = mission
            .message_timestamps
            .iter()
            .filter(|m| m.sensor_id == "lidar_0")
            .count();
        assert_eq!(lidar_msgs, 100);
    }

    #[test]
    fn test_detection_failure_generation() {
        let mission = TestDataGenerator::detection_failure_mission();
        assert_eq!(mission.mission_id, "test_detection_failure");
        assert!(!mission.detection_results.is_empty());
        assert!(!mission.camera_frames.is_empty());
    }

    #[test]
    fn test_healthy_mission_generation() {
        let mission = TestDataGenerator::healthy_mission();
        assert_eq!(mission.mission_id, "test_healthy");
        assert!(!mission.control_messages.is_empty());
        assert!(!mission.motor_currents.is_empty());
    }
}
