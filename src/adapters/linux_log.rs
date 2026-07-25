/// Adapter for parsing Linux/kernel logs (Layer 2)
///
/// Supports:
/// - journalctl output
/// - dmesg kernel messages
/// - syslog entries
///
/// Normalizes to MissionEvent::KernelEvent and MissionEvent::LinuxLogEvent

use crate::core::event::MissionEvent;
use crate::adapters::AdapterError;
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelEventSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl KernelEventSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            KernelEventSeverity::Critical => "critical",
            KernelEventSeverity::High => "high",
            KernelEventSeverity::Medium => "medium",
            KernelEventSeverity::Low => "low",
        }
    }

    pub fn from_log_level(level: &str) -> Self {
        match level.to_uppercase().as_str() {
            "EMERGENCY" | "ALERT" | "CRITICAL" => KernelEventSeverity::Critical,
            "ERROR" | "ERR" => KernelEventSeverity::High,
            "WARNING" | "WARN" => KernelEventSeverity::Medium,
            "NOTICE" | "INFO" => KernelEventSeverity::Low,
            _ => KernelEventSeverity::Low,
        }
    }
}

/// Parser for Linux/kernel logs
pub struct LinuxLogAdapter {
    journalctl_patterns: Vec<(Regex, &'static str)>,
}

impl LinuxLogAdapter {
    pub fn new() -> Self {
        Self {
            journalctl_patterns: vec![
                // OOM kill pattern: "systemd[1]: eviction_cgroup_cleanup[pid]: Killed"
                (
                    Regex::new(r"(?i)killed.*out.of.memory|oom.killer").unwrap(),
                    "oom_kill",
                ),
                // USB disconnect pattern
                (
                    Regex::new(r"(?i)usb.*(disconnect|removed|reset)").unwrap(),
                    "usb_disconnect",
                ),
                // Kernel panic pattern
                (
                    Regex::new(r"(?i)kernel.panic|panic.cpu:|oops:|segfault").unwrap(),
                    "kernel_panic",
                ),
                // Thermal throttle pattern
                (
                    Regex::new(r"(?i)thermal.throttle|cpu.throttle|overheat").unwrap(),
                    "thermal_throttle",
                ),
                // Filesystem error pattern
                (
                    Regex::new(r"(?i)filesystem.error|io.error|i/o.*error|FS.*error").unwrap(),
                    "filesystem_error",
                ),
                // Process crash pattern
                (
                    Regex::new(r"(?i)segmentation.fault|signal.11|coredump|core.dump").unwrap(),
                    "process_crash",
                ),
            ],
        }
    }

    /// Detect if a line matches any known kernel event pattern
    fn detect_kernel_event(&self, line: &str) -> Option<&'static str> {
        for (pattern, event_type) in &self.journalctl_patterns {
            if pattern.is_match(line) {
                return Some(event_type);
            }
        }
        None
    }

    /// Parse journalctl format
    pub fn parse_journalctl(&self, content: &str) -> Result<Vec<MissionEvent>, AdapterError> {
        let mut events = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse journalctl line format
            // Format: YYYY-MM-DD HH:MM:SS HOSTNAME UNIT[PID]: MESSAGE
            // Example: 2024-07-25 14:22:15 robot1 nav_stack[2341]: Planner timeout

            let parts: Vec<&str> = line.splitn(5, ' ').collect();
            if parts.len() < 5 {
                // Try alternative format
                if let Some(event) = self.parse_generic_log_line(line, "journalctl") {
                    events.push(event);
                }
                continue;
            }

            let date_str = parts[0];
            let time_str = parts[1];
            let hostname = parts[2];
            let unit_and_msg = parts[4];

            // Parse timestamp
            let timestamp_str = format!("{} {}", date_str, time_str);
            let timestamp = match NaiveDateTime::parse_from_str(&timestamp_str, "%Y-%m-%d %H:%M:%S")
            {
                Ok(naive) => DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc),
                Err(_) => {
                    // Try ISO format
                    match chrono::DateTime::parse_from_rfc3339(&format!("{}T{}Z", date_str, time_str)) {
                        Ok(dt) => dt.with_timezone(&Utc),
                        Err(_) => Utc::now(),
                    }
                }
            };

