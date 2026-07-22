//! Fleet-Wide Gap Aggregation & Trending
//!
//! Aggregates gaps across entire robot fleet to identify:
//! - Which gaps are most common across fleet
//! - Trending gaps (getting worse over time)
//! - Gaps unique to specific robot types
//! - Predictable gap chains (gap X often precedes gap Y)

use crate::analyzers::RealityGapFinding;
use std::collections::HashMap;

/// Gap statistics across entire fleet
#[derive(Debug, Clone)]
pub struct FleetGapStatistics {
    /// Total number of missions analyzed
    pub total_missions: usize,

    /// Unique robots in fleet
    pub robot_count: usize,

    /// Most common gaps (name, occurrence count, avg confidence)
    pub most_common_gaps: Vec<(String, usize, f32)>,

    /// Trending gaps (getting worse: confidence increasing)
    pub trending_gaps: Vec<(String, f32, f32)>, // name, old_confidence, new_confidence

    /// Gaps unique to specific robot types
    pub robot_type_gaps: HashMap<String, Vec<(String, usize)>>,

    /// Predictive chains (gap X → gap Y)
    pub predictive_chains: Vec<(String, String, f32)>, // from, to, co_occurrence_rate

    /// Overall fleet health score (0.0-1.0)
    pub fleet_health_score: f32,

    /// Risk level: "Critical", "High", "Medium", "Low"
    pub fleet_risk_level: String,
}

/// Per-robot-type calibration profile
#[derive(Debug, Clone)]
pub struct RobotTypeProfile {
    /// Robot type identifier
    pub robot_type: String,

    /// Number of instances of this type
    pub instance_count: usize,

    /// Sensitivity multiplier for gap detection (default 1.0)
    pub sensitivity_multiplier: f32,

    /// Average gap confidence for this type
    pub avg_gap_confidence: f32,

    /// Most problematic gaps for this type
    pub problematic_gaps: Vec<(String, usize, f32)>,

    /// Environmental conditions this type is sensitive to
    pub environmental_sensitivities: HashMap<String, f32>,

    /// Recommended thresholds for alerting
    pub alert_thresholds: HashMap<String, f32>,

    /// Calibration recency (0.0=stale, 1.0=very recent)
    pub calibration_recency: f32,
}

/// Aggregates gaps from multiple missions
pub struct FleetLearningEngine;

impl FleetLearningEngine {
    /// Aggregate gaps from multiple missions into fleet statistics
    pub fn aggregate_fleet_gaps(
        missions: &[FleetMission],
    ) -> FleetGapStatistics {
        let total_missions = missions.len();
        let mut robot_types: HashMap<String, usize> = HashMap::new();
        let mut gap_frequency: HashMap<String, (usize, f32)> = HashMap::new(); // count, sum_confidence
        let mut robot_type_gaps: HashMap<String, HashMap<String, usize>> = HashMap::new();
        let mut gap_confidence_trend: HashMap<String, Vec<(f32, f32)>> = HashMap::new(); // (timestamp, confidence)

        // Aggregate all gaps
        for mission in missions {
            *robot_types.entry(mission.robot_type.clone()).or_insert(0) += 1;

            for gap in &mission.gaps {
                let entry = gap_frequency.entry(gap.category.clone()).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += gap.confidence;

                robot_type_gaps
                    .entry(mission.robot_type.clone())
                    .or_insert_with(HashMap::new)
                    .entry(gap.category.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);

                gap_confidence_trend
                    .entry(gap.category.clone())
                    .or_insert_with(Vec::new)
                    .push((mission.timestamp_sec, gap.confidence));
            }
        }

        // Compute trending gaps
        let trending_gaps = Self::compute_trending_gaps(&gap_confidence_trend);

        // Compute most common gaps
        let mut most_common_gaps: Vec<_> = gap_frequency
            .into_iter()
            .map(|(name, (count, sum_conf))| (name, count, sum_conf / count as f32))
            .collect();
        most_common_gaps.sort_by(|a, b| b.1.cmp(&a.1));
        most_common_gaps.truncate(10);

        // Convert robot type gaps
        let robot_type_gaps_formatted = robot_type_gaps
            .into_iter()
            .map(|(robot_type, gaps)| {
                let mut gap_vec: Vec<_> = gaps.into_iter().map(|(name, count)| (name, count)).collect();
                gap_vec.sort_by(|a, b| b.1.cmp(&a.1));
                (robot_type, gap_vec)
            })
            .collect();

        // Compute predictive chains
        let predictive_chains = Self::compute_predictive_chains(missions);

        // Compute fleet health
        let fleet_health_score = Self::compute_fleet_health(&most_common_gaps, total_missions);
        let fleet_risk_level = Self::classify_fleet_risk(fleet_health_score);

        FleetGapStatistics {
            total_missions,
            robot_count: robot_types.len(),
            most_common_gaps,
            trending_gaps,
            robot_type_gaps: robot_type_gaps_formatted,
            predictive_chains,
            fleet_health_score,
            fleet_risk_level,
        }
    }

