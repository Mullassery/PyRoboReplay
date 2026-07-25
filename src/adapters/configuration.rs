/// Adapter for parsing configuration files (Layer 4)
///
/// Supports:
/// - YAML config files (Nav2, SLAM, launch parameters)
/// - Hardware configuration
/// - Parameter validation
///
/// Normalizes to MissionEvent::ConfigurationEvent and MissionEvent::ParameterValidationEvent

use crate::core::event::MissionEvent;
use crate::adapters::AdapterError;
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConfigValidationError {
    pub parameter: String,
    pub reason: String,
    pub expected: Option<String>,
    pub actual: String,
}

/// Parser for configuration files
pub struct ConfigurationAdapter;

impl ConfigurationAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Parse YAML configuration file
    pub fn parse_yaml(&self, content: &str, config_type: &str, filename: &str) -> Result<Vec<MissionEvent>, AdapterError> {
        let mut events = Vec::new();
        let timestamp = Utc::now();

        // Parse YAML
        let config: serde_yaml::Value = serde_yaml::from_str(content)
            .map_err(|e| AdapterError::ParseError(format!("Invalid YAML: {}", e)))?;

        // Extract parameters
        if let Some(map) = config.as_mapping() {
            for (key, value) in map.iter() {
                let param_name = key
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{:?}", key));

                let value_str = match value {
                    serde_yaml::Value::String(s) => s.clone(),
                    serde_yaml::Value::Number(n) => n.to_string(),
                    serde_yaml::Value::Bool(b) => b.to_string(),
                    _ => format!("{:?}", value),
                };

                // Create configuration event
                events.push(MissionEvent::ConfigurationEvent {
                    timestamp,
                    config_type: config_type.to_string(),
                    parameter_name: Some(param_name.clone()),
                    old_value: None,
                    new_value: Some(value_str.clone()),
                    config_file: filename.to_string(),
                    description: None,
                });

                // Validate parameter
                if let Some(validation) = self.validate_parameter(&param_name, &value_str, config_type) {
                    events.push(MissionEvent::ParameterValidationEvent {
                        timestamp,
                        parameter_name: param_name,
                        current_value: value_str,
                        expected_range: validation.expected,
                        severity: if validation.reason.contains("critical") {
                            "error".to_string()
                        } else {
                            "warning".to_string()
                        },
                        message: validation.reason,
                    });
                }
            }
        }

        Ok(events)
    }

    /// Validate a parameter against expected ranges and known issues
    fn validate_parameter(&self, param_name: &str, value_str: &str, config_type: &str) -> Option<ConfigValidationError> {
        let param_lower = param_name.to_lowercase();
        let value_lower = value_str.to_lowercase();

        // Navigation2 validations
        if config_type == "nav2" {
            // Planner timeout validation
            if param_lower.contains("timeout") {
                if let Ok(timeout) = value_str.parse::<f32>() {
                    if timeout < 0.5 {
                        return Some(ConfigValidationError {
                            parameter: param_name.to_string(),
                            reason: "Planner timeout too low (critical: may cause rapid replanning)".to_string(),
                            expected: Some("0.5-5.0 seconds".to_string()),
                            actual: value_str.to_string(),
                        });
                    }
                }
            }

            // Costmap update frequency validation
            if param_lower.contains("update_frequency") || param_lower.contains("hz") {
                if let Ok(hz) = value_str.parse::<f32>() {
                    if hz < 1.0 {
                        return Some(ConfigValidationError {
                            parameter: param_name.to_string(),
                            reason: "Costmap update frequency too low (warning: may miss obstacles)".to_string(),
                            expected: Some("1.0-10.0 Hz".to_string()),
                            actual: value_str.to_string(),
                        });
                    }
                }
            }

            // Inflation radius validation
            if param_lower.contains("inflation_radius") {
                if let Ok(radius) = value_str.parse::<f32>() {
                    if radius < 0.1 {
                        return Some(ConfigValidationError {
                            parameter: param_name.to_string(),
                            reason: "Inflation radius too small (warning: robot may clip obstacles)".to_string(),
                            expected: Some("0.1-0.5 meters".to_string()),
                            actual: value_str.to_string(),
                        });
                    }
                }
            }

            // Transform tolerance validation
            if param_lower.contains("transform_tolerance") {
                if let Ok(tolerance) = value_str.parse::<f32>() {
                    if tolerance > 1.0 {
                        return Some(ConfigValidationError {
                            parameter: param_name.to_string(),
                            reason: "Transform tolerance too high (warning: loose TF matching)".to_string(),
                            expected: Some("0.01-0.5 seconds".to_string()),
                            actual: value_str.to_string(),
                        });
                    }
                }
            }
        }

        // SLAM validations
        if config_type == "slam" {
            // Map update frequency
            if param_lower.contains("update_frequency") || param_lower.contains("hz") {
                if let Ok(hz) = value_str.parse::<f32>() {
                    if hz > 50.0 {
                        return Some(ConfigValidationError {
                            parameter: param_name.to_string(),
                            reason: "Map update frequency too high (warning: may cause CPU overload)".to_string(),
                            expected: Some("1.0-20.0 Hz".to_string()),
                            actual: value_str.to_string(),
                        });
                    }
                }
            }

            // Loop closure threshold
            if param_lower.contains("loop_closure") && param_lower.contains("threshold") {
                if let Ok(threshold) = value_str.parse::<f32>() {
                    if threshold < 0.5 || threshold > 0.99 {
                        return Some(ConfigValidationError {
                            parameter: param_name.to_string(),
                            reason: "Loop closure threshold out of range (warning: may miss/falsely detect loop closures)".to_string(),
                            expected: Some("0.5-0.99".to_string()),
                            actual: value_str.to_string(),
                        });
                    }
                }
            }
        }

        None
    }

    /// Detect known anti-patterns in configuration
    pub fn detect_anti_patterns(&self, config_type: &str, params: &HashMap<String, String>) -> Vec<String> {
        let mut issues = Vec::new();

        if config_type == "nav2" {
            // Anti-pattern: Low timeout + High costmap frequency (thrashing)
            let has_low_timeout = params
                .iter()
                .any(|(k, v)| k.to_lowercase().contains("timeout") && v.parse::<f32>().unwrap_or(1.0) < 1.0);

            let has_high_costmap_freq = params
                .iter()
                .any(|(k, v)| {
                    k.to_lowercase().contains("update_frequency") && v.parse::<f32>().unwrap_or(10.0) > 20.0
                });

            if has_low_timeout && has_high_costmap_freq {
                issues.push("Anti-pattern: Low planner timeout + high costmap frequency may cause rapid replanning".to_string());
            }

            // Anti-pattern: Very small inflation radius
            if params
                .iter()
                .any(|(k, v)| {
                    k.to_lowercase().contains("inflation_radius") && v.parse::<f32>().unwrap_or(0.2) < 0.05
                })
            {
                issues.push("Anti-pattern: Very small inflation radius may cause collision".to_string());
            }

            // Anti-pattern: Large transform tolerance with poor localization
            if params
                .iter()
                .any(|(k, v)| {
                    k.to_lowercase().contains("transform_tolerance") && v.parse::<f32>().unwrap_or(0.1) > 0.5
                })
            {
                issues.push("Anti-pattern: Large transform tolerance may cause navigation in wrong frame".to_string());
            }
        }

        issues
    }

    /// Get expected parameter ranges for a config type
    pub fn expected_ranges(&self, config_type: &str) -> HashMap<String, (f32, f32)> {
        let mut ranges = HashMap::new();

        if config_type == "nav2" {
            ranges.insert("planner_timeout".to_string(), (0.5, 10.0));
            ranges.insert("update_frequency".to_string(), (1.0, 50.0));
            ranges.insert("inflation_radius".to_string(), (0.1, 1.0));
            ranges.insert("transform_tolerance".to_string(), (0.01, 1.0));
        } else if config_type == "slam" {
            ranges.insert("update_frequency".to_string(), (1.0, 50.0));
            ranges.insert("loop_closure_threshold".to_string(), (0.5, 0.99));
        }

        ranges
    }
}