            // Extract unit name and PID if present
            let (unit, message) = if let Some(bracket_pos) = unit_and_msg.find(':') {
                let unit_part = &unit_and_msg[..bracket_pos];
                let msg_part = &unit_and_msg[bracket_pos + 1..].trim();
                (unit_part.to_string(), msg_part.to_string())
            } else {
                (String::new(), unit_and_msg.to_string())
            };

            // Determine log level
            let log_level = if message.contains("ERROR") || message.contains("error") {
                "ERROR"
            } else if message.contains("WARNING") || message.contains("warning") {
                "WARNING"
            } else if message.contains("CRITICAL") || message.contains("critical") {
                "CRITICAL"
            } else {
                "INFO"
            };

            // Check for kernel events
            if let Some(event_type) = self.detect_kernel_event(&message) {
                let severity = match event_type {
                    "oom_kill" | "kernel_panic" => KernelEventSeverity::Critical,
                    "usb_disconnect" | "filesystem_error" => KernelEventSeverity::High,
                    "thermal_throttle" => KernelEventSeverity::Medium,
                    _ => KernelEventSeverity::Low,
                };

                // Extract PID if present in unit (e.g., "process[1234]")
                let pid = unit_and_msg
                    .split('[')
                    .nth(1)
                    .and_then(|s| s.split(']').next())
                    .and_then(|s| s.parse::<u32>().ok());

                events.push(MissionEvent::KernelEvent {
                    timestamp,
                    event_type: event_type.to_string(),
                    severity: severity.as_str().to_string(),
                    description: message.clone(),
                    source_file: Some("journalctl.log".to_string()),
                    process_id: pid,
                    process_name: if !unit.is_empty() {
                        Some(unit.clone())
                    } else {
                        None
                    },
                });
            } else {
                // Regular log entry
                events.push(MissionEvent::LinuxLogEvent {
                    timestamp,
                    log_source: "journalctl".to_string(),
                    log_level: log_level.to_string(),
                    unit: if unit.is_empty() { None } else { Some(unit) },
                    message,
                    metadata: None,
                });
            }
        }

        Ok(events)
    }

    /// Parse dmesg format
    pub fn parse_dmesg(&self, content: &str) -> Result<Vec<MissionEvent>, AdapterError> {
        let mut events = Vec::new();

        // dmesg format: [timestamp] message
        // Example: [    0.000000] Linux version 5.10.0
        // Example: [ 1234.567890] Out of memory: Kill process 2341 (nav_stack)

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Extract timestamp
            if let Some(bracket_end) = line.find(']') {
                let timestamp_str = &line[1..bracket_end];
                let message = &line[bracket_end + 1..].trim();

                // Parse timestamp (seconds since boot)
                let boot_seconds: f64 = timestamp_str.trim().parse().unwrap_or(0.0);

                // Use approximate timestamp (would be improved with system boot time)
                let timestamp = Utc::now() - chrono::Duration::milliseconds((boot_seconds * 1000.0) as i64);

                // Detect kernel events
                if let Some(event_type) = self.detect_kernel_event(message) {
                    let severity = match event_type {
                        "oom_kill" | "kernel_panic" => KernelEventSeverity::Critical,
                        "usb_disconnect" | "filesystem_error" => KernelEventSeverity::High,
                        "thermal_throttle" => KernelEventSeverity::Medium,
                        _ => KernelEventSeverity::Low,
                    };

                    events.push(MissionEvent::KernelEvent {
                        timestamp,
                        event_type: event_type.to_string(),
                        severity: severity.as_str().to_string(),
                        description: message.to_string(),
                        source_file: Some("dmesg.log".to_string()),
                        process_id: None,
                        process_name: None,
                    });
                } else {
                    events.push(MissionEvent::KernelEvent {
                        timestamp,
                        event_type: "kernel_message".to_string(),
                        severity: KernelEventSeverity::Low.as_str().to_string(),
                        description: message.to_string(),
                        source_file: Some("dmesg.log".to_string()),
                        process_id: None,
                        process_name: None,
                    });
                }
            }
        }

        Ok(events)
    }

    /// Parse generic syslog format
    pub fn parse_syslog(&self, content: &str) -> Result<Vec<MissionEvent>, AdapterError> {
        let mut events = Vec::new();

        // syslog format: MMM DD HH:MM:SS hostname process[pid]: message
        // Example: Jul 25 14:22:15 robot1 nav_stack[2341]: Planner timeout

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Some(event) = self.parse_generic_log_line(line, "syslog") {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Parse generic log line (format-agnostic)
    fn parse_generic_log_line(&self, line: &str, source: &str) -> Option<MissionEvent> {
        // Try to extract timestamp from common formats
        let timestamp = self.extract_timestamp(line).unwrap_or_else(Utc::now);

        // Determine log level
        let log_level = if line.contains("ERROR") {
            "ERROR"
        } else if line.contains("WARNING") {
            "WARNING"
        } else if line.contains("CRITICAL") {
            "CRITICAL"
        } else {
            "INFO"
        };

        // Check for kernel events
        if let Some(event_type) = self.detect_kernel_event(line) {
            let severity = match event_type {
                "oom_kill" | "kernel_panic" => KernelEventSeverity::Critical,
                "usb_disconnect" | "filesystem_error" => KernelEventSeverity::High,
                _ => KernelEventSeverity::Medium,
            };

            Some(MissionEvent::KernelEvent {
                timestamp,
                event_type: event_type.to_string(),
                severity: severity.as_str().to_string(),
                description: line.to_string(),
                source_file: Some(format!("{}.log", source)),
                process_id: None,
                process_name: None,
            })
        } else {
            Some(MissionEvent::LinuxLogEvent {
                timestamp,
                log_source: source.to_string(),
                log_level: log_level.to_string(),
                unit: None,
                message: line.to_string(),
                metadata: None,
            })
        }
    }

    /// Try to extract timestamp from a log line
    fn extract_timestamp(&self, line: &str) -> Option<DateTime<Utc>> {
        // Try various timestamp formats
        let timestamp_patterns = vec![
            "%Y-%m-%d %H:%M:%S",
            "%b %d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S",
        ];

        for pattern in timestamp_patterns {
            if let Ok(naive) = NaiveDateTime::parse_from_str(
                &line.chars().take(20).collect::<String>(),
                pattern,
            ) {
                return Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
            }
        }

        None
    }
}

impl Default for LinuxLogAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = LinuxLogAdapter::new();
        assert!(!adapter.journalctl_patterns.is_empty());
    }

    #[test]
    fn test_severity_from_log_level() {
        assert_eq!(
            KernelEventSeverity::from_log_level("ERROR"),
            KernelEventSeverity::High
        );
        assert_eq!(
            KernelEventSeverity::from_log_level("WARNING"),
            KernelEventSeverity::Medium
        );
        assert_eq!(
            KernelEventSeverity::from_log_level("INFO"),
            KernelEventSeverity::Low
        );
    }

    #[test]
    fn test_oom_kill_detection() {
        let adapter = LinuxLogAdapter::new();
        let message = "Out of memory: Kill process 2341 (nav_stack)";
        assert_eq!(adapter.detect_kernel_event(message), Some("oom_kill"));
    }

    #[test]
    fn test_usb_disconnect_detection() {
        let adapter = LinuxLogAdapter::new();
        let message = "usb 1-1: USB disconnect, device number 2";
        assert_eq!(adapter.detect_kernel_event(message), Some("usb_disconnect"));
    }

    #[test]
    fn test_parse_dmesg() {
        let adapter = LinuxLogAdapter::new();
        let content = "[  100.123456] Linux version 5.10.0\n[  200.654321] Out of memory: Kill process 2341";

        let events = adapter.parse_dmesg(content).unwrap();
        assert_eq!(events.len(), 2);
    }
}
