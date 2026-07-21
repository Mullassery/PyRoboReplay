/// JSON output format for AI-agent integration
/// All CLI commands can output structured JSON for programmatic parsing

use crate::core::event::MissionRecord;
use serde::{Serialize, Deserialize};
use chrono::Utc;

/// Mission analysis in JSON format
#[derive(Debug, Serialize, Deserialize)]
pub struct MissionAnalysisJson {
    pub mission_id: String,
    pub mission_name: String,
    pub duration_seconds: Option<i64>,
    pub total_events: usize,
    pub sensors: Vec<String>,
    pub event_breakdown: Vec<EventTypeCount>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventTypeCount {
    pub event_type: String,
    pub count: usize,
    pub percentage: f32,
}

/// Single event in JSON format (for queries)
#[derive(Debug, Serialize, Deserialize)]
pub struct EventJson {
    pub timestamp: String,
    pub event_type: String,
    pub robot_id: Option<String>,
    pub sensor_type: Option<String>,
}

/// Sensor frames query result
#[derive(Debug, Serialize, Deserialize)]
pub struct SensorFramesJson {
    pub sensor_type: String,
    pub frame_count: usize,
    pub frames: Vec<EventJson>,
}

/// Mission timeline export
#[derive(Debug, Serialize, Deserialize)]
pub struct MissionTimelineJson {
    pub mission_id: String,
    pub mission_name: String,
    pub events: Vec<EventJson>,
}

/// Sensor statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct SensorStatsJson {
    pub sensor_name: String,
    pub frame_count: usize,
    pub first_timestamp: String,
    pub last_timestamp: String,
    pub duration_seconds: Option<i64>,
    pub average_hz: f32,
}

impl MissionAnalysisJson {
    /// Create from MissionRecord
    pub fn from_mission(mission: &MissionRecord) -> Self {
        let mut event_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for event in &mission.events {
            let key = event.event_type().to_string();
            *event_counts.entry(key).or_insert(0) += 1;
        }

        let mut breakdown: Vec<_> = event_counts
            .into_iter()
            .map(|(event_type, count)| {
                let percentage = (count as f32 / mission.events.len() as f32) * 100.0;
                EventTypeCount {
                    event_type,
                    count,
                    percentage,
                }
            })
            .collect();

        breakdown.sort_by(|a, b| b.count.cmp(&a.count));

        let sensors: Vec<String> = mission
            .events
            .iter()
            .filter_map(|e| e.sensor_type())
            .map(|s| s.to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        Self {
            mission_id: mission.id.to_string(),
            mission_name: mission.name.clone(),
            duration_seconds: mission.duration().map(|d| d.num_seconds()),
            total_events: mission.events.len(),
            sensors,
            event_breakdown: breakdown,
            created_at: mission.created_at.to_rfc3339(),
        }
    }
}

impl EventJson {
    /// Create from a MissionEvent
    pub fn from_event(event: &crate::core::event::MissionEvent) -> Self {
        Self {
            timestamp: event.timestamp().to_rfc3339(),
            event_type: event.event_type().to_string(),
            robot_id: event.robot_id().map(|s| s.to_string()),
            sensor_type: event.sensor_type().map(|s| s.to_string()),
        }
    }
}

impl SensorFramesJson {
    /// Create from query results
    pub fn from_events(sensor_type: &str, events: Vec<&crate::core::event::MissionEvent>) -> Self {
        let frames = events.iter().map(|e| EventJson::from_event(e)).collect();
        Self {
            sensor_type: sensor_type.to_string(),
            frame_count: events.len(),
            frames,
        }
    }
}

impl MissionTimelineJson {
    /// Export entire mission as JSON timeline
    pub fn from_mission(mission: &MissionRecord) -> Self {
        let events = mission
            .events
            .iter()
            .map(|e| EventJson::from_event(e))
            .collect();

        Self {
            mission_id: mission.id.to_string(),
            mission_name: mission.name.clone(),
            events,
        }
    }
}

impl SensorStatsJson {
    /// Calculate statistics for a sensor type
    pub fn from_events(
        sensor_name: &str,
        events: &[&crate::core::event::MissionEvent],
    ) -> Option<Self> {
        if events.is_empty() {
            return None;
        }

        let first_timestamp = events.first()?.timestamp();
        let last_timestamp = events.last()?.timestamp();
        let duration = last_timestamp
            .signed_duration_since(first_timestamp)
            .num_seconds() as f32;

        let average_hz = if duration > 0.0 {
            events.len() as f32 / duration
        } else {
            0.0
        };

        Some(Self {
            sensor_name: sensor_name.to_string(),
            frame_count: events.len(),
            first_timestamp: first_timestamp.to_rfc3339(),
            last_timestamp: last_timestamp.to_rfc3339(),
            duration_seconds: if duration > 0.0 {
                Some(duration as i64)
            } else {
                None
            },
            average_hz,
        })
    }
}

/// Generic JSON response wrapper for all queries
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: String,
}

impl<T> JsonResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_analysis_json() {
        let mission = MissionRecord::new("test");
        let analysis = MissionAnalysisJson::from_mission(&mission);

        assert_eq!(analysis.mission_name, "test");
        assert_eq!(analysis.total_events, 0);
        assert!(analysis.duration_seconds.is_none());
    }

    #[test]
    fn test_json_response() {
        let response: JsonResponse<String> = JsonResponse::success("test data".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test data".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_response_error() {
        let response: JsonResponse<String> = JsonResponse::error("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }
}
