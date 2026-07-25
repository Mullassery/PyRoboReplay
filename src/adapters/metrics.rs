/// Adapter for parsing resource metrics (Layer 3)
///
/// Supports:
/// - CSV time-series data (CPU, memory, disk, temperature)
/// - JSON metrics (DDS telemetry, network stats)
///
/// Normalizes to MissionEvent::ResourceMetric and MissionEvent::NetworkEvent

use crate::core::event::MissionEvent;
use crate::adapters::AdapterError;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    CpuPercent,
    MemoryMb,
    DiskPercent,
    TemperatureCelsius,
    NetworkRxBytes,
    NetworkTxBytes,
    NetworkRxPackets,
    NetworkTxPackets,
}

impl MetricType {
    pub fn as_str(&self) -> &str {
        match self {
            MetricType::CpuPercent => "cpu_percent",
            MetricType::MemoryMb => "memory_mb",
            MetricType::DiskPercent => "disk_percent",
            MetricType::TemperatureCelsius => "temp_celsius",
            MetricType::NetworkRxBytes => "network_rx_bytes",
            MetricType::NetworkTxBytes => "network_tx_bytes",
            MetricType::NetworkRxPackets => "network_rx_packets",
            MetricType::NetworkTxPackets => "network_tx_packets",
        }
    }

    pub fn from_filename(filename: &str) -> Option<Self> {
        match filename.to_lowercase().as_str() {
            s if s.contains("cpu") => Some(MetricType::CpuPercent),
            s if s.contains("memory") || s.contains("mem") => Some(MetricType::MemoryMb),
            s if s.contains("disk") => Some(MetricType::DiskPercent),
            s if s.contains("thermal") || s.contains("temp") => Some(MetricType::TemperatureCelsius),
            s if s.contains("network") && s.contains("rx") => Some(MetricType::NetworkRxBytes),
            s if s.contains("network") && s.contains("tx") => Some(MetricType::NetworkTxBytes),
            _ => None,
        }
    }

    pub fn unit(&self) -> &str {
        match self {
            MetricType::CpuPercent | MetricType::DiskPercent => "%",
            MetricType::MemoryMb => "MB",
            MetricType::TemperatureCelsius => "°C",
            MetricType::NetworkRxBytes | MetricType::NetworkTxBytes => "bytes",
            MetricType::NetworkRxPackets | MetricType::NetworkTxPackets => "packets",
        }
    }
}

/// Parser for resource metrics
pub struct MetricsAdapter;

