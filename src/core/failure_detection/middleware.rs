/// Middleware Failure Detector
///
/// Detects:
/// - DDS discovery timeout (nodes can't find each other)
/// - QoS mismatch (incompatible pub/sub settings)
/// - Topic starvation (subscription receiving no messages)
/// - Message latency spikes
/// - DDS buffer overflow

use super::{DetectedFailure, FailureDetector, FailureDomain, FailureSeverity};
use crate::core::timeline_correlation::NormalizedEvent;

pub struct MiddlewareFailureDetector;

impl MiddlewareFailureDetector {
    /// Detect DDS discovery timeout
    fn detect_discovery_timeout(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::CommunicationEvent {
                timestamp,
                event_type,
                ..
            } = &event.event
            {
                if event_type.contains("discovery_timeout") || event_type.contains("discovery_failed") {
                    failures.push(
                        DetectedFailure::new(
                            "dds_discovery_timeout",
                            FailureDomain::Middleware,
                            *timestamp,
                            1.0,
                            FailureSeverity::Critical,
                            "DDS discovery timeout: nodes unable to find each other".to_string(),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }

            if let crate::core::event::MissionEvent::DDSMetric {
                timestamp,
                event_type,
                severity,
                ..
            } = &event.event
            {
                if event_type.contains("discovery") {
                    let sev = if severity == "critical" {
                        FailureSeverity::Critical
                    } else {
                        FailureSeverity::High
                    };

                    failures.push(
                        DetectedFailure::new(
                            "dds_discovery_timeout",
                            FailureDomain::Middleware,
                            *timestamp,
                            0.95,
                            sev,
                            "DDS discovery issue detected".to_string(),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }

    /// Detect QoS mismatch between publishers and subscribers
    fn detect_qos_mismatch(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::CommunicationEvent {
                timestamp,
                event_type,
                data,
                ..
            } = &event.event
            {
                if event_type.contains("qos_incompatible") || event_type.contains("qos_mismatch") {
                    let reason = data
                        .as_ref()
                        .and_then(|d| d.get("reason"))
                        .and_then(|r| r.as_str())
                        .unwrap_or("Incompatible QoS");

                    failures.push(
                        DetectedFailure::new(
                            "dds_qos_mismatch",
                            FailureDomain::Middleware,
                            *timestamp,
                            0.95,
                            FailureSeverity::Medium,
                            format!("DDS QoS mismatch: {}", reason),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }

            if let crate::core::event::MissionEvent::DDSMetric {
                timestamp,
                event_type,
                ..
            } = &event.event
            {
                if event_type.contains("qos") {
                    failures.push(
                        DetectedFailure::new(
                            "dds_qos_mismatch",
                            FailureDomain::Middleware,
                            *timestamp,
                            0.90,
                            FailureSeverity::Medium,
                            "DDS QoS violation detected".to_string(),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }

    /// Detect topic starvation: subscription receiving no messages
    fn detect_topic_starvation(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();
        let mut topic_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for event in events {
            if let crate::core::event::MissionEvent::CommunicationEvent {
                event_type,
                data,
                ..
            } = &event.event
            {
                if let Some(topic) = data.as_ref().and_then(|d| d.get("topic")) {
                    if let Some(topic_str) = topic.as_str() {
                        *topic_counts.entry(topic_str.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Topics with 0-2 messages are suspicious (should have more over the mission)
        for (topic, count) in topic_counts {
            if count < 3 {
                if let Some(event) = events.first() {
                    failures.push(
                        DetectedFailure::new(
                            "topic_starvation",
                            FailureDomain::Middleware,
                            event.timestamp,
                            0.80,
                            FailureSeverity::High,
                            format!("Topic {} starved: only {} messages", topic, count),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }

    /// Detect message latency spikes
    fn detect_latency_spike(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();
        const LATENCY_SPIKE_MS: i64 = 500; // 500ms spike is concerning

        for event in events {
            if let crate::core::event::MissionEvent::CommunicationEvent {
                timestamp,
                event_type,
                data,
                ..
            } = &event.event
            {
                if event_type.contains("latency") {
                    if let Some(latency_val) = data.as_ref().and_then(|d| d.get("latency_ms")) {
                        if let Some(latency_ms) = latency_val.as_i64() {
                            if latency_ms > LATENCY_SPIKE_MS {
                                failures.push(
                                    DetectedFailure::new(
                                        "latency_spike",
                                        FailureDomain::Middleware,
                                        *timestamp,
                                        0.85,
                                        FailureSeverity::Medium,
                                        format!("DDS latency spike: {}ms", latency_ms),
                                    )
                                    .with_event_ids(vec![event.id.clone()]),
                                );
                            }
                        }
                    }
                }
            }
        }

        failures
    }
}

impl FailureDetector for MiddlewareFailureDetector {
    fn detect(&self, events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut all_failures = Vec::new();

        all_failures.extend(Self::detect_discovery_timeout(events));
        all_failures.extend(Self::detect_qos_mismatch(events));
        all_failures.extend(Self::detect_topic_starvation(events));
        all_failures.extend(Self::detect_latency_spike(events));

        all_failures
    }

    fn domain(&self) -> FailureDomain {
        FailureDomain::Middleware
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = MiddlewareFailureDetector;
        assert_eq!(detector.domain(), FailureDomain::Middleware);
    }
}
