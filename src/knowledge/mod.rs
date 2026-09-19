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

pub mod change_detection;
pub mod fleet_terrain_learning;
pub mod knowledge_graph;
pub mod longitudinal_reasoning;
pub mod multi_mission_learning;
pub mod spatial_grounding;
pub mod temporal_analysis;
pub mod terrain_integration;
pub mod world_model;

pub use change_detection::ChangeDetector;
pub use fleet_terrain_learning::{
    FleetTerrainModel, RobotProfile, RobotTraversabilityObservation, TerrainConsensus,
};
pub use knowledge_graph::KnowledgeGraph;
pub use longitudinal_reasoning::LongitudinalAnalyzer;
pub use multi_mission_learning::{
    LearningProgression, MissionContext, MissionTrace, MultiMissionLearner,
};
pub use spatial_grounding::{
    GroundedEntity, SpatialCoordinates, SpatialGroundingEngine, SpatialTemporalTrend,
};
pub use temporal_analysis::TemporalAnalyzer;
pub use terrain_integration::{
    EntityTerrainContext, TerrainIntegrationEngine, TerrainObstacle, TerrainZone,
};
pub use world_model::{Entity, Location, Observation, WorldState};