    /// Compute which gaps are trending (getting worse)
    fn compute_trending_gaps(
        gap_confidence_trend: &HashMap<String, Vec<(f32, f32)>>,
    ) -> Vec<(String, f32, f32)> {
        let mut trending = Vec::new();

        for (gap_name, trend_data) in gap_confidence_trend {
            if trend_data.len() < 2 {
                continue;
            }

            // Sort by timestamp
            let mut sorted_data = trend_data.clone();
            sorted_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            // Split into old and recent
            let midpoint = sorted_data.len() / 2;
            let old_avg: f32 = sorted_data[..midpoint].iter().map(|(_, conf)| conf).sum::<f32>()
                / (midpoint as f32).max(1.0);
            let new_avg: f32 = sorted_data[midpoint..].iter().map(|(_, conf)| conf).sum::<f32>()
                / ((sorted_data.len() - midpoint) as f32).max(1.0);

            // If average confidence is increasing over time, it's trending worse
            if new_avg > old_avg {
                trending.push((gap_name.clone(), old_avg, new_avg));
            }
        }

        trending.sort_by(|a, b| (b.2 - b.1).partial_cmp(&(a.2 - a.1)).unwrap_or(std::cmp::Ordering::Equal));
        trending.truncate(5);
        trending
    }

    /// Identify predictive gap chains (gap X → gap Y)
    fn compute_predictive_chains(missions: &[FleetMission]) -> Vec<(String, String, f32)> {
        let mut chains: HashMap<(String, String), usize> = HashMap::new();
        let mut total_pairs = 0;

        for mission in missions {
            if mission.gaps.len() < 2 {
                continue;
            }

            // Look for gaps that occur in sequence
            for i in 0..mission.gaps.len() - 1 {
                for j in i + 1..mission.gaps.len() {
                    let time_diff = mission.gaps[j].detection_time_sec.unwrap_or(0.0)
                        - mission.gaps[i].detection_time_sec.unwrap_or(0.0);

                    // Only consider gaps within 30 seconds of each other
                    if time_diff > 0.0 && time_diff < 30.0 {
                        let key = (
                            mission.gaps[i].category.clone(),
                            mission.gaps[j].category.clone(),
                        );
                        *chains.entry(key).or_insert(0) += 1;
                        total_pairs += 1;
                    }
                }
            }
        }

        if total_pairs == 0 {
            return Vec::new();
        }

        let mut chain_vec: Vec<_> = chains
            .into_iter()
            .map(|((from, to), count)| (from, to, count as f32 / total_pairs as f32))
            .collect();

        chain_vec.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        chain_vec.truncate(10);
        chain_vec
    }

    /// Compute overall fleet health score
    fn compute_fleet_health(most_common_gaps: &[(String, usize, f32)], total_missions: usize) -> f32 {
        if most_common_gaps.is_empty() || total_missions == 0 {
            return 1.0;
        }

        let gap_rate = most_common_gaps[0].1 as f32 / total_missions as f32;
        let avg_confidence = most_common_gaps.iter().map(|g| g.2).sum::<f32>() / most_common_gaps.len() as f32;

        // Lower health if gap rate is high or confidence is high
        (1.0 - gap_rate * 0.5 - avg_confidence * 0.2).max(0.0).min(1.0)
    }

