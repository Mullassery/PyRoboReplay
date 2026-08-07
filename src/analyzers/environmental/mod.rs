//! Environmental Domain Gap Analyzer
//!
//! Detects gaps related to environmental factors:
//! - Dynamic lighting and shadows
//! - Seasonal and long-term environmental changes
//! - Weather effects (wind, rain, snow)
//! - Human interactions and unpredictability
//!
//! `MissionAnalysisData` doesn't carry raw pixel data or direct lux/weather
//! telemetry (see `CameraFrame`'s doc comment — "simplified, no actual image
//! bytes"), so this analyzer works from the proxies that ARE available:
//! frame-quality metrics (entropy/sharpness, which correlate with lighting
//! conditions — under/overexposed frames have depressed entropy) and thermal
//! readings (temperature swings correlate with weather/time-of-day, which
//! simulation typically holds constant).

pub mod lighting_variability;
pub mod thermal_drift;

use crate::analyzers::{GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding};
use lighting_variability::LightingVariabilityDetector;
use thermal_drift::ThermalDriftDetector;

pub struct EnvironmentalDomainAnalyzer {
    lighting_detector: LightingVariabilityDetector,
    thermal_detector: ThermalDriftDetector,
}

impl EnvironmentalDomainAnalyzer {
    pub fn new() -> Self {
        EnvironmentalDomainAnalyzer {
            lighting_detector: LightingVariabilityDetector::new(),
            thermal_detector: ThermalDriftDetector::new(),
        }
    }
}

impl Default for EnvironmentalDomainAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GapDetector for EnvironmentalDomainAnalyzer {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();
        findings.extend(self.lighting_detector.analyze(&mission_data.camera_frames));
        findings.extend(self.thermal_detector.analyze(&mission_data.thermal_readings));
        findings
    }

    fn domain(&self) -> RealityDomain {
        RealityDomain::Environmental
    }
}
