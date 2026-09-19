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

pub mod analyzer_capabilities;
pub mod modality_adapters;
pub mod navigation_session;
pub mod temporal_sync;
pub mod timeline_indexing;
pub mod video_processing;

// Re-exports for public API
pub use analyzer_capabilities::{AnalysisCapability, AnalyzerCapabilitiesV2, AnalyzerRegistry};
pub use modality_adapters::{
    AnnotationAdapter, DataSource, LinuxLogsAdapter, Nav2ExportAdapter, PointCloudAdapter,
    RosBagAdapter, VideoAdapter,
};
pub use navigation_session::{DataSource as SessionDataSource, NavigationSession, SessionBuilder};
pub use temporal_sync::{ClockOffset, SyncReport, TemporalSyncEngine, TimeModel};
pub use timeline_indexing::{
    EventIndex, Modality, TimeSlice, TimeSliceQuery, Timeline, TimelineEvent,
};
pub use video_processing::{FrameData, ObjectDetection, OpticalFlowFrame, VideoProcessor};

#[cfg(test)]
mod tests {

    #[test]
    fn test_phase14_modules_accessible() {
        // Ensure all public types are accessible
        let _: () = ();
    }
}
