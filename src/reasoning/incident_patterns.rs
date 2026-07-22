//! Incident Pattern Analysis
//!
//! Discovers patterns across fleet of missions to identify systemic issues.

use std::collections::HashMap;

/// Pattern discovered across multiple missions
#[derive(Debug, Clone)]
pub struct IncidentPattern {
    /// Pattern name
    pub pattern_type: String,

    /// How many missions exhibit this pattern
    pub occurrence_count: usize,

    /// Percentage of fleet affected
    pub fleet_percentage: f32,

    /// Affected robot types
    pub robot_types: Vec<String>,

    /// Common root causes
    pub root_causes: Vec<String>,

    /// Confidence in this pattern (0.0-1.0)
    pub confidence: f32,

    /// Recommended fleet-wide action
    pub fleet_action: String,
}

/// Pattern analysis engine
pub struct IncidentPatternAnalyzer;

impl IncidentPatternAnalyzer {
    /// Analyze fleet for patterns
    pub fn analyze_fleet(incidents: &[FleetIncident]) -> Vec<IncidentPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: Collision by object type
        let collision_patterns = Self::analyze_collision_patterns(incidents);
        patterns.extend(collision_patterns);

        // Pattern 2: Sensor blind spots
        let sensor_patterns = Self::analyze_sensor_patterns(incidents);
        patterns.extend(sensor_patterns);

        // Pattern 3: Environmental failures
        let env_patterns = Self::analyze_environmental_patterns(incidents);
        patterns.extend(env_patterns);

        // Sort by severity (occurrence count)
        patterns.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));

        patterns
    }

    /// Analyze collision patterns
    fn analyze_collision_patterns(incidents: &[FleetIncident]) -> Vec<IncidentPattern> {
        let mut patterns = Vec::new();
        let mut collision_by_object: HashMap<String, usize> = HashMap::new();

        for incident in incidents {
            if incident.outcome == "collision" {
                for obj in &incident.objects_involved {
                    *collision_by_object.entry(obj.clone()).or_insert(0) += 1;
                }
            }
        }

        for (obj_type, count) in collision_by_object {
            if count > 1 {
                patterns.push(IncidentPattern {
                    pattern_type: format!("Collision with {}", obj_type),
                    occurrence_count: count,
                    fleet_percentage: (count as f32 / incidents.len() as f32) * 100.0,
                    robot_types: Self::extract_robot_types(incidents, &obj_type),
                    root_causes: vec!["Perception gap".to_string(), "Sensor limitation".to_string()],
                    confidence: 0.85,
                    fleet_action: format!(
                        "Enhance detection for {} objects; add redundant sensors",
                        obj_type
                    ),
                });
            }
        }

        patterns
    }

    /// Analyze sensor blind spot patterns
    fn analyze_sensor_patterns(incidents: &[FleetIncident]) -> Vec<IncidentPattern> {
        let mut patterns = Vec::new();
        let mut perception_failures = 0;

        for incident in incidents {
            if incident.had_perception_failure {
                perception_failures += 1;
            }
        }

        if perception_failures > 1 {
            patterns.push(IncidentPattern {
                pattern_type: "Sensor blind spots".to_string(),
                occurrence_count: perception_failures,
                fleet_percentage: (perception_failures as f32 / incidents.len() as f32) * 100.0,
                robot_types: incidents
                    .iter()
                    .filter(|i| i.had_perception_failure)
                    .map(|i| i.robot_type.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect(),
                root_causes: vec!["Limited sensor FOV".to_string(), "Range limitations".to_string()],
                confidence: 0.88,
                fleet_action: "Conduct sensor placement analysis; consider additional cameras"
                    .to_string(),
            });
        }

        patterns
    }

    /// Analyze environmental factor patterns
    fn analyze_environmental_patterns(incidents: &[FleetIncident]) -> Vec<IncidentPattern> {
        let mut patterns = Vec::new();
        let mut failures_by_environment: HashMap<String, usize> = HashMap::new();

        for incident in incidents {
            if incident.outcome == "collision" || incident.had_near_miss {
                *failures_by_environment
                    .entry(incident.environment.clone())
                    .or_insert(0) += 1;
            }
        }

        for (env, count) in failures_by_environment {
            if count > 0 {
                patterns.push(IncidentPattern {
                    pattern_type: format!("Failures in {}", env),
                    occurrence_count: count,
                    fleet_percentage: (count as f32 / incidents.len() as f32) * 100.0,
                    robot_types: vec![], // Would filter from incidents
                    root_causes: vec!["Environment-specific challenge".to_string()],
                    confidence: 0.75,
                    fleet_action: format!(
                        "Add environment-specific training; enhance {} perception",
                        env
                    ),
                });
            }
        }

        patterns
    }

    /// Extract robot types from incidents
    fn extract_robot_types(incidents: &[FleetIncident], obj_type: &str) -> Vec<String> {
        incidents
            .iter()
            .filter(|i| i.objects_involved.contains(&obj_type.to_string()))
            .map(|i| i.robot_type.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

/// Fleet incident for pattern analysis
#[derive(Debug, Clone)]
pub struct FleetIncident {
    pub mission_id: String,
    pub robot_type: String,
    pub outcome: String, // "collision", "success", "near_miss"
    pub objects_involved: Vec<String>,
    pub environment: String,
    pub had_perception_failure: bool,
    pub had_near_miss: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_analysis() {
        let incidents = vec![
            FleetIncident {
                mission_id: "m1".to_string(),
                robot_type: "mobile".to_string(),
                outcome: "collision".to_string(),
                objects_involved: vec!["pedestrian".to_string()],
                environment: "warehouse".to_string(),
                had_perception_failure: true,
                had_near_miss: false,
            },
            FleetIncident {
                mission_id: "m2".to_string(),
                robot_type: "mobile".to_string(),
                outcome: "collision".to_string(),
                objects_involved: vec!["pedestrian".to_string()],
                environment: "warehouse".to_string(),
                had_perception_failure: true,
                had_near_miss: false,
            },
        ];

        let patterns = IncidentPatternAnalyzer::analyze_fleet(&incidents);

        assert!(!patterns.is_empty());
        assert!(patterns[0].occurrence_count >= 1);
    }
}
