pub mod decision_clustering;
pub mod fleet_learning;
/// Phase 18: Pattern Discovery & Fleet Learning
///
/// Mine failure patterns across millions of missions, cluster similar decisions,
/// and generate fleet-wide recommendations based on observed patterns.
pub mod pattern_discovery;

pub use decision_clustering::{ClusterAnalyzer, DecisionTemplate};
pub use fleet_learning::{FleetLearner, LeaderboardEntry, OptimizationTip};
pub use pattern_discovery::{FailurePattern, PatternCluster, PatternMiner};
