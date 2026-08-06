/// Phase 20: Causal Counterfactual Analysis & Decision Tree Generation
///
/// Generate decision trees from causal graphs, perform advanced what-if analysis,
/// and extract interpretable decision rules from complex causal relationships.

pub mod decision_tree;
pub mod counterfactual_scenarios;
pub mod rule_extraction;

pub use decision_tree::{DecisionTree, TreeNode, SplitCriterion};
pub use counterfactual_scenarios::{CounterfactualScenario, ScenarioAnalyzer};
pub use rule_extraction::{DecisionRule, RuleExtractor, RuleSet};
