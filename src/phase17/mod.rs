/// Phase 17: Counterfactual Analysis & Outcome Influence
///
/// Generate alternative histories ("what if?") and calculate which factors
/// had the greatest impact on outcomes using causal graphs from Phase 16.

pub mod counterfactual;
pub mod outcome_influence;
pub mod alternative_timeline;

pub use counterfactual::{CounterfactualQuery, CounterfactualAnalyzer, QueryType};
pub use outcome_influence::{InfluenceScore, OutcomeInfluenceAnalyzer};
pub use alternative_timeline::{AlternativeTimeline, TimelineComparison};
