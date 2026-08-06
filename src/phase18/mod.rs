/// Phase 18: Pattern Discovery & Fleet Learning
///
/// Mine failure patterns across millions of missions, cluster similar decisions,
/// and generate fleet-wide recommendations based on observed patterns.

pub mod pattern_discovery;
pub mod decision_clustering;
pub mod fleet_learning;

pub use pattern_discovery::{FailurePattern, PatternMiner, PatternCluster};
pub use decision_clustering::{DecisionTemplate, ClusterAnalyzer};
pub use fleet_learning::{FleetLearner, LeaderboardEntry, OptimizationTip};
