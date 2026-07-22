//! Sensor Fusion Layer
//!
//! Phase 13: Multispectral Perception & Invisible Person Detection
//!
//! Fuses RGB and thermal/infrared imagery to achieve perception
//! beyond either sensor alone. Discovers people and objects missed by RGB.

pub mod thermal_model;
pub mod rgb_thermal_fusion;
pub mod invisible_person_detector;
pub mod forensic_reporter;

pub use thermal_model::{ThermalCameraConfig, ThermalFrame, ThermalHotspot, ThermalSource};
pub use rgb_thermal_fusion::{
    FusedDetection, ThermalOnlyDetection, RGBThermalFusionEngine, FusionStatistics,
};
pub use invisible_person_detector::{InvisiblePersonScenario, InvisiblePersonDetector};
pub use forensic_reporter::{ForensicReport, ForensicReporter};
