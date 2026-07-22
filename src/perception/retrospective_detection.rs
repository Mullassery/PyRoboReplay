//! Phase 12: Retrospective Detection with DINO + SAM
//!
//! Discovers invisible objects: What robot should have seen vs what it detected.
//!
//! Flow:
//! 1. Robot runs YOLO detection (real-time, Phase 7)
//! 2. Mission fails → analyze with DINO retrospectively
//! 3. DINO finds open-vocabulary objects (person, vehicle, box, obstacle, etc.)
//! 4. SAM segments each discovery with precise boundaries
//! 5. Compare YOLO vs DINO: identify missed detections
//! 6. Score invisibility factors (occlusion, distance, blur, etc.)

use crate::perception::object_detection::{BoundingBox, DetectedObject, DetectionFrame, ObjectClass};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DINO (open-vocabulary detection) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DINOConfig {
    /// Model checkpoint
    pub model_path: String,
    /// Text prompt (e.g., "person . vehicle . box . obstacle")
    pub text_prompt: String,
    /// Confidence threshold for detections
    pub confidence_threshold: f32,
    /// Device: "cpu" or "cuda:0"
    pub device: String,
}

impl Default for DINOConfig {
    fn default() -> Self {
        DINOConfig {
            model_path: "dino_v1.pt".to_string(),
            text_prompt: "person . vehicle . bicycle . box . pallet . obstacle . wall . door . window . ramp . stairs . chair . table . machinery".to_string(),
            confidence_threshold: 0.35,
            device: "cpu".to_string(),
        }
    }
}

/// DINO detection result (open-vocabulary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DINODetection {
    /// Object description (from text prompt match)
    pub class_name: String,
    /// Bounding box
    pub bbox: BoundingBox,
    /// Confidence (0.0-1.0)
    pub confidence: f32,
    /// Estimated distance (if available)
    pub distance_m: Option<f32>,
}

impl DINODetection {
    /// Convert DINO detection to ObjectClass (best effort)
    pub fn to_object_class(&self) -> ObjectClass {
        match self.class_name.to_lowercase().as_str() {
            s if s.contains("person") || s.contains("human") => ObjectClass::Person,
            s if s.contains("vehicle") || s.contains("car") || s.contains("truck") => {
                ObjectClass::Vehicle
            }
            s if s.contains("bicycle") || s.contains("bike") => ObjectClass::Bicycle,
            s if s.contains("animal") || s.contains("dog") || s.contains("cat") => {
                ObjectClass::Animal
            }
            s if s.contains("cone") => ObjectClass::TrafficCone,
            s if s.contains("pallet") => ObjectClass::Pallet,
            s if s.contains("forklift") => ObjectClass::Forklift,
            s if s.contains("machinery") || s.contains("machine") => ObjectClass::Machinery,
            s if s.contains("tool") => ObjectClass::Tool,
            s if s.contains("obstacle") || s.contains("wall") || s.contains("barrier") => {
                ObjectClass::StaticObstacle
            }
            _ => ObjectClass::Unknown,
        }
    }
}

/// SAM segmentation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SAMSegmentation {
    /// Segmentation mask (simplified as percentage of image)
    pub mask_area_percentage: f32,
    /// Bounding box from mask
    pub bbox: BoundingBox,
    /// Quality score (0.0-1.0)
    pub quality_score: f32,
    /// Contour complexity (0.0=simple, 1.0=complex)
    pub contour_complexity: f32,
}

/// Detection gap: object found by DINO but missed by YOLO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionGap {
    /// DINO detection
    pub dino_detection: DINODetection,
    /// SAM segmentation
    pub sam_segmentation: Option<SAMSegmentation>,
    /// Why robot missed it
    pub invisibility_factors: Vec<InvisibilityFactor>,
    /// Severity (0.0-1.0)
    pub severity: f32,
    /// Recommendation to avoid in future
    pub recommendation: String,
}

/// Factors contributing to invisibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InvisibilityFactor {
    /// Object partially or fully occluded
    Occlusion(f32), // occlusion percentage
    /// Object too far from camera
    Distance(f32), // distance in meters
    /// Image blur or motion blur
    ImageBlur(f32), // blur amount 0.0-1.0
    /// Low contrast with background
    LowContrast(f32), // contrast 0.0-1.0
    /// Outside camera field of view
    OutOfFOV,
    /// Lighting too dark
    DarkLighting(f32), // darkness 0.0-1.0
    /// Object too small in image
    TooSmall(f32), // size relative to image
    /// Similar color to background (camouflage)
    ColorCamouflage(f32), // similarity 0.0-1.0
}

