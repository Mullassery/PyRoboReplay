//! Timeline indexing and efficient time-series storage for Phase 14
//!
//! Provides a unified timeline representation where all modalities (ROS, video,
//! logs, etc.) are indexed by synchronized time. Supports fast range queries,
//! nearest-neighbor lookups, and per-modality filtering.

use std::collections::{BTreeMap, HashMap, HashSet};
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimelineEvent {
    /// ROS message from a topic
    RosEvent {
        topic: String,
        msg_type: String,
        payload: Vec<u8>,
        frame_id: Option<String>,
    },

    /// Video frame with timestamp
    VideoFrame {
        camera_name: String,
        frame_index: u32,
        data: FrameMetadata,
    },

    /// System log entry (syslog, dmesg, etc.)
    LogEntry {
        source: String,
        level: LogLevel,
        message: String,
    },

    /// Raw sensor data (LiDAR, depth, etc.)
    SensorReading {
        sensor_id: String,
        sensor_type: String,
        data: Vec<u8>,
    },

    /// User annotation
    Annotation {
        text: String,
        confidence: f32,
    },

    /// System metric (CPU%, memory, etc.)
    SystemMetric {
        metric_name: String,
        value: f32,
        unit: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub resolution: (u32, u32),
    pub format: String,
    pub size_bytes: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Error)]
pub enum TimelineError {
    #[error("Time not found in timeline")]
    TimeNotFound,

    #[error("Invalid time range: {0}")]
    InvalidRange(String),

    #[error("Modality not available: {0}")]
    ModalityNotAvailable(String),

    #[error("No events in range")]
    NoEventsInRange,
}

pub type TimelineResult<T> = Result<T, TimelineError>;

/// Unified timeline with multi-modal events indexed by timestamp
#[derive(Clone)]
pub struct Timeline {
    /// BTreeMap: timestamp (ROS ns) → events at that time
    events: BTreeMap<i64, Vec<(TimelineEvent, Modality)>>,
    /// Per-modality index for fast filtering
    modality_index: HashMap<Modality, Vec<i64>>,
    /// Global start/end times
    start_time: Option<i64>,
    end_time: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    RosBag,
    LinuxLogs,
    Video,
    Sensors,
    Annotations,
    SystemMetrics,
}

impl std::fmt::Display for Modality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Modality::RosBag => write!(f, "ROS Bag"),
            Modality::LinuxLogs => write!(f, "Linux Logs"),
            Modality::Video => write!(f, "Video"),
            Modality::Sensors => write!(f, "Sensors"),
            Modality::Annotations => write!(f, "Annotations"),
            Modality::SystemMetrics => write!(f, "System Metrics"),
        }
    }
}

impl Timeline {
    pub fn new() -> Self {
        Timeline {
            events: BTreeMap::new(),
            modality_index: HashMap::new(),
            start_time: None,
            end_time: None,
        }
    }

    /// Add event to timeline at specific timestamp
    pub fn insert(&mut self, timestamp: i64, event: TimelineEvent, modality: Modality) {
        // Update start/end times
        if self.start_time.is_none() || timestamp < self.start_time.unwrap() {
            self.start_time = Some(timestamp);
        }
        if self.end_time.is_none() || timestamp > self.end_time.unwrap() {
            self.end_time = Some(timestamp);
        }

        // Insert into main event map
        self.events.entry(timestamp)
            .or_insert_with(Vec::new)
            .push((event, modality));

        // Update modality index
        self.modality_index.entry(modality)
            .or_insert_with(Vec::new)
            .push(timestamp);
    }

    /// Get all events at exact timestamp
    pub fn get_at(&self, timestamp: i64) -> TimelineResult<Vec<&(TimelineEvent, Modality)>> {
        self.events.get(&timestamp)
            .map(|v| v.iter().collect())
            .ok_or(TimelineError::TimeNotFound)
    }

    /// Query time slice: all events within [time - window, time + window]
    pub fn query_slice(&self, time: i64, window: i64) -> TimelineResult<TimeSlice> {
        let start = time - window;
        let end = time + window;

        let mut slice_events = Vec::new();
        for (ts, events) in self.events.range(start..=end) {
            for (event, modality) in events {
                slice_events.push((*ts, event.clone(), *modality));
            }
        }

        if slice_events.is_empty() {
            return Err(TimelineError::NoEventsInRange);
        }

        Ok(TimeSlice {
            center_time: time,
            window,
            events: slice_events,
        })
    }

