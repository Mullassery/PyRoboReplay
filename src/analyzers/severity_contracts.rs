//! Severity Contracts: Quality-Driven Rules
//!
//! Defines auditable quality contracts for gap severity classification.
//! Replaces hand-coded decision trees with versioned, composable contracts.

use std::collections::HashMap;

/// A severity contract defines when a gap reaches a certain severity level
#[derive(Debug, Clone)]
pub struct SeverityContract {
    /// Contract identifier (e.g., "mechanical_degradation_critical")
    pub id: String,

    /// Target severity level this contract defines
    pub target_severity: String, // "critical", "high", "medium", "low"

    /// Contract conditions: metric_name → (min_threshold, max_threshold)
    pub conditions: HashMap<String, (f32, f32)>,

    /// Logical operator: "AND" (all must match) or "OR" (any matches)
    pub operator: ContractOperator,

    /// Contract version (for auditing)
    pub version: String,

    /// Description of what this contract represents
    pub description: String,

    /// Confidence in this contract (0.0-1.0)
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContractOperator {
    And,
    Or,
}

impl SeverityContract {
    /// Create new contract
    pub fn new(id: &str, target_severity: &str) -> Self {
        SeverityContract {
            id: id.to_string(),
            target_severity: target_severity.to_string(),
            conditions: HashMap::new(),
            operator: ContractOperator::And,
            version: "1.0.0".to_string(),
            description: String::new(),
            confidence: 0.8,
        }
    }

    /// Add a condition to the contract
    pub fn add_condition(&mut self, metric: &str, min_threshold: f32, max_threshold: f32) {
        self.conditions.insert(metric.to_string(), (min_threshold, max_threshold));
    }

    /// Check if a finding satisfies this contract
    pub fn matches(&self, metrics: &HashMap<String, f32>) -> bool {
        if self.conditions.is_empty() {
            return false;
        }

        let matches: Vec<bool> = self
            .conditions
            .iter()
            .map(|(metric, (min, max))| {
                if let Some(value) = metrics.get(metric) {
                    *value >= *min && *value <= *max
                } else {
                    false
                }
            })
            .collect();

        match self.operator {
            ContractOperator::And => matches.iter().all(|&m| m),
            ContractOperator::Or => matches.iter().any(|&m| m),
        }
    }

    /// Get matching conditions for a finding
    pub fn matching_conditions(&self, metrics: &HashMap<String, f32>) -> Vec<String> {
        self.conditions
            .iter()
            .filter_map(|(metric, (min, max))| {
                if let Some(value) = metrics.get(metric) {
                    if *value >= *min && *value <= *max {
                        return Some(metric.clone());
                    }
                }
                None
            })
            .collect()
    }
}

/// Contract catalog: defines all severity contracts for a system
pub struct SeverityContractCatalog {
    contracts: HashMap<String, SeverityContract>,
}

impl SeverityContractCatalog {
    /// Create new catalog with standard contracts
    pub fn new() -> Self {
        let mut catalog = SeverityContractCatalog {
            contracts: HashMap::new(),
        };

        // CRITICAL severity contracts
        catalog.add_critical_contract_timestamp_reversal();
        catalog.add_critical_contract_safety_collision();
        catalog.add_critical_contract_performance_catastrophic();

        // HIGH severity contracts
        catalog.add_high_contract_response_degradation();
        catalog.add_high_contract_efficiency_decline();
        catalog.add_high_contract_detection_confidence_drop();

        // MEDIUM severity contracts
        catalog.add_medium_contract_environmental_correlation();
        catalog.add_medium_contract_thermal_gradual();

        // LOW severity contracts
        catalog.add_low_contract_minor_quality();

        catalog
    }

    /// Add contract to catalog
    pub fn add_contract(&mut self, contract: SeverityContract) {
        self.contracts.insert(contract.id.clone(), contract);
    }

    /// Get contract by ID
    pub fn get_contract(&self, id: &str) -> Option<&SeverityContract> {
        self.contracts.get(id)
    }

