//! Build a Phase 14 [`NavigationSession`] from real on-disk sources.
//!
//! This is the missing end-to-end wiring between the Phase 14 adapters and
//! [`Timeline`]: each adapter parses a real file, its stream is inserted as a
//! typed [`TimelineEvent`], and the resulting modalities/completeness are
//! reported through [`NavigationSessionSummary`]. No live ROS connection is
//! required.

use crate::phase14::modality_adapters::{
    DataSource, LinuxLogsAdapter, LogType, Nav2ExportAdapter, RosBagAdapter,
};
use crate::phase14::navigation_session::{
    NavigationSession, SessionError, SessionMetadata, SummaryStats,
};
use crate::phase14::temporal_sync::TemporalSyncEngine;
use crate::phase14::timeline_indexing::{LogLevel, Modality, Timeline, TimelineEvent};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// A compact, serializable report of what a session contains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationSessionSummary {
    pub stats: SummaryStats,
    pub modalities: Vec<String>,
    pub topic_event_counts: Vec<(String, usize)>,
    pub first_error_time_ns: Option<i64>,
}

impl NavigationSession {
    /// Summarize modalities, per-topic event counts, and first error timestamp.
    pub fn public_summary(&self) -> NavigationSessionSummary {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for (ts, event, _) in self.timeline.all_events() {
            let _ = ts;
            let label = match &event {
                TimelineEvent::RosEvent { topic, .. } => topic.clone(),
                TimelineEvent::VideoFrame { camera_name, .. } => format!("video:{camera_name}"),
                TimelineEvent::LogEntry { source, .. } => format!("log:{source}"),
                TimelineEvent::SensorReading { sensor_id, .. } => format!("sensor:{sensor_id}"),
                TimelineEvent::Annotation { .. } => "annotation".to_string(),
                TimelineEvent::SystemMetric { metric_name, .. } => {
                    format!("metric:{metric_name}")
                }
            };
            *counts.entry(label).or_insert(0) += 1;
        }
        let mut topic_event_counts: Vec<(String, usize)> = counts.into_iter().collect();
        topic_event_counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let mut modalities: Vec<String> = self
            .available_modalities
            .iter()
            .map(|m| m.to_string())
            .collect();
        modalities.sort();

        NavigationSessionSummary {
            stats: self.summary_stats(),
            modalities,
            topic_event_counts,
            first_error_time_ns: self.find_failure_time(),
        }
    }
}

/// Sources accepted by [`build_navigation_session`].
#[derive(Debug, Clone, Default)]
pub struct SessionSources<'a> {
    pub bag: Option<&'a Path>,
    pub log_file: Option<&'a Path>,
    pub log_type: Option<LogType>,
    pub nav2_export_dir: Option<&'a Path>,
}

