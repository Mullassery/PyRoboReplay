use crate::streaming::channel::StreamEvent;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type EventFilter = Box<dyn Fn(&StreamEvent) -> bool + Send + Sync>;

pub struct ProcessorConfig {
    pub mission_id_filter: Option<String>,
    pub event_type_filter: Option<Vec<String>>,
    pub robot_id_filter: Option<String>,
    pub max_events: Option<usize>,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        ProcessorConfig {
            mission_id_filter: None,
            event_type_filter: None,
            robot_id_filter: None,
            max_events: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub event_count: usize,
    pub event_types: HashMap<String, usize>,
    pub robots_seen: Vec<String>,
    pub events_per_second: f64,
}

pub struct StreamProcessor {
    config: ProcessorConfig,
}

impl StreamProcessor {
    pub fn new(config: ProcessorConfig) -> Self {
        StreamProcessor { config }
    }

    pub fn process_event(&self, event: &StreamEvent) -> bool {
        if let Some(ref mission_filter) = self.config.mission_id_filter {
            if event.mission_id != *mission_filter {
                return false;
            }
        }

        if let Some(ref event_types) = self.config.event_type_filter {
            if !event_types.contains(&event.event_type) {
                return false;
            }
        }

        if let Some(ref robot_filter) = self.config.robot_id_filter {
            if let Some(ref robot_id) = event.robot_id {
                if robot_id != robot_filter {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn aggregate_window(&self, events: &[StreamEvent], window_ms: u64) -> Vec<AggregationResult> {
        if events.is_empty() {
            return Vec::new();
        }

        let window_duration = Duration::milliseconds(window_ms as i64);
        let mut windows: Vec<AggregationResult> = Vec::new();

        let mut current_window_start = events[0].timestamp;
        let mut current_window_end = current_window_start + window_duration;
        let mut window_events: Vec<&StreamEvent> = Vec::new();

        for event in events {
            if event.timestamp > current_window_end {
                if !window_events.is_empty() {
                    windows.push(Self::create_aggregation_result(
                        current_window_start,
                        current_window_end,
                        &window_events,
                    ));
                }

                current_window_start = event.timestamp;
                current_window_end = current_window_start + window_duration;
                window_events.clear();
            }

            if self.process_event(event) {
                window_events.push(event);
            }
        }

        if !window_events.is_empty() {
            windows.push(Self::create_aggregation_result(
                current_window_start,
                current_window_end,
                &window_events,
            ));
        }

        windows
    }

    fn create_aggregation_result(
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
        events: &[&StreamEvent],
    ) -> AggregationResult {
        let mut event_types: HashMap<String, usize> = HashMap::new();
        let mut robots_set: std::collections::HashSet<String> = std::collections::HashSet::new();

        for event in events {
            *event_types.entry(event.event_type.clone()).or_insert(0) += 1;
            if let Some(ref robot_id) = event.robot_id {
                robots_set.insert(robot_id.clone());
            }
        }

        let duration = window_end - window_start;
        let duration_secs = duration.num_milliseconds() as f64 / 1000.0;
        let events_per_second = if duration_secs > 0.0 {
            events.len() as f64 / duration_secs
        } else {
            0.0
        };

        let mut robots_seen: Vec<String> = robots_set.into_iter().collect();
        robots_seen.sort();

        AggregationResult {
            window_start,
            window_end,
            event_count: events.len(),
            event_types,
            robots_seen,
            events_per_second,
        }
    }

    pub fn drain_filtered(
        &self,
        consumer: &crate::streaming::channel::EventStreamConsumer,
        max_wait_ms: u64,
    ) -> Vec<StreamEvent> {
        let mut events = Vec::new();
        let timeout = std::time::Duration::from_millis(max_wait_ms);

        loop {
            match consumer.recv_timeout(timeout) {
                Ok(event) => {
                    if self.process_event(&event) {
                        if let Some(max) = self.config.max_events {
                            if events.len() >= max {
                                break;
                            }
                        }
                        events.push(event);
                    }
                }
                Err(_) => break,
            }
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::channel::create_stream;

    #[test]
    fn test_processor_no_filter_passes_all() {
        let config = ProcessorConfig::default();
        let processor = StreamProcessor::new(config);

        let event = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "lidar_scan".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 0,
        };

        assert!(processor.process_event(&event));
    }

    #[test]
    fn test_processor_event_type_filter() {
        let config = ProcessorConfig {
            event_type_filter: Some(vec!["lidar_scan".to_string()]),
            ..Default::default()
        };
        let processor = StreamProcessor::new(config);

        let lidar_event = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "lidar_scan".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 0,
        };

        let camera_event = StreamEvent {
            event_id: "event_2".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "camera_frame".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 1,
        };

        assert!(processor.process_event(&lidar_event));
        assert!(!processor.process_event(&camera_event));
    }

    #[test]
    fn test_processor_robot_id_filter() {
        let config = ProcessorConfig {
            robot_id_filter: Some("robot_1".to_string()),
            ..Default::default()
        };
        let processor = StreamProcessor::new(config);

        let event1 = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "sensor".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 0,
        };

        let event2 = StreamEvent {
            event_id: "event_2".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "sensor".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("robot_2".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 1,
        };

        assert!(processor.process_event(&event1));
        assert!(!processor.process_event(&event2));
    }

    #[test]
    fn test_processor_mission_id_filter() {
        let config = ProcessorConfig {
            mission_id_filter: Some("mission_1".to_string()),
            ..Default::default()
        };
        let processor = StreamProcessor::new(config);

        let event1 = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "sensor".to_string(),
            timestamp: Utc::now(),
            robot_id: None,
            payload: serde_json::json!({}),
            sequence_number: 0,
        };

        let event2 = StreamEvent {
            event_id: "event_2".to_string(),
            mission_id: "mission_2".to_string(),
            event_type: "sensor".to_string(),
            timestamp: Utc::now(),
            robot_id: None,
            payload: serde_json::json!({}),
            sequence_number: 1,
        };

        assert!(processor.process_event(&event1));
        assert!(!processor.process_event(&event2));
    }

    #[test]
    fn test_aggregate_window_groups_correctly() {
        let config = ProcessorConfig::default();
        let processor = StreamProcessor::new(config);

        let now = Utc::now();
        let events = vec![
            StreamEvent {
                event_id: "event_1".to_string(),
                mission_id: "mission_1".to_string(),
                event_type: "lidar_scan".to_string(),
                timestamp: now,
                robot_id: Some("robot_1".to_string()),
                payload: serde_json::json!({}),
                sequence_number: 0,
            },
            StreamEvent {
                event_id: "event_2".to_string(),
                mission_id: "mission_1".to_string(),
                event_type: "camera_frame".to_string(),
                timestamp: now + Duration::milliseconds(500),
                robot_id: Some("robot_1".to_string()),
                payload: serde_json::json!({}),
                sequence_number: 1,
            },
            StreamEvent {
                event_id: "event_3".to_string(),
                mission_id: "mission_1".to_string(),
                event_type: "lidar_scan".to_string(),
                timestamp: now + Duration::milliseconds(2000),
                robot_id: Some("robot_1".to_string()),
                payload: serde_json::json!({}),
                sequence_number: 2,
            },
        ];

        let results = processor.aggregate_window(&events, 1000);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].event_count, 2);
        assert_eq!(results[1].event_count, 1);
    }

    #[test]
    fn test_drain_filtered_respects_max_events() {
        let config = ProcessorConfig {
            max_events: Some(2),
            ..Default::default()
        };
        let processor = StreamProcessor::new(config);

        let (producer, consumer) = create_stream(crate::streaming::StreamConfig::default());

        for i in 0..5 {
            let event = StreamEvent {
                event_id: format!("event_{}", i),
                mission_id: "mission_1".to_string(),
                event_type: "sensor".to_string(),
                timestamp: Utc::now(),
                robot_id: None,
                payload: serde_json::json!({}),
                sequence_number: 0,
            };
            producer.publish(event).ok();
        }

        let drained = processor.drain_filtered(&consumer, 100);
        assert_eq!(drained.len(), 2);
    }
}
