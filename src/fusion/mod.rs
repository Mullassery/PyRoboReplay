//! Sensor Fusion Layer
//!
//! Phase 13: Multispectral Perception & Invisible Person Detection
//!
//! Fuses RGB and thermal/infrared imagery to achieve perception
//! beyond either sensor alone. Discovers people and objects missed by RGB.

pub mod forensic_reporter;
pub mod invisible_person_detector;
pub mod rgb_thermal_fusion;
pub mod thermal_model;

pub use forensic_reporter::{ForensicReport, ForensicReporter};
pub use invisible_person_detector::{InvisiblePersonDetector, InvisiblePersonScenario};
pub use rgb_thermal_fusion::{
    FusedDetection, FusionStatistics, RGBThermalFusionEngine, ThermalOnlyDetection,
};
pub use thermal_model::{ThermalCameraConfig, ThermalFrame, ThermalHotspot, ThermalSource};
