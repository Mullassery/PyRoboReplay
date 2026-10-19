//! MCAP export for the Rust CLI.
//!
//! The Python package also exposes MCAP/Rerun export; this keeps the `export`
//! subcommand usable from the native binary without a Python runtime.

use crate::core::anomaly_detector::AnomalyDetector;
use crate::core::event::MissionRecord;
use mcap::{records::MessageHeader, Writer};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs::File;
use std::path::Path;

fn to_ns(timestamp: chrono::DateTime<chrono::Utc>) -> u64 {
    timestamp
        .timestamp_nanos_opt()
        .map(|ns| ns.max(0) as u64)
        .unwrap_or(0)
}

/// Write mission events and detected failures to an MCAP file.
pub fn export_mcap(mission: &MissionRecord, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();
    let file = File::create(path)?;
    let mut writer = Writer::new(file)?;

    let schema = json!({
        "type": "object",
        "properties": {
            "event_type": {"type": "string"},
            "robot_id": {"type": ["string", "null"]},
            "sensor_type": {"type": ["string", "null"]}
        }
    });
    let schema_id = writer.add_schema(
        "pyroboreplay.Event",
        "jsonschema",
        schema.to_string().as_bytes(),
    )?;

    let mut channels: HashMap<String, u16> = HashMap::new();
    let metadata = BTreeMap::new();
    for event in &mission.events {
        let sensor = event.sensor_type().unwrap_or("other");
        let topic = format!("/events/{sensor}");
        let channel_id = if let Some(id) = channels.get(&topic) {
            *id
        } else {
            let id = writer.add_channel(schema_id, &topic, "json", &metadata)?;
            channels.insert(topic.clone(), id);
            id
        };
        let payload = json!({
            "event_type": event.event_type(),
            "robot_id": event.robot_id(),
            "sensor_type": event.sensor_type(),
        });
        let timestamp = to_ns(event.timestamp());
        writer.write_to_known_channel(
            &MessageHeader {
                channel_id,
                sequence: 0,
                log_time: timestamp,
                publish_time: timestamp,
            },
            payload.to_string().as_bytes(),
        )?;
    }

    let failures_channel = channels
        .get("/failures")
        .copied()
        .unwrap_or(writer.add_channel(schema_id, "/failures", "json", &metadata)?);
    for failure in AnomalyDetector::new(mission.events.clone()).detect_all() {
        let payload = json!({
            "failure_type": failure.failure_type,
            "severity": failure.severity,
            "description": failure.description,
            "confidence": failure.confidence,
        });
        let timestamp = to_ns(failure.timestamp);
        writer.write_to_known_channel(
            &MessageHeader {
                channel_id: failures_channel,
                sequence: 0,
                log_time: timestamp,
                publish_time: timestamp,
            },
            payload.to_string().as_bytes(),
        )?;
    }

    writer.finish()?;
    Ok(())
}
