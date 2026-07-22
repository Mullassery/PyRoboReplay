//! Persistent World Knowledge Framework
//!
//! Phase 10: Transforms replay from single-mission analysis into
//! longitudinal temporal reasoning system.
//!
//! Core principle: Every mission builds a persistent world model.
//! Future missions are analyzed in context of everything observed before.
//!
//! Enables:
//! - "What changed since last visit?"
//! - "Is this behavior anomalous compared to history?"
//! - "How did the environment evolve to cause this failure?"
//! - Cross-mission pattern detection
//! - Temporal anomaly scoring

pub mod world_model;
pub mod knowledge_graph;
pub mod temporal_analysis;
pub mod change_detection;
pub mod longitudinal_reasoning;

pub use world_model::{WorldState, Entity, Location, Observation};
pub use knowledge_graph::KnowledgeGraph;
pub use temporal_analysis::TemporalAnalyzer;
pub use change_detection::ChangeDetector;
pub use longitudinal_reasoning::LongitudinalAnalyzer;
