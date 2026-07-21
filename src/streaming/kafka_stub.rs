use crate::streaming::channel::StreamEvent;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub trait KafkaConnector: Send + Sync {
    fn connect(&mut self, brokers: &str, topic: &str) -> Result<(), String>;
    fn publish_event(&self, event: &StreamEvent) -> Result<(), String>;
    fn subscribe(&mut self, group_id: &str) -> Result<(), String>;
    fn poll_event(&self, timeout_ms: u64) -> Result<Option<StreamEvent>, String>;
    fn disconnect(&self) -> Result<(), String>;
}

pub struct KafkaStub {
    events: Arc<Mutex<VecDeque<StreamEvent>>>,
    connected: bool,
}

impl KafkaStub {
    pub fn new() -> Self {
        KafkaStub {
            events: Arc::new(Mutex::new(VecDeque::new())),
            connected: false,
        }
    }
}

impl Default for KafkaStub {
    fn default() -> Self {
        Self::new()
    }
}

impl KafkaConnector for KafkaStub {
    fn connect(&mut self, _brokers: &str, _topic: &str) -> Result<(), String> {
        self.connected = true;
        Ok(())
    }

    fn publish_event(&self, event: &StreamEvent) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        self.events.lock().unwrap().push_back(event.clone());
        Ok(())
    }

    fn subscribe(&mut self, _group_id: &str) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        Ok(())
    }

    fn poll_event(&self, _timeout_ms: u64) -> Result<Option<StreamEvent>, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        Ok(self.events.lock().unwrap().pop_front())
    }

    fn disconnect(&self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_kafka_stub_connect_and_publish() {
        let mut stub = KafkaStub::new();
        assert!(stub.connect("localhost:9092", "events").is_ok());

        let event = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "sensor".to_string(),
            timestamp: Utc::now(),
            robot_id: None,
            payload: serde_json::json!({}),
            sequence_number: 0,
        };

        assert!(stub.publish_event(&event).is_ok());
    }

    #[test]
    fn test_kafka_stub_poll_returns_published() {
        let mut stub = KafkaStub::new();
        stub.connect("localhost:9092", "events").unwrap();

        let event = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "sensor".to_string(),
            timestamp: Utc::now(),
            robot_id: None,
            payload: serde_json::json!({}),
            sequence_number: 0,
        };

        stub.publish_event(&event).unwrap();

        match stub.poll_event(1000) {
            Ok(Some(polled_event)) => {
                assert_eq!(polled_event.event_id, "event_1");
            }
            _ => panic!("Expected to poll the published event"),
        }
    }
}
