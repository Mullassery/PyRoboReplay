use crate::core::event::{MissionEvent, MissionRecord};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimelineError {
    #[error("Mission not found")]
    MissionNotFound,
    #[error("Event index out of bounds")]
    IndexOutOfBounds,
    #[error("Invalid time range")]
    InvalidTimeRange,
}

pub struct Timeline {
    missions: HashMap<String, MissionRecord>,
    cursor_positions: HashMap<String, usize>,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            missions: HashMap::new(),
            cursor_positions: HashMap::new(),
        }
    }

    pub fn add_mission(&mut self, mission: MissionRecord) {
        self.missions.insert(mission.id.to_string(), mission);
    }

    pub fn get_mission(&self, mission_id: &str) -> Result<&MissionRecord, TimelineError> {
        self.missions
            .get(mission_id)
            .ok_or(TimelineError::MissionNotFound)
    }

    pub fn get_events_by_type(
        &self,
        mission_id: &str,
        event_type: &str,
    ) -> Result<Vec<&MissionEvent>, TimelineError> {
        let mission = self.get_mission(mission_id)?;
        Ok(mission
            .events
            .iter()
            .filter(|e| e.event_type() == event_type)
            .collect())
    }

    pub fn get_events_in_time_range(
        &self,
        mission_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<&MissionEvent>, TimelineError> {
        if start > end {
            return Err(TimelineError::InvalidTimeRange);
        }

        let mission = self.get_mission(mission_id)?;
        Ok(mission
            .events
            .iter()
            .filter(|e| {
                let ts = e.timestamp();
                ts >= start && ts <= end
            })
            .collect())
    }

    pub fn get_events_by_robot(
        &self,
        mission_id: &str,
        robot_id: &str,
    ) -> Result<Vec<&MissionEvent>, TimelineError> {
        let mission = self.get_mission(mission_id)?;
        Ok(mission
            .events
            .iter()
            .filter(|e| e.robot_id().map_or(false, |rid| rid == robot_id))
            .collect())
    }

    pub fn jump_to_event(
        &mut self,
        mission_id: &str,
        event_index: usize,
    ) -> Result<&MissionEvent, TimelineError> {
        let mission = self
            .missions
            .get(mission_id)
            .ok_or(TimelineError::MissionNotFound)?;

        if event_index >= mission.events.len() {
            return Err(TimelineError::IndexOutOfBounds);
        }

        self.cursor_positions.insert(mission_id.to_string(), event_index);
        Ok(&mission.events[event_index])
    }

    pub fn next_event(&mut self, mission_id: &str) -> Result<Option<&MissionEvent>, TimelineError> {
        let mission = self
            .missions
            .get(mission_id)
            .ok_or(TimelineError::MissionNotFound)?;

        let current_pos = *self.cursor_positions.get(mission_id).unwrap_or(&0);
        if current_pos + 1 < mission.events.len() {
            let next_pos = current_pos + 1;
            self.cursor_positions.insert(mission_id.to_string(), next_pos);
            Ok(Some(&mission.events[next_pos]))
        } else {
            Ok(None)
        }
    }

    pub fn previous_event(
        &mut self,
        mission_id: &str,
    ) -> Result<Option<&MissionEvent>, TimelineError> {
        let current_pos = *self.cursor_positions.get(mission_id).unwrap_or(&0);
        if current_pos > 0 {
            let prev_pos = current_pos - 1;
            self.cursor_positions.insert(mission_id.to_string(), prev_pos);
            let mission = self
                .missions
                .get(mission_id)
                .ok_or(TimelineError::MissionNotFound)?;
            Ok(Some(&mission.events[prev_pos]))
        } else {
            Ok(None)
        }
    }

    pub fn current_event(&self, mission_id: &str) -> Result<Option<&MissionEvent>, TimelineError> {
        let mission = self
            .missions
            .get(mission_id)
            .ok_or(TimelineError::MissionNotFound)?;

        let current_pos = *self.cursor_positions.get(mission_id).unwrap_or(&0);
        if current_pos < mission.events.len() {
            Ok(Some(&mission.events[current_pos]))
        } else {
            Ok(None)
        }
    }

    pub fn mission_count(&self) -> usize {
        self.missions.len()
    }

    /// Get all events of a specific sensor type (lidar, camera, imu, etc.)
    pub fn get_sensor_frames(
        &self,
        mission_id: &str,
        sensor_type: &str,
    ) -> Result<Vec<&MissionEvent>, TimelineError> {
        let mission = self.get_mission(mission_id)?;
        Ok(mission
            .events
            .iter()
            .filter(|e| e.sensor_type().map_or(false, |st| st == sensor_type))
            .collect())
    }

    /// Get events matching multiple sensor types (e.g., lidar and camera)
    pub fn get_multi_sensor_frames(
        &self,
        mission_id: &str,
        sensor_types: &[&str],
    ) -> Result<Vec<&MissionEvent>, TimelineError> {
        let mission = self.get_mission(mission_id)?;
        Ok(mission
            .events
            .iter()
            .filter(|e| {
                e.sensor_type()
                    .map_or(false, |st| sensor_types.contains(&st))
            })
            .collect())
    }

    /// Get all available sensor types in a mission
    pub fn get_available_sensors(&self, mission_id: &str) -> Result<Vec<String>, TimelineError> {
        let mission = self.get_mission(mission_id)?;
        let mut sensors = std::collections::HashSet::new();

        for event in &mission.events {
            if let Some(sensor) = event.sensor_type() {
                sensors.insert(sensor.to_string());
            }
        }

        let mut sensor_list: Vec<String> = sensors.into_iter().collect();
        sensor_list.sort();
        Ok(sensor_list)
    }

    /// Get all events at a specific timestamp (holistic view across all sensors)
    pub fn get_events_at_timestamp(
        &self,
        mission_id: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<&MissionEvent>, TimelineError> {
        let mission = self.get_mission(mission_id)?;
        Ok(mission
            .events
            .iter()
            .filter(|e| e.timestamp() == timestamp)
            .collect())
    }

    /// Get sensor frames within a time window (e.g., all lidar frames in range [t1, t2])
    pub fn get_sensor_frames_in_range(
        &self,
        mission_id: &str,
        sensor_type: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<&MissionEvent>, TimelineError> {
        if start > end {
            return Err(TimelineError::InvalidTimeRange);
        }

        let mission = self.get_mission(mission_id)?;
        Ok(mission
            .events
            .iter()
            .filter(|e| {
                let ts = e.timestamp();
                let is_sensor = e.sensor_type().map_or(false, |st| st == sensor_type);
                is_sensor && ts >= start && ts <= end
            })
            .collect())
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::Pose;

    fn create_test_mission() -> MissionRecord {
        let mut mission = MissionRecord::new("test_mission");
        let now = Utc::now();

        mission.add_event(MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            pose: Pose {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95),
        });

        mission.add_event(MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: now + chrono::Duration::seconds(1),
            pose: Pose {
                x: 1.0,
                y: 1.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.90),
        });

        mission
    }

    #[test]
    fn test_timeline_creation() {
        let timeline = Timeline::new();
        assert_eq!(timeline.mission_count(), 0);
    }

    #[test]
    fn test_add_mission() {
        let mut timeline = Timeline::new();
        let mission = create_test_mission();
        let mission_id = mission.id.to_string();
        timeline.add_mission(mission);
        assert_eq!(timeline.mission_count(), 1);
        assert!(timeline.get_mission(&mission_id).is_ok());
    }

    #[test]
    fn test_get_events_by_type() {
        let mut timeline = Timeline::new();
        let mission = create_test_mission();
        let mission_id = mission.id.to_string();
        timeline.add_mission(mission);

        let events = timeline.get_events_by_type(&mission_id, "robot_pose").unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_next_previous_event() {
        let mut timeline = Timeline::new();
        let mission = create_test_mission();
        let mission_id = mission.id.to_string();
        timeline.add_mission(mission);

        timeline.jump_to_event(&mission_id, 0).unwrap();
        assert!(timeline.next_event(&mission_id).is_ok());
        assert!(timeline.previous_event(&mission_id).is_ok());
    }

    #[test]
    fn test_get_sensor_frames() {
        let mut timeline = Timeline::new();
        let mut mission = MissionRecord::new("sensor_test");
        let now = Utc::now();

        // Add lidar frames
        mission.add_event(MissionEvent::LidarScan {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            data: crate::core::event::LidarData {
                ranges: vec![1.0, 2.0],
                intensities: None,
                frame_id: "laser".to_string(),
                min_angle: -3.14,
                max_angle: 3.14,
                angle_increment: 0.01,
                range_min: 0.0,
                range_max: 30.0,
            },
        });

        mission.add_event(MissionEvent::LidarScan {
            robot_id: "robot_1".to_string(),
            timestamp: now + chrono::Duration::seconds(1),
            data: crate::core::event::LidarData {
                ranges: vec![1.5, 2.5],
                intensities: None,
                frame_id: "laser".to_string(),
                min_angle: -3.14,
                max_angle: 3.14,
                angle_increment: 0.01,
                range_min: 0.0,
                range_max: 30.0,
            },
        });

        let mission_id = mission.id.to_string();
        timeline.add_mission(mission);

        // Query sensor frames
        let lidar_frames = timeline.get_sensor_frames(&mission_id, "lidar").unwrap();
        assert_eq!(lidar_frames.len(), 2);
    }

    #[test]
    fn test_get_available_sensors() {
        let mut timeline = Timeline::new();
        let mut mission = MissionRecord::new("multi_sensor");
        let now = Utc::now();

        // Add different sensor types
        mission.add_event(MissionEvent::LidarScan {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            data: crate::core::event::LidarData {
                ranges: vec![],
                intensities: None,
                frame_id: "laser".to_string(),
                min_angle: -3.14,
                max_angle: 3.14,
                angle_increment: 0.01,
                range_min: 0.0,
                range_max: 30.0,
            },
        });

        mission.add_event(MissionEvent::CameraFrame {
            robot_id: "robot_1".to_string(),
            timestamp: now,
            data: crate::core::event::CameraFrame {
                sensor_id: "camera_1".to_string(),
                frame_id: "camera".to_string(),
                width: 640,
                height: 480,
                encoding: "rgb8".to_string(),
                image_data: vec![],
                camera_info: None,
            },
        });

        let mission_id = mission.id.to_string();
        timeline.add_mission(mission);

        // Get available sensors
        let sensors = timeline.get_available_sensors(&mission_id).unwrap();
        assert!(sensors.contains(&"lidar".to_string()));
        assert!(sensors.contains(&"camera".to_string()));
    }

    #[test]
    fn test_independent_cursors_two_missions() {
        let mut timeline = Timeline::new();
        let mission1 = create_test_mission();
        let mission2 = create_test_mission();
        let m1_id = mission1.id.to_string();
        let m2_id = mission2.id.to_string();

        timeline.add_mission(mission1);
        timeline.add_mission(mission2);

        timeline.jump_to_event(&m1_id, 0).unwrap();
        timeline.jump_to_event(&m2_id, 1).unwrap();

        assert_eq!(timeline.current_event(&m1_id).unwrap().unwrap().timestamp(),
                   timeline.get_mission(&m1_id).unwrap().events[0].timestamp());
        assert_eq!(timeline.current_event(&m2_id).unwrap().unwrap().timestamp(),
                   timeline.get_mission(&m2_id).unwrap().events[1].timestamp());
    }

    #[test]
    fn test_cursor_resets_per_mission() {
        let mut timeline = Timeline::new();
        let mission1 = create_test_mission();
        let mission2 = create_test_mission();
        let m1_id = mission1.id.to_string();
        let m2_id = mission2.id.to_string();

        timeline.add_mission(mission1);
        timeline.add_mission(mission2);

        timeline.jump_to_event(&m1_id, 1).unwrap();
        timeline.jump_to_event(&m2_id, 0).unwrap();
        timeline.jump_to_event(&m1_id, 0).unwrap();

        let evt1 = timeline.current_event(&m1_id).unwrap().unwrap();
        assert_eq!(evt1.timestamp(), timeline.get_mission(&m1_id).unwrap().events[0].timestamp());
    }
}
