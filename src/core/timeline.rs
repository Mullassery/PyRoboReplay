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
    current_position: usize,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            missions: HashMap::new(),
            current_position: 0,
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

        self.current_position = event_index;
        Ok(&mission.events[event_index])
    }

    pub fn next_event(&mut self, mission_id: &str) -> Result<Option<&MissionEvent>, TimelineError> {
        let mission = self
            .missions
            .get(mission_id)
            .ok_or(TimelineError::MissionNotFound)?;

        if self.current_position + 1 < mission.events.len() {
            self.current_position += 1;
            Ok(Some(&mission.events[self.current_position]))
        } else {
            Ok(None)
        }
    }

    pub fn previous_event(
        &mut self,
        mission_id: &str,
    ) -> Result<Option<&MissionEvent>, TimelineError> {
        if self.current_position > 0 {
            self.current_position -= 1;
            let mission = self
                .missions
                .get(mission_id)
                .ok_or(TimelineError::MissionNotFound)?;
            Ok(Some(&mission.events[self.current_position]))
        } else {
            Ok(None)
        }
    }

    pub fn current_event(&self, mission_id: &str) -> Result<Option<&MissionEvent>, TimelineError> {
        let mission = self
            .missions
            .get(mission_id)
            .ok_or(TimelineError::MissionNotFound)?;

        if self.current_position < mission.events.len() {
            Ok(Some(&mission.events[self.current_position]))
        } else {
            Ok(None)
        }
    }

    pub fn mission_count(&self) -> usize {
        self.missions.len()
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
}
