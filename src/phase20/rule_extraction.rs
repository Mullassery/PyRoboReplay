/// Rule Extraction - Extract interpretable decision rules from models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRule {
    pub rule_id: String,
    pub antecedent: Vec<String>,        // Conditions (e.g., "sensor_drift > 0.7")
    pub consequent: String,             // Conclusion (e.g., "navigation_failure")
    pub support: f32,                   // % of data matching this rule
    pub confidence: f32,                // % of matches that lead to conclusion
    pub lift: f32,                      // How much better than baseline?
    pub complexity: usize,              // Number of conditions
}

impl DecisionRule {
    pub fn new(consequent: String) -> Self {
        DecisionRule {
            rule_id: format!("rule_{}", uuid::Uuid::new_v4()),
            antecedent: Vec::new(),
            consequent,
            support: 0.0,
            confidence: 0.0,
            lift: 1.0,
            complexity: 0,
        }
    }

    pub fn quality_score(&self) -> f32 {
        // Quality = confidence * support * (1 / complexity)
        (self.confidence * self.support) / (1.0 + (self.complexity as f32 * 0.1))
    }

    pub fn is_actionable(&self) -> bool {
        // Rule is actionable if: high confidence AND high lift
        self.confidence > 0.7 && self.lift > 1.2
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub rules: Vec<DecisionRule>,
    pub total_patterns: usize,
    pub coverage: f32,                  // % of data covered by rules
    pub avg_confidence: f32,
}

impl RuleSet {
    pub fn new() -> Self {
        RuleSet {
            rules: Vec::new(),
            total_patterns: 0,
            coverage: 0.0,
            avg_confidence: 0.0,
        }
    }

    pub fn add_rule(&mut self, rule: DecisionRule) {
        self.rules.push(rule);
    }

    pub fn calculate_statistics(&mut self) {
        if self.rules.is_empty() {
            return;
        }

        let avg_confidence: f32 = self.rules.iter().map(|r| r.confidence).sum::<f32>()
            / self.rules.len() as f32;
        let total_support: f32 = self.rules.iter().map(|r| r.support).sum::<f32>();

        self.avg_confidence = avg_confidence;
        self.coverage = (total_support / self.total_patterns as f32).min(1.0);
    }

    pub fn get_top_rules(&self, n: usize) -> Vec<DecisionRule> {
        let mut sorted = self.rules.clone();
        sorted.sort_by(|a, b| b.quality_score().partial_cmp(&a.quality_score()).unwrap());
        sorted.into_iter().take(n).collect()
    }

    pub fn get_actionable_rules(&self) -> Vec<DecisionRule> {
        self.rules.iter().filter(|r| r.is_actionable()).cloned().collect()
    }
}

pub struct RuleExtractor {
    min_support: f32,
    min_confidence: f32,
}

impl RuleExtractor {
    pub fn new() -> Self {
        RuleExtractor {
            min_support: 0.05,        // At least 5% of data
            min_confidence: 0.6,      // At least 60% confidence
        }
    }

    pub fn set_min_support(&mut self, support: f32) {
        self.min_support = support;
    }

    pub fn set_min_confidence(&mut self, confidence: f32) {
        self.min_confidence = confidence;
    }

    /// Extract rules from decision tree paths
    pub fn extract_from_tree_paths(
        &self,
        paths: &[Vec<String>],
        target: String,
        data_size: usize,
    ) -> RuleSet {
        let mut ruleset = RuleSet::new();
        ruleset.total_patterns = data_size;

        for path in paths {
            if path.is_empty() {
                continue;
            }

            // Simulate support and confidence calculation
            let rule_support = (path.len() as f32 / data_size as f32).max(self.min_support);
            let rule_confidence = 0.5 + (path.len() as f32 * 0.1).min(0.4);  // Simplified

            if rule_support >= self.min_support && rule_confidence >= self.min_confidence {
                let mut rule = DecisionRule::new(target.clone());
                rule.antecedent = path.clone();
                rule.support = rule_support;
                rule.confidence = rule_confidence;
                rule.complexity = path.len();
                rule.lift = rule_confidence / 0.5;  // Simplified baseline

                ruleset.add_rule(rule);
            }
        }

        ruleset.calculate_statistics();
        ruleset
    }

