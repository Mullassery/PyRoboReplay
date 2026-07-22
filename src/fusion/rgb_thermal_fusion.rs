//! RGB-Thermal Sensor Fusion
//!
//! Fuses RGB and thermal imagery to achieve perception robustness
//! beyond either sensor alone.

use crate::fusion::thermal_model::{ThermalFrame, ThermalSource};
use crate::perception::object_detection::{BoundingBox, DetectedObject, ObjectClass};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RGB detection with thermal corroboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedDetection {
    /// Detection ID
    pub id: u32,
    /// Object class
    pub class: ObjectClass,
    /// RGB confidence (0.0-1.0)
    pub rgb_confidence: f32,
    /// Thermal evidence strength (0.0-1.0)
    pub thermal_confidence: f32,
    /// Fused confidence (combined assessment)
    pub fused_confidence: f32,
    /// Bounding box from RGB
    pub rgb_bbox: BoundingBox,
    /// Thermal evidence present
    pub thermal_evidence: bool,
    /// Thermal source (if present)
    pub thermal_source: Option<ThermalSource>,
    /// Sensor agreement (0.0-1.0, 1.0 = perfect agreement)
    pub sensor_agreement: f32,
    /// Which sensor detected first (in temporal sequence)
    pub first_detector: String, // "rgb", "thermal", or "both"
}

/// Missed RGB detection (thermal only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalOnlyDetection {
    /// Estimated object class (from thermal signature)
    pub estimated_class: ObjectClass,
    /// Thermal confidence
    pub thermal_confidence: f32,
    /// Bounding box (thermal estimate)
    pub bbox: BoundingBox,
    /// Thermal source
    pub source: ThermalSource,
    /// Why RGB likely missed it
    pub rgb_miss_reason: String,
    /// Invisibility factors (darkness, occlusion, etc.)
    pub invisibility_factors: Vec<String>,
}

/// Sensor disagreement (both detectors, different classes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorDisagreement {
    /// RGB detection
    pub rgb_detection: DetectedObject,
    /// Thermal assessment (what thermal thinks it is)
    pub thermal_source: ThermalSource,
    /// Disagreement type
    pub disagreement_type: String,
    /// Likely explanation
    pub explanation: String,
    /// Recommended assessment
    pub recommended_class: ObjectClass,
}

/// RGB-Thermal fusion engine
pub struct RGBThermalFusionEngine {
    /// RGB detections
    pub rgb_detections: Vec<DetectedObject>,
    /// Thermal frame
    pub thermal_frame: Option<ThermalFrame>,
    /// Fused detections
    pub fused_detections: Vec<FusedDetection>,
    /// Thermal-only detections (RGB missed)
    pub thermal_only: Vec<ThermalOnlyDetection>,
    /// Sensor disagreements
    pub disagreements: Vec<SensorDisagreement>,
}

impl RGBThermalFusionEngine {
    /// Create new fusion engine
    pub fn new() -> Self {
        RGBThermalFusionEngine {
            rgb_detections: Vec::new(),
            thermal_frame: None,
            fused_detections: Vec::new(),
            thermal_only: Vec::new(),
            disagreements: Vec::new(),
        }
    }

    /// Load RGB detections
    pub fn load_rgb_detections(&mut self, detections: Vec<DetectedObject>) {
        self.rgb_detections = detections;
    }

    /// Load thermal frame
    pub fn load_thermal_frame(&mut self, frame: ThermalFrame) {
        self.thermal_frame = Some(frame);
    }