impl InvisibilityFactor {
    /// Get severity weight (0.0-1.0)
    pub fn severity_weight(&self) -> f32 {
        match self {
            InvisibilityFactor::Occlusion(pct) => pct.min(1.0) * 0.9,
            InvisibilityFactor::Distance(m) => (*m / 20.0).min(1.0) * 0.7,
            InvisibilityFactor::ImageBlur(amt) => amt.min(1.0) * 0.6,
            InvisibilityFactor::LowContrast(amt) => amt.min(1.0) * 0.8,
            InvisibilityFactor::OutOfFOV => 1.0,
            InvisibilityFactor::DarkLighting(amt) => amt.min(1.0) * 0.85,
            InvisibilityFactor::TooSmall(amt) => amt.min(1.0) * 0.7,
            InvisibilityFactor::ColorCamouflage(amt) => amt.min(1.0) * 0.65,
        }
    }

    /// Get description
    pub fn description(&self) -> String {
        match self {
            InvisibilityFactor::Occlusion(pct) => format!("{}% occluded", (pct * 100.0) as u32),
            InvisibilityFactor::Distance(m) => format!("{:.1}m away", m),
            InvisibilityFactor::ImageBlur(amt) => format!("Blur: {:.0}%", amt * 100.0),
            InvisibilityFactor::LowContrast(amt) => {
                format!("Low contrast: {:.0}%", amt * 100.0)
            }
            InvisibilityFactor::OutOfFOV => "Outside field of view".to_string(),
            InvisibilityFactor::DarkLighting(amt) => format!("Dark: {:.0}%", amt * 100.0),
            InvisibilityFactor::TooSmall(amt) => format!("Too small: {:.0}% of image", amt * 100.0),
            InvisibilityFactor::ColorCamouflage(amt) => {
                format!("Color camouflage: {:.0}%", amt * 100.0)
            }
        }
    }
}

/// Retrospective detection engine
pub struct RetrospectiveDetectionEngine {
    /// DINO configuration
    pub dino_config: DINOConfig,
    /// YOLO detections (robot saw)
    pub yolo_detections: Vec<DetectedObject>,
    /// DINO detections (retrospective analysis)
    pub dino_detections: Vec<DINODetection>,
    /// SAM segmentations
    pub segmentations: HashMap<usize, SAMSegmentation>, // dino_detection index -> segmentation
    /// Detection gaps identified
    pub gaps: Vec<DetectionGap>,
}

impl RetrospectiveDetectionEngine {
    /// Create new engine
    pub fn new(dino_config: DINOConfig) -> Self {
        RetrospectiveDetectionEngine {
            dino_config,
            yolo_detections: Vec::new(),
            dino_detections: Vec::new(),
            segmentations: HashMap::new(),
            gaps: Vec::new(),
        }
    }

    /// Load YOLO detections (what robot saw)
    pub fn load_yolo_detections(&mut self, detections: Vec<DetectedObject>) {
        self.yolo_detections = detections;
    }

    /// Simulate DINO detection (stub - real would use model)
    pub fn run_dino_detection(
        &mut self,
        _image_data: &[u8],
        width: u32,
        height: u32,
    ) {
        // Stub: in reality this would run DINO model
        // For now, simulate discovering extra objects
        self.dino_detections = vec![
            DINODetection {
                class_name: "person".to_string(),
                bbox: BoundingBox {
                    x: 100.0,
                    y: 50.0,
                    width: 80.0,
                    height: 150.0,
                },
                confidence: 0.72,
                distance_m: Some(3.5),
            },
            DINODetection {
                class_name: "obstacle".to_string(),
                bbox: BoundingBox {
                    x: 800.0,
                    y: 400.0,
                    width: 120.0,
                    height: 100.0,
                },
                confidence: 0.68,
                distance_m: Some(5.2),
            },
        ];
    }

    /// Simulate SAM segmentation
    pub fn run_sam_segmentation(&mut self) {
        for (idx, _dino_det) in self.dino_detections.iter().enumerate() {
            let seg = SAMSegmentation {
                mask_area_percentage: 0.15,
                bbox: BoundingBox {
                    x: 100.0,
                    y: 50.0,
                    width: 85.0,
                    height: 155.0,
                },
                quality_score: 0.89,
                contour_complexity: 0.45,
            };
            self.segmentations.insert(idx, seg);
        }
    }