    /// Classify overall fleet risk
    fn classify_fleet_risk(health_score: f32) -> String {
        match health_score {
            h if h < 0.3 => "Critical".to_string(),
            h if h < 0.5 => "High".to_string(),
            h if h < 0.8 => "Medium".to_string(),
            _ => "Low".to_string(),
        }
    }

    /// Build calibration profile for a specific robot type
    pub fn calibrate_robot_type(
        robot_type: &str,
        missions: &[FleetMission],
    ) -> RobotTypeProfile {
        let relevant_missions: Vec<_> = missions
            .iter()
            .filter(|m| m.robot_type == robot_type)
            .collect();

        let instance_count = relevant_missions.len();
        if instance_count == 0 {
            return RobotTypeProfile {
                robot_type: robot_type.to_string(),
                instance_count: 0,
                sensitivity_multiplier: 1.0,
                avg_gap_confidence: 0.0,
                problematic_gaps: Vec::new(),
                environmental_sensitivities: HashMap::new(),
                alert_thresholds: HashMap::new(),
                calibration_recency: 0.0,
            };
        }

        let mut all_gaps: Vec<&RealityGapFinding> = relevant_missions
            .iter()
            .flat_map(|m| m.gaps.iter())
            .collect();

        let avg_gap_confidence: f32 = all_gaps.iter().map(|g| g.confidence).sum::<f32>()
            / all_gaps.len().max(1) as f32;

        // Find problematic gaps for this type
        let mut gap_counts: HashMap<String, (usize, f32)> = HashMap::new();
        for gap in &all_gaps {
            let entry = gap_counts.entry(gap.category.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += gap.confidence;
        }

        let mut problematic_gaps: Vec<_> = gap_counts
            .into_iter()
            .map(|(name, (count, sum_conf))| (name, count, sum_conf / count as f32))
            .collect();
        problematic_gaps.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Environmental sensitivities
        let environmental_sensitivities = Self::analyze_environmental_sensitivities(&relevant_missions);

        // Alert thresholds (adjust based on observed confidence)
        let mut alert_thresholds = HashMap::new();
        for gap in problematic_gaps.iter().take(3) {
            alert_thresholds.insert(gap.0.clone(), gap.2 * 0.8); // 80% of observed avg
        }

        let sensitivity_multiplier = if avg_gap_confidence > 0.8 {
            1.2 // High gaps → increase sensitivity
        } else if avg_gap_confidence < 0.5 {
            0.8 // Low gaps → decrease sensitivity
        } else {
            1.0
        };

        RobotTypeProfile {
            robot_type: robot_type.to_string(),
            instance_count,
            sensitivity_multiplier,
            avg_gap_confidence,
            problematic_gaps,
            environmental_sensitivities,
            alert_thresholds,
            calibration_recency: 1.0,
        }
    }

    /// Analyze how environmental conditions affect gaps
    fn analyze_environmental_sensitivities(
        missions: &[&FleetMission],
    ) -> HashMap<String, f32> {
        let mut sensitivities = HashMap::new();

        for mission in missions {
            for (env_factor, env_value) in &mission.environmental_conditions {
                let gap_rate = if mission.gaps.is_empty() {
                    0.0
                } else {
                    1.0
                };

                sensitivities
                    .entry(env_factor.clone())
                    .and_modify(|v| *v = (*v + gap_rate) / 2.0)
                    .or_insert(gap_rate);
            }
        }

        sensitivities
    }
}

/// Single mission data from a robot
#[derive(Debug, Clone)]
pub struct FleetMission {
    /// Robot identifier (unique per robot)
    pub robot_id: String,

    /// Robot type ("mobile_robot", "drone", etc.)
    pub robot_type: String,

    /// When the mission started (unix timestamp or seconds)
    pub timestamp_sec: f32,

    /// All gaps detected in this mission
    pub gaps: Vec<RealityGapFinding>,

    /// Environmental conditions during mission
    pub environmental_conditions: HashMap<String, f32>,

    /// Mission success/failure
    pub mission_outcome: String, // "success", "partial_failure", "complete_failure"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzers::{Evidence, RealityDomain, Severity};

