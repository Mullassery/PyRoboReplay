//! System Domain Gap Analyzer
//!
//! Detects gaps related to system-level effects:
//! - CPU contention and thermal throttling
//! - Memory pressure and leaks
//! - Network congestion and packet loss
//! - Clock drift and timing issues

pub mod clock_drift;

use crate::analyzers::{GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding};
use clock_drift::ClockDriftDetector;

pub struct SystemDomainAnalyzer {
    clock_drift_detector: ClockDriftDetector,
}

impl SystemDomainAnalyzer {
    pub fn new() -> Self {
        SystemDomainAnalyzer {
            clock_drift_detector: ClockDriftDetector::new(),
        }
    }
}

impl Default for SystemDomainAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GapDetector for SystemDomainAnalyzer {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();
        findings.extend(self.clock_drift_detector.analyze(mission_data));
        findings
    }

    fn domain(&self) -> RealityDomain {
        RealityDomain::System
    }
}
