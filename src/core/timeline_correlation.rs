/// Timeline Correlation Engine for MLRIAS
///
/// Synchronizes events across 4 layers of evidence:
/// 1. Normalizes timestamps (clock sync, skew correction)
/// 2. Aligns multi-robot events to shared time reference
/// 3. Reconstructs causal chains with confidence scores
///
/// This is separate from the replay Timeline - focused on forensic analysis.

use crate::core::event::MissionEvent;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Represents an event with normalized timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedEvent {
    /// Unique event identifier
    pub id: String,

    /// Which layer this event came from (1-4)
    pub layer: u32,

    /// Normalized UTC timestamp
    pub timestamp: DateTime<Utc>,

    /// Confidence in timestamp accuracy (0.0-1.0)
    pub timestamp_confidence: f32,

    /// Original event data
    pub event: MissionEvent,

    /// Robot or host system that generated this
    pub origin: String,
}

/// Clock synchronization state for a robot/system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockSyncState {
    /// Estimated clock offset vs. reference (milliseconds)
    pub clock_offset_ms: i64,

    /// Clock skew rate (ppm - parts per million)
    pub clock_skew_ppm: i32,

    /// Confidence in clock sync (0.0-1.0)
    pub sync_confidence: f32,

    /// Last synchronization timestamp
    pub last_sync_at: DateTime<Utc>,

    /// Number of sync anchors used to compute this
    pub anchor_count: usize,
}

/// Causal link metadata for MLRIAS (uses existing CausalLink from causality.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLRIASCausalLink {
    /// ID of source event
    pub source_event_id: String,

    /// ID of target event
    pub target_event_id: String,

    /// Type of causal relationship (e.g., "sensor_triggers_detection")
    pub relationship_type: String,

    /// Confidence in the causal link (0.0-1.0)
    pub confidence: f32,

    /// Expected latency between events (milliseconds)
    pub expected_latency_ms: i64,

    /// Actual latency between events (milliseconds)
    pub actual_latency_ms: i64,
}

/// Timeline Correlation Engine - core of MLRIAS
pub struct TimelineCorrelationEngine {
    /// All normalized events from all layers
    pub unified_timeline: Vec<NormalizedEvent>,

    /// Clock sync state for each robot/system
    pub clock_sync: HashMap<String, ClockSyncState>,

    /// Temporal correlation window (default 2000ms)
    pub correlation_window_ms: i64,

    /// Reference robot for multi-robot alignment
    pub reference_robot: Option<String>,

    /// Causal links discovered between events
    pub causal_links: Vec<MLRIASCausalLink>,
}

impl TimelineCorrelationEngine {
    /// Create new correlation engine from raw events
    pub fn new(events: Vec<MissionEvent>) -> Self {
        // Convert raw events to normalized events (initial pass - no correction yet)
        let normalized: Vec<NormalizedEvent> = events
            .into_iter()
            .enumerate()
            .map(|(idx, event)| {
                let origin = event.robot_id().unwrap_or("system").to_string();
                NormalizedEvent {
                    id: format!("event_{}", idx),
                    layer: Self::detect_layer(&event),
                    timestamp: event.timestamp(),
                    timestamp_confidence: 1.0, // Will be adjusted after clock sync
                    event,
                    origin,
                }
            })
            .collect();

        Self {
            unified_timeline: normalized,
            clock_sync: HashMap::new(),
            correlation_window_ms: 2000, // 2-second temporal window
            reference_robot: None,
            causal_links: Vec::new(),
        }
    }

    /// Detect which layer an event came from
    fn detect_layer(event: &MissionEvent) -> u32 {
        match event {
            // Layer 1: ROS events
            MissionEvent::LidarScan { .. }
            | MissionEvent::CameraFrame { .. }
            | MissionEvent::IMUData { .. }
            | MissionEvent::OdometryUpdate { .. }
            | MissionEvent::CostmapUpdate { .. }
            | MissionEvent::RobotPose { .. }
            | MissionEvent::NavigationDecision { .. }
            | MissionEvent::ObstacleDetected { .. }
            | MissionEvent::CommunicationEvent { .. }
            | MissionEvent::CoordinationEvent { .. }
            | MissionEvent::EnvironmentalChange { .. }
            | MissionEvent::MissionLifecycle { .. } => 1,

            // Layer 2: Linux/Kernel events
            MissionEvent::KernelEvent { .. }
            | MissionEvent::LinuxLogEvent { .. }
            | MissionEvent::HardwareEvent { .. } => 2,

            // Layer 3: Resource metrics
            MissionEvent::ResourceMetric { .. }
            | MissionEvent::DDSMetric { .. }
            | MissionEvent::NetworkEvent { .. } => 3,

            // Layer 4: Configuration events
            MissionEvent::ConfigurationEvent { .. }
            | MissionEvent::ParameterValidationEvent { .. } => 4,
        }
    }

