use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub event_id: String,
    pub mission_id: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub robot_id: Option<String>,
    pub payload: serde_json::Value,
    pub sequence_number: u64,
}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub buffer_size: usize,
    pub mission_id: String,
}

impl Default for StreamConfig {
    fn default() -> Self {
        StreamConfig {
            buffer_size: 1000,
            mission_id: "default_mission".to_string(),
        }
    }
}

pub struct EventStreamProducer {
    senders: Vec<SyncSender<StreamEvent>>,
    mission_id: String,
    sequence: Arc<AtomicU64>,
}

impl EventStreamProducer {
    pub fn publish(&self, mut event: StreamEvent) -> Result<(), String> {
        event.sequence_number = self.sequence.fetch_add(1, Ordering::SeqCst);

        let mut errors = Vec::new();
        for sender in &self.senders {
            if let Err(_) = sender.try_send(event.clone()) {
                errors.push("Channel full - backpressure triggered".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    pub fn mission_id(&self) -> &str {
        &self.mission_id
    }

    pub fn subscriber_count(&self) -> usize {
        self.senders.len()
    }
}

impl Clone for EventStreamProducer {
    fn clone(&self) -> Self {
        EventStreamProducer {
            senders: self.senders.clone(),
            mission_id: self.mission_id.clone(),
            sequence: Arc::clone(&self.sequence),
        }
    }
}

pub struct EventStreamConsumer {
    receiver: Receiver<StreamEvent>,
}

impl EventStreamConsumer {
    pub fn recv(&self) -> Result<StreamEvent, String> {
        self.receiver.recv().map_err(|_| "Channel closed".to_string())
    }

    pub fn try_recv(&self) -> Result<StreamEvent, String> {
        self.receiver.try_recv().map_err(|_| "No event available".to_string())
    }

    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Result<StreamEvent, String> {
        self.receiver
            .recv_timeout(timeout)
            .map_err(|_| "Timeout waiting for event".to_string())
    }
}

pub struct EventStream {
    producers: Vec<SyncSender<StreamEvent>>,
    mission_id: String,
    sequence: Arc<AtomicU64>,
}

impl EventStream {
    pub fn subscribe(&self) -> EventStreamConsumer {
        let (_tx, rx) = sync_channel(1);
        EventStreamConsumer { receiver: rx }
    }

    pub fn publisher(&self) -> EventStreamProducer {
        EventStreamProducer {
            senders: self.producers.clone(),
            mission_id: self.mission_id.clone(),
            sequence: Arc::clone(&self.sequence),
        }
    }

    pub fn mission_id(&self) -> &str {
        &self.mission_id
    }

    pub fn publisher_count(&self) -> usize {
        self.producers.len()
    }
}

pub fn create_stream(config: StreamConfig) -> (EventStreamProducer, EventStreamConsumer) {
    let (tx, rx) = sync_channel(config.buffer_size);

    let producer = EventStreamProducer {
        senders: vec![tx],
        mission_id: config.mission_id.clone(),
        sequence: Arc::new(AtomicU64::new(0)),
    };

    let consumer = EventStreamConsumer { receiver: rx };

    (producer, consumer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_stream() {
        let config = StreamConfig {
            buffer_size: 100,
            mission_id: "test_mission".to_string(),
        };
        let (producer, _consumer) = create_stream(config);
        assert_eq!(producer.mission_id(), "test_mission");
    }

    #[test]
    fn test_send_and_receive_event() {
        let config = StreamConfig::default();
        let (producer, consumer) = create_stream(config);

        let event = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "lidar_scan".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({"ranges": [1.0, 2.0, 3.0]}),
            sequence_number: 0,
        };

        producer.publish(event.clone()).unwrap();
        let received = consumer.recv().unwrap();

        assert_eq!(received.event_id, "event_1");
        assert_eq!(received.event_type, "lidar_scan");
    }

    #[test]
    fn test_sequence_numbers_increment() {
        let config = StreamConfig::default();
        let (producer, consumer) = create_stream(config);

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
            producer.publish(event).unwrap();
        }

        for i in 0..5 {
            let received = consumer.recv().unwrap();
            assert_eq!(received.sequence_number, i as u64);
        }
    }

    #[test]
    fn test_buffer_backpressure() {
        let config = StreamConfig {
            buffer_size: 2,
            mission_id: "mission_1".to_string(),
        };
        let (producer, _consumer) = create_stream(config);

        // Fill the buffer
        for i in 0..2 {
            let event = StreamEvent {
                event_id: format!("event_{}", i),
                mission_id: "mission_1".to_string(),
                event_type: "sensor".to_string(),
                timestamp: Utc::now(),
                robot_id: None,
                payload: serde_json::json!({}),
                sequence_number: 0,
            };
            producer.publish(event).unwrap();
        }

        // Third event should trigger backpressure
        let event = StreamEvent {
            event_id: "event_3".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "sensor".to_string(),
            timestamp: Utc::now(),
            robot_id: None,
            payload: serde_json::json!({}),
            sequence_number: 0,
        };
        match producer.publish(event) {
            Err(msg) => assert!(msg.contains("backpressure")),
            Ok(_) => panic!("Expected backpressure error"),
        }
    }

    #[test]
    fn test_producer_clone() {
        let config = StreamConfig::default();
        let (producer, consumer) = create_stream(config);
        let producer2 = producer.clone();

        let event = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "sensor".to_string(),
            timestamp: Utc::now(),
            robot_id: None,
            payload: serde_json::json!({}),
            sequence_number: 0,
        };

        producer2.publish(event).unwrap();
        let received = consumer.recv().unwrap();
        assert_eq!(received.event_id, "event_1");
    }

    #[test]
    fn test_stream_event_serialization() {
        let event = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_1".to_string(),
            event_type: "lidar_scan".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({"ranges": [1.0, 2.0, 3.0]}),
            sequence_number: 42,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: StreamEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.event_id, event.event_id);
        assert_eq!(deserialized.sequence_number, 42);
    }
}
