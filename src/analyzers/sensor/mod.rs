//! Sensor Domain Gap Analyzer
//!
//! Detects gaps related to sensor systems:
//! - Optical contamination (lens dirt, water droplets)
//! - Calibration drift (intrinsic, extrinsic)
//! - Sensor timing issues (clock drift, synchronization)
//! - Signal corruption (multipath, interference)

pub mod optical_contamination;

use crate::analyzers::{GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding};
use optical_contamination::OpticalContaminationDetector;

/// Analyzer for sensor domain gaps
pub struct SensorDomainAnalyzer {
    optical_detector: OpticalContaminationDetector,
}

impl SensorDomainAnalyzer {
    pub fn new() -> Self {
        SensorDomainAnalyzer {
            optical_detector: OpticalContaminationDetector::new(),
        }
    }
}

impl Default for SensorDomainAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GapDetector for SensorDomainAnalyzer {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();
        findings.extend(self.optical_detector.analyze(mission_data));
        findings
    }

    fn domain(&self) -> RealityDomain {
        RealityDomain::Sensor
    }
}
