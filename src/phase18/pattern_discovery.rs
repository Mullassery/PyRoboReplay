/// Failure Pattern Discovery - Mine patterns from fleet missions

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FailurePattern {
    pub pattern_id: String,
    pub name: String,
    pub description: String,
    pub causal_chain: Vec<String>,  // A → B → C → Failure
    pub frequency: usize,            // How many missions exhibit this?
    pub success_rate_without: f32,   // Success rate if this pattern is prevented
    pub avg_recovery_time_ms: u32,
    pub affected_subsystems: Vec<String>,
    pub confidence: f32,             // How certain? (0-1)
}

impl FailurePattern {
    pub fn new(name: String) -> Self {
        FailurePattern {
            pattern_id: format!("pattern_{}", uuid::Uuid::new_v4()),
            name,
            description: String::new(),
            causal_chain: Vec::new(),
            frequency: 0,
            success_rate_without: 0.0,
            avg_recovery_time_ms: 0,
            affected_subsystems: Vec::new(),
            confidence: 0.0,
        }
    }

    pub fn impact_score(&self) -> f32 {
        // Higher impact = more frequent + affects recovery + more certain
        (self.frequency as f32 / 1000.0) * (1.0 - self.success_rate_without) * self.confidence
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCluster {
    pub cluster_id: String,
    pub patterns: Vec<FailurePattern>,
    pub cluster_center: Vec<f32>,    // Vector representation
    pub size: usize,
    pub silhouette_score: f32,       // Clustering quality (0-1)
}

impl PatternCluster {
    pub fn new(cluster_id: String) -> Self {
        PatternCluster {
            cluster_id,
            patterns: Vec::new(),
            cluster_center: Vec::new(),
            size: 0,
            silhouette_score: 0.0,
        }
    }

    pub fn average_impact(&self) -> f32 {
        if self.patterns.is_empty() {
            return 0.0;
        }
        self.patterns.iter().map(|p| p.impact_score()).sum::<f32>() / self.patterns.len() as f32
    }
}

pub struct PatternMiner {
    missions: Vec<MissionData>,
    min_pattern_frequency: usize,
}

#[derive(Debug, Clone)]
struct MissionData {
    mission_id: String,
    failure_occurred: bool,
    causal_chain: Vec<String>,
    recovery_time_ms: u32,
    affected_subsystems: Vec<String>,
}

impl PatternMiner {
    pub fn new(min_frequency: usize) -> Self {
        PatternMiner {
            missions: Vec::new(),
            min_pattern_frequency: min_frequency,
        }
    }

    pub fn add_mission(&mut self, mission_data: MissionData) {
        self.missions.push(mission_data);
    }

    /// Discover failure patterns from mission dataset
    pub fn discover_patterns(&self) -> Vec<FailurePattern> {
        let mut pattern_map: HashMap<Vec<String>, Vec<&MissionData>> = HashMap::new();

        // Group missions by causal chain
        for mission in &self.missions {
            if mission.failure_occurred {
                pattern_map
                    .entry(mission.causal_chain.clone())
                    .or_insert_with(Vec::new)
                    .push(mission);
            }
        }

        // Convert to patterns with statistics
        let mut patterns = Vec::new();

        for (causal_chain, missions) in pattern_map {
            if missions.len() >= self.min_pattern_frequency {
                let mut pattern = FailurePattern::new(format!("Pattern: {}", causal_chain.join(" → ")));
                pattern.causal_chain = causal_chain.clone();
                pattern.frequency = missions.len();

                // Calculate success rate without this pattern
                let total_missions = self.missions.len();
                let failures_with_pattern = missions.len();
                let success_rate_prevention = 1.0 - (failures_with_pattern as f32 / total_missions as f32);
                pattern.success_rate_without = success_rate_prevention;

                // Average recovery time
                pattern.avg_recovery_time_ms =
                    (missions.iter().map(|m| m.recovery_time_ms as u64).sum::<u64>() / missions.len() as u64) as u32;

                // Affected subsystems
                let mut subsystem_counts: HashMap<String, usize> = HashMap::new();
                for mission in &missions {
                    for subsys in &mission.affected_subsystems {
                        *subsystem_counts.entry(subsys.clone()).or_insert(0) += 1;
                    }
                }
                pattern.affected_subsystems = subsystem_counts.keys().cloned().collect();

                // Confidence based on consistency
                pattern.confidence = (failures_with_pattern as f32 / (failures_with_pattern + 10) as f32).min(0.95);

                patterns.push(pattern);
            }
        }

        // Sort by impact
        patterns.sort_by(|a, b| b.impact_score().partial_cmp(&a.impact_score()).unwrap());

        patterns
    }

    /// Get top N patterns by impact
    pub fn get_top_patterns(&self, n: usize) -> Vec<FailurePattern> {
        let mut patterns = self.discover_patterns();
        patterns.truncate(n);
        patterns
    }

    /// Get pattern statistics
    pub fn get_statistics(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();

        let failed_missions = self.missions.iter().filter(|m| m.failure_occurred).count();
        let total_missions = self.missions.len();

        stats.insert(
            "overall_failure_rate".to_string(),
            failed_missions as f32 / total_missions as f32,
        );
        stats.insert("total_missions".to_string(), total_missions as f32);
        stats.insert("unique_failure_patterns".to_string(), failed_missions as f32);

        if failed_missions > 0 {
            let avg_recovery: f32 = self.missions
                .iter()
                .filter(|m| m.failure_occurred)
                .map(|m| m.recovery_time_ms as f32)
                .sum::<f32>()
                / failed_missions as f32;
            stats.insert("avg_recovery_time_ms".to_string(), avg_recovery);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_pattern_creation() {
        let pattern = FailurePattern::new("Sensor Drift".to_string());
        assert_eq!(pattern.name, "Sensor Drift");
        assert_eq!(pattern.frequency, 0);
    }

    #[test]
    fn test_pattern_impact_score() {
        let mut pattern = FailurePattern::new("Test Pattern".to_string());
        pattern.frequency = 100;
        pattern.success_rate_without = 0.5;
        pattern.confidence = 0.9;

        let score = pattern.impact_score();
        assert!(score > 0.0);
    }

    #[test]
    fn test_pattern_cluster() {
        let cluster = PatternCluster::new("cluster_1".to_string());
        assert_eq!(cluster.patterns.len(), 0);
    }

    #[test]
    fn test_pattern_miner() {
        let miner = PatternMiner::new(2);
        assert_eq!(miner.missions.len(), 0);
    }

    #[test]
    fn test_discover_patterns() {
        let mut miner = PatternMiner::new(1);

        for i in 0..3 {
            let mission = MissionData {
                mission_id: format!("m_{}", i),
                failure_occurred: i < 2,  // 2 failures
                causal_chain: vec!["sensor_drift".to_string(), "localization_error".to_string()],
                recovery_time_ms: 500 * (i as u32 + 1),
                affected_subsystems: vec!["navigation".to_string()],
            };
            miner.add_mission(mission);
        }

        let patterns = miner.discover_patterns();
        assert!(!patterns.is_empty());
    }
}
