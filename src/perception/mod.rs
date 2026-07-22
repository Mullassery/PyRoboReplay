//! Perception Layer: Object Detection, Tracking, and Scene Understanding
//!
//! Reconstructs robot's perception of the world for debugging.
//! Answers: What did the robot see? What did it understand?

pub mod object_detection;
pub mod object_tracking;
pub mod scene_understanding;
pub mod perception_analysis;

pub use object_detection::{DetectedObject, ObjectClass, DetectionFrame};
pub use object_tracking::{TrackedObject, TrackingEngine};
pub use scene_understanding::{SpatialRelationship, EnvironmentalCondition, SemanticContext};
pub use perception_analysis::{PerceptionFailure, FailureType};