    /// Evaluate all contracts against a finding's metrics
    pub fn evaluate(
        &self,
        metrics: &HashMap<String, f32>,
    ) -> Vec<(&SeverityContract, Vec<String>)> {
        self.contracts
            .values()
            .filter_map(|contract| {
                if contract.matches(metrics) {
                    let matching = contract.matching_conditions(metrics);
                    Some((contract, matching))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Determine severity from contract evaluation
    pub fn determine_severity(
        &self,
        metrics: &HashMap<String, f32>,
    ) -> Option<(String, f32)> {
        // Priority: critical > high > medium > low
        let matches = self.evaluate(metrics);

        for (contract, _) in &matches {
            if contract.target_severity == "critical" {
                return Some(("critical".to_string(), contract.confidence));
            }
        }

        for (contract, _) in &matches {
            if contract.target_severity == "high" {
                return Some(("high".to_string(), contract.confidence));
            }
        }

        for (contract, _) in &matches {
            if contract.target_severity == "medium" {
                return Some(("medium".to_string(), contract.confidence));
            }
        }

        for (contract, _) in &matches {
            if contract.target_severity == "low" {
                return Some(("low".to_string(), contract.confidence));
            }
        }

        None
    }

    fn add_critical_contract_timestamp_reversal(&mut self) {
        let mut contract = SeverityContract::new(
            "critical_timestamp_reversal",
            "critical",
        );
        contract.description = "Time running backwards = critical safety issue".to_string();
        contract.add_condition("clock_drift_direction", -1.0, -0.1); // Negative drift
        contract.confidence = 0.99;
        self.add_contract(contract);
    }

    fn add_critical_contract_safety_collision(&mut self) {
        let mut contract = SeverityContract::new(
            "critical_safety_collision",
            "critical",
        );
        contract.description = "Collision detection disabled or failing".to_string();
        contract.operator = ContractOperator::Or;
        contract.add_condition("detection_confidence_decline_pct", 80.0, 101.0); // >80% drop
        contract.add_condition("obstacle_miss_rate", 0.5, 1.1); // >50% missing obstacles
        contract.confidence = 0.95;
        self.add_contract(contract);
    }

    fn add_critical_contract_performance_catastrophic(&mut self) {
        let mut contract = SeverityContract::new(
            "critical_performance_catastrophic",
            "critical",
        );
        contract.description = "Mission performance degraded beyond acceptable".to_string();
        contract.add_condition("response_time_increase_pct", 100.0, 500.0); // 100%+ slower
        contract.add_condition("trend_slope_ms_per_hour", 0.5, 10.0); // Rapid degradation
        contract.confidence = 0.92;
        self.add_contract(contract);
    }

    fn add_high_contract_response_degradation(&mut self) {
        let mut contract = SeverityContract::new(
            "high_response_degradation",
            "high",
        );
        contract.description = "Control response time degrading > 5%".to_string();
        contract.add_condition("trend_slope_ms_per_hour", 0.01, 0.5);
        contract.confidence = 0.85;
        self.add_contract(contract);
    }

    fn add_high_contract_efficiency_decline(&mut self) {
        let mut contract = SeverityContract::new(
            "high_efficiency_decline",
            "high",
        );
        contract.description = "Motor efficiency declining, likely thermal or mechanical wear".to_string();
        contract.add_condition("efficiency_decline_pct", 10.0, 40.0);
        contract.confidence = 0.80;
        self.add_contract(contract);
    }

    fn add_high_contract_detection_confidence_drop(&mut self) {
        let mut contract = SeverityContract::new(
            "high_detection_drop",
            "high",
        );
        contract.description = "Object detection confidence declining rapidly".to_string();
        contract.add_condition("confidence_decline_pct", 20.0, 80.0);
        contract.confidence = 0.78;
        self.add_contract(contract);
    }

    fn add_medium_contract_environmental_correlation(&mut self) {
        let mut contract = SeverityContract::new(
            "medium_environmental_correlation",
            "medium",
        );
        contract.description = "Gap correlates with environmental factors (rain, lighting, etc)".to_string();
        contract.add_condition("quality_confidence_correlation", 0.6, 1.0);
        contract.confidence = 0.75;
        self.add_contract(contract);
    }

    fn add_medium_contract_thermal_gradual(&mut self) {
        let mut contract = SeverityContract::new(
            "medium_thermal_gradual",
            "medium",
        );
        contract.description = "Gradual thermal degradation over mission".to_string();
        contract.add_condition("efficiency_decline_pct", 5.0, 15.0);
        contract.add_condition("temperature_rise_c", 20.0, 60.0);
        contract.confidence = 0.72;
        self.add_contract(contract);
    }

    fn add_low_contract_minor_quality(&mut self) {
        let mut contract = SeverityContract::new(
            "low_minor_quality",
            "low",
        );
        contract.description = "Minor quality degradation, not impacting performance".to_string();
        contract.add_condition("sharpness_decline_pct", 5.0, 15.0);
        contract.confidence = 0.65;
        self.add_contract(contract);
    }
}

impl Default for SeverityContractCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_creation() {
        let mut contract = SeverityContract::new("test", "high");
        contract.add_condition("metric1", 0.5, 1.0);

        assert_eq!(contract.id, "test");
        assert_eq!(contract.target_severity, "high");
        assert_eq!(contract.conditions.len(), 1);
    }

    #[test]
    fn test_contract_matches_and() {
        let mut contract = SeverityContract::new("test", "high");
        contract.operator = ContractOperator::And;
        contract.add_condition("metric1", 0.5, 1.0);
        contract.add_condition("metric2", 10.0, 20.0);

        let mut metrics = HashMap::new();
        metrics.insert("metric1".to_string(), 0.75);
        metrics.insert("metric2".to_string(), 15.0);

        assert!(contract.matches(&metrics));
    }

    #[test]
    fn test_contract_matches_or() {
        let mut contract = SeverityContract::new("test", "high");
        contract.operator = ContractOperator::Or;
        contract.add_condition("metric1", 0.5, 1.0);
        contract.add_condition("metric2", 10.0, 20.0);

        let mut metrics = HashMap::new();
        metrics.insert("metric1".to_string(), 0.75); // Matches
        metrics.insert("metric2".to_string(), 100.0); // Doesn't match

        assert!(contract.matches(&metrics));
    }

    #[test]
    fn test_contract_no_match() {
        let mut contract = SeverityContract::new("test", "high");
        contract.add_condition("metric1", 0.5, 1.0);

        let mut metrics = HashMap::new();
        metrics.insert("metric1".to_string(), 0.2); // Outside range

        assert!(!contract.matches(&metrics));
    }

    #[test]
    fn test_catalog_creation() {
        let catalog = SeverityContractCatalog::new();
        assert!(catalog.contracts.len() > 0);
    }

    #[test]
    fn test_catalog_evaluate_critical() {
        let catalog = SeverityContractCatalog::new();

        let mut metrics = HashMap::new();
        metrics.insert("clock_drift_direction".to_string(), -0.5); // Negative drift

        let matches = catalog.evaluate(&metrics);
        assert!(matches.len() > 0);

        let (contract, _) = &matches[0];
        assert_eq!(contract.target_severity, "critical");
    }

    #[test]
    fn test_determine_severity() {
        let catalog = SeverityContractCatalog::new();

        let mut metrics = HashMap::new();
        metrics.insert("response_time_increase_pct".to_string(), 150.0);
        metrics.insert("trend_slope_ms_per_hour".to_string(), 1.0);

        let result = catalog.determine_severity(&metrics);
        assert!(result.is_some());

        let (severity, confidence) = result.unwrap();
        assert_eq!(severity, "critical");
        assert!(confidence > 0.9);
    }

    #[test]
    fn test_matching_conditions() {
        let mut contract = SeverityContract::new("test", "high");
        contract.add_condition("metric1", 0.5, 1.0);
        contract.add_condition("metric2", 10.0, 20.0);
        contract.add_condition("metric3", 50.0, 100.0);

        let mut metrics = HashMap::new();
        metrics.insert("metric1".to_string(), 0.75); // Matches
        metrics.insert("metric2".to_string(), 15.0); // Matches
        metrics.insert("metric3".to_string(), 30.0); // Doesn't match

        let matching = contract.matching_conditions(&metrics);
        assert_eq!(matching.len(), 2);
        assert!(matching.contains(&"metric1".to_string()));
        assert!(matching.contains(&"metric2".to_string()));
    }

    #[test]
    fn test_contract_priority() {
        let catalog = SeverityContractCatalog::new();

        // Create metrics that match both high and medium contracts
        let mut metrics = HashMap::new();
        metrics.insert("response_time_increase_pct".to_string(), 150.0);
        metrics.insert("trend_slope_ms_per_hour".to_string(), 1.0); // Critical
        metrics.insert("quality_confidence_correlation".to_string(), 0.8); // Medium

        let (severity, _) = catalog.determine_severity(&metrics).unwrap();
        assert_eq!(severity, "critical"); // Critical takes priority
    }
}
