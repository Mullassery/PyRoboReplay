pub mod adapters;
pub mod core;
pub mod cli;

use pyo3::prelude::*;
use core::event::{MissionEvent, MissionRecord};
use core::Timeline;
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

#[pymodule]
fn pyroboreplay(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Mission>()?;
    m.add_class::<Event>()?;

    // Module docstring
    m.setattr(
        "__doc__",
        "PyRoboReplay: Time-travel debugger for robot fleets\n\n\
         Load and replay missions from ROS 2 bag files with sensor filtering.",
    )?;

    Ok(())
}