    /// Perform fusion analysis
    pub fn fuse(&mut self) {
        self.fused_detections.clear();
        self.thermal_only.clear();
        self.disagreements.clear();

        if self.thermal_frame.is_none() {
            return;
        }

        let thermal = self.thermal_frame.as_ref().unwrap();

        // Process RGB detections with thermal evidence
        for rgb_det in &self.rgb_detections {
            let thermal_evidence = thermal.get_region_avg(
                rgb_det.bbox.x as u32,
                rgb_det.bbox.y as u32,
                rgb_det.bbox.width as u32,
                rgb_det.bbox.height as u32,
            );

            let thermal_confidence =
                thermal.estimate_human_likelihood(
                    rgb_det.bbox.x as u32,
                    rgb_det.bbox.y as u32,
                    rgb_det.bbox.width as u32,
                    rgb_det.bbox.height as u32,
                );

            let thermal_present = thermal_evidence > 280.0; // Above ambient

            // Compute sensor agreement
            let agreement = if thermal_present {
                0.9 // Good agreement if thermal confirms
            } else if rgb_det.class == ObjectClass::Person {
                0.3 // Poor agreement for person without thermal
            } else {
                0.7 // Moderate agreement for non-person
            };

            // Fused confidence (weight both sensors)
            let fused_conf = (rgb_det.confidence * 0.6 + thermal_confidence * 0.4).min(1.0);

            let thermal_source = if thermal_present {
                let hotspots = thermal.detect_hotspots(5.0);
                hotspots
                    .iter()
                    .find(|h| {
                        let dist = ((h.center_x - rgb_det.bbox.x).powi(2)
                            + (h.center_y - rgb_det.bbox.y).powi(2))
                        .sqrt();
                        dist < 50.0
                    })
                    .map(|h| h.estimate_source())
            } else {
                None
            };

            let fused = FusedDetection {
                id: rgb_det.id,
                class: rgb_det.class,
                rgb_confidence: rgb_det.confidence,
                thermal_confidence,
                fused_confidence: fused_conf,
                rgb_bbox: rgb_det.bbox.clone(),
                thermal_evidence: thermal_present,
                thermal_source,
                sensor_agreement: agreement,
                first_detector: "rgb".to_string(),
            };

            self.fused_detections.push(fused);
        }

        // Find thermal-only detections (RGB missed)
        let hotspots = thermal.detect_hotspots(5.0);
        for hotspot in hotspots {
            let is_explained = self.fused_detections.iter().any(|f| {
                let dist = ((f.rgb_bbox.x - hotspot.center_x).powi(2)
                    + (f.rgb_bbox.y - hotspot.center_y).powi(2))
                .sqrt();
                dist < 75.0
            });

            if !is_explained {
                let source = hotspot.estimate_source();
                let estimated_class = match source {
                    ThermalSource::Human | ThermalSource::HumanPartial => ObjectClass::Person,
                    ThermalSource::Animal => ObjectClass::Animal,
                    ThermalSource::Engine => ObjectClass::Machinery,
                    _ => ObjectClass::Unknown,
                };

                let thermal_only_det = ThermalOnlyDetection {
                    estimated_class,
                    thermal_confidence: hotspot.source_confidence(),
                    bbox: BoundingBox {
                        x: hotspot.center_x - 30.0,
                        y: hotspot.center_y - 30.0,
                        width: 60.0,
                        height: 60.0,
                    },
                    source,
                    rgb_miss_reason: "No RGB detection at thermal hotspot".to_string(),
                    invisibility_factors: vec![
                        "low_light".to_string(),
                        "thermal_signature_dominant".to_string(),
                    ],
                };

                self.thermal_only.push(thermal_only_det);
            }
        }
    }

    /// Get fusion statistics
    pub fn get_statistics(&self) -> FusionStatistics {
        let total_rgb = self.rgb_detections.len();
        let fused_count = self.fused_detections.len();
        let thermal_only_count = self.thermal_only.len();
        let total_detections = fused_count + thermal_only_count;

        let avg_rgb_conf = if total_rgb > 0 {
            self.rgb_detections
                .iter()
                .map(|d| d.confidence)
                .sum::<f32>()
                / total_rgb as f32
        } else {
            0.0
        };

        let avg_fused_conf = if fused_count > 0 {
            self.fused_detections
                .iter()
                .map(|d| d.fused_confidence)
                .sum::<f32>()
                / fused_count as f32
        } else {
            0.0
        };

        let avg_agreement = if fused_count > 0 {
            self.fused_detections
                .iter()
                .map(|d| d.sensor_agreement)
                .sum::<f32>()
                / fused_count as f32
        } else {
            0.0
        };

        FusionStatistics {
            rgb_detections: total_rgb,
            thermal_only_detections: thermal_only_count,
            fused_detections: fused_count,
            total_detections,
            rgb_miss_rate: (thermal_only_count as f32 / total_detections as f32).min(1.0),
            avg_rgb_confidence: avg_rgb_conf,
            avg_fused_confidence: avg_fused_conf,
            avg_sensor_agreement: avg_agreement,
            confidence_improvement: avg_fused_conf - avg_rgb_conf,
        }
    }