impl Default for ConfigurationAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = ConfigurationAdapter::new();
        let _ = adapter.parse_yaml("key: value", "test", "test.yaml");
    }

    #[test]
    fn test_validate_low_timeout() {
        let adapter = ConfigurationAdapter::new();
        let validation = adapter.validate_parameter("planner_timeout", "0.2", "nav2");
        assert!(validation.is_some());
        if let Some(err) = validation {
            assert!(err.reason.contains("too low"));
        }
    }

    #[test]
    fn test_validate_low_costmap_frequency() {
        let adapter = ConfigurationAdapter::new();
        let validation = adapter.validate_parameter("costmap_update_frequency", "0.5", "nav2");
        assert!(validation.is_some());
    }

    #[test]
    fn test_detect_anti_patterns() {
        let adapter = ConfigurationAdapter::new();
        let mut params = HashMap::new();
        params.insert("planner_timeout".to_string(), "0.5".to_string());
        params.insert("update_frequency".to_string(), "30.0".to_string());

        let issues = adapter.detect_anti_patterns("nav2", &params);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_expected_ranges() {
        let adapter = ConfigurationAdapter::new();
        let ranges = adapter.expected_ranges("nav2");
        assert!(ranges.contains_key("planner_timeout"));
        assert!(ranges.contains_key("inflation_radius"));
    }
}
