//! Perception Layer: Object Detection, Tracking, and Scene Understanding
//!
//! Reconstructs robot's perception of the world for debugging.
//! Answers: What did the robot see? What did it understand?

pub mod object_detection;
pub mod object_tracking;
pub mod scene_understanding;
pub mod perception_analysis;
pub mod detection_backends;
pub mod detection_orchestrator;
pub mod retrospective_detection;
pub mod gap_context_analyzer;

pub use object_detection::{DetectedObject, ObjectClass, DetectionFrame};
pub use object_tracking::{TrackedObject, TrackingEngine};
pub use scene_understanding::{SpatialRelationship, EnvironmentalCondition, SemanticContext};
pub use perception_analysis::{PerceptionFailure, FailureType};
pub use detection_backends::{DetectionBackend, YOLOBackend, SAMBackend, TemplateBackend, DetectionBackendType, YOLOConfig, SAMConfig};
pub use detection_orchestrator::{DetectionOrchestrator, DetectionStrategy, DetectionOrchestrationStats};
pub use retrospective_detection::{DINOConfig, DINODetection, DetectionGap, InvisibilityFactor, RetrospectiveDetectionEngine, RetrospectiveDetectionStats};
pub use gap_context_analyzer::{ContextualGap, GapSeverityAssessment, GapContextAnalyzer, GapPattern};