/// Build a navigation session from a ROS 2 bag and optional logs/Nav2 exports.
pub fn build_navigation_session(
    mission_id: impl Into<String>,
    sources: &SessionSources<'_>,
) -> Result<NavigationSession, SessionError> {
    if sources.bag.is_none() && sources.log_file.is_none() && sources.nav2_export_dir.is_none() {
        return Err(SessionError::BuildFailed(
            "at least one source (bag, log_file, or nav2_export_dir) is required".to_string(),
        ));
    }

    let mut timeline = Timeline::new();
    let mut available_modalities: HashSet<Modality> = HashSet::new();
    let mut start_time: Option<i64> = None;
    let mut end_time: Option<i64> = None;

    if let Some(path) = sources.bag {
        let adapter = RosBagAdapter::new();
        let source = adapter
            .load(path)
            .map_err(|e| SessionError::BuildFailed(format!("ROS bag: {e}")))?;
        for topic in &source.topics {
            let points = adapter.extract_stream(&topic.name).map_err(|e| {
                SessionError::BuildFailed(format!("ROS bag topic {}: {e}", topic.name))
            })?;
            for point in points {
                start_time = Some(start_time.map_or(point.timestamp, |s| s.min(point.timestamp)));
                end_time = Some(end_time.map_or(point.timestamp, |e| e.max(point.timestamp)));
                timeline.insert(
                    point.timestamp,
                    TimelineEvent::RosEvent {
                        topic: point.topic,
                        msg_type: topic.msg_type.clone(),
                        payload: point.value,
                        frame_id: None,
                    },
                    Modality::RosBag,
                );
            }
        }
        if !source.topics.is_empty() {
            available_modalities.insert(Modality::RosBag);
        }
    }

    if let Some(path) = sources.log_file {
        let log_type = sources.log_type.unwrap_or(LogType::Syslog);
        let adapter = LinuxLogsAdapter::new(log_type);
        let source = adapter
            .load(path)
            .map_err(|e| SessionError::BuildFailed(format!("Linux logs: {e}")))?;
        let points = adapter
            .extract_stream("")
            .map_err(|e| SessionError::BuildFailed(format!("Linux logs: {e}")))?;
        for point in points {
            let value: serde_json::Value = serde_json::from_slice(&point.value)
                .map_err(|e| SessionError::BuildFailed(format!("log record: {e}")))?;
            let source_name = value
                .get("process")
                .or_else(|| value.get("host"))
                .and_then(|v| v.as_str())
                .unwrap_or("linux")
                .to_string();
            let severity = value
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("info");
            let message = value
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            start_time = Some(start_time.map_or(point.timestamp, |s| s.min(point.timestamp)));
            end_time = Some(end_time.map_or(point.timestamp, |e| e.max(point.timestamp)));
            timeline.insert(
                point.timestamp,
                TimelineEvent::LogEntry {
                    source: source_name,
                    level: parse_log_level(severity),
                    message,
                },
                Modality::LinuxLogs,
            );
        }
        if !source.topics.is_empty() {
            available_modalities.insert(Modality::LinuxLogs);
        }
    }

    if let Some(path) = sources.nav2_export_dir {
        let adapter = Nav2ExportAdapter::new();
        let source = adapter
            .load(path)
            .map_err(|e| SessionError::BuildFailed(format!("Nav2 export: {e}")))?;
        for topic in &source.topics {
            let points = adapter.extract_stream(&topic.name).map_err(|e| {
                SessionError::BuildFailed(format!("Nav2 topic {}: {e}", topic.name))
            })?;
            for point in points {
                start_time = Some(start_time.map_or(point.timestamp, |s| s.min(point.timestamp)));
                end_time = Some(end_time.map_or(point.timestamp, |e| e.max(point.timestamp)));
                timeline.insert(
                    point.timestamp,
                    TimelineEvent::RosEvent {
                        topic: point.topic,
                        msg_type: topic.msg_type.clone(),
                        payload: point.value,
                        frame_id: None,
                    },
                    Modality::RosBag,
                );
            }
        }
        if !source.topics.is_empty() {
            available_modalities.insert(Modality::RosBag);
        }
    }

    let duration_ns = match (start_time, end_time) {
        (Some(start), Some(end)) if end >= start => Some((end - start) as f64 / 1e9),
        _ => None,
    };
    let metadata = SessionMetadata {
        start_time,
        end_time,
        duration_seconds: duration_ns,
        ..Default::default()
    };

    Ok(NavigationSession {
        mission_id: mission_id.into(),
        timeline,
        sync_engine: TemporalSyncEngine::new(0),
        available_modalities,
        data_completeness: 0.0,
        metadata,
    }
    .with_computed_completeness())
}

trait Completeness {
    fn with_computed_completeness(self) -> Self;
}

impl Completeness for NavigationSession {
    fn with_computed_completeness(mut self) -> Self {
        self.data_completeness = self.available_modalities.len() as f32 / 6.0;
        self
    }
}

fn parse_log_level(severity: &str) -> LogLevel {
    match severity.to_ascii_lowercase().as_str() {
        "emergency" | "alert" | "critical" | "error" => LogLevel::Error,
        "warning" => LogLevel::Warn,
        "notice" | "info" => LogLevel::Info,
        "debug" => LogLevel::Debug,
        _ => LogLevel::Trace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_bag() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ros2_cdr_fixture.db3")
    }

    #[test]
    fn builds_session_from_real_ros2_fixture() {
        let bag = fixture_bag();
        let session = build_navigation_session(
            "fixture",
            &SessionSources {
                bag: Some(&bag),
                ..Default::default()
            },
        )
        .expect("fixture should build");
        let summary = session.public_summary();
        assert!(summary.stats.total_events > 0);
        assert!(summary.modalities.contains(&"ROS Bag".to_string()));
        assert!(summary.stats.duration_seconds >= 0.0);
    }

    #[test]
    fn rejects_empty_sources() {
        let result = build_navigation_session("empty", &SessionSources::default());
        assert!(matches!(result, Err(SessionError::BuildFailed(_))));
    }
}
