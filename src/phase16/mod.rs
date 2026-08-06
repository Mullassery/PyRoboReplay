/// Phase 16: Causal Graph Construction & Decision Reconstruction
///
/// Automated causal graph generation from mission timelines and full decision context recovery
/// with temporal fusion (Phase 14) and root cause inference (Phase 15) integration.

pub mod causal_builder;
pub mod decision_reconstructor;
pub mod graph_validator;
pub mod pattern_matcher;

pub use causal_builder::{CausalGraphBuilderV2, EdgeDetector, TemporalProximityDetector, MagnitudeChangeDetector, DecisionTriggerDetector, MultiModalDetector, HistoricalDetector};
pub use decision_reconstructor::{Decision, DecisionCategory, DecisionContext, Alternative, DecisionOutcome, DecisionReconstructor};
pub use graph_validator::CausalGraphValidator;
pub use pattern_matcher::{DecisionPattern, DecisionPatternMatcher, DecisionTemplate};
