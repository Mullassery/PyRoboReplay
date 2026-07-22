//! Per-Robot Type Calibration
//!
//! Learn robot-type specific gap patterns and customize severity thresholds by fleet.

use std::collections::HashMap;

/// Robot-type specific gap profile
#[derive(Debug, Clone)]
pub struct RobotTypeProfile {
    pub robot_type: String,
    pub mission_count: usize,
    pub failure_count: usize,
    pub gap_frequencies: HashMap<String, f32>, // category -> frequency (0.0-1.0)
    pub gap_severities: HashMap<String, (f32, f32)>, // category -> (avg_severity_score, std_dev)
    pub thermal_sensitivity: f32,      // How much thermal effects matter (0.0-1.0)
    pub mechanical_sensitivity: f32,   // How much mechanical wear matters
    pub sensor_sensitivity: f32,       // How much sensor issues matter
    pub learned_severity_threshold: f32, // Customized threshold for this robot type
}

impl RobotTypeProfile {
    /// Create new profile for a robot type
    pub fn new(robot_type: &str) -> Self {
        RobotTypeProfile {
            robot_type: robot_type.to_string(),
            mission_count: 0,
            failure_count: 0,
            gap_frequencies: HashMap::new(),
            gap_severities: HashMap::new(),
            thermal_sensitivity: 1.0,      // Default: neutral
            mechanical_sensitivity: 1.0,
            sensor_sensitivity: 1.0,
            learned_severity_threshold: 0.6, // Default: medium/high cutoff
        }
    }

    /// Update gap frequency for a category
    pub fn record_gap(&mut self, category: &str, severity_score: f32) {
        *self.gap_frequencies.entry(category.to_string()).or_insert(0.0) += 0.01; // Incremental
        self.gap_frequencies
            .entry(category.to_string())
            .and_modify(|f| *f = f.min(1.0)); // Cap at 1.0

        let (sum, count) = *self
            .gap_severities
            .entry(category.to_string())
            .or_insert((0.0, 0.0));

        let new_count = count + 1.0;
        let new_sum = sum + severity_score;
        let avg = new_sum / new_count;
        let std_dev = ((sum * sum + severity_score * severity_score) / new_count - avg * avg)
            .sqrt();

        self.gap_severities
            .insert(category.to_string(), (avg, std_dev));
    }

    /// Record mission outcome
    pub fn record_mission(&mut self, success: bool) {
        self.mission_count += 1;
        if !success {
            self.failure_count += 1;
        }
    }

    /// Get failure rate for this robot type
    pub fn failure_rate(&self) -> f32 {
        if self.mission_count == 0 {
            return 0.0;
        }
        self.failure_count as f32 / self.mission_count as f32
    }

    /// Get most common gap type for this robot
    pub fn most_common_gap(&self) -> Option<(String, f32)> {
        self.gap_frequencies
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, v)| (k.clone(), *v))
    }

    /// Get gap severity patterns (used for robot-specific severity scaling)
    pub fn severity_pattern(&self, category: &str) -> Option<(f32, f32)> {
        self.gap_severities.get(category).copied()
    }
}

/// Robot calibration manager for the fleet
pub struct RobotCalibrationManager {
    profiles: HashMap<String, RobotTypeProfile>,
    fleet_size: usize,
}

impl RobotCalibrationManager {
    /// Create new calibration manager
    pub fn new() -> Self {
        RobotCalibrationManager {
            profiles: HashMap::new(),
            fleet_size: 0,
        }
    }

    /// Register a new robot type in the fleet
    pub fn register_robot_type(&mut self, robot_type: &str) {
        self.profiles.insert(
            robot_type.to_string(),
            RobotTypeProfile::new(robot_type),
        );
    }

    /// Record gap observation for a robot type
    pub fn record_gap(&mut self, robot_type: &str, category: &str, severity_score: f32) {
        if !self.profiles.contains_key(robot_type) {
            self.register_robot_type(robot_type);
        }

        if let Some(profile) = self.profiles.get_mut(robot_type) {
            profile.record_gap(category, severity_score);
        }
    }

    /// Record mission result
    pub fn record_mission(&mut self, robot_type: &str, success: bool) {
        if !self.profiles.contains_key(robot_type) {
            self.register_robot_type(robot_type);
        }

        if let Some(profile) = self.profiles.get_mut(robot_type) {
            profile.record_mission(success);
        }

        self.fleet_size += 1;
    }