    /// Find detection gaps (DINO found, YOLO missed)
    pub fn analyze_gaps(&mut self) {
        self.gaps.clear();

        for (idx, dino_det) in self.dino_detections.iter().enumerate() {
            // Check if YOLO detected nearby
            let yolo_detected_nearby = self.yolo_detections.iter().any(|yolo_det| {
                let dx = dino_det.bbox.x - yolo_det.bbox.x;
                let dy = dino_det.bbox.y - yolo_det.bbox.y;
                let distance = (dx * dx + dy * dy).sqrt();
                distance < 100.0 && dino_det.to_object_class() == yolo_det.class
            });

            if !yolo_detected_nearby {
                // This is a gap - DINO found it, YOLO missed it
                let invisibility_factors = self.assess_invisibility(&dino_det);
                let severity = self.compute_severity(&invisibility_factors);

                let recommendation = self.generate_recommendation(&dino_det, &invisibility_factors);

                let gap = DetectionGap {
                    dino_detection: dino_det.clone(),
                    sam_segmentation: self.segmentations.get(&idx).cloned(),
                    invisibility_factors,
                    severity,
                    recommendation,
                };

                self.gaps.push(gap);
            }
        }
    }

    /// Assess why object is invisible to YOLO
    fn assess_invisibility(&self, dino_det: &DINODetection) -> Vec<InvisibilityFactor> {
        let mut factors = Vec::new();

        // Distance factor
        if let Some(dist) = dino_det.distance_m {
            if dist > 8.0 {
                factors.push(InvisibilityFactor::Distance(dist));
            }
        }

        // Confidence suggests potential issues
        if dino_det.confidence < 0.6 {
            factors.push(InvisibilityFactor::ImageBlur(0.4));
        }

        // Bounding box size suggests small object
        let bbox_area = dino_det.bbox.width * dino_det.bbox.height;
        if bbox_area < 1000.0 {
            factors.push(InvisibilityFactor::TooSmall(0.3));
        }

        // Simulate occlusion detection
        if dino_det.bbox.x < 50.0 || dino_det.bbox.x > 1870.0 {
            factors.push(InvisibilityFactor::Occlusion(0.3));
        }

        factors
    }

    /// Compute overall severity from invisibility factors
    fn compute_severity(&self, factors: &[InvisibilityFactor]) -> f32 {
        if factors.is_empty() {
            return 0.0;
        }

        let weights: Vec<f32> = factors.iter().map(|f| f.severity_weight()).collect();
        weights.iter().sum::<f32>() / weights.len() as f32
    }

    /// Generate recommendation to avoid this gap in future
    fn generate_recommendation(
        &self,
        dino_det: &DINODetection,
        factors: &[InvisibilityFactor],
    ) -> String {
        let class_name = &dino_det.class_name;
        let primary_factor = factors.first().map(|f| f.description());

        if let Some(factor) = primary_factor {
            format!(
                "Detected {} but missed: {}. Consider: longer range sensor, enhanced preprocessing for {}",
                class_name, factor, factor.to_lowercase()
            )
        } else {
            format!("Found {} that detection model missed", class_name)
        }
    }

    /// Get gap summary report
    pub fn gap_summary(&self) -> String {
        let mut summary = format!("Detection Gap Analysis: {} gaps found\n", self.gaps.len());

        for (idx, gap) in self.gaps.iter().enumerate() {
            summary.push_str(&format!(
                "\nGap {}: {} (confidence {:.0}%)\n",
                idx + 1,
                gap.dino_detection.class_name,
                gap.dino_detection.confidence * 100.0
            ));

            summary.push_str(&format!("  Severity: {:.0}%\n", gap.severity * 100.0));

            if !gap.invisibility_factors.is_empty() {
                summary.push_str("  Invisibility factors:\n");
                for factor in &gap.invisibility_factors {
                    summary.push_str(&format!("    • {}\n", factor.description()));
                }
            }

            summary.push_str(&format!("  Recommendation: {}\n", gap.recommendation));
        }

        summary
    }

    /// Get statistics
    pub fn get_stats(&self) -> RetrospectiveDetectionStats {
        let total_gaps = self.gaps.len();
        let avg_severity = if total_gaps > 0 {
            self.gaps.iter().map(|g| g.severity).sum::<f32>() / total_gaps as f32
        } else {
            0.0
        };

        let critical_gaps = self.gaps.iter().filter(|g| g.severity > 0.7).count();
        let moderate_gaps = self.gaps.iter().filter(|g| g.severity > 0.4 && g.severity <= 0.7).count();

        RetrospectiveDetectionStats {
            yolo_detections: self.yolo_detections.len(),
            dino_detections: self.dino_detections.len(),
            detection_gaps: total_gaps,
            avg_gap_severity: avg_severity,
            critical_gaps,
            moderate_gaps,
        }
    }
}

