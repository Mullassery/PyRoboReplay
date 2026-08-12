//! Unified navigation session representation for Phase 14
//!
//! NavigationSession integrates all modalities into a single coherent data model
//! supporting multi-modal queries and cross-modality correlation.

use crate::phase14::timeline_indexing::{Timeline, TimelineEvent, Modality, TimeSliceQuery, TimeSlice};
use crate::phase14::temporal_sync::TemporalSyncEngine;
use crate::phase14::modality_adapters::{DataSourceType, DataSource as DataSourceTrait};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Failed to build session: {0}")]
    BuildFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Modality not available: {0}")]
    ModalityNotAvailable(String),

    #[error("Unsupported: {0}")]
    Unsupported(String),
}

pub type SessionResult<T> = Result<T, SessionError>;

/// Extension point for 3D scene reconstruction from point-cloud/sensor data.
///
/// Deliberately *not* implemented in this crate: PyRoboReplay's own
/// architectural boundary (docs/CLAUDE.md, "Principle 2: Mapping-Independent")
/// assigns 3D reconstruction, real-time SLAM, and traversability analysis to
/// PyTerrainMap, a sibling SHER-stack system, so PyRoboReplay stays a
/// replay/analysis layer that *consumes* maps rather than building them. This
/// trait is the seam a future PyTerrainMap integration (once it becomes an
/// actual declared `Cargo.toml`/`pyproject.toml` dependency — it is not one
/// today) would implement and hand to
/// [`NavigationSession::reconstruct_3d_with`].
pub trait SceneReconstructor: Send + Sync {
    /// Reconstruct a 3D scene representation from the session's available
    /// sensor/point-cloud timeline data. The returned bytes are an opaque,
    /// reconstructor-owned serialization (e.g. a mesh format defined by
    /// PyTerrainMap) — this crate does not interpret them.
    fn reconstruct(&self, session: &NavigationSession) -> SessionResult<Vec<u8>>;
}

/// Unified representation of a robot navigation mission
#[derive(Clone)]
pub struct NavigationSession {
    /// Mission identifier
    pub mission_id: String,

    /// Global timeline with all synchronized events
    pub timeline: Timeline,

    /// Temporal sync engine for cross-modality timestamp conversion
    pub sync_engine: TemporalSyncEngine,

    /// Available data modalities
    pub available_modalities: HashSet<Modality>,

    /// Data completeness score (0-1)
    pub data_completeness: f32,

