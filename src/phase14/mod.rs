//! Phase 14: Universal Temporal Fusion Foundation
//!
//! Transforms PyRoboReplay from a single-modality replay engine into a multi-modal
//! temporal fusion platform. Accepts heterogeneous data sources (ROS bags, video,
//! Linux logs, Nav2 diagnostics, sensor streams, annotations) and aligns them to
//! a unified timeline for forensic reconstruction and Nav2 limitation analysis.
//!
//! Architecture:
//! 1. Modality Adapters: Parse diverse input formats
//! 2. Temporal Sync Engine: Align all sources to unified timeline
//! 3. Timeline Indexing: Efficient time-series storage and queries
//! 4. Navigation Session: Unified data model with multi-modal API
//! 5. Video Processing: Frame extraction, YOLO detection, optical flow
//! 6. Analyzer Capabilities: Extended registry for multi-modal analysis

pub mod modality_adapters;
pub mod temporal_sync;
pub mod timeline_indexing;
pub mod navigation_session;
pub mod video_processing;
pub mod analyzer_capabilities;

// Re-exports for public API
pub use modality_adapters::{
    DataSource, RosBagAdapter, LinuxLogsAdapter, Nav2ExportAdapter,
    VideoAdapter, PointCloudAdapter, AnnotationAdapter,
};
pub use temporal_sync::{
    TemporalSyncEngine, TimeModel, ClockOffset, SyncReport,
};
pub use timeline_indexing::{
    Timeline, TimelineEvent, TimeSlice, TimeSliceQuery, EventIndex, Modality,
};
pub use navigation_session::{
    NavigationSession, SessionBuilder, DataSource as SessionDataSource,
};
pub use video_processing::{
    VideoProcessor, FrameData, ObjectDetection, OpticalFlowFrame,
};
pub use analyzer_capabilities::{
    AnalyzerCapabilitiesV2, AnalyzerRegistry, AnalysisCapability,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase14_modules_accessible() {
        // Ensure all public types are accessible
        let _: () = ();
    }
}
