pub mod adapters;
pub mod core;
pub mod cli;
pub mod storage;
pub mod streaming;

use pyo3::prelude::*;
use core::event::{MissionEvent, MissionRecord};
use core::Timeline;
use core::anomaly_detector::{AnomalyDetector, Failure as RustFailure};
use core::root_cause::{RootCauseAnalysis as RustRootCauseAnalysis, RootCauseHypothesis as RustHypothesis};
use adapters::ros2::Ros2Adapter;
use adapters::MissionAdapter;

/// Python wrapper for a Mission
#[pyclass]
pub struct Mission {
    inner: MissionRecord,
    timeline: Timeline,
}

#[pymethods]
impl Mission {
    /// Load a mission from a ROS 2 bag file
    #[staticmethod]
    pub fn from_ros_bag(path: &str) -> PyResult<Mission> {
        let adapter = Ros2Adapter::new();
        let inner = adapter
            .read(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        let mut timeline = Timeline::new();
        timeline.add_mission(inner.clone());

        Ok(Mission { inner, timeline })
    }

    /// Get the mission ID
    pub fn mission_id(&self) -> String {
        self.inner.id.to_string()
    }

    /// Get the mission name
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get total event count
    pub fn event_count(&self) -> usize {
        self.inner.event_count()
    }

    /// Get mission duration in seconds
    pub fn duration_seconds(&self) -> Option<i64> {
        self.inner.duration().map(|d| d.num_seconds())
    }

    /// Get all available sensor types
    pub fn get_available_sensors(&self) -> PyResult<Vec<String>> {
        self.timeline
            .get_available_sensors(&self.inner.id.to_string())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get sensor frames of a specific type
    ///
    /// Args:
    ///     sensor_type (str): Type of sensor ("lidar", "camera", "imu", "odometry", "costmap")
    ///
    /// Returns:
    ///     List of Event objects for that sensor type
    pub fn get_sensor_frames(&self, sensor_type: &str) -> PyResult<Vec<Event>> {
        let events = self
            .timeline
            .get_sensor_frames(&self.inner.id.to_string(), sensor_type)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(events.iter().map(|e| Event::from_event((*e).clone())).collect())
    }

    /// Get multiple sensor types at once
    pub fn get_multi_sensor_frames(&self, sensor_types: Vec<String>) -> PyResult<Vec<Event>> {
        let sensor_refs: Vec<&str> = sensor_types.iter().map(|s| s.as_str()).collect();
        let events = self
            .timeline
            .get_multi_sensor_frames(&self.inner.id.to_string(), &sensor_refs)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(events.iter().map(|e| Event::from_event((*e).clone())).collect())
    }

    /// Get all events at a specific timestamp
    pub fn get_events_at_timestamp(&self, timestamp_iso8601: &str) -> PyResult<Vec<Event>> {
        let ts = chrono::DateTime::parse_from_rfc3339(timestamp_iso8601)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
            .with_timezone(&chrono::Utc);

        let events = self
            .timeline
            .get_events_at_timestamp(&self.inner.id.to_string(), ts)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(events.iter().map(|e| Event::from_event((*e).clone())).collect())
    }

    /// Get event counts by type
    pub fn get_event_counts(&self) -> PyResult<Vec<(String, usize)>> {
        let mut counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for event in &self.inner.events {
            let key = event.event_type().to_string();
            *counts.entry(key).or_insert(0) += 1;
        }

        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by_key(|k| std::cmp::Reverse(k.1));
        Ok(result)
    }

    /// Get all events
    pub fn get_all_events(&self) -> Vec<Event> {
        self.inner
            .events
            .iter()
            .map(|e| Event::from_event(e.clone()))
            .collect()
    }

    /// Export mission to JSON
    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
    }

    /// Detect all failures and anomalies in the mission
    ///
    /// Returns a list of failures detected across 8 categories:
    /// - near_collision: LiDAR obstacle too close
    /// - localization_loss: Odometry covariance spike
    /// - navigation_deadlock: Oscillating path with no progress
    /// - perception_failure: Low detection confidence
    /// - communication_loss: Message dropout
    /// - sensor_dropout: Sensor stopped reporting
    /// - oscillation: Back-and-forth movement pattern
    /// - costmap_anomaly: Sudden changes in obstacle map
    pub fn detect_failures(&self) -> PyResult<Vec<Failure>> {
        let detector = AnomalyDetector::new(self.inner.events.clone());
        let rust_failures = detector.detect_all();

        Ok(rust_failures
            .into_iter()
            .map(|f| Failure::from_rust_failure(f))
            .collect())
    }

    /// Analyze root causes for a specific failure timestamp
    ///
    /// Args:
    ///     timestamp (float): Unix timestamp in seconds
    ///
    /// Returns:
    ///     RootCauseAnalysis with primary hypothesis and ranked alternatives
    ///
    /// Example:
    ///     failures = mission.detect_failures()
    ///     first_failure = failures[0]
    ///     analysis = mission.analyze_failure(first_failure.get_timestamp())
    ///     print(analysis.get_primary_hypothesis())
    pub fn analyze_failure(&self, timestamp: f64) -> PyResult<RootCauseAnalysis> {
        use core::root_cause::RootCauseAnalyzer;
        use core::causality::CausalGraphBuilder;

        // Find failure event at or near this timestamp
        let secs = timestamp.floor() as i64;
        let nanos = ((timestamp - secs as f64) * 1_000_000_000.0) as u32;
        let target_timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(secs, nanos)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid timestamp"))?;

        // Find the closest event to this timestamp
        let failure_idx = self
            .inner
            .events
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| {
                let diff = e.timestamp() - target_timestamp;
                let abs_diff = if diff.num_seconds() < 0 { -diff } else { diff };
                abs_diff.num_milliseconds()
            })
            .map(|(idx, _)| idx);

        let failure_idx = match failure_idx {
            Some(idx) => idx,
            None => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "No events found near timestamp",
                ))
            }
        };

        // Build causal graph
        let mut builder = CausalGraphBuilder::new(self.inner.events.clone());
        let graph = builder.build();

        // Analyze failure
        let mut analyzer = RootCauseAnalyzer::new(self.inner.events.clone());
        analyzer = analyzer.with_causal_graph(graph);

        let analysis_opt = analyzer.analyze_failure(failure_idx);
        let rust_analysis = match analysis_opt {
            Some(a) => a,
            None => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Could not analyze failure at this timestamp",
                ))
            }
        };

        Ok(RootCauseAnalysis::from_rust_analysis(rust_analysis))
    }
}

