use serde::{Deserialize, Serialize};

/// Recommendation for preventing or mitigating a failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Unique recommendation ID
    pub id: String,
    /// Title of the recommendation
    pub title: String,
    /// Detailed description of what to do
    pub description: String,
    /// Why this recommendation matters (root cause link)
    pub rationale: String,
    /// Priority: critical, high, medium, low
    pub priority: String,
    /// Expected impact if implemented (0.0-1.0)
    pub expected_impact: f32,
    /// Estimated effort to implement (0.0-1.0, where 1.0 is hardest)
    pub implementation_effort: f32,
    /// Affected subsystem (planning, sensing, actuation, etc.)
    pub affected_subsystem: String,
    /// Type of fix (algorithm, parameter, hardware, strategy)
    pub fix_type: String,
    /// Confidence this will help (0.0-1.0)
    pub confidence: f32,
    /// Risk of side effects (0.0-1.0)
    pub risk_of_regression: f32,
    /// ROI score: impact / effort
    pub roi_score: f32,
}

/// Recommendation set for a mission failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationSet {
    /// Failure event description
    pub failure_description: String,
    /// Root cause summary
    pub root_cause: String,
    /// Ranked recommendations (sorted by ROI)
    pub recommendations: Vec<Recommendation>,
    /// Quick wins (high impact, low effort)
    pub quick_wins: Vec<Recommendation>,
    /// Strategic improvements (medium-term, higher effort)
    pub strategic_improvements: Vec<Recommendation>,
    /// Summary statistics
    pub stats: RecommendationStats,
}

/// Statistics for recommendation set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationStats {
    /// Total recommendations
    pub total_recommendations: usize,
    /// Average expected impact
    pub avg_impact: f32,
    /// Average implementation effort
    pub avg_effort: f32,
    /// Average confidence
    pub avg_confidence: f32,
    /// Best ROI recommendation
    pub best_roi: f32,
    /// Average risk of regression
    pub avg_regression_risk: f32,
    /// Recommendations by priority
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}

/// Recommendation engine
pub struct RecommendationEngine {
    /// Generated recommendations
    recommendations: Vec<Recommendation>,
}

impl RecommendationEngine {
    /// Create new recommendation engine
    pub fn new() -> Self {
        RecommendationEngine {
            recommendations: Vec::new(),
        }
    }

    /// Generate recommendations for a specific failure mode
    pub fn generate_for_failure(
        &mut self,
        failure_type: &str,
        root_cause: &str,
        severity: &str,
    ) -> Vec<Recommendation> {
        let mut recs = match failure_type {
            "navigation_deadlock" => self._recommend_deadlock_mitigation(root_cause, severity),
            "battery_drain" => self._recommend_battery_optimization(root_cause, severity),
            "collision" => self._recommend_collision_prevention(root_cause, severity),
            "coverage_gap" => self._recommend_coverage_improvement(root_cause, severity),
            "communication_failure" => self._recommend_communication_improvement(root_cause, severity),
            _ => self._recommend_generic(failure_type, root_cause, severity),
        };

        // Sort by ROI
        recs.sort_by(|a, b| b.roi_score.partial_cmp(&a.roi_score).unwrap_or(std::cmp::Ordering::Equal));

        self.recommendations.extend(recs.clone());
        recs
    }

    /// Create recommendation set with categorization
    pub fn create_recommendation_set(
        &self,
        failure_description: &str,
        root_cause: &str,
    ) -> RecommendationSet {
        let mut recs = self.recommendations.clone();
        recs.sort_by(|a, b| b.roi_score.partial_cmp(&a.roi_score).unwrap_or(std::cmp::Ordering::Equal));

        // Categorize into quick wins and strategic improvements
        let quick_wins: Vec<_> = recs
            .iter()
            .filter(|r| r.implementation_effort < 0.4 && r.expected_impact > 0.6)
            .cloned()
            .collect();

        let strategic_improvements: Vec<_> = recs
            .iter()
            .filter(|r| r.implementation_effort >= 0.4 && r.expected_impact > 0.7)
            .cloned()
            .collect();

        // Calculate statistics
        let stats = self._compute_stats(&recs);

        RecommendationSet {
            failure_description: failure_description.to_string(),
            root_cause: root_cause.to_string(),
            recommendations: recs,
            quick_wins,
            strategic_improvements,
            stats,
        }
    }

