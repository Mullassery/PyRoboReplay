/// Counterfactual Scenario Analysis - What-if analysis with impact prediction

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualScenario {
    pub scenario_id: String,
    pub name: String,
    pub description: String,
    pub modifications: HashMap<String, f32>,  // Feature -> hypothetical value
    pub predicted_outcome: String,
    pub confidence: f32,
    pub impact_magnitude: f32,                // How much different from baseline?
    pub affected_features: Vec<String>,
}

impl CounterfactualScenario {
    pub fn new(name: String) -> Self {
        CounterfactualScenario {
            scenario_id: format!("scenario_{}", uuid::Uuid::new_v4()),
            name,
            description: String::new(),
            modifications: HashMap::new(),
            predicted_outcome: String::new(),
            confidence: 0.0,
            impact_magnitude: 0.0,
            affected_features: Vec::new(),
        }
    }

    pub fn impact_score(&self) -> f32 {
        // Impact based on magnitude and confidence
        self.impact_magnitude * self.confidence
    }
}

pub struct ScenarioAnalyzer {
    baseline_features: HashMap<String, f32>,
    scenario_history: Vec<CounterfactualScenario>,
}

impl ScenarioAnalyzer {
    pub fn new(baseline: HashMap<String, f32>) -> Self {
        ScenarioAnalyzer {
            baseline_features: baseline,
            scenario_history: Vec::new(),
        }
    }

    /// Create a counterfactual scenario by modifying baseline features
    pub fn create_scenario(
        &mut self,
        scenario_name: String,
        modifications: HashMap<String, f32>,
    ) -> CounterfactualScenario {
        let mut scenario = CounterfactualScenario::new(scenario_name);

        scenario.modifications = modifications.clone();
        scenario.affected_features = modifications.keys().cloned().collect();

        // Calculate impact magnitude
        let mut total_change = 0.0;
        for (feature, hypothetical_value) in &modifications {
            let baseline_value = self.baseline_features.get(feature).unwrap_or(&0.0);
            let change = (hypothetical_value - baseline_value).abs();
            total_change += change;
        }

        scenario.impact_magnitude = total_change / scenario.affected_features.len().max(1) as f32;

        self.scenario_history.push(scenario.clone());
        scenario
    }