    fn create_test_gap(category: &str, confidence: f32, time: f32) -> RealityGapFinding {
        RealityGapFinding {
            domain: RealityDomain::Sensor,
            category: category.to_string(),
            finding_type: "Test".to_string(),
            severity: Severity::Medium,
            confidence,
            reality_gap_score: 0.7,
            description: "Test gap".to_string(),
            evidence: vec![Evidence {
                signal: "test".to_string(),
                value: 0.5,
                timestamp: time,
                confidence: 0.8,
            }],
            metrics: HashMap::new(),
            sim_recreation_suggestion: "Test".to_string(),
            remediation: "Test".to_string(),
            detection_time_sec: Some(time),
        }
    }

    fn create_test_mission(robot_type: &str, gaps: Vec<RealityGapFinding>) -> FleetMission {
        FleetMission {
            robot_id: format!("{}_1", robot_type),
            robot_type: robot_type.to_string(),
            timestamp_sec: 1000.0,
            gaps,
            environmental_conditions: {
                let mut map = HashMap::new();
                map.insert("temperature".to_string(), 25.0);
                map.insert("humidity".to_string(), 0.5);
                map
            },
            mission_outcome: "success".to_string(),
        }
    }

    #[test]
    fn test_fleet_aggregation() {
        let missions = vec![
            create_test_mission(
                "mobile_robot",
                vec![
                    create_test_gap("Optical Contamination", 0.82, 50.0),
                    create_test_gap("Thermal Effects", 0.78, 75.0),
                ],
            ),
            create_test_mission(
                "mobile_robot",
                vec![create_test_gap("Optical Contamination", 0.85, 100.0)],
            ),
            create_test_mission(
                "drone",
                vec![create_test_gap("Clock Drift", 0.90, 50.0)],
            ),
        ];

        let stats = FleetLearningEngine::aggregate_fleet_gaps(&missions);

        assert_eq!(stats.total_missions, 3);
        assert_eq!(stats.robot_count, 2);
        assert!(stats.fleet_health_score >= 0.0);
        assert!(stats.fleet_health_score <= 1.0);
    }

    #[test]
    fn test_trending_detection() {
        let missions = vec![
            create_test_mission(
                "mobile_robot",
                vec![create_test_gap("Optical Contamination", 0.60, 10.0)],
            ),
            create_test_mission(
                "mobile_robot",
                vec![create_test_gap("Optical Contamination", 0.75, 20.0)],
            ),
            create_test_mission(
                "mobile_robot",
                vec![create_test_gap("Optical Contamination", 0.90, 30.0)],
            ),
        ];

        let stats = FleetLearningEngine::aggregate_fleet_gaps(&missions);

        // Should detect Optical Contamination as trending
        let optical_trending = stats
            .trending_gaps
            .iter()
            .any(|(name, _, _)| name.contains("Optical"));
        assert!(optical_trending);
    }

    #[test]
    fn test_robot_type_calibration() {
        let missions = vec![
            create_test_mission(
                "mobile_robot",
                vec![create_test_gap("Mechanical Degradation", 0.80, 50.0)],
            ),
            create_test_mission(
                "mobile_robot",
                vec![create_test_gap("Mechanical Degradation", 0.85, 100.0)],
            ),
        ];

        let profile = FleetLearningEngine::calibrate_robot_type("mobile_robot", &missions);

        assert_eq!(profile.robot_type, "mobile_robot");
        assert_eq!(profile.instance_count, 2);
        assert!(profile.avg_gap_confidence > 0.7);
        assert!(!profile.problematic_gaps.is_empty());
    }

    #[test]
    fn test_predictive_chains() {
        let missions = vec![
            create_test_mission(
                "mobile_robot",
                vec![
                    create_test_gap("Optical Contamination", 0.80, 10.0),
                    create_test_gap("Detection Robustness", 0.75, 20.0),
                ],
            ),
            create_test_mission(
                "mobile_robot",
                vec![
                    create_test_gap("Optical Contamination", 0.82, 15.0),
                    create_test_gap("Detection Robustness", 0.78, 25.0),
                ],
            ),
        ];

        let stats = FleetLearningEngine::aggregate_fleet_gaps(&missions);

        // Should detect Optical → Detection chain
        assert!(!stats.predictive_chains.is_empty());
    }
}