    /// Synchronize clocks across all robots/systems
    pub fn synchronize_clocks(&mut self) -> Result<(), String> {
        // Step 1: Find sync anchors (events with both ROS and system timestamps)
        let anchors = self.find_sync_anchors();

        if anchors.is_empty() {
            return Err("No clock sync anchors found".to_string());
        }

        // Step 2: Extract unique systems
        let systems: Vec<String> = self
            .unified_timeline
            .iter()
            .map(|e| e.origin.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Step 3: Compute clock state for each system
        for system in systems {
            let system_anchors: Vec<_> = anchors
                .iter()
                .filter(|(origin, _, _)| origin == &system)
                .collect();

            if !system_anchors.is_empty() {
                let sync_state = self.compute_clock_correction(&system, &system_anchors);
                self.clock_sync.insert(system, sync_state);
            }
        }

        // Step 4: Apply corrections to all events
        self.apply_clock_corrections();

        Ok(())
    }

    /// Find events that have both ROS and system timestamps (sync anchors)
    fn find_sync_anchors(&self) -> Vec<(String, DateTime<Utc>, DateTime<Utc>)> {
        let mut anchors = Vec::new();

        for event in &self.unified_timeline {
            // Look for MissionLifecycle start events (marked with system time)
            // or communication events that bridge layers
            match &event.event {
                MissionEvent::MissionLifecycle { timestamp, .. } => {
                    // These usually have explicit timestamps
                    anchors.push((event.origin.clone(), *timestamp, *timestamp));
                }
                MissionEvent::LinuxLogEvent { timestamp, .. } if event.layer == 2 => {
                    // System log entries are good anchors
                    anchors.push((event.origin.clone(), *timestamp, *timestamp));
                }
                _ => {}
            }
        }

        anchors
    }

    /// Compute clock offset and skew for a system
    fn compute_clock_correction(
        &self,
        _system: &str,
        anchors: &[&(String, DateTime<Utc>, DateTime<Utc>)],
    ) -> ClockSyncState {
        if anchors.is_empty() {
            return ClockSyncState {
                clock_offset_ms: 0,
                clock_skew_ppm: 0,
                sync_confidence: 0.0,
                last_sync_at: Utc::now(),
                anchor_count: 0,
            };
        }

        // Simple approach: use mean offset from anchors
        let offsets: Vec<i64> = anchors
            .iter()
            .map(|(_, ros_ts, sys_ts)| (*sys_ts - *ros_ts).num_milliseconds())
            .collect();

        let mean_offset = if !offsets.is_empty() {
            offsets.iter().sum::<i64>() / offsets.len() as i64
        } else {
            0
        };

        // Confidence based on anchor consistency
        let variance: f32 = if offsets.len() > 1 {
            let mean = mean_offset as f32;
            let sum_sq_diff: f32 = offsets
                .iter()
                .map(|&o| {
                    let diff = o as f32 - mean;
                    diff * diff
                })
                .sum();
            sum_sq_diff / offsets.len() as f32
        } else {
            0.0
        };

        let std_dev = variance.sqrt();
        let confidence_val: f32 = if std_dev > 0.0 {
            1.0 / (1.0 + std_dev / 100.0) // Higher variance → lower confidence
        } else {
            1.0
        };

        ClockSyncState {
            clock_offset_ms: mean_offset,
            clock_skew_ppm: 0, // Could be computed from multiple anchors over time
            sync_confidence: confidence_val.max(0.0).min(1.0),
            last_sync_at: Utc::now(),
            anchor_count: anchors.len(),
        }
    }

    /// Apply clock corrections to all events
    fn apply_clock_corrections(&mut self) {
        for event in &mut self.unified_timeline {
            if let Some(sync_state) = self.clock_sync.get(&event.origin) {
                // Apply offset correction
                let correction = Duration::milliseconds(sync_state.clock_offset_ms);
                event.timestamp = event.timestamp - correction;
                event.timestamp_confidence = sync_state.sync_confidence;
            }
        }
    }

    /// Select reference robot for multi-robot alignment
    pub fn select_reference_robot(&mut self) {
        let robots: Vec<String> = self
            .unified_timeline
            .iter()
            .filter_map(|e| e.event.robot_id().map(|s| s.to_string()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Choose robot with most events (likely most data)
        if let Some(reference) = robots
            .iter()
            .max_by_key(|r| {
                self.unified_timeline
                    .iter()
                    .filter(|e| e.origin == **r)
                    .count()
            })
            .cloned()
        {
            self.reference_robot = Some(reference);
        }
    }

    /// Build causal chains between events
    pub fn build_causal_chains(&mut self) {
        for i in 0..self.unified_timeline.len() {
            for j in i + 1..self.unified_timeline.len() {
                let event_a = &self.unified_timeline[i];
                let event_b = &self.unified_timeline[j];

                // Check temporal proximity
                let time_delta = event_b.timestamp - event_a.timestamp;
                if time_delta.num_milliseconds() > self.correlation_window_ms {
                    continue;
                }

                // Check semantic correlation
                if let Some(link) = self.infer_causal_link(event_a, event_b) {
                    self.causal_links.push(link);
                }
            }
        }
    }

    /// Infer causal link between two events
    fn infer_causal_link(&self, event_a: &NormalizedEvent, event_b: &NormalizedEvent) -> Option<MLRIASCausalLink> {
        use crate::core::event::MissionEvent::*;

        let latency_ms = (event_b.timestamp - event_a.timestamp).num_milliseconds();

        match (&event_a.event, &event_b.event) {
            // Layer 1: Sensor → Detection → Navigation
            (LidarScan { .. }, ObstacleDetected { .. }) => Some(MLRIASCausalLink {
                source_event_id: event_a.id.clone(),
                target_event_id: event_b.id.clone(),
                relationship_type: "sensor_triggers_detection".to_string(),
                confidence: 0.95,
                expected_latency_ms: 50,
                actual_latency_ms: latency_ms,
            }),

            (ObstacleDetected { .. }, NavigationDecision { .. }) => Some(MLRIASCausalLink {
                source_event_id: event_a.id.clone(),
                target_event_id: event_b.id.clone(),
                relationship_type: "detection_triggers_planning".to_string(),
                confidence: 0.85,
                expected_latency_ms: 100,
                actual_latency_ms: latency_ms,
            }),

            // Cross-layer: System event → ROS consequence
            (ResourceMetric { .. }, NavigationDecision { .. }) => Some(MLRIASCausalLink {
                source_event_id: event_a.id.clone(),
                target_event_id: event_b.id.clone(),
                relationship_type: "resource_affects_planning".to_string(),
                confidence: 0.60,
                expected_latency_ms: 200,
                actual_latency_ms: latency_ms,
            }),

            (KernelEvent { .. }, NavigationDecision { .. }) => Some(MLRIASCausalLink {
                source_event_id: event_a.id.clone(),
                target_event_id: event_b.id.clone(),
                relationship_type: "kernel_event_affects_ros".to_string(),
                confidence: 0.70,
                expected_latency_ms: 300,
                actual_latency_ms: latency_ms,
            }),

            _ => None,
        }
    }

    /// Get events within a time window
    pub fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&NormalizedEvent> {
        self.unified_timeline
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    /// Get events from a specific layer
    pub fn get_events_by_layer(&self, layer: u32) -> Vec<&NormalizedEvent> {
        self.unified_timeline
            .iter()
            .filter(|e| e.layer == layer)
            .collect()
    }

    /// Get causal chain leading to a specific event
    pub fn get_causal_chain_to(&self, event_id: &str) -> Vec<&MLRIASCausalLink> {
        self.causal_links
            .iter()
            .filter(|link| link.target_event_id == event_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let events = vec![];
        let engine = TimelineCorrelationEngine::new(events);
        assert_eq!(engine.unified_timeline.len(), 0);
    }

    #[test]
    fn test_layer_detection() {
        let event = MissionEvent::RobotPose {
            robot_id: "robot1".to_string(),
            timestamp: Utc::now(),
            pose: crate::core::event::Pose {
                x: 0.0, y: 0.0, z: 0.0,
                qx: 0.0, qy: 0.0, qz: 0.0, qw: 1.0,
            },
            confidence: None,
        };
        assert_eq!(TimelineCorrelationEngine::detect_layer(&event), 1);

        let event2 = MissionEvent::KernelEvent {
            timestamp: Utc::now(),
            event_type: "oom_kill".to_string(),
            severity: "critical".to_string(),
            description: "Out of memory".to_string(),
            source_file: None,
            process_id: None,
            process_name: None,
        };
        assert_eq!(TimelineCorrelationEngine::detect_layer(&event2), 2);
    }

    #[test]
    fn test_reference_robot_selection() {
        let mut engine = TimelineCorrelationEngine::new(vec![]);
        engine.select_reference_robot();
        // Should not panic even with empty timeline
        assert!(engine.reference_robot.is_none());
    }
}
