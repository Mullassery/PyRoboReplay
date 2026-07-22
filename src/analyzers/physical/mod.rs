//! Physical Domain Gap Analyzer
//!
//! Detects gaps related to mechanical systems:
//! - Mechanical degradation (wear, response time increase)
//! - Structural dynamics (vibration, flex, resonance)
//! - Thermal effects
//! - Calibration drift

pub mod mechanical_degradation;
pub mod thermal_effects;
pub mod structural_dynamics;

use crate::analyzers::{GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding};
use mechanical_degradation::MechanicalDegradationDetector;
use thermal_effects::ThermalEffectsDetector;
use structural_dynamics::StructuralDynamicsDetector;

/// Analyzer for physical domain gaps
pub struct PhysicalDomainAnalyzer {
    mechanical_detector: MechanicalDegradationDetector,
    thermal_detector: ThermalEffectsDetector,
    structural_detector: StructuralDynamicsDetector,
}

impl PhysicalDomainAnalyzer {
    pub fn new() -> Self {
        PhysicalDomainAnalyzer {
            mechanical_detector: MechanicalDegradationDetector::new(),
            thermal_detector: ThermalEffectsDetector::new(),
            structural_detector: StructuralDynamicsDetector::new(),
        }
    }
}

impl Default for PhysicalDomainAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GapDetector for PhysicalDomainAnalyzer {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        let mut findings = Vec::new();

        findings.extend(self.mechanical_detector.analyze(mission_data));
        findings.extend(self.thermal_detector.analyze(mission_data));
        findings.extend(self.structural_detector.analyze(mission_data));

        findings
    }

    fn domain(&self) -> RealityDomain {
        RealityDomain::Physical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_analyzer_creation() {
        let analyzer = PhysicalDomainAnalyzer::new();
        assert_eq!(analyzer.domain(), RealityDomain::Physical);
    }
}