    /// Calibrate sensitivities for all robot types
    pub fn calibrate_sensitivities(&mut self) {
        for profile in self.profiles.values_mut() {
            // Thermal sensitivity: how common are thermal gaps?
            let thermal_freq = profile.gap_frequencies.get("Thermal Effects").copied().unwrap_or(0.0);
            profile.thermal_sensitivity = 1.0 + thermal_freq; // Range: 1.0-2.0

            // Mechanical sensitivity: how common are mechanical gaps?
            let mech_freq = profile.gap_frequencies.get("Mechanical Degradation").copied().unwrap_or(0.0);
            profile.mechanical_sensitivity = 1.0 + mech_freq * 0.5; // Range: 1.0-1.5

            // Sensor sensitivity: how common are sensor gaps?
            let sensor_freqs: f32 = [
                "Optical Contamination",
                "Detection Robustness",
                "Sensor Calibration Drift",
            ]
            .iter()
            .filter_map(|cat| profile.gap_frequencies.get(*cat).copied())
            .sum();
            profile.sensor_sensitivity = 1.0 + sensor_freqs * 0.3; // Range: 1.0-1.9
        }
    }

    /// Learn severity threshold for a robot type
    pub fn learn_severity_threshold(&mut self, robot_type: &str) {
        if let Some(profile) = self.profiles.get_mut(robot_type) {
            // Threshold inversely correlated with failure rate:
            // High failure rate -> lower threshold (be more conservative)
            // Low failure rate -> higher threshold (fewer false alarms)
            let failure_rate = profile.failure_rate();
            profile.learned_severity_threshold = 0.5 + (0.4 * (1.0 - failure_rate));
            // Range: 0.1 (all failures) to 0.9 (no failures)
        }
    }

    /// Get calibration for a robot type
    pub fn get_profile(&self, robot_type: &str) -> Option<&RobotTypeProfile> {
        self.profiles.get(robot_type)
    }

    /// Get all profiles
    pub fn all_profiles(&self) -> Vec<&RobotTypeProfile> {
        self.profiles.values().collect()
    }

    /// Get fleet-wide statistics
    pub fn fleet_statistics(&self) -> FleetStats {
        let mut total_missions = 0;
        let mut total_failures = 0;
        let mut avg_thermal_sensitivity = 0.0;
        let mut avg_mechanical_sensitivity = 0.0;

        let profile_count = self.profiles.len() as f32;

        for profile in self.profiles.values() {
            total_missions += profile.mission_count;
            total_failures += profile.failure_count;
            avg_thermal_sensitivity += profile.thermal_sensitivity;
            avg_mechanical_sensitivity += profile.mechanical_sensitivity;
        }

        FleetStats {
            robot_type_count: self.profiles.len(),
            total_missions,
            total_failures,
            overall_failure_rate: if total_missions > 0 {
                total_failures as f32 / total_missions as f32
            } else {
                0.0
            },
            avg_thermal_sensitivity: avg_thermal_sensitivity / profile_count.max(1.0),
            avg_mechanical_sensitivity: avg_mechanical_sensitivity / profile_count.max(1.0),
        }
    }

    /// Predict severity score for a gap given robot type
    pub fn predict_severity(
        &self,
        robot_type: &str,
        category: &str,
        base_severity: f32,
    ) -> f32 {
        if let Some(profile) = self.get_profile(robot_type) {
            let mut multiplier = 1.0;

            // Apply sensitivity multipliers
            match category {
                "Thermal Effects" => multiplier = profile.thermal_sensitivity,
                "Mechanical Degradation" => multiplier = profile.mechanical_sensitivity,
                "Optical Contamination" | "Detection Robustness" => multiplier = profile.sensor_sensitivity,
                _ => {}
            }

            // Apply learned threshold: if gap is below threshold for this robot type, reduce severity
            let threshold_adjustment = if base_severity < profile.learned_severity_threshold {
                0.6 // Scale down non-critical gaps for this robot type (more aggressive)
            } else {
                1.3 // Scale up critical gaps
            };

            (base_severity * multiplier * threshold_adjustment).clamp(0.0, 1.0)
        } else {
            base_severity // Fallback: no adjustment
        }
    }
}