    /// Analyze how features cascade through a causal network
    pub fn analyze_cascade(
        &self,
        scenario: &CounterfactualScenario,
        causal_map: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, f32> {
        let mut cascading_effects: HashMap<String, f32> = HashMap::new();

        // Start with direct modifications
        for (feature, value) in &scenario.modifications {
            cascading_effects.insert(feature.clone(), *value);
        }

        // Propagate effects through causal chain
        let mut iterations = 0;
        let max_iterations = 10;

        while iterations < max_iterations {
            let mut new_effects = cascading_effects.clone();
            let mut changed = false;

            for (source, targets) in causal_map {
                if let Some(&source_value) = cascading_effects.get(source) {
                    for target in targets {
                        // Simple propagation: effect attenuates by 0.7x per hop
                        let indirect_effect = source_value * 0.7;

                        if !new_effects.contains_key(target)
                            || (new_effects.get(target).unwrap_or(&0.0) - indirect_effect).abs() > 0.01
                        {
                            new_effects.insert(target.clone(), indirect_effect);
                            changed = true;
                        }
                    }
                }
            }

            cascading_effects = new_effects;

            if !changed {
                break;
            }

            iterations += 1;
        }

        cascading_effects
    }

    /// Generate multiple scenarios to test robustness
    pub fn generate_sensitivity_analysis(
        &mut self,
        feature: String,
        range: (f32, f32),
        steps: usize,
    ) -> Vec<CounterfactualScenario> {
        let mut scenarios = Vec::new();
        let baseline = self.baseline_features.get(&feature).unwrap_or(&0.0);

        let step_size = (range.1 - range.0) / (steps as f32 - 1.0);

        for i in 0..steps {
            let value = range.0 + (i as f32 * step_size);
            let mut modifications = HashMap::new();
            modifications.insert(feature.clone(), value);

            let scenario_name = format!("{}_sensitivity_{}", feature, i);
            let scenario = self.create_scenario(scenario_name, modifications);

            scenarios.push(scenario);
        }

        scenarios
    }

    /// Compare two scenarios to understand trade-offs
    pub fn compare_scenarios(
        &self,
        scenario_a: &CounterfactualScenario,
        scenario_b: &CounterfactualScenario,
    ) -> HashMap<String, f32> {
        let mut comparison = HashMap::new();

        // Find common modifications
        for feature in &scenario_a.affected_features {
            let mod_a = scenario_a.modifications.get(feature).unwrap_or(&0.0);
            let mod_b = scenario_b.modifications.get(feature).unwrap_or(&0.0);

            let diff = (mod_b - mod_a).abs();
            comparison.insert(format!("diff_{}", feature), diff);
        }

        comparison.insert("impact_diff".to_string(),
            (scenario_b.impact_magnitude - scenario_a.impact_magnitude).abs());

        comparison
    }

    /// Get scenario statistics
    pub fn get_statistics(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();

        stats.insert("total_scenarios".to_string(), self.scenario_history.len() as f32);

        if !self.scenario_history.is_empty() {
            let avg_impact: f32 = self.scenario_history
                .iter()
                .map(|s| s.impact_magnitude)
                .sum::<f32>()
                / self.scenario_history.len() as f32;

            let avg_confidence: f32 = self.scenario_history
                .iter()
                .map(|s| s.confidence)
                .sum::<f32>()
                / self.scenario_history.len() as f32;

            stats.insert("avg_impact".to_string(), avg_impact);
            stats.insert("avg_confidence".to_string(), avg_confidence);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_creation() {
        let scenario = CounterfactualScenario::new("test_scenario".to_string());
        assert_eq!(scenario.name, "test_scenario");
    }

    #[test]
    fn test_scenario_impact_score() {
        let mut scenario = CounterfactualScenario::new("test".to_string());
        scenario.impact_magnitude = 0.5;
        scenario.confidence = 0.8;

        assert_eq!(scenario.impact_score(), 0.4);
    }

    #[test]
    fn test_analyzer_creation() {
        let baseline = HashMap::new();
        let analyzer = ScenarioAnalyzer::new(baseline);
        assert!(analyzer.scenario_history.is_empty());
    }

    #[test]
    fn test_create_scenario() {
        let baseline = {
            let mut m = HashMap::new();
            m.insert("feature1".to_string(), 0.5);
            m
        };

        let mut analyzer = ScenarioAnalyzer::new(baseline);
        let mut modifications = HashMap::new();
        modifications.insert("feature1".to_string(), 0.8);

        let scenario = analyzer.create_scenario("test".to_string(), modifications);
        assert_eq!(scenario.affected_features.len(), 1);
    }

    #[test]
    fn test_analyze_cascade() {
        let baseline = HashMap::new();
        let analyzer = ScenarioAnalyzer::new(baseline);

        let mut scenario = CounterfactualScenario::new("test".to_string());
        scenario.modifications.insert("f1".to_string(), 1.0);

        let mut causal_map = HashMap::new();
        causal_map.insert("f1".to_string(), vec!["f2".to_string()]);

        let effects = analyzer.analyze_cascade(&scenario, &causal_map);
        assert!(effects.contains_key("f1"));
    }

    #[test]
    fn test_sensitivity_analysis() {
        let baseline = {
            let mut m = HashMap::new();
            m.insert("feature1".to_string(), 0.5);
            m
        };

        let mut analyzer = ScenarioAnalyzer::new(baseline);
        let scenarios = analyzer.generate_sensitivity_analysis("feature1".to_string(), (0.0, 1.0), 5);

        assert_eq!(scenarios.len(), 5);
    }

    #[test]
    fn test_compare_scenarios() {
        let baseline = HashMap::new();
        let analyzer = ScenarioAnalyzer::new(baseline);

        let mut s1 = CounterfactualScenario::new("s1".to_string());
        s1.modifications.insert("f1".to_string(), 0.5);

        let mut s2 = CounterfactualScenario::new("s2".to_string());
        s2.modifications.insert("f1".to_string(), 0.8);

        let comparison = analyzer.compare_scenarios(&s1, &s2);
        assert!(!comparison.is_empty());
    }

    #[test]
    fn test_analyzer_statistics() {
        let baseline = HashMap::new();
        let mut analyzer = ScenarioAnalyzer::new(baseline);

        let mut mods = HashMap::new();
        mods.insert("f1".to_string(), 0.5);
        analyzer.create_scenario("test".to_string(), mods);

        let stats = analyzer.get_statistics();
        assert_eq!(stats.get("total_scenarios"), Some(&1.0));
    }
}
