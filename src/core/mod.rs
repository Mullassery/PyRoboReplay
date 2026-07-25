pub mod event;
pub mod timeline;
pub mod causality;
pub mod correlation;
pub mod spatial_causality;
pub mod pyterrain_bridge;
pub mod coverage_evolution;
pub mod multi_robot;
pub mod root_cause;
pub mod counterfactual;
pub mod recommendation;
pub mod diagnostic_report;
pub mod deterministic_replay;
pub mod compliance;
pub mod cross_mission;
pub mod anomaly_detector;
pub mod explanation;
pub mod failure_actions;
pub mod geospatial_export;
pub mod incident_bundle;
pub mod evidence_discovery;
pub mod timeline_correlation;
pub mod failure_detection;
pub mod confidence_scoring;
pub mod recommendations_engine;
pub mod incident_analysis;

pub use event::{MissionEvent, MissionRecord, Pose, Location};
pub use timeline::Timeline;
pub use causality::{
    CausalGraph, CausalGraphBuilder, CausalLink, CausalChain, CausalQuery, CausalHypothesis,
};
pub use correlation::{CorrelationAnalyzer, EventCorrelation, CorrelationStats, AnomalyPattern, EventChain};
pub use spatial_causality::{
    SpatialCausalityAnalyzer, SpatialContext, SpatialCausalLink, SpatialCausalQuery, SpatialRegion, SpatialCausalStats,
};
pub use pyterrain_bridge::{PyTerrainBridge, TerrainKnowledgeGraph, Obstacle, TraversabilityZone, CoverageMap, CoverageEvolution};
pub use coverage_evolution::{CoverageEvolutionAnalyzer, CoverageEvolutionQuery, CoverageSnapshot, CoverageGap, CoverageHotspot, CoverageEvolutionStats};
pub use multi_robot::{MultiRobotCoordinationAnalyzer, CoordinationEvent, CommunicationLink, FleetSnapshot, RobotState, InterRobotCausalLink, CoordinationPattern, MultiRobotCoordinationStats};
pub use root_cause::{RootCauseAnalyzer, RootCauseAnalysis, RootCauseHypothesis, FailureMode, DiagnosticStats};
pub use counterfactual::{CounterfactualAnalyzer, CounterfactualAnalysis, CounterfactualScenario, ScenarioImpact, CriticalCausalLink, CounterfactualStats};
pub use recommendation::{RecommendationEngine, Recommendation, RecommendationSet, RecommendationStats};
pub use diagnostic_report::{DiagnosticReportGenerator, DiagnosticReport, ExecutiveSummary, DiagnosticSection, ReportFormat};
pub use deterministic_replay::{DeterministicReplay, ReplayManifest, DeterministicReplayError, EventHasher};
pub use compliance::{
    ComplianceEvent, ComplianceReport, ComplianceReportGenerator, ComplianceConfig,
    ComplianceViolation, ViolationType, ViolationSeverity, ProximityZoneEvent, EmergencyStopEvent,
    SpeedComplianceEvent, OperatorPresenceEvent, ProximityZoneType,
};
pub use cross_mission::{
    MissionPattern, MissionOccurrence, PatternMatch, PatternLibrary, CrossMissionAnalyzer,
};
pub use anomaly_detector::{Failure, AnomalyDetector};
pub use explanation::ExplanationGenerator;
pub use failure_actions::{Action, ActionRecommender};
pub use geospatial_export::{GeospatialExporter, GeoJsonExport, CoverageRaster, GeoHotspot};
pub use incident_bundle::{
    IncidentBundle, BundleManifest, LayerAvailability, LayerFileInventory,
    TimeRange, BundleError,
};
pub use evidence_discovery::EvidenceDiscovery;
pub use timeline_correlation::{
    TimelineCorrelationEngine, NormalizedEvent, ClockSyncState,
};
pub use failure_detection::{
    FailureDetectionEngine, DetectedFailure, FailureDomain, FailureSeverity,
    NavigationFailureDetector, LocalizationFailureDetector, PerceptionFailureDetector,
    MiddlewareFailureDetector, SystemFailureDetector,
};
pub use confidence_scoring::{
    ConfidenceScoringEngine, ConfidenceChain, ConfidenceTier, EvidenceItem,
};
pub use recommendations_engine::{
    MLRIASRecommendationsEngine, MLRIASRecommendation, Priority,
};
pub use incident_analysis::{
    IncidentAnalysisOrchestrator, IncidentAnalysisReport, AnalysisResult,
    FailureReport, RecommendationReport, AnalysisSummary,
};
