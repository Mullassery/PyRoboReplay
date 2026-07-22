//! Retrospective Intelligence Platform
//!
//! Extracts latent information from historical recordings using modern AI.
//! Reconstructs scene understanding the robot never had during operation.
//!
//! Philosophy: The replay engine is smarter than the robot was.
//!
//! Core capability: Derive insights from recorded pixels that weren't available
//! to the robot's onboard software in real time.

pub mod scene_reconstruction;
pub mod perception_gap_analysis;
pub mod hidden_explanations;
pub mod event_extraction;
pub mod agent_friendly_output;

pub use scene_reconstruction::{RetrospectiveScene, SceneTimeline};
pub use perception_gap_analysis::{PerceptionGap, GapAnalysis};
pub use hidden_explanations::{HiddenFact, CausalNarrative};
pub use event_extraction::{StructuredEvent, EventStream};
pub use agent_friendly_output::{AgentEvent, AgentMission};