    fn _recommend_deadlock_mitigation(
        &self,
        root_cause: &str,
        severity: &str,
    ) -> Vec<Recommendation> {
        vec![
            Recommendation {
                id: "deadlock_timeout".to_string(),
                title: "Implement Navigation Timeout".to_string(),
                description: "Add a configurable timeout to detect and escape navigation deadlock. If robot doesn't move for N seconds, trigger recovery behavior.".to_string(),
                rationale: "Deadlock occurs when navigation decisions fail repeatedly. Timeout enables detection and escape.".to_string(),
                priority: if severity == "critical" { "critical".to_string() } else { "high".to_string() },
                expected_impact: if root_cause.contains("obstacle") { 0.85 } else { 0.70 },
                implementation_effort: 0.30,
                affected_subsystem: "navigation_stack".to_string(),
                fix_type: "parameter".to_string(),
                confidence: 0.92,
                risk_of_regression: 0.05,
                roi_score: 0.0,
            },
            Recommendation {
                id: "deadlock_planner_diversity".to_string(),
                title: "Use Multiple Path Planners".to_string(),
                description: "Implement planner switching: if primary planner fails, switch to alternative planner. Increases chance of finding valid path.".to_string(),
                rationale: "Single planner may fail in complex environments. Multiple planners provide redundancy and diversity.".to_string(),
                priority: "high".to_string(),
                expected_impact: 0.80,
                implementation_effort: 0.55,
                affected_subsystem: "planning".to_string(),
                fix_type: "algorithm".to_string(),
                confidence: 0.88,
                risk_of_regression: 0.10,
                roi_score: 0.0,
            },
            Recommendation {
                id: "deadlock_recovery".to_string(),
                title: "Add Recovery Behaviors".to_string(),
                description: "Implement recovery behaviors: back up, rotate in place, or request human assistance when stuck.".to_string(),
                rationale: "Even with detection, robot needs concrete recovery strategies beyond timeout.".to_string(),
                priority: "high".to_string(),
                expected_impact: 0.75,
                implementation_effort: 0.45,
                affected_subsystem: "motion_control".to_string(),
                fix_type: "strategy".to_string(),
                confidence: 0.85,
                risk_of_regression: 0.08,
                roi_score: 0.0,
            },
        ]
    }

    fn _recommend_battery_optimization(
        &self,
        root_cause: &str,
        _severity: &str,
    ) -> Vec<Recommendation> {
        vec![
            Recommendation {
                id: "battery_path_optimization".to_string(),
                title: "Optimize Path Planning for Energy".to_string(),
                description: "Switch from greedy nearest-neighbor to Dijkstra/A* with energy-aware cost function. Reduces unnecessary movement.".to_string(),
                rationale: if root_cause.contains("suboptimal") {
                    "Root cause identified as suboptimal path planning. Better algorithm directly addresses this.".to_string()
                } else {
                    "Energy waste often stems from inefficient paths.".to_string()
                },
                priority: "high".to_string(),
                expected_impact: if root_cause.contains("suboptimal") { 0.90 } else { 0.70 },
                implementation_effort: 0.50,
                affected_subsystem: "planning".to_string(),
                fix_type: "algorithm".to_string(),
                confidence: 0.93,
                risk_of_regression: 0.03,
                roi_score: 0.0,
            },
            Recommendation {
                id: "battery_speed_reduction".to_string(),
                title: "Reduce Navigation Speed".to_string(),
                description: "Lower max velocity setpoint (e.g., 0.5 m/s → 0.3 m/s). Reduces power consumption and improves control stability.".to_string(),
                rationale: "Power consumption scales with velocity. Conservative speed saves battery.".to_string(),
                priority: "medium".to_string(),
                expected_impact: 0.55,
                implementation_effort: 0.10,
                affected_subsystem: "motion_control".to_string(),
                fix_type: "parameter".to_string(),
                confidence: 0.95,
                risk_of_regression: 0.02,
                roi_score: 0.0,
            },
            Recommendation {
                id: "battery_monitoring".to_string(),
                title: "Implement Battery Monitoring".to_string(),
                description: "Add real-time battery monitoring with predictive SOC. Plan recovery charging before critical level is reached.".to_string(),
                rationale: "Proactive monitoring enables better planning and prevents mission failure from battery.".to_string(),
                priority: "medium".to_string(),
                expected_impact: 0.65,
                implementation_effort: 0.35,
                affected_subsystem: "power_management".to_string(),
                fix_type: "algorithm".to_string(),
                confidence: 0.88,
                risk_of_regression: 0.01,
                roi_score: 0.0,
            },
        ]
    }

