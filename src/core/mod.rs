pub mod anomaly_detector;
pub mod causality;
pub mod compliance;
pub mod confidence_scoring;
pub mod correlation;
pub mod counterfactual;
pub mod coverage_evolution;
pub mod cross_mission;
pub mod deterministic_replay;
pub mod diagnostic_report;
pub mod event;
pub mod evidence_discovery;
pub mod explanation;
pub mod failure_actions;
pub mod failure_detection;
pub mod geospatial_export;
pub mod incident_analysis;
pub mod incident_bundle;
pub mod multi_robot;
pub mod pyterrain_bridge;
pub mod recommendation;
pub mod recommendations_engine;
pub mod root_cause;
pub mod spatial_causality;
pub mod timeline;
pub mod timeline_correlation;

pub use anomaly_detector::{AnomalyDetector, Failure};
pub use causality::{
    CausalChain, CausalGraph, CausalGraphBuilder, CausalHypothesis, CausalLink, CausalQuery,
};
pub use compliance::{
    ComplianceConfig, ComplianceEvent, ComplianceReport, ComplianceReportGenerator,
    ComplianceViolation, EmergencyStopEvent, OperatorPresenceEvent, ProximityZoneEvent,
    ProximityZoneType, SpeedComplianceEvent, ViolationSeverity, ViolationType,
};
pub use confidence_scoring::{
    ConfidenceChain, ConfidenceScoringEngine, ConfidenceTier, EvidenceItem,
};
pub use correlation::{
    AnomalyPattern, CorrelationAnalyzer, CorrelationStats, EventChain, EventCorrelation,
};
pub use counterfactual::{
    CounterfactualAnalysis, CounterfactualAnalyzer, CounterfactualScenario, CounterfactualStats,
    CriticalCausalLink, ScenarioImpact,
};
pub use coverage_evolution::{
    CoverageEvolutionAnalyzer, CoverageEvolutionQuery, CoverageEvolutionStats, CoverageGap,
    CoverageHotspot, CoverageSnapshot,
};
pub use cross_mission::{
    CrossMissionAnalyzer, MissionOccurrence, MissionPattern, PatternLibrary, PatternMatch,
};
pub use deterministic_replay::{
    DeterministicReplay, DeterministicReplayError, EventHasher, ReplayManifest,
};
pub use diagnostic_report::{
    DiagnosticReport, DiagnosticReportGenerator, DiagnosticSection, ExecutiveSummary, ReportFormat,
};
pub use event::{Location, MissionEvent, MissionRecord, Pose};
pub use evidence_discovery::EvidenceDiscovery;
pub use explanation::ExplanationGenerator;
pub use failure_actions::{Action, ActionRecommender};
pub use failure_detection::{
    DetectedFailure, FailureDetectionEngine, FailureDomain, FailureSeverity,
    LocalizationFailureDetector, MiddlewareFailureDetector, NavigationFailureDetector,
    PerceptionFailureDetector, SystemFailureDetector,
};
pub use geospatial_export::{CoverageRaster, GeoHotspot, GeoJsonExport, GeospatialExporter};
pub use incident_analysis::{
    AnalysisResult, AnalysisSummary, FailureReport, IncidentAnalysisOrchestrator,
    IncidentAnalysisReport, RecommendationReport,
};
pub use incident_bundle::{
    BundleError, BundleManifest, IncidentBundle, LayerAvailability, LayerFileInventory, TimeRange,
};
pub use multi_robot::{
    CommunicationLink, CoordinationEvent, CoordinationPattern, FleetSnapshot, InterRobotCausalLink,
    MultiRobotCoordinationAnalyzer, MultiRobotCoordinationStats, RobotState,
};
pub use pyterrain_bridge::{
    CoverageEvolution, CoverageMap, Obstacle, PyTerrainBridge, TerrainKnowledgeGraph,
    TraversabilityZone,
};
pub use recommendation::{
    Recommendation, RecommendationEngine, RecommendationSet, RecommendationStats,
};
pub use recommendations_engine::{MLRIASRecommendation, MLRIASRecommendationsEngine, Priority};
pub use root_cause::{
    DiagnosticStats, FailureMode, RootCauseAnalysis, RootCauseAnalyzer, RootCauseHypothesis,
};
pub use spatial_causality::{
    SpatialCausalLink, SpatialCausalQuery, SpatialCausalStats, SpatialCausalityAnalyzer,
    SpatialContext, SpatialRegion,
};
pub use timeline::Timeline;
pub use timeline_correlation::{ClockSyncState, NormalizedEvent, TimelineCorrelationEngine};
