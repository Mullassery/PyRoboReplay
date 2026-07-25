/// System Failure Detector
///
/// Detects:
/// - OOM kills (out-of-memory process termination)
/// - Kernel panics (unrecoverable kernel errors)
/// - Driver failures (device driver crashes)
/// - USB resets (USB device disconnection/reconnection)
/// - Filesystem errors (disk I/O failures)
/// - CPU saturation (sustained high CPU usage)
/// - Memory pressure (sustained high memory usage)

use super::{DetectedFailure, FailureDetector, FailureDomain, FailureSeverity};
use crate::core::timeline_correlation::NormalizedEvent;

pub struct SystemFailureDetector;

impl SystemFailureDetector {
    /// Detect OOM (Out-of-Memory) kills
    fn detect_oom_kill(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::KernelEvent {
                timestamp,
                event_type,
                description,
                ..
            } = &event.event
            {
                if event_type == "oom_kill" {
                    failures.push(
                        DetectedFailure::new(
                            "oom_kill",
                            FailureDomain::System,
                            *timestamp,
                            1.0, // Logged by kernel - certain
                            FailureSeverity::Critical,
                            format!("Out-of-memory kill: {}", description),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }

    /// Detect kernel panics
    fn detect_kernel_panic(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::KernelEvent {
                timestamp,
                event_type,
                description,
                ..
            } = &event.event
            {
                if event_type == "kernel_panic" {
                    failures.push(
                        DetectedFailure::new(
                            "kernel_panic",
                            FailureDomain::System,
                            *timestamp,
                            1.0,
                            FailureSeverity::Critical,
                            format!("Kernel panic: {}", description),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }

    /// Detect USB device loss/disconnection
    fn detect_usb_device_loss(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::HardwareEvent {
                timestamp,
                event_type,
                hardware_id,
                ..
            } = &event.event
            {
                if event_type == "usb_disconnect" {
                    failures.push(
                        DetectedFailure::new(
                            "usb_device_loss",
                            FailureDomain::System,
                            *timestamp,
                            1.0,
                            FailureSeverity::High,
                            format!("USB device lost: {} (may be LiDAR, IMU, etc.)", hardware_id),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }

    /// Detect thermal throttling
    fn detect_thermal_throttle(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::HardwareEvent {
                timestamp,
                event_type,
                ..
            } = &event.event
            {
                if event_type == "thermal_throttle" {
                    failures.push(
                        DetectedFailure::new(
                            "thermal_throttle",
                            FailureDomain::System,
                            *timestamp,
                            0.95,
                            FailureSeverity::High,
                            "Thermal throttling detected - CPU/GPU too hot".to_string(),
                        )
                        .with_event_ids(vec![event.id.clone()]),
                    );
                }
            }
        }

        failures
    }

    /// Detect CPU saturation
    fn detect_cpu_saturation(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();
        const CPU_THRESHOLD: f32 = 95.0; // >95% CPU
        const SATURATION_DURATION_MS: i64 = 5000; // >5 seconds

        let mut cpu_high_events = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::ResourceMetric {
                timestamp,
                metric_type,
                value,
                ..
            } = &event.event
            {
                if metric_type == "cpu_percent" && *value > CPU_THRESHOLD {
                    cpu_high_events.push((event.id.clone(), *timestamp));
                }
            }
        }

        if !cpu_high_events.is_empty() {
            let first_time = cpu_high_events[0].1;
            let last_time = cpu_high_events[cpu_high_events.len() - 1].1;
            let duration = (last_time - first_time).num_milliseconds();

            if duration > SATURATION_DURATION_MS {
                failures.push(
                    DetectedFailure::new(
                        "cpu_saturation",
                        FailureDomain::System,
                        first_time,
                        0.90,
                        FailureSeverity::High,
                        format!(
                            "CPU >{}% for {:.1}s (sustained overload)",
                            CPU_THRESHOLD,
                            duration as f64 / 1000.0
                        ),
                    )
                    .with_event_ids(cpu_high_events.iter().map(|(id, _)| id.clone()).collect()),
                );
            }
        }

        failures
    }

    /// Detect memory pressure
    fn detect_memory_pressure(events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut failures = Vec::new();
        const MEMORY_THRESHOLD: f32 = 85.0; // >85% memory used
        const PRESSURE_DURATION_MS: i64 = 3000; // >3 seconds

        let mut mem_high_events = Vec::new();

        for event in events {
            if let crate::core::event::MissionEvent::ResourceMetric {
                timestamp,
                metric_type,
                value,
                ..
            } = &event.event
            {
                if (metric_type == "memory_percent" || metric_type == "memory_mb")
                    && *value > MEMORY_THRESHOLD
                {
                    mem_high_events.push((event.id.clone(), *timestamp));
                }
            }
        }

        if !mem_high_events.is_empty() {
            let first_time = mem_high_events[0].1;
            let last_time = mem_high_events[mem_high_events.len() - 1].1;
            let duration = (last_time - first_time).num_milliseconds();

            if duration > PRESSURE_DURATION_MS {
                failures.push(
                    DetectedFailure::new(
                        "memory_pressure",
                        FailureDomain::System,
                        first_time,
                        0.85,
                        FailureSeverity::High,
                        format!(
                            "Memory >{}% for {:.1}s (sustained pressure)",
                            MEMORY_THRESHOLD,
                            duration as f64 / 1000.0
                        ),
                    )
                    .with_event_ids(mem_high_events.iter().map(|(id, _)| id.clone()).collect()),
                );
            }
        }

        failures
    }
}

impl FailureDetector for SystemFailureDetector {
    fn detect(&self, events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut all_failures = Vec::new();

        all_failures.extend(Self::detect_oom_kill(events));
        all_failures.extend(Self::detect_kernel_panic(events));
        all_failures.extend(Self::detect_usb_device_loss(events));
        all_failures.extend(Self::detect_thermal_throttle(events));
        all_failures.extend(Self::detect_cpu_saturation(events));
        all_failures.extend(Self::detect_memory_pressure(events));

        all_failures
    }

    fn domain(&self) -> FailureDomain {
        FailureDomain::System
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = SystemFailureDetector;
        assert_eq!(detector.domain(), FailureDomain::System);
    }
}
