/// Outcome Influence Analyzer - Calculate which factors had most impact on results

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfluenceScore {
    pub factor_id: String,
    pub factor_name: String,
    pub influence_percent: f32,  // 0-100
    pub direction: String,       // "positive", "negative", "neutral"
    pub magnitude: f32,          // How strong the effect
    pub confidence: f32,         // How certain (0-1)
}

impl InfluenceScore {
    pub fn new(factor_id: String, factor_name: String, influence: f32) -> Self {
        let direction = if influence > 0.0 {
            "positive".to_string()
        } else if influence < 0.0 {
            "negative".to_string()
        } else {
            "neutral".to_string()
        };

        InfluenceScore {
            factor_id,
            factor_name,
            influence_percent: influence.abs(),
            direction,
            magnitude: influence.abs(),
            confidence: 0.75,
        }
    }
}

pub struct OutcomeInfluenceAnalyzer {
    mission_id: String,
    outcome: String,
    factors: Vec<(String, f32)>,  // (factor_name, contribution_score)
}

impl OutcomeInfluenceAnalyzer {
    pub fn new(mission_id: String, outcome: String) -> Self {
        OutcomeInfluenceAnalyzer {
            mission_id,
            outcome,
            factors: Vec::new(),
        }
    }

    /// Add a factor that contributed to the outcome
    pub fn add_factor(&mut self, name: String, influence_magnitude: f32) {
        self.factors.push((name, influence_magnitude));
    }

    /// Analyze and rank factors by influence
    pub fn analyze(&self) -> Vec<InfluenceScore> {
        if self.factors.is_empty() {
            return Vec::new();
        }

        // Calculate total magnitude
        let total_magnitude: f32 = self.factors.iter().map(|(_, mag)| mag.abs()).sum();

        // Convert to percentages
        let mut scores: Vec<InfluenceScore> = self
            .factors
            .iter()
            .map(|(name, magnitude)| {
                let percent = (magnitude.abs() / total_magnitude) * 100.0;
                InfluenceScore {
                    factor_id: format!("factor_{}", name.to_lowercase().replace(" ", "_")),
                    factor_name: name.clone(),
                    influence_percent: percent,
                    direction: if magnitude > &0.0 {
                        "positive".to_string()
                    } else {
                        "negative".to_string()
                    },
                    magnitude: magnitude.abs(),
                    confidence: 0.80,
                }
            })
            .collect();

        // Sort by influence (descending)
        scores.sort_by(|a, b| b.influence_percent.partial_cmp(&a.influence_percent).unwrap());

        scores
    }

    /// Get top N factors
    pub fn get_top_factors(&self, n: usize) -> Vec<InfluenceScore> {
        let mut all = self.analyze();
        all.truncate(n);
        all
    }

    /// Generate influence report
    pub fn generate_report(&self) -> HashMap<String, String> {
        let mut report = HashMap::new();

        let scores = self.analyze();
        if scores.is_empty() {
            report.insert("summary".to_string(), "No factors identified".to_string());
            return report;
        }

        // Summary
        let top = scores.first().unwrap();
        report.insert(
            "summary".to_string(),
            format!(
                "Mission outcome '{}' was primarily influenced by {} ({}%)",
                self.outcome, top.factor_name, top.influence_percent as i32
            ),
        );

        // Top 3 factors
        for (i, score) in scores.iter().take(3).enumerate() {
            report.insert(
                format!("factor_{}", i + 1),
                format!(
                    "{}: {}% influence ({})",
                    score.factor_name, score.influence_percent as i32, score.direction
                ),
            );
        }

        // Ranking string
        let ranking: String = scores
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {} ({}%)", i + 1, s.factor_name, s.influence_percent as i32))
            .collect::<Vec<_>>()
            .join(" → ");

        report.insert("ranking".to_string(), ranking);

        report
    }

    /// Calculate impact of removing each factor
    pub fn calculate_removal_impact(&self) -> HashMap<String, f32> {
        let mut impact = HashMap::new();

        let baseline_score: f32 = self.factors.iter().map(|(_, mag)| mag.abs()).sum();

        for (name, magnitude) in &self.factors {
            let remaining = baseline_score - magnitude.abs();
            let impact_percent = (magnitude.abs() / baseline_score) * 100.0;
            impact.insert(name.clone(), impact_percent);
        }

        impact
    }

    /// Identify critical factors (those with high influence and high confidence)
    pub fn identify_critical_factors(&self, threshold_percent: f32) -> Vec<InfluenceScore> {
        self.analyze()
            .into_iter()
            .filter(|s| s.influence_percent >= threshold_percent && s.confidence > 0.7)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_influence_score_creation() {
        let score = InfluenceScore::new("sensor_1".to_string(), "Sensor Drift".to_string(), -0.35);
        assert_eq!(score.factor_name, "Sensor Drift");
        assert_eq!(score.direction, "negative");
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = OutcomeInfluenceAnalyzer::new(
            "mission_123".to_string(),
            "Failed".to_string(),
        );
        assert_eq!(analyzer.mission_id, "mission_123");
        assert_eq!(analyzer.outcome, "Failed");
    }

    #[test]
    fn test_factor_ranking() {
        let mut analyzer = OutcomeInfluenceAnalyzer::new(
            "mission_1".to_string(),
            "Success".to_string(),
        );

        analyzer.add_factor("Good Path Planning".to_string(), 0.42);
        analyzer.add_factor("Fast Recovery".to_string(), 0.27);
        analyzer.add_factor("High Battery".to_string(), 0.18);
        analyzer.add_factor("Weather".to_string(), 0.13);

        let ranked = analyzer.analyze();
        assert_eq!(ranked.len(), 4);
        assert_eq!(ranked[0].factor_name, "Good Path Planning");
        assert!((ranked[0].influence_percent - 42.0).abs() < 1.0);
    }

    #[test]
    fn test_top_factors() {
        let mut analyzer = OutcomeInfluenceAnalyzer::new(
            "mission_1".to_string(),
            "Success".to_string(),
        );

        for i in 0..10 {
            analyzer.add_factor(format!("Factor {}", i), (10 - i) as f32);
        }

        let top_3 = analyzer.get_top_factors(3);
        assert_eq!(top_3.len(), 3);
    }

    #[test]
    fn test_removal_impact() {
        let mut analyzer = OutcomeInfluenceAnalyzer::new(
            "mission_1".to_string(),
            "Success".to_string(),
        );

        analyzer.add_factor("Factor A".to_string(), 0.60);
        analyzer.add_factor("Factor B".to_string(), 0.40);

        let impact = analyzer.calculate_removal_impact();
        assert!((impact.get("Factor A").unwrap() - 60.0).abs() < 0.1);
        assert!((impact.get("Factor B").unwrap() - 40.0).abs() < 0.1);
    }

    #[test]
    fn test_critical_factors() {
        let mut analyzer = OutcomeInfluenceAnalyzer::new(
            "mission_1".to_string(),
            "Success".to_string(),
        );

        analyzer.add_factor("Critical Factor".to_string(), 0.75);
        analyzer.add_factor("Minor Factor".to_string(), 0.25);

        let critical = analyzer.identify_critical_factors(50.0);
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].factor_name, "Critical Factor");
    }
}