impl Default for RobotCalibrationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Fleet-wide statistics
#[derive(Debug, Clone)]
pub struct FleetStats {
    pub robot_type_count: usize,
    pub total_missions: usize,
    pub total_failures: usize,
    pub overall_failure_rate: f32,
    pub avg_thermal_sensitivity: f32,
    pub avg_mechanical_sensitivity: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = RobotTypeProfile::new("mobile_robot");
        assert_eq!(profile.robot_type, "mobile_robot");
        assert_eq!(profile.mission_count, 0);
        assert_eq!(profile.failure_rate(), 0.0);
    }

    #[test]
    fn test_record_gap() {
        let mut profile = RobotTypeProfile::new("mobile_robot");

        profile.record_gap("Mechanical Degradation", 0.7);
        profile.record_gap("Mechanical Degradation", 0.8);

        let freq = profile.gap_frequencies.get("Mechanical Degradation").unwrap();
        assert!(*freq > 0.0);
    }

    #[test]
    fn test_failure_rate() {
        let mut profile = RobotTypeProfile::new("mobile_robot");

        profile.record_mission(true);
        profile.record_mission(true);
        profile.record_mission(false);
        profile.record_mission(false);

        let rate = profile.failure_rate();
        assert!((rate - 0.5).abs() < 0.01); // 2/4 = 0.5
    }

    #[test]
    fn test_calibration_manager_creation() {
        let _manager = RobotCalibrationManager::new();
    }

    #[test]
    fn test_register_robot_type() {
        let mut manager = RobotCalibrationManager::new();
        manager.register_robot_type("mobile_robot");
        manager.register_robot_type("drone");

        assert_eq!(manager.profiles.len(), 2);
    }

    #[test]
    fn test_record_gap_auto_register() {
        let mut manager = RobotCalibrationManager::new();

        manager.record_gap("mobile_robot", "Mechanical Degradation", 0.7);

        assert!(manager.profiles.contains_key("mobile_robot"));
    }

    #[test]
    fn test_calibrate_sensitivities() {
        let mut manager = RobotCalibrationManager::new();
        manager.register_robot_type("thermal_bot");

        let profile = manager.profiles.get_mut("thermal_bot").unwrap();
        profile.gap_frequencies.insert("Thermal Effects".to_string(), 0.8);

        drop(profile);
        manager.calibrate_sensitivities();

        let profile = manager.get_profile("thermal_bot").unwrap();
        assert!(profile.thermal_sensitivity > 1.5); // 1.0 + 0.8
    }

    #[test]
    fn test_learn_severity_threshold() {
        let mut manager = RobotCalibrationManager::new();
        manager.register_robot_type("reliable_bot");

        let profile = manager.profiles.get_mut("reliable_bot").unwrap();
        profile.record_mission(true);
        profile.record_mission(true);
        profile.record_mission(true);
        profile.record_mission(true);
        profile.record_mission(false); // 80% success rate

        drop(profile);
        manager.learn_severity_threshold("reliable_bot");

        let profile = manager.get_profile("reliable_bot").unwrap();
        // Failure rate = 0.2, threshold = 0.5 + 0.4 * 0.8 = 0.82
        assert!(profile.learned_severity_threshold > 0.8);
    }

    #[test]
    fn test_predict_severity() {
        let mut manager = RobotCalibrationManager::new();
        manager.register_robot_type("thermal_bot");

        let profile = manager.profiles.get_mut("thermal_bot").unwrap();
        profile.thermal_sensitivity = 1.5;
        profile.learned_severity_threshold = 0.6;

        drop(profile);

        // Thermal gap with high base severity
        let adjusted = manager.predict_severity("thermal_bot", "Thermal Effects", 0.8);
        assert!(adjusted > 0.8); // Should be boosted

        // Thermal gap with low base severity
        let adjusted = manager.predict_severity("thermal_bot", "Thermal Effects", 0.4);
        assert!(adjusted < 0.4); // Should be scaled down
    }

    #[test]
    fn test_fleet_statistics() {
        let mut manager = RobotCalibrationManager::new();

        manager.record_mission("mobile_robot", true);
        manager.record_mission("mobile_robot", false);
        manager.record_mission("drone", true);

        let stats = manager.fleet_statistics();
        assert_eq!(stats.total_missions, 3);
        assert_eq!(stats.total_failures, 1);
        assert_eq!(stats.robot_type_count, 2);
    }
}