    /// Generate fusion report
    pub fn generate_report(&self) -> String {
        let stats = self.get_statistics();

        let mut report = String::from("RGB-THERMAL FUSION ANALYSIS\n");
        report.push_str("================================\n\n");

        report.push_str(&format!("RGB Detections: {}\n", stats.rgb_detections));
        report.push_str(&format!("Thermal-Only Detections: {}\n", stats.thermal_only_detections));
        report.push_str(&format!("Total Detections: {}\n", stats.total_detections));
        report.push_str(&format!("RGB Miss Rate: {:.1}%\n\n", stats.rgb_miss_rate * 100.0));

        report.push_str(&format!("Average RGB Confidence: {:.0}%\n", stats.avg_rgb_confidence * 100.0));
        report.push_str(&format!("Average Fused Confidence: {:.0}%\n", stats.avg_fused_confidence * 100.0));
        report.push_str(&format!(
            "Confidence Improvement: +{:.1}%\n\n",
            stats.confidence_improvement * 100.0
        ));

        report.push_str(&format!("Average Sensor Agreement: {:.0}%\n", stats.avg_sensor_agreement * 100.0));

        if !self.thermal_only.is_empty() {
            report.push_str("\nTHERMAL-ONLY DETECTIONS (RGB MISSED):\n");
            for (idx, thermal_only) in self.thermal_only.iter().enumerate() {
                report.push_str(&format!(
                    "  {}. {} (thermal confidence {:.0}%)\n",
                    idx + 1,
                    thermal_only.estimated_class,
                    thermal_only.thermal_confidence * 100.0
                ));
                report.push_str(&format!(
                    "     Source: {} | Reason: {}\n",
                    thermal_only.source, thermal_only.rgb_miss_reason
                ));
            }
        }

        report
    }
}

impl Default for RGBThermalFusionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Fusion statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionStatistics {
    pub rgb_detections: usize,
    pub thermal_only_detections: usize,
    pub fused_detections: usize,
    pub total_detections: usize,
    pub rgb_miss_rate: f32,
    pub avg_rgb_confidence: f32,
    pub avg_fused_confidence: f32,
    pub avg_sensor_agreement: f32,
    pub confidence_improvement: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_fusion_engine_creation() {
        let engine = RGBThermalFusionEngine::new();
        assert_eq!(engine.rgb_detections.len(), 0);
    }

    #[test]
    fn test_load_rgb_detections() {
        let mut engine = RGBThermalFusionEngine::new();
        let rgb_det = DetectedObject {
            id: 1,
            class: ObjectClass::Person,
            confidence: 0.95,
            bbox: BoundingBox {
                x: 100.0,
                y: 100.0,
                width: 50.0,
                height: 100.0,
            },
            distance_m: Some(3.0),
            velocity_ms: None,
            position_3d: None,
            trajectory_id: None,
            attributes: HashMap::new(),
        };

        engine.load_rgb_detections(vec![rgb_det]);
        assert_eq!(engine.rgb_detections.len(), 1);
    }

    #[test]
    fn test_fusion_report() {
        let engine = RGBThermalFusionEngine::new();
        let report = engine.generate_report();
        assert!(report.contains("RGB-THERMAL FUSION ANALYSIS"));
    }

    #[test]
    fn test_statistics_generation() {
        let engine = RGBThermalFusionEngine::new();
        let stats = engine.get_statistics();
        assert_eq!(stats.rgb_detections, 0);
    }
}