/// Python wrapper for an Event
#[pyclass]
pub struct Event {
    event_type: String,
    timestamp: String,
    robot_id: Option<String>,
    sensor_type: Option<String>,
}

impl Event {
    fn from_event(event: MissionEvent) -> Self {
        let timestamp = event.timestamp().to_rfc3339();
        let robot_id = event.robot_id().map(|s| s.to_string());
        let sensor_type = event.sensor_type().map(|s| s.to_string());
        let event_type = event.event_type().to_string();

        Event {
            event_type,
            timestamp,
            robot_id,
            sensor_type,
        }
    }
}

#[pymethods]
impl Event {
    /// Get event type (e.g., "lidar_scan", "camera_frame", "imu_data")
    pub fn get_event_type(&self) -> String {
        self.event_type.clone()
    }

    /// Get timestamp in ISO 8601 format
    pub fn get_timestamp(&self) -> String {
        self.timestamp.clone()
    }

    /// Get robot ID (if applicable)
    pub fn get_robot_id(&self) -> Option<String> {
        self.robot_id.clone()
    }

    /// Get sensor type (if applicable)
    pub fn get_sensor_type(&self) -> Option<String> {
        self.sensor_type.clone()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Event(type='{}', timestamp='{}', robot={:?}, sensor={:?})",
            self.event_type, self.timestamp, self.robot_id, self.sensor_type
        )
    }
}

/// Python wrapper for a detected failure/anomaly
#[pyclass]
pub struct Failure {
    id: String,
    failure_type: String,
    timestamp: f64,
    confidence: f32,
    severity: String,
    description: String,
    affected_systems: Vec<String>,
    evidence: std::collections::HashMap<String, String>,
}

impl Failure {
    fn from_rust_failure(rust_failure: RustFailure) -> Self {
        Failure {
            id: rust_failure.id,
            failure_type: rust_failure.failure_type,
            timestamp: rust_failure.timestamp_seconds,
            confidence: rust_failure.confidence,
            severity: rust_failure.severity,
            description: rust_failure.description,
            affected_systems: rust_failure.affected_systems,
            evidence: rust_failure.evidence,
        }
    }
}

