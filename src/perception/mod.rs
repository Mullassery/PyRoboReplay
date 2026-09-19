//! Perception Layer: Object Detection, Tracking, and Scene Understanding
//!
//! Reconstructs robot's perception of the world for debugging.
//! Answers: What did the robot see? What did it understand?

pub mod detection_backends;
pub mod detection_orchestrator;
pub mod gap_context_analyzer;
pub mod object_detection;
pub mod object_tracking;
pub mod perception_analysis;
pub mod retrospective_detection;
pub mod scene_understanding;

pub use detection_backends::{
    DetectionBackend, DetectionBackendType, SAMBackend, SAMConfig, TemplateBackend, YOLOBackend,
    YOLOConfig,
};
pub use detection_orchestrator::{
    DetectionOrchestrationStats, DetectionOrchestrator, DetectionStrategy,
};
pub use gap_context_analyzer::{
    ContextualGap, GapContextAnalyzer, GapPattern, GapSeverityAssessment,
};
pub use object_detection::{DetectedObject, DetectionFrame, ObjectClass};
pub use object_tracking::{TrackedObject, TrackingEngine};
pub use perception_analysis::{FailureType, PerceptionFailure};
pub use retrospective_detection::{
    DINOConfig, DINODetection, DetectionGap, InvisibilityFactor, RetrospectiveDetectionEngine,
    RetrospectiveDetectionStats,
};
pub use scene_understanding::{EnvironmentalCondition, SemanticContext, SpatialRelationship};
