/// Phase 19: Temporal Pattern Discovery - Analyze patterns across time windows
///
/// Discover patterns that emerge over different temporal scales:
/// - Short-term (seconds): Real-time failure cascades
/// - Medium-term (minutes): Multi-step recovery procedures
/// - Long-term (hours/days): Fleet-wide optimization trends
pub mod temporal_patterns;
pub mod trend_detector;
pub mod window_analyzer;

pub use temporal_patterns::{TemporalPattern, TemporalPatternMiner, TimeWindow};
pub use trend_detector::{Trend, TrendDetector, TrendType};
pub use window_analyzer::{WindowAnalyzer, WindowStatistics};
