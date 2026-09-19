//! Retrospective Intelligence Platform
//!
//! Extracts latent information from historical recordings using modern AI.
//! Reconstructs scene understanding the robot never had during operation.
//!
//! Philosophy: The replay engine is smarter than the robot was.
//!
//! Core capability: Derive insights from recorded pixels that weren't available
//! to the robot's onboard software in real time.

pub mod agent_friendly_output;
pub mod event_extraction;
pub mod hidden_explanations;
pub mod perception_gap_analysis;
pub mod scene_reconstruction;

pub use agent_friendly_output::{AgentEvent, AgentMission};
pub use event_extraction::{EventStream, StructuredEvent};
pub use hidden_explanations::{CausalNarrative, HiddenFact};
pub use perception_gap_analysis::{GapAnalysis, PerceptionGap};
pub use scene_reconstruction::{RetrospectiveScene, SceneTimeline};
