/// Fleet Learning - Generate fleet-wide recommendations and leaderboards

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationTip {
    pub tip_id: String,
    pub title: String,
    pub description: String,
    pub potential_improvement: f32,  // % improvement expected (0-100)
    pub affected_robots: Vec<String>,
    pub priority: OptimizationPriority,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl OptimizationTip {
    pub fn new(title: String) -> Self {
        OptimizationTip {
            tip_id: format!("tip_{}", uuid::Uuid::new_v4()),
            title,
            description: String::new(),
            potential_improvement: 0.0,
            affected_robots: Vec::new(),
            priority: OptimizationPriority::Medium,
            category: String::new(),
        }
    }

    pub fn impact_score(&self) -> f32 {
        let priority_weight = match self.priority {
            OptimizationPriority::Critical => 4.0,
            OptimizationPriority::High => 3.0,
            OptimizationPriority::Medium => 2.0,
            OptimizationPriority::Low => 1.0,
        };

        (self.potential_improvement / 100.0) * priority_weight
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub robot_id: String,
    pub rank: usize,
    pub success_rate: f32,
    pub avg_mission_time_ms: u32,
    pub total_missions: usize,
    pub efficiency_score: f32,  // 0-100
    pub learning_curve_slope: f32,  // How fast is it improving?
}

impl LeaderboardEntry {
    pub fn new(robot_id: String) -> Self {
        LeaderboardEntry {
            robot_id,
            rank: 0,
            success_rate: 0.0,
            avg_mission_time_ms: 0,
            total_missions: 0,
            efficiency_score: 0.0,
            learning_curve_slope: 0.0,
        }
    }

    pub fn overall_score(&self) -> f32 {
        // Composite: success_rate (50%) + efficiency (30%) + learning_curve (20%)
        (self.success_rate * 50.0 + self.efficiency_score * 30.0 + (self.learning_curve_slope.max(0.0) * 100.0).min(100.0) * 20.0) / 100.0
    }
}

pub struct FleetLearner {
    robots: Vec<RobotStats>,
    tip_categories: Vec<String>,
}

#[derive(Debug, Clone)]
struct RobotStats {
    robot_id: String,
    missions: Vec<MissionResult>,
    success_rate: f32,
    avg_time_ms: u32,
}

#[derive(Debug, Clone)]
struct MissionResult {
    mission_id: String,
    succeeded: bool,
    time_ms: u32,
    timestamp: u64,
}

impl FleetLearner {
    pub fn new() -> Self {
        FleetLearner {
            robots: Vec::new(),
            tip_categories: vec![
                "Navigation".to_string(),
                "Perception".to_string(),
                "Decision Making".to_string(),
                "Energy Management".to_string(),
                "Hardware Tuning".to_string(),
            ],
        }
    }

    pub fn add_robot(&mut self, robot_id: String) {
        self.robots.push(RobotStats {
            robot_id,
            missions: Vec::new(),
            success_rate: 0.0,
            avg_time_ms: 0,
        });
    }

    pub fn add_mission_result(&mut self, robot_id: &str, succeeded: bool, time_ms: u32) {
        if let Some(robot) = self.robots.iter_mut().find(|r| r.robot_id == robot_id) {
            robot.missions.push(MissionResult {
                mission_id: format!("m_{}", robot.missions.len()),
                succeeded,
                time_ms,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            });

            // Recalculate stats
            let successful = robot.missions.iter().filter(|m| m.succeeded).count();
            robot.success_rate = successful as f32 / robot.missions.len() as f32;
            robot.avg_time_ms = (robot.missions.iter().map(|m| m.time_ms as u64).sum::<u64>()
                / robot.missions.len() as u64) as u32;
        }
    }

    /// Generate leaderboard
    pub fn generate_leaderboard(&self) -> Vec<LeaderboardEntry> {
        let mut entries: Vec<LeaderboardEntry> = self.robots.iter().map(|r| {
            let mut entry = LeaderboardEntry::new(r.robot_id.clone());
            entry.success_rate = r.success_rate;
            entry.avg_mission_time_ms = r.avg_time_ms;
            entry.total_missions = r.missions.len();

            // Efficiency: inverse of time (faster is better)
            entry.efficiency_score = (1.0 - (r.avg_time_ms as f32 / 5000.0)).max(0.0) * 100.0;

            // Learning curve: improvement over time
            if r.missions.len() > 1 {
                let first_half = r.missions.len() / 2;
                let early_success = r.missions[..first_half].iter()
                    .filter(|m| m.succeeded)
                    .count() as f32 / first_half as f32;
                let late_success = r.missions[first_half..].iter()
                    .filter(|m| m.succeeded)
                    .count() as f32 / (r.missions.len() - first_half) as f32;

                entry.learning_curve_slope = late_success - early_success;
            }

            entry
        }).collect();

        // Rank by overall score
        entries.sort_by(|a, b| b.overall_score().partial_cmp(&a.overall_score()).unwrap());
        for (i, entry) in entries.iter_mut().enumerate() {
            entry.rank = i + 1;
        }

        entries
    }

    /// Generate optimization tips for the fleet
    pub fn generate_optimization_tips(&self, leaderboard: &[LeaderboardEntry]) -> Vec<OptimizationTip> {
        let mut tips = Vec::new();

        // Tip 1: Low success rate
        for entry in leaderboard {
            if entry.success_rate < 0.75 {
                let mut tip = OptimizationTip::new(
                    format!("Improve success rate for {}", entry.robot_id)
                );
                tip.description = format!(
                    "Current success rate: {:.1}%. Target: 90%+",
                    entry.success_rate * 100.0
                );
                tip.potential_improvement = (0.90 - entry.success_rate).max(0.0) * 100.0;
                tip.affected_robots = vec![entry.robot_id.clone()];
                tip.priority = if entry.success_rate < 0.50 {
                    OptimizationPriority::Critical
                } else {
                    OptimizationPriority::High
                };
                tip.category = "Navigation".to_string();

                tips.push(tip);
            }
        }

        // Tip 2: Slow execution
        if let Some(entry) = leaderboard.first() {
            let avg_time = leaderboard.iter().map(|e| e.avg_mission_time_ms).sum::<u32>() / leaderboard.len() as u32;

            for entry in leaderboard.iter().skip(1) {
                if entry.avg_mission_time_ms > avg_time * 2 {
                    let mut tip = OptimizationTip::new(
                        "Optimize mission execution time".to_string()
                    );
                    tip.description = format!(
                        "{}: {:.1}ms (fleet avg: {:.1}ms)",
                        entry.robot_id, entry.avg_mission_time_ms, avg_time
                    );
                    tip.potential_improvement = ((entry.avg_mission_time_ms as f32 / avg_time as f32) - 1.0).min(50.0);
                    tip.affected_robots = vec![entry.robot_id.clone()];
                    tip.priority = OptimizationPriority::High;
                    tip.category = "Performance".to_string();

                    tips.push(tip);
                }
            }
        }

        // Tip 3: Fleet-wide learning
        let non_learners = leaderboard.iter()
            .filter(|e| e.learning_curve_slope < 0.01)
            .collect::<Vec<_>>();

        if !non_learners.is_empty() {
            let mut tip = OptimizationTip::new("Enable peer learning".to_string());
            tip.description = format!(
                "{} robots showing minimal improvement. Enable knowledge sharing.",
                non_learners.len()
            );
            tip.potential_improvement = 15.0;
            tip.affected_robots = non_learners.iter().map(|e| e.robot_id.clone()).collect();
            tip.priority = OptimizationPriority::Medium;
            tip.category = "Fleet Learning".to_string();

            tips.push(tip);
        }

        // Sort by impact
        tips.sort_by(|a, b| b.impact_score().partial_cmp(&a.impact_score()).unwrap());

        tips
    }

    /// Get fleet statistics
    pub fn get_fleet_stats(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();

        stats.insert("num_robots".to_string(), self.robots.len() as f32);

        if !self.robots.is_empty() {
            let avg_success = self.robots.iter().map(|r| r.success_rate).sum::<f32>() / self.robots.len() as f32;
            let avg_time = self.robots.iter().map(|r| r.avg_time_ms as f32).sum::<f32>() / self.robots.len() as f32;

            stats.insert("fleet_avg_success_rate".to_string(), avg_success);
            stats.insert("fleet_avg_mission_time_ms".to_string(), avg_time);
            stats.insert("total_missions".to_string(),
                self.robots.iter().map(|r| r.missions.len() as f32).sum::<f32>());
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_tip_creation() {
        let tip = OptimizationTip::new("Test Tip".to_string());
        assert_eq!(tip.title, "Test Tip");
    }

    #[test]
    fn test_optimization_tip_impact() {
        let mut tip = OptimizationTip::new("Test".to_string());
        tip.potential_improvement = 50.0;
        tip.priority = OptimizationPriority::High;

        let impact = tip.impact_score();
        assert!(impact > 0.0);
    }

    #[test]
    fn test_leaderboard_entry_creation() {
        let entry = LeaderboardEntry::new("robot_1".to_string());
        assert_eq!(entry.robot_id, "robot_1");
        assert_eq!(entry.rank, 0);
    }

    #[test]
    fn test_leaderboard_overall_score() {
        let mut entry = LeaderboardEntry::new("robot_1".to_string());
        entry.success_rate = 0.9;
        entry.efficiency_score = 80.0;
        entry.learning_curve_slope = 0.05;

        let score = entry.overall_score();
        assert!(score > 0.0);
    }

    #[test]
    fn test_fleet_learner_creation() {
        let learner = FleetLearner::new();
        assert_eq!(learner.robots.len(), 0);
    }

    #[test]
    fn test_add_robot() {
        let mut learner = FleetLearner::new();
        learner.add_robot("robot_1".to_string());
        assert_eq!(learner.robots.len(), 1);
    }

    #[test]
    fn test_add_mission_result() {
        let mut learner = FleetLearner::new();
        learner.add_robot("robot_1".to_string());
        learner.add_mission_result("robot_1", true, 1000);

        assert_eq!(learner.robots[0].missions.len(), 1);
    }

    #[test]
    fn test_generate_leaderboard() {
        let mut learner = FleetLearner::new();
        learner.add_robot("robot_1".to_string());
        learner.add_mission_result("robot_1", true, 1000);

        let leaderboard = learner.generate_leaderboard();
        assert!(!leaderboard.is_empty());
        assert_eq!(leaderboard[0].robot_id, "robot_1");
    }

    #[test]
    fn test_generate_optimization_tips() {
        let mut learner = FleetLearner::new();
        learner.add_robot("robot_1".to_string());
        learner.add_mission_result("robot_1", false, 5000);

        let leaderboard = learner.generate_leaderboard();
        let tips = learner.generate_optimization_tips(&leaderboard);

        assert!(!tips.is_empty());
    }

    #[test]
    fn test_fleet_stats() {
        let mut learner = FleetLearner::new();
        learner.add_robot("robot_1".to_string());
        learner.add_mission_result("robot_1", true, 1000);

        let stats = learner.get_fleet_stats();
        assert_eq!(stats.get("num_robots"), Some(&1.0));
    }
}