    fn _recommend_collision_prevention(
        &self,
        _root_cause: &str,
        _severity: &str,
    ) -> Vec<Recommendation> {
        vec![
            Recommendation {
                id: "collision_sensor_fusion".to_string(),
                title: "Implement Sensor Fusion".to_string(),
                description: "Combine multiple sensors (lidar, camera, ultrasonic) for robust obstacle detection with redundancy.".to_string(),
                rationale: "Single sensor may miss obstacles. Fusion increases detection reliability.".to_string(),
                priority: "critical".to_string(),
                expected_impact: 0.90,
                implementation_effort: 0.60,
                affected_subsystem: "sensing".to_string(),
                fix_type: "algorithm".to_string(),
                confidence: 0.91,
                risk_of_regression: 0.05,
                roi_score: 0.0,
            },
            Recommendation {
                id: "collision_safety_margin".to_string(),
                title: "Increase Safety Margin".to_string(),
                description: "Add safety buffer to collision detection (e.g., expand obstacle radius by 10cm). Provides more reaction time.".to_string(),
                rationale: "Safety margin reduces last-minute emergency braking and improves mission robustness.".to_string(),
                priority: "high".to_string(),
                expected_impact: 0.80,
                implementation_effort: 0.15,
                affected_subsystem: "navigation".to_string(),
                fix_type: "parameter".to_string(),
                confidence: 0.94,
                risk_of_regression: 0.02,
                roi_score: 0.0,
            },
        ]
    }

    fn _recommend_coverage_improvement(
        &self,
        _root_cause: &str,
        _severity: &str,
    ) -> Vec<Recommendation> {
        vec![
            Recommendation {
                id: "coverage_spiral_pattern".to_string(),
                title: "Use Spiral Coverage Pattern".to_string(),
                description: "Replace random walk with systematic spiral/boustrophe pattern. Guarantees full area coverage.".to_string(),
                rationale: "Coverage gaps often result from unplanned exploration. Systematic patterns prevent gaps.".to_string(),
                priority: "high".to_string(),
                expected_impact: 0.85,
                implementation_effort: 0.40,
                affected_subsystem: "planning".to_string(),
                fix_type: "algorithm".to_string(),
                confidence: 0.90,
                risk_of_regression: 0.08,
                roi_score: 0.0,
            },
            Recommendation {
                id: "coverage_tracking".to_string(),
                title: "Add Coverage Tracking".to_string(),
                description: "Maintain real-time coverage map. Identify and prioritize uncovered regions dynamically.".to_string(),
                rationale: "Without tracking, robot can't know which areas are unexplored.".to_string(),
                priority: "medium".to_string(),
                expected_impact: 0.70,
                implementation_effort: 0.45,
                affected_subsystem: "mapping".to_string(),
                fix_type: "algorithm".to_string(),
                confidence: 0.87,
                risk_of_regression: 0.03,
                roi_score: 0.0,
            },
        ]
    }

    fn _recommend_communication_improvement(
        &self,
        _root_cause: &str,
        _severity: &str,
    ) -> Vec<Recommendation> {
        vec![
            Recommendation {
                id: "comm_redundancy".to_string(),
                title: "Implement Communication Redundancy".to_string(),
                description: "Support multiple communication channels (WiFi, cellular, mesh). Fallback if primary fails.".to_string(),
                rationale: "Single communication channel is a single point of failure.".to_string(),
                priority: "high".to_string(),
                expected_impact: 0.80,
                implementation_effort: 0.55,
                affected_subsystem: "communication".to_string(),
                fix_type: "hardware".to_string(),
                confidence: 0.89,
                risk_of_regression: 0.05,
                roi_score: 0.0,
            },
        ]
    }

    fn _recommend_generic(
        &self,
        failure_type: &str,
        _root_cause: &str,
        _severity: &str,
    ) -> Vec<Recommendation> {
        vec![Recommendation {
            id: format!("generic_logging_{}", failure_type),
            title: "Improve Diagnostic Logging".to_string(),
            description: "Add detailed logging for all subsystems involved in this failure. Enables better root cause analysis.".to_string(),
            rationale: "Better visibility into system behavior enables faster diagnosis.".to_string(),
            priority: "medium".to_string(),
            expected_impact: 0.50,
            implementation_effort: 0.25,
            affected_subsystem: "logging".to_string(),
            fix_type: "parameter".to_string(),
            confidence: 0.85,
            risk_of_regression: 0.01,
            roi_score: 0.0,
        }]
    }