    /// Query range: all events in [start, end)
    pub fn query_range(&self, start: i64, end: i64) -> TimelineResult<TimeSlice> {
        if start >= end {
            return Err(TimelineError::InvalidRange(
                format!("start ({}) >= end ({})", start, end),
            ));
        }

        let mut slice_events = Vec::new();
        for (ts, events) in self.events.range(start..end) {
            for (event, modality) in events {
                slice_events.push((*ts, event.clone(), *modality));
            }
        }

        if slice_events.is_empty() {
            return Err(TimelineError::NoEventsInRange);
        }

        Ok(TimeSlice {
            center_time: (start + end) / 2,
            window: (end - start) / 2,
            events: slice_events,
        })
    }

    /// Get events for specific modality within time range
    pub fn query_modality(
        &self,
        modality: Modality,
        start: i64,
        end: i64,
    ) -> TimelineResult<Vec<(i64, TimelineEvent)>> {
        let timestamps = self.modality_index.get(&modality)
            .ok_or(TimelineError::ModalityNotAvailable(modality.to_string()))?;

        let mut results = Vec::new();
        for &ts in timestamps {
            if ts >= start && ts < end {
                if let Ok(events) = self.get_at(ts) {
                    for (event, m) in events {
                        if *m == modality {
                            results.push((ts, event.clone()));
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            return Err(TimelineError::NoEventsInRange);
        }

        Ok(results)
    }

    /// Find nearest event before timestamp
    pub fn nearest_before(&self, timestamp: i64) -> Option<(i64, &(TimelineEvent, Modality))> {
        self.events.range(..timestamp)
            .next_back()
            .map(|(ts, events)| (*ts, events.first().unwrap()))
    }

    /// Find nearest event after timestamp
    pub fn nearest_after(&self, timestamp: i64) -> Option<(i64, &(TimelineEvent, Modality))> {
        self.events.range(timestamp + 1..)
            .next()
            .map(|(ts, events)| (*ts, events.first().unwrap()))
    }

    /// Get available modalities in timeline
    pub fn available_modalities(&self) -> Vec<Modality> {
        self.modality_index.keys().copied().collect()
    }

    /// Get timeline bounds
    pub fn time_range(&self) -> Option<(i64, i64)> {
        match (self.start_time, self.end_time) {
            (Some(s), Some(e)) => Some((s, e)),
            _ => None,
        }
    }

    /// Get total event count
    pub fn event_count(&self) -> usize {
        self.events.values().map(|v| v.len()).sum()
    }

    /// Get event count for modality
    pub fn modality_count(&self, modality: Modality) -> usize {
        self.modality_index.get(&modality)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Time slice: a window of events around a center time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlice {
    pub center_time: i64,
    pub window: i64,
    pub events: Vec<(i64, TimelineEvent, Modality)>,
}

impl TimeSlice {
    pub fn ros_events(&self) -> Vec<(i64, &TimelineEvent)> {
        self.events.iter()
            .filter_map(|(ts, event, _modality)| {
                if matches!(event, TimelineEvent::RosEvent { .. }) {
                    Some((*ts, event))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn video_frames(&self) -> Vec<(i64, &TimelineEvent)> {
        self.events.iter()
            .filter_map(|(ts, event, _modality)| {
                if matches!(event, TimelineEvent::VideoFrame { .. }) {
                    Some((*ts, event))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn log_entries(&self) -> Vec<(i64, &TimelineEvent)> {
        self.events.iter()
            .filter_map(|(ts, event, _modality)| {
                if matches!(event, TimelineEvent::LogEntry { .. }) {
                    Some((*ts, event))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn annotations(&self) -> Vec<(i64, &TimelineEvent)> {
        self.events.iter()
            .filter_map(|(ts, event, _modality)| {
                if matches!(event, TimelineEvent::Annotation { .. }) {
                    Some((*ts, event))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Query specification for timeline access
pub struct TimeSliceQuery {
    pub time: i64,
    pub window: i64,
    pub modalities: Option<Vec<Modality>>,
}

impl TimeSliceQuery {
    pub fn new(time: i64, window: i64) -> Self {
        TimeSliceQuery {
            time,
            window,
            modalities: None,
        }
    }

    pub fn with_modalities(mut self, modalities: Vec<Modality>) -> Self {
        self.modalities = Some(modalities);
        self
    }
}

/// Event index for fast lookups by topic/sensor
pub struct EventIndex {
    topic_index: HashMap<String, Vec<i64>>,
    sensor_index: HashMap<String, Vec<i64>>,
}

impl EventIndex {
    pub fn new() -> Self {
        EventIndex {
            topic_index: HashMap::new(),
            sensor_index: HashMap::new(),
        }
    }

    pub fn index_topic(&mut self, topic: String, timestamp: i64) {
        self.topic_index.entry(topic)
            .or_insert_with(Vec::new)
            .push(timestamp);
    }

    pub fn index_sensor(&mut self, sensor_id: String, timestamp: i64) {
        self.sensor_index.entry(sensor_id)
            .or_insert_with(Vec::new)
            .push(timestamp);
    }

    pub fn get_topic_events(&self, topic: &str) -> Option<&[i64]> {
        self.topic_index.get(topic).map(|v| v.as_slice())
    }

    pub fn get_sensor_events(&self, sensor_id: &str) -> Option<&[i64]> {
        self.sensor_index.get(sensor_id).map(|v| v.as_slice())
    }
}

impl Default for EventIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_creation() {
        let timeline = Timeline::new();
        assert_eq!(timeline.event_count(), 0);
    }

    #[test]
    fn test_timeline_insert() {
        let mut timeline = Timeline::new();
        let event = TimelineEvent::LogEntry {
            source: "test".to_string(),
            level: LogLevel::Info,
            message: "test message".to_string(),
        };

        timeline.insert(1000, event, Modality::LinuxLogs);
        assert_eq!(timeline.event_count(), 1);
    }

    #[test]
    fn test_timeline_time_range() {
        let mut timeline = Timeline::new();
        let event = TimelineEvent::LogEntry {
            source: "test".to_string(),
            level: LogLevel::Info,
            message: "test".to_string(),
        };

        timeline.insert(1000, event.clone(), Modality::LinuxLogs);
        timeline.insert(2000, event, Modality::LinuxLogs);

        let (start, end) = timeline.time_range().unwrap();
        assert_eq!(start, 1000);
        assert_eq!(end, 2000);
    }

    #[test]
    fn test_timeline_get_at() {
        let mut timeline = Timeline::new();
        let event = TimelineEvent::LogEntry {
            source: "test".to_string(),
            level: LogLevel::Info,
            message: "test".to_string(),
        };

        timeline.insert(1000, event, Modality::LinuxLogs);
        let events = timeline.get_at(1000).unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_query_modality() {
        let mut timeline = Timeline::new();
        let event = TimelineEvent::LogEntry {
            source: "test".to_string(),
            level: LogLevel::Info,
            message: "test".to_string(),
        };

        timeline.insert(1000, event, Modality::LinuxLogs);
        let results = timeline.query_modality(Modality::LinuxLogs, 0, 2000).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_nearest_neighbors() {
        let mut timeline = Timeline::new();
        let event = TimelineEvent::LogEntry {
            source: "test".to_string(),
            level: LogLevel::Info,
            message: "test".to_string(),
        };

        timeline.insert(1000, event.clone(), Modality::LinuxLogs);
        timeline.insert(3000, event, Modality::LinuxLogs);

        let before = timeline.nearest_before(2000).unwrap();
        assert_eq!(before.0, 1000);

        let after = timeline.nearest_after(2000).unwrap();
        assert_eq!(after.0, 3000);
    }

    #[test]
    fn test_event_index() {
        let mut index = EventIndex::new();
        index.index_topic("/scan".to_string(), 1000);
        index.index_topic("/scan".to_string(), 1100);

        let events = index.get_topic_events("/scan").unwrap();
        assert_eq!(events.len(), 2);
    }
}