#[pymethods]
impl Failure {
    /// Get failure ID
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    /// Get failure type (e.g., "near_collision", "localization_loss")
    pub fn get_failure_type(&self) -> String {
        self.failure_type.clone()
    }

    /// Get timestamp when failure was detected
    pub fn get_timestamp(&self) -> f64 {
        self.timestamp
    }

    /// Get confidence score (0.0-1.0)
    pub fn get_confidence(&self) -> f32 {
        self.confidence
    }

    /// Get severity level (critical, high, medium, low)
    pub fn get_severity(&self) -> String {
        self.severity.clone()
    }

    /// Get human-readable description
    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    /// Get list of affected systems
    pub fn get_affected_systems(&self) -> Vec<String> {
        self.affected_systems.clone()
    }

    /// Get evidence that triggered this failure detection
    pub fn get_evidence(&self) -> std::collections::HashMap<String, String> {
        self.evidence.clone()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Failure(type='{}', timestamp={:.2}, confidence={:.2}, severity='{}')",
            self.failure_type, self.timestamp, self.confidence, self.severity
        )
    }
}

/// Python wrapper for a root cause hypothesis
#[pyclass]
#[derive(Clone)]
pub struct Hypothesis {
    description: String,
    confidence: f32,
    causal_chain: Vec<String>,
}

#[pymethods]
impl Hypothesis {
    /// Get hypothesis description
    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    /// Get confidence score (0.0-1.0)
    pub fn get_confidence(&self) -> f32 {
        self.confidence
    }

    /// Get causal chain (event sequence)
    pub fn get_causal_chain(&self) -> Vec<String> {
        self.causal_chain.clone()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Hypothesis(confidence={:.2}, chain_length={})",
            self.confidence,
            self.causal_chain.len()
        )
    }
}

/// Python wrapper for root cause analysis result
#[pyclass]
pub struct RootCauseAnalysis {
    primary_hypothesis: String,
    hypotheses: Vec<Hypothesis>,
    diagnostic_confidence: f32,
}

impl RootCauseAnalysis {
    fn from_rust_analysis(rust_analysis: RustRootCauseAnalysis) -> Self {
        let primary_hypothesis = rust_analysis
            .most_likely_cause
            .as_ref()
            .map(|h| h.explanation.clone())
            .unwrap_or_else(|| "Unknown cause".to_string());

        let hypotheses = rust_analysis
            .hypotheses
            .iter()
            .map(|h| Hypothesis {
                description: h.explanation.clone(),
                confidence: h.confidence,
                causal_chain: h
                    .causal_chain
                    .iter()
                    .map(|idx| format!("event_{}", idx))
                    .collect(),
            })
            .collect();

        RootCauseAnalysis {
            primary_hypothesis,
            hypotheses,
            diagnostic_confidence: rust_analysis.diagnostic_confidence,
        }
    }
}

#[pymethods]
impl RootCauseAnalysis {
    /// Get primary (most likely) hypothesis
    pub fn get_primary_hypothesis(&self) -> String {
        self.primary_hypothesis.clone()
    }

    /// Get all ranked hypotheses
    pub fn get_hypotheses(&self) -> Vec<Hypothesis> {
        self.hypotheses.clone()
    }

    /// Get diagnostic confidence (0.0-1.0)
    pub fn get_diagnostic_confidence(&self) -> f32 {
        self.diagnostic_confidence
    }

    pub fn __repr__(&self) -> String {
        format!(
            "RootCauseAnalysis(confidence={:.2}, hypotheses={})",
            self.diagnostic_confidence,
            self.hypotheses.len()
        )
    }
}

#[pymodule]
fn pyroboreplay(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Mission>()?;
    m.add_class::<Event>()?;
    m.add_class::<Failure>()?;
    m.add_class::<Hypothesis>()?;
    m.add_class::<RootCauseAnalysis>()?;

    // Module docstring
    m.setattr(
        "__doc__",
        "PyRoboReplay: Time-travel debugger for robot fleets\n\n\
         Load and replay missions from ROS 2 bag files with sensor filtering.",
    )?;

    Ok(())
}