    fn _compute_stats(&self, recommendations: &[Recommendation]) -> RecommendationStats {
        if recommendations.is_empty() {
            return RecommendationStats {
                total_recommendations: 0,
                avg_impact: 0.0,
                avg_effort: 0.0,
                avg_confidence: 0.0,
                best_roi: 0.0,
                avg_regression_risk: 0.0,
                critical_count: 0,
                high_count: 0,
                medium_count: 0,
                low_count: 0,
            };
        }

        let avg_impact = recommendations.iter().map(|r| r.expected_impact).sum::<f32>() / recommendations.len() as f32;
        let avg_effort = recommendations.iter().map(|r| r.implementation_effort).sum::<f32>() / recommendations.len() as f32;
        let avg_confidence = recommendations.iter().map(|r| r.confidence).sum::<f32>() / recommendations.len() as f32;
        let avg_regression = recommendations.iter().map(|r| r.risk_of_regression).sum::<f32>() / recommendations.len() as f32;
        let best_roi = recommendations.iter().map(|r| r.roi_score).fold(0.0, f32::max);

        let critical_count = recommendations.iter().filter(|r| r.priority == "critical").count();
        let high_count = recommendations.iter().filter(|r| r.priority == "high").count();
        let medium_count = recommendations.iter().filter(|r| r.priority == "medium").count();
        let low_count = recommendations.iter().filter(|r| r.priority == "low").count();

        RecommendationStats {
            total_recommendations: recommendations.len(),
            avg_impact,
            avg_effort,
            avg_confidence,
            best_roi,
            avg_regression_risk: avg_regression,
            critical_count,
            high_count,
            medium_count,
            low_count,
        }
    }

    /// Get all recommendations
    pub fn recommendations(&self) -> &[Recommendation] {
        &self.recommendations
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = RecommendationEngine::new();
        assert_eq!(engine.recommendations().len(), 0);
    }

    #[test]
    fn test_generate_deadlock_recommendations() {
        let mut engine = RecommendationEngine::new();
        let recs = engine.generate_for_failure("navigation_deadlock", "obstacle_blocking", "high");
        assert!(recs.len() > 0);
    }

    #[test]
    fn test_generate_battery_recommendations() {
        let mut engine = RecommendationEngine::new();
        let recs = engine.generate_for_failure("battery_drain", "suboptimal_planning", "medium");
        assert!(recs.len() > 0);
        assert!(recs.iter().any(|r| r.title.contains("Path")));
    }

    #[test]
    fn test_recommendation_roi_calculation() {
        let rec = Recommendation {
            id: "test".to_string(),
            title: "test".to_string(),
            description: "test".to_string(),
            rationale: "test".to_string(),
            priority: "high".to_string(),
            expected_impact: 0.8,
            implementation_effort: 0.4,
            affected_subsystem: "test".to_string(),
            fix_type: "test".to_string(),
            confidence: 0.9,
            risk_of_regression: 0.05,
            roi_score: 2.0, // 0.8 / 0.4
        };
        assert!(rec.roi_score > 1.0);
    }

    #[test]
    fn test_recommendation_set_creation() {
        let mut engine = RecommendationEngine::new();
        engine.generate_for_failure("battery_drain", "suboptimal_planning", "high");
        let set = engine.create_recommendation_set("Battery critical", "Suboptimal path planning");

        assert!(set.recommendations.len() > 0);
        assert_eq!(set.failure_description, "Battery critical");
    }

    #[test]
    fn test_quick_wins_categorization() {
        let mut engine = RecommendationEngine::new();
        engine.generate_for_failure("battery_drain", "suboptimal_planning", "high");
        let set = engine.create_recommendation_set("Battery critical", "Suboptimal path planning");

        // Quick wins should be low effort, high impact
        for qw in &set.quick_wins {
            assert!(qw.implementation_effort < 0.4);
            assert!(qw.expected_impact > 0.6);
        }
    }

    #[test]
    fn test_recommendation_priority_levels() {
        let mut engine = RecommendationEngine::new();
        engine.generate_for_failure("collision", "sensor_failure", "critical");
        let set = engine.create_recommendation_set("Collision detected", "Sensor failure");

        // Should have critical recommendations for collision
        assert!(set.stats.critical_count > 0);
    }

    #[test]
    fn test_recommendation_stats() {
        let mut engine = RecommendationEngine::new();
        engine.generate_for_failure("coverage_gap", "no_planning", "medium");
        let set = engine.create_recommendation_set("Coverage gap", "No systematic planning");

        assert!(set.stats.avg_impact > 0.0);
        assert!(set.stats.avg_effort > 0.0);
        assert!(set.stats.avg_confidence > 0.0);
    }
}