    /// Extract association rules from feature combinations
    pub fn extract_associations(
        &self,
        patterns: &HashMap<Vec<String>, (usize, usize)>,  // Pattern -> (occurrences, success_count)
        data_size: usize,
    ) -> RuleSet {
        let mut ruleset = RuleSet::new();
        ruleset.total_patterns = data_size;

        for (pattern, (occurrences, successes)) in patterns {
            if pattern.is_empty() {
                continue;
            }

            let support = *occurrences as f32 / data_size as f32;
            let confidence = *successes as f32 / *occurrences as f32;

            if support >= self.min_support && confidence >= self.min_confidence {
                let mut rule = DecisionRule::new("success".to_string());
                rule.antecedent = pattern.clone();
                rule.support = support;
                rule.confidence = confidence;
                rule.complexity = pattern.len();
                rule.lift = confidence / (*successes as f32 / data_size as f32).max(0.1);

                ruleset.add_rule(rule);
            }
        }

        ruleset.calculate_statistics();
        ruleset
    }

    /// Convert rule to natural language
    pub fn rule_to_text(&self, rule: &DecisionRule) -> String {
        let antecedents = rule.antecedent.join(" AND ");
        format!(
            "IF {} THEN {} (confidence: {:.1}%, support: {:.1}%)",
            antecedents,
            rule.consequent,
            rule.confidence * 100.0,
            rule.support * 100.0
        )
    }

    /// Get extraction statistics
    pub fn get_statistics(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();
        stats.insert("min_support".to_string(), self.min_support);
        stats.insert("min_confidence".to_string(), self.min_confidence);
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_rule_creation() {
        let rule = DecisionRule::new("success".to_string());
        assert_eq!(rule.consequent, "success");
    }

    #[test]
    fn test_decision_rule_quality() {
        let mut rule = DecisionRule::new("success".to_string());
        rule.confidence = 0.8;
        rule.support = 0.5;
        rule.complexity = 2;

        let quality = rule.quality_score();
        assert!(quality > 0.0);
    }

    #[test]
    fn test_rule_is_actionable() {
        let mut rule = DecisionRule::new("success".to_string());
        rule.confidence = 0.75;
        rule.lift = 1.3;

        assert!(rule.is_actionable());
    }

    #[test]
    fn test_ruleset_creation() {
        let ruleset = RuleSet::new();
        assert!(ruleset.rules.is_empty());
    }

    #[test]
    fn test_ruleset_add_rule() {
        let mut ruleset = RuleSet::new();
        let rule = DecisionRule::new("success".to_string());
        ruleset.add_rule(rule);

        assert_eq!(ruleset.rules.len(), 1);
    }

    #[test]
    fn test_ruleset_statistics() {
        let mut ruleset = RuleSet::new();
        ruleset.total_patterns = 100;

        let mut rule = DecisionRule::new("success".to_string());
        rule.confidence = 0.8;
        rule.support = 0.2;
        ruleset.add_rule(rule);

        ruleset.calculate_statistics();
        assert!(ruleset.avg_confidence > 0.0);
    }

    #[test]
    fn test_rule_extractor_creation() {
        let extractor = RuleExtractor::new();
        assert_eq!(extractor.min_support, 0.05);
    }

    #[test]
    fn test_extract_from_tree_paths() {
        let extractor = RuleExtractor::new();
        let paths = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()],
        ];

        let ruleset = extractor.extract_from_tree_paths(&paths, "target".to_string(), 100);
        assert!(!ruleset.rules.is_empty());
    }

    #[test]
    fn test_rule_to_text() {
        let extractor = RuleExtractor::new();
        let mut rule = DecisionRule::new("success".to_string());
        rule.antecedent = vec!["x > 0.5".to_string(), "y < 0.3".to_string()];
        rule.confidence = 0.85;
        rule.support = 0.10;

        let text = extractor.rule_to_text(&rule);
        assert!(text.contains("IF"));
        assert!(text.contains("THEN"));
    }
}
