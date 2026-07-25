/// Failure Detection Framework for MLRIAS
///
/// Detects and classifies failures across 5 domains:
/// 1. Navigation (planner timeouts, oscillation, recovery loops)
/// 2. Localization (AMCL divergence, map mismatch, TF issues)
/// 3. Perception (sensor dropout, frame loss, sync issues)
/// 4. Middleware (DDS discovery, QoS, topic starvation)
/// 5. System (OOM kills, kernel panics, USB resets)

pub mod navigation;
pub mod localization;
pub mod perception;
pub mod middleware;
pub mod system;

use crate::core::event::MissionEvent;
use crate::core::timeline_correlation::NormalizedEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use navigation::NavigationFailureDetector;
pub use localization::LocalizationFailureDetector;
pub use perception::PerceptionFailureDetector;
pub use middleware::MiddlewareFailureDetector;
pub use system::SystemFailureDetector;

/// A detected failure with evidence and diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFailure {
    /// Unique failure identifier
    pub id: String,

    /// Type of failure (e.g., "planner_timeout", "amcl_divergence")
    pub failure_type: String,

    /// Domain this failure belongs to
    pub domain: FailureDomain,

    /// When the failure occurred
    pub timestamp: DateTime<Utc>,

    /// Confidence in this failure detection (0.0-1.0)
    pub confidence: f32,

    /// Severity level (critical, high, medium, low)
    pub severity: FailureSeverity,

    /// Human-readable description
    pub description: String,

    /// Evidence supporting this failure
    pub evidence: Vec<String>,

    /// Event IDs involved in this failure
    pub event_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureDomain {
    Navigation,
    Localization,
    Perception,
    Middleware,
    System,
}

impl FailureDomain {
    pub fn as_str(&self) -> &str {
        match self {
            FailureDomain::Navigation => "navigation",
            FailureDomain::Localization => "localization",
            FailureDomain::Perception => "perception",
            FailureDomain::Middleware => "middleware",
            FailureDomain::System => "system",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FailureSeverity {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

impl FailureSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            FailureSeverity::Low => "low",
            FailureSeverity::Medium => "medium",
            FailureSeverity::High => "high",
            FailureSeverity::Critical => "critical",
        }
    }
}

impl DetectedFailure {
    pub fn new(
        failure_type: impl Into<String>,
        domain: FailureDomain,
        timestamp: DateTime<Utc>,
        confidence: f32,
        severity: FailureSeverity,
        description: impl Into<String>,
    ) -> Self {
        let failure_type_str = failure_type.into();
        Self {
            id: format!("{}_{}_{}", domain.as_str(), failure_type_str, timestamp.timestamp()),
            failure_type: failure_type_str,
            domain,
            timestamp,
            confidence: confidence.max(0.0).min(1.0),
            severity,
            description: description.into(),
            evidence: Vec::new(),
            event_ids: Vec::new(),
        }
    }

    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }

    pub fn with_event_ids(mut self, event_ids: Vec<String>) -> Self {
        self.event_ids = event_ids;
        self
    }
}

/// Trait for failure detectors
pub trait FailureDetector: Send + Sync {
    /// Detect failures in the timeline
    fn detect(&self, events: &[NormalizedEvent]) -> Vec<DetectedFailure>;

    /// Get the domain this detector handles
    fn domain(&self) -> FailureDomain;
}

/// Orchestrator for all failure detectors
pub struct FailureDetectionEngine {
    detectors: Vec<Box<dyn FailureDetector>>,
}

impl FailureDetectionEngine {
    pub fn new() -> Self {
        Self {
            detectors: vec![
                Box::new(NavigationFailureDetector),
                Box::new(LocalizationFailureDetector),
                Box::new(PerceptionFailureDetector),
                Box::new(MiddlewareFailureDetector),
                Box::new(SystemFailureDetector),
            ],
        }
    }

    /// Detect all failures across all domains
    pub fn detect_all(&self, events: &[NormalizedEvent]) -> Vec<DetectedFailure> {
        let mut all_failures = Vec::new();

        for detector in &self.detectors {
            let failures = detector.detect(events);
            all_failures.extend(failures);
        }

        // Sort by timestamp for consistent ordering
        all_failures.sort_by_key(|f| f.timestamp);

        all_failures
    }

    /// Detect failures in a specific domain only
    pub fn detect_by_domain(&self, events: &[NormalizedEvent], domain: FailureDomain) -> Vec<DetectedFailure> {
        self.detectors
            .iter()
            .filter(|d| d.domain() == domain)
            .flat_map(|d| d.detect(events))
            .collect()
    }

    /// Add a custom detector
    pub fn add_detector(&mut self, detector: Box<dyn FailureDetector>) {
        self.detectors.push(detector);
    }
}

impl Default for FailureDetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_creation() {
        let failure = DetectedFailure::new(
            "test_failure",
            FailureDomain::Navigation,
            Utc::now(),
            0.85,
            FailureSeverity::High,
            "Test failure",
        );
        assert_eq!(failure.confidence, 0.85);
        assert_eq!(failure.severity, FailureSeverity::High);
    }

    #[test]
    fn test_engine_creation() {
        let engine = FailureDetectionEngine::new();
        // Should have 5 detectors
        assert_eq!(engine.detectors.len(), 5);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(FailureSeverity::Low < FailureSeverity::High);
        assert!(FailureSeverity::Critical > FailureSeverity::Medium);
    }
}