/// Statistics from retrospective analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrospectiveDetectionStats {
    /// Detections robot made
    pub yolo_detections: usize,
    /// Detections DINO found
    pub dino_detections: usize,
    /// Gaps identified
    pub detection_gaps: usize,
    /// Average severity of gaps
    pub avg_gap_severity: f32,
    /// Critical gaps (>70% severity)
    pub critical_gaps: usize,
    /// Moderate gaps (40-70% severity)
    pub moderate_gaps: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dino_config_default() {
        let config = DINOConfig::default();
        assert!(!config.text_prompt.is_empty());
        assert_eq!(config.confidence_threshold, 0.35);
    }

    #[test]
    fn test_dino_detection_to_class() {
        let det = DINODetection {
            class_name: "person".to_string(),
            bbox: BoundingBox {
                x: 100.0,
                y: 100.0,
                width: 50.0,
                height: 100.0,
            },
            confidence: 0.8,
            distance_m: Some(3.0),
        };

        assert_eq!(det.to_object_class(), ObjectClass::Person);
    }

    #[test]
    fn test_invisibility_factor_severity() {
        let factor = InvisibilityFactor::Occlusion(0.5);
        assert!(factor.severity_weight() > 0.0);
        assert!(factor.severity_weight() <= 1.0);
    }

    #[test]
    fn test_retrospective_engine_creation() {
        let config = DINOConfig::default();
        let engine = RetrospectiveDetectionEngine::new(config);
        assert_eq!(engine.gaps.len(), 0);
    }

    #[test]
    fn test_load_yolo_detections() {
        let config = DINOConfig::default();
        let mut engine = RetrospectiveDetectionEngine::new(config);

        let yolo_det = DetectedObject {
            id: 1,
            class: ObjectClass::Vehicle,
            confidence: 0.95,
            bbox: BoundingBox {
                x: 500.0,
                y: 400.0,
                width: 100.0,
                height: 100.0,
            },
            distance_m: Some(5.0),
            velocity_ms: None,
            position_3d: None,
            trajectory_id: None,
            attributes: HashMap::new(),
        };

        engine.load_yolo_detections(vec![yolo_det]);
        assert_eq!(engine.yolo_detections.len(), 1);
    }

    #[test]
    fn test_dino_detection() {
        let config = DINOConfig::default();
        let mut engine = RetrospectiveDetectionEngine::new(config);

        engine.run_dino_detection(&vec![], 1920, 1080);
        assert!(!engine.dino_detections.is_empty());
    }

    #[test]
    fn test_sam_segmentation() {
        let config = DINOConfig::default();
        let mut engine = RetrospectiveDetectionEngine::new(config);

        engine.run_dino_detection(&vec![], 1920, 1080);
        engine.run_sam_segmentation();

        assert!(!engine.segmentations.is_empty());
    }

    #[test]
    fn test_gap_analysis() {
        let config = DINOConfig::default();
        let mut engine = RetrospectiveDetectionEngine::new(config);

        // Load YOLO detections (robot saw vehicle)
        let yolo_det = DetectedObject {
            id: 1,
            class: ObjectClass::Vehicle,
            confidence: 0.95,
            bbox: BoundingBox {
                x: 500.0,
                y: 400.0,
                width: 100.0,
                height: 100.0,
            },
            distance_m: Some(5.0),
            velocity_ms: None,
            position_3d: None,
            trajectory_id: None,
            attributes: HashMap::new(),
        };
        engine.load_yolo_detections(vec![yolo_det]);

        // Run DINO (finds person that YOLO missed)
        engine.run_dino_detection(&vec![], 1920, 1080);

        // Analyze
        engine.analyze_gaps();

        // Should find gap for person
        assert!(!engine.gaps.is_empty());
    }

    #[test]
    fn test_gap_summary() {
        let config = DINOConfig::default();
        let mut engine = RetrospectiveDetectionEngine::new(config);

        engine.run_dino_detection(&vec![], 1920, 1080);
        engine.analyze_gaps();

        let summary = engine.gap_summary();
        assert!(summary.contains("Detection Gap Analysis"));
    }

    #[test]
    fn test_statistics() {
        let config = DINOConfig::default();
        let mut engine = RetrospectiveDetectionEngine::new(config);

        engine.run_dino_detection(&vec![], 1920, 1080);
        engine.analyze_gaps();

        let stats = engine.get_stats();
        assert!(stats.dino_detections > 0);
    }
}
