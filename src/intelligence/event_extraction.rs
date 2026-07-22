//! Event Extraction: Structured Event Stream Generation
//!
//! Converts raw sensor streams into structured events that can be
//! processed by AI agents and monitoring systems.

use serde::{Deserialize, Serialize};

/// Structured event from mission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredEvent {
    pub timestamp_sec: f32,
    pub event_type: String,
    pub entity_type: Option<String>,
    pub confidence: f32,
    pub payload: std::collections::HashMap<String, String>,
}

/// Stream of events
#[derive(Debug, Clone)]
pub struct EventStream {
    pub mission_id: String,
    pub events: Vec<StructuredEvent>,
}

/// Event extraction engine
pub struct EventExtractor;

impl EventExtractor {
    /// Extract structured events from scenes
    pub fn extract_events(
        scenes: &[(f32, crate::intelligence::scene_reconstruction::RetrospectiveScene)],
    ) -> EventStream {
        let mut events = Vec::new();

        for (timestamp, scene) in scenes {
            for obj in &scene.detected_objects {
                if obj.confidence > 0.7 {
                    let mut payload = std::collections::HashMap::new();
                    payload.insert("entity".to_string(), obj.entity_type.clone());
                    if let Some(dist) = obj.distance_m {
                        payload.insert("distance_m".to_string(), format!("{:.1}", dist));
                    }

                    events.push(StructuredEvent {
                        timestamp_sec: *timestamp,
                        event_type: "object_detected".to_string(),
                        entity_type: Some(obj.entity_type.clone()),
                        confidence: obj.confidence,
                        payload,
                    });
                }
            }
        }

        EventStream {
            mission_id: "mission_x".to_string(),
            events,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_extraction() {
        let events = vec![];
        let stream = EventExtractor::extract_events(&events);
        assert_eq!(stream.events.len(), 0);
    }
}