    /// Mission metadata
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub duration_seconds: Option<f64>,
    pub robot_name: Option<String>,
    pub environment_type: Option<String>,
    pub mission_notes: Option<String>,
    pub custom_fields: HashMap<String, String>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        SessionMetadata {
            start_time: None,
            end_time: None,
            duration_seconds: None,
            robot_name: None,
            environment_type: None,
            mission_notes: None,
            custom_fields: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataSource {
    RosBag,
    LinuxLogs,
    Nav2Export,
    Video,
    PointCloud,
    Annotation,
    SensorCalibration,
    EnvironmentMap,
    RobotModel,
}

/// Builder pattern for constructing NavigationSession
pub struct SessionBuilder {
    mission_id: String,
    data_sources: HashMap<DataSource, bool>,
    metadata: SessionMetadata,
    timeline: Timeline,
    sync_engine: Option<TemporalSyncEngine>,
}

impl SessionBuilder {
    pub fn new(mission_id: String) -> Self {
        SessionBuilder {
            mission_id,
            data_sources: HashMap::new(),
            metadata: SessionMetadata::default(),
            timeline: Timeline::new(),
            sync_engine: None,
        }
    }

    pub fn with_ros_bag(mut self) -> Self {
        self.data_sources.insert(DataSource::RosBag, true);
        self
    }

    pub fn with_linux_logs(mut self) -> Self {
        self.data_sources.insert(DataSource::LinuxLogs, true);
        self
    }

    pub fn with_nav2_export(mut self) -> Self {
        self.data_sources.insert(DataSource::Nav2Export, true);
        self
    }

    pub fn with_video(mut self) -> Self {
        self.data_sources.insert(DataSource::Video, true);
        self
    }

    pub fn with_point_cloud(mut self) -> Self {
        self.data_sources.insert(DataSource::PointCloud, true);
        self
    }

    pub fn with_annotation(mut self) -> Self {
        self.data_sources.insert(DataSource::Annotation, true);
        self
    }

    pub fn with_metadata(mut self, metadata: SessionMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_sync_engine(mut self, engine: TemporalSyncEngine) -> Self {
        self.sync_engine = Some(engine);
        self
    }

    pub fn build(self) -> SessionResult<NavigationSession> {
        let available = self.data_sources.keys()
            .filter(|_| self.data_sources[&self.data_sources.keys().next().unwrap()])
            .copied()
            .collect::<HashSet<_>>();

        let modality_set: HashSet<Modality> = available.iter()
            .map(|ds| datasource_to_modality(*ds))
            .collect();

        let data_completeness = modality_set.len() as f32 / 6.0; // 6 possible modalities

        let sync_engine = self.sync_engine
            .unwrap_or_else(|| TemporalSyncEngine::new(0));

        Ok(NavigationSession {
            mission_id: self.mission_id,
            timeline: self.timeline,
            sync_engine,
            available_modalities: modality_set,
            data_completeness,
            metadata: self.metadata,
        })
    }
}

impl NavigationSession {
    /// Query state at specific time
    pub fn query_at(&self, time: i64) -> SessionResult<TimeSlice> {
        self.timeline.query_slice(time, 100_000_000) // ±100ms default window
            .map_err(|e| SessionError::QueryFailed(e.to_string()))
    }

    /// Query state within time range
    pub fn query_range(&self, start: i64, end: i64) -> SessionResult<TimeSlice> {
        self.timeline.query_range(start, end)
            .map_err(|e| SessionError::QueryFailed(e.to_string()))
    }

    /// Query specific modality
    pub fn query_modality(
        &self,
        modality: Modality,
        start: i64,
        end: i64,
    ) -> SessionResult<Vec<(i64, TimelineEvent)>> {
        self.timeline.query_modality(modality, start, end)
            .map_err(|e| SessionError::QueryFailed(e.to_string()))
    }

    /// Find time of failure based on timeline events
    pub fn find_failure_time(&self) -> Option<i64> {
        // Heuristic: look for sudden discontinuity in robot state or error-level logs
        if let Ok(range) = self.timeline.query_range(0, i64::MAX) {
            for (ts, event, _modality) in range.events {
                if let TimelineEvent::LogEntry { level, .. } = event {
                    if matches!(level, crate::phase14::timeline_indexing::LogLevel::Error) {
                        return Some(ts);
                    }
                }
            }
        }
        None
    }

    /// Get all available modalities
    pub fn modalities(&self) -> Vec<Modality> {
        self.available_modalities.iter().copied().collect()
    }

    /// Check if modality is available
    pub fn has_modality(&self, modality: Modality) -> bool {
        self.available_modalities.contains(&modality)
    }

    /// Get data completeness percentage
    pub fn completeness_percentage(&self) -> f32 {
        self.data_completeness * 100.0
    }

    /// Reconstruct 3D scene from multi-sensor data.
    ///
    /// PyRoboReplay does not, and deliberately will not, implement 3D
    /// reconstruction (SLAM-grade point-cloud fusion, meshing, etc.) itself.
    /// Per this repo's own architectural boundary (docs/CLAUDE.md,
    /// "Principle 2: Mapping-Independent" — "Does not generate maps. Consumes
    /// maps from PyTerrainMap, SLAM systems, GIS platforms.") that
    /// responsibility belongs to PyTerrainMap, a sibling system in the SHER
    /// stack. As of this pass PyTerrainMap is not a declared dependency of
    /// this crate (checked `Cargo.toml`/`pyproject.toml` — no path/git
    /// dependency present), so there is nothing real to call yet; building a
    /// local reconstruction implementation here would duplicate work another
    /// subsystem owns and would need to be thrown away once the real
    /// integration lands.
    ///
    /// This method is therefore the stable call site plus extension point:
    /// it delegates to an injected [`SceneReconstructor`] (the seam a future
    /// PyTerrainMap adapter would implement) and reports explicitly via
    /// `SessionError::Unsupported` when none is configured, rather than
    /// silently returning an empty/fake scene.
    pub fn reconstruct_3d(&self) -> SessionResult<Vec<u8>> {
        self.reconstruct_3d_with(None)
    }

    /// Same as [`Self::reconstruct_3d`], but with an explicit reconstructor
    /// injected (e.g. by a future PyTerrainMap integration). Pass `None` to
    /// get the same "not configured" behavior as `reconstruct_3d()`.
    pub fn reconstruct_3d_with(
        &self,
        reconstructor: Option<&dyn SceneReconstructor>,
    ) -> SessionResult<Vec<u8>> {
        if !self.has_modality(Modality::Sensors) {
            return Err(SessionError::ModalityNotAvailable("Sensors".to_string()));
        }

        match reconstructor {
            Some(r) => r.reconstruct(self),
            None => Err(SessionError::Unsupported(
                "3D reconstruction requires an external SceneReconstructor (e.g. a PyTerrainMap \
                 integration); none is configured. PyRoboReplay is mapping-independent by design \
                 (docs/CLAUDE.md Principle 2) and does not implement reconstruction locally."
                    .to_string(),
            )),
        }
    }

    /// Get summary statistics
    pub fn summary_stats(&self) -> SummaryStats {
        let (start, end) = self.timeline.time_range()
            .unwrap_or((0, 0));

        let duration_ns = end - start;
        let duration_s = duration_ns as f64 / 1e9;

        SummaryStats {
            mission_id: self.mission_id.clone(),
            available_modalities: self.available_modalities.len(),
            data_completeness: self.data_completeness,
            start_time: start,
            end_time: end,
            duration_seconds: duration_s,
            total_events: self.timeline.event_count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStats {
    pub mission_id: String,
    pub available_modalities: usize,
    pub data_completeness: f32,
    pub start_time: i64,
    pub end_time: i64,
    pub duration_seconds: f64,
    pub total_events: usize,
}

fn datasource_to_modality(ds: DataSource) -> Modality {
    match ds {
        DataSource::RosBag => Modality::RosBag,
        DataSource::LinuxLogs => Modality::LinuxLogs,
        DataSource::Nav2Export => Modality::RosBag, // Part of ROS ecosystem
        DataSource::Video => Modality::Video,
        DataSource::PointCloud => Modality::Sensors,
        DataSource::Annotation => Modality::Annotations,
        DataSource::SensorCalibration => Modality::Sensors,
        DataSource::EnvironmentMap => Modality::RosBag, // Static context
        DataSource::RobotModel => Modality::RosBag,    // Static context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_builder_creation() {
        let result = SessionBuilder::new("test_mission".to_string())
            .with_ros_bag()
            .with_video()
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_session_metadata() {
        let metadata = SessionMetadata {
            robot_name: Some("robot_a".to_string()),
            environment_type: Some("warehouse".to_string()),
            ..Default::default()
        };

        let session = SessionBuilder::new("test".to_string())
            .with_metadata(metadata.clone())
            .build()
            .unwrap();

        assert_eq!(session.metadata.robot_name, Some("robot_a".to_string()));
    }

    #[test]
    fn test_session_modality_check() {
        let session = SessionBuilder::new("test".to_string())
            .with_ros_bag()
            .build()
            .unwrap();

        assert!(session.has_modality(Modality::RosBag));
        assert!(!session.has_modality(Modality::Video));
    }

    #[test]
    fn test_summary_stats() {
        let session = SessionBuilder::new("test".to_string())
            .with_ros_bag()
            .build()
            .unwrap();

        let stats = session.summary_stats();
        assert_eq!(stats.mission_id, "test");
        assert_eq!(stats.available_modalities, 1);
    }
}