impl MetricsAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Parse CSV metrics file (time-series data)
    ///
    /// Expected format:
    /// timestamp,value
    /// 2024-07-25T14:22:00Z,45.5
    /// 2024-07-25T14:22:01Z,46.2
    pub fn parse_csv(&self, content: &str, metric_type: MetricType) -> Result<Vec<MissionEvent>, AdapterError> {
        let mut events = Vec::new();
        let mut line_count = 0;

        for line in content.lines() {
            line_count += 1;

            // Skip header line
            if line_count == 1 && (line.contains("timestamp") || line.contains("time")) {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() < 2 {
                continue;
            }

            let timestamp_str = parts[0];
            let value_str = parts[1];

            // Parse timestamp
            let timestamp = self.parse_iso_timestamp(timestamp_str)
                .or_else(|| self.parse_unix_timestamp(timestamp_str))
                .unwrap_or_else(Utc::now);

            // Parse value
            let value: f32 = match value_str.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Determine threshold based on metric type
            let threshold = match metric_type {
                MetricType::CpuPercent => Some(80.0),
                MetricType::MemoryMb => None,
                MetricType::DiskPercent => Some(85.0),
                MetricType::TemperatureCelsius => Some(80.0),
                _ => None,
            };

            events.push(MissionEvent::ResourceMetric {
                timestamp,
                robot_id: None,
                metric_type: metric_type.as_str().to_string(),
                value,
                unit: metric_type.unit().to_string(),
                threshold,
                metadata: None,
            });
        }

        Ok(events)
    }

    /// Parse JSON metrics (DDS telemetry, network stats)
    pub fn parse_json(&self, content: &str) -> Result<Vec<MissionEvent>, AdapterError> {
        let mut events = Vec::new();

        // Try to parse as JSON array of metric objects
        match serde_json::from_str::<Vec<serde_json::Value>>(content) {
            Ok(array) => {
                for obj in array {
                    if let Some(event) = self.json_to_event(&obj) {
                        events.push(event);
                    }
                }
            }
            Err(_) => {
                // Try as single JSON object
                match serde_json::from_str::<serde_json::Value>(content) {
                    Ok(obj) => {
                        if let Some(event) = self.json_to_event(&obj) {
                            events.push(event);
                        }
                    }
                    Err(e) => {
                        return Err(AdapterError::ParseError(format!(
                            "Invalid JSON format: {}",
                            e
                        )));
                    }
                }
            }
        }

        Ok(events)
    }

    /// Convert JSON object to MissionEvent
    fn json_to_event(&self, obj: &serde_json::Value) -> Option<MissionEvent> {
        let obj = obj.as_object()?;

        // Try to parse as DDS metric
        if let (Some(timestamp_val), Some(event_type_val)) =
            (obj.get("timestamp"), obj.get("event_type"))
        {
            let timestamp_str = timestamp_val.as_str()?;
            let event_type = event_type_val.as_str()?;

            let timestamp = self.parse_iso_timestamp(timestamp_str)
                .or_else(|| self.parse_unix_timestamp(timestamp_str))
                .unwrap_or_else(Utc::now);

            return Some(MissionEvent::DDSMetric {
                timestamp,
                event_type: event_type.to_string(),
                participant_id: obj
                    .get("participant_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                severity: obj
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("medium")
                    .to_string(),
                details: Some(serde_json::to_value(obj).unwrap_or(serde_json::json!({}))),
            });
        }

        // Try to parse as network metric
        if let Some(timestamp_val) = obj.get("timestamp") {
            let timestamp_str = timestamp_val.as_str()?;
            let timestamp = self.parse_iso_timestamp(timestamp_str)
                .or_else(|| self.parse_unix_timestamp(timestamp_str))
                .unwrap_or_else(Utc::now);

            if let (Some(rx_val), Some(tx_val)) = (obj.get("rx_bytes"), obj.get("tx_bytes")) {
                let rx_bytes = rx_val.as_u64();
                let tx_bytes = tx_val.as_u64();

                return Some(MissionEvent::NetworkEvent {
                    timestamp,
                    event_type: "interface_stats".to_string(),
                    interface: obj
                        .get("interface")
                        .and_then(|v| v.as_str())
                        .unwrap_or("eth0")
                        .to_string(),
                    severity: "low".to_string(),
                    rx_packets: obj.get("rx_packets").and_then(|v| v.as_u64()),
                    tx_packets: obj.get("tx_packets").and_then(|v| v.as_u64()),
                    rx_bytes,
                    tx_bytes,
                    details: None,
                });
            }
        }

        None
    }

    /// Parse ISO 8601 timestamp
    fn parse_iso_timestamp(&self, timestamp_str: &str) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(timestamp_str)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }

    /// Parse Unix timestamp (seconds or milliseconds)
    fn parse_unix_timestamp(&self, timestamp_str: &str) -> Option<DateTime<Utc>> {
        // Try parsing as seconds
        if let Ok(seconds) = timestamp_str.parse::<i64>() {
            return Some(DateTime::<Utc>::from_timestamp(seconds, 0)?);
        }

        // Try parsing as milliseconds
        if let Ok(millis) = timestamp_str.parse::<i64>() {
            if millis > 10_000_000_000 {
                let seconds = millis / 1000;
                let nanos = ((millis % 1000) * 1_000_000) as u32;
                return Some(DateTime::<Utc>::from_timestamp(seconds, nanos)?);
            }
        }

        None
    }

    /// Detect metric type from filename
    pub fn detect_metric_type(&self, filename: &str) -> Option<MetricType> {
        MetricType::from_filename(filename)
    }
}

impl Default for MetricsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = MetricsAdapter::new();
        assert!(adapter.parse_csv("", MetricType::CpuPercent).is_ok());
    }

    #[test]
    fn test_metric_type_unit() {
        assert_eq!(MetricType::CpuPercent.unit(), "%");
        assert_eq!(MetricType::MemoryMb.unit(), "MB");
        assert_eq!(MetricType::TemperatureCelsius.unit(), "°C");
    }

    #[test]
    fn test_parse_csv_metrics() {
        let adapter = MetricsAdapter::new();
        let csv_content = "timestamp,value\n2024-07-25T14:22:00Z,45.5\n2024-07-25T14:22:01Z,46.2";

        let events = adapter.parse_csv(csv_content, MetricType::CpuPercent).unwrap();
        assert_eq!(events.len(), 2);

        // Verify first event
        if let MissionEvent::ResourceMetric { value, .. } = &events[0] {
            assert_eq!(*value, 45.5);
        }
    }

    #[test]
    fn test_detect_metric_from_filename() {
        let adapter = MetricsAdapter::new();
        assert_eq!(adapter.detect_metric_type("cpu.csv"), Some(MetricType::CpuPercent));
        assert_eq!(adapter.detect_metric_type("memory.csv"), Some(MetricType::MemoryMb));
        assert_eq!(adapter.detect_metric_type("temperature.csv"), Some(MetricType::TemperatureCelsius));
    }

    #[test]
    fn test_parse_iso_timestamp() {
        let adapter = MetricsAdapter::new();
        let ts = adapter.parse_iso_timestamp("2024-07-25T14:22:00Z");
        assert!(ts.is_some());
    }
}
