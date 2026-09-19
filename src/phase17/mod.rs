pub mod alternative_timeline;
/// Phase 17: Counterfactual Analysis & Outcome Influence
///
/// Generate alternative histories ("what if?") and calculate which factors
/// had the greatest impact on outcomes using causal graphs from Phase 16.
pub mod counterfactual;
pub mod outcome_influence;

pub use alternative_timeline::{AlternativeTimeline, TimelineComparison};
pub use counterfactual::{CounterfactualAnalyzer, CounterfactualQuery, QueryType};
pub use outcome_influence::{InfluenceScore, OutcomeInfluenceAnalyzer};
