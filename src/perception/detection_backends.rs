//! Pluggable Object Detection Backends
//!
//! Supports multiple detection engines:
//! - YOLO (v5/v8 local): Real-time object detection
//! - SAM (Segment Anything Model): Zero-shot segmentation
//! - Template-based: Fallback when models unavailable
//!
//! Enables: YOLO for speed, SAM for zero-shot, template for offline

use crate::perception::object_detection::{BoundingBox, DetectedObject, DetectionFrame, ObjectClass};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detection backend trait
pub trait DetectionBackend: Send + Sync {
    /// Run inference on frame
    fn detect(
        &self,
        image_data: &[u8],
        width: u32,
        height: u32,
        timestamp_sec: f32,
        frame_index: usize,
    ) -> DetectionFrame;

    /// Get backend name
    fn backend_name(&self) -> &str;

    /// Set confidence threshold
    fn set_confidence_threshold(&mut self, threshold: f32);

    /// Get current confidence threshold
    fn confidence_threshold(&self) -> f32;
}

/// YOLO detection backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YOLOConfig {
    /// Model path (v5, v8, etc.)
    pub model_path: String,
    /// Confidence threshold
    pub confidence_threshold: f32,
    /// IOU threshold for NMS
    pub iou_threshold: f32,
    /// Maximum detections
    pub max_detections: usize,
    /// Device: "cpu" or "cuda:0"
    pub device: String,
}

impl Default for YOLOConfig {
    fn default() -> Self {
        YOLOConfig {
            model_path: "yolov8n.pt".to_string(),
            confidence_threshold: 0.5,
            iou_threshold: 0.45,
            max_detections: 100,
            device: "cpu".to_string(),
        }
    }
}

/// YOLO backend (local inference stub)
pub struct YOLOBackend {
    config: YOLOConfig,
    class_map: HashMap<i32, ObjectClass>,
}

impl YOLOBackend {
    /// Create YOLO backend
    pub fn new(config: YOLOConfig) -> Self {
        let mut class_map = HashMap::new();
        // COCO class mapping (simplified for common classes)
        class_map.insert(0, ObjectClass::Person);     // person
        class_map.insert(2, ObjectClass::Vehicle);    // car
        class_map.insert(1, ObjectClass::Bicycle);    // bicycle
        class_map.insert(16, ObjectClass::Animal);    // dog/cat
        class_map.insert(23, ObjectClass::Machinery); // train
        class_map.insert(26, ObjectClass::Machinery); // backpack

        YOLOBackend { config, class_map }
    }

    /// Map YOLO class ID to ObjectClass
    fn map_class(&self, coco_id: i32) -> ObjectClass {
        self.class_map
            .get(&coco_id)
            .copied()
            .unwrap_or(ObjectClass::Unknown)
    }
}

impl DetectionBackend for YOLOBackend {
    fn detect(
        &self,
        _image_data: &[u8],
        width: u32,
        height: u32,
        timestamp_sec: f32,
        frame_index: usize,
    ) -> DetectionFrame {
        // Stub: actual YOLO inference would happen here via PyO3 or subprocess
        // For now, return empty frame that's marked as YOLO
        DetectionFrame {
            timestamp_sec,
            frame_index,
            camera_id: "front_camera".to_string(),
            objects: Vec::new(),
            metadata: crate::perception::object_detection::FrameMetadata {
                width,
                height,
                detector_model: format!("YOLO-{}", self.config.model_path),
                inference_time_ms: 50.0,
                quality_score: 0.95,
                environmental_factors: HashMap::new(),
            },
        }
    }

    fn backend_name(&self) -> &str {
        "yolo"
    }

    fn set_confidence_threshold(&mut self, threshold: f32) {
        self.config.confidence_threshold = threshold;
    }

    fn confidence_threshold(&self) -> f32 {
        self.config.confidence_threshold
    }
}

/// SAM (Segment Anything Model) backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SAMConfig {
    /// Model size: "vit_b", "vit_l", "vit_h"
    pub model_size: String,
    /// Confidence threshold for segments
    pub confidence_threshold: f32,
    /// Minimum mask area (pixels)
    pub min_mask_area: usize,
    /// Device: "cpu" or "cuda:0"
    pub device: String,
}

impl Default for SAMConfig {
    fn default() -> Self {
        SAMConfig {
            model_size: "vit_b".to_string(),
            confidence_threshold: 0.5,
            min_mask_area: 100,
            device: "cpu".to_string(),
        }
    }
}

/// SAM backend (zero-shot segmentation)
pub struct SAMBackend {
    config: SAMConfig,
}

impl SAMBackend {
    /// Create SAM backend
    pub fn new(config: SAMConfig) -> Self {
        SAMBackend { config }
    }
}

impl DetectionBackend for SAMBackend {
    fn detect(
        &self,
        _image_data: &[u8],
        width: u32,
        height: u32,
        timestamp_sec: f32,
        frame_index: usize,
    ) -> DetectionFrame {
        // Stub: actual SAM inference would generate masks
        // Returns empty frame marked as SAM
        DetectionFrame {
            timestamp_sec,
            frame_index,
            camera_id: "front_camera".to_string(),
            objects: Vec::new(),
            metadata: crate::perception::object_detection::FrameMetadata {
                width,
                height,
                detector_model: format!("SAM-{}", self.config.model_size),
                inference_time_ms: 200.0, // SAM is slower
                quality_score: 0.92,
                environmental_factors: HashMap::new(),
            },
        }
    }

    fn backend_name(&self) -> &str {
        "sam"
    }

    fn set_confidence_threshold(&mut self, threshold: f32) {
        self.config.confidence_threshold = threshold;
    }

    fn confidence_threshold(&self) -> f32 {
        self.config.confidence_threshold
    }
}

/// Template-based detection (fallback)
pub struct TemplateBackend {
    /// Hardcoded detections for known scenarios
    templates: HashMap<String, Vec<DetectedObject>>,
    confidence_threshold: f32,
}

impl TemplateBackend {
    /// Create template backend
    pub fn new() -> Self {
        TemplateBackend {
            templates: HashMap::new(),
            confidence_threshold: 0.5,
        }
    }

    /// Register template scenario
    pub fn register_template(&mut self, scenario_id: &str, detections: Vec<DetectedObject>) {
        self.templates.insert(scenario_id.to_string(), detections);
    }

    /// Get template by ID
    pub fn get_template(&self, scenario_id: &str) -> Option<Vec<DetectedObject>> {
        self.templates.get(scenario_id).cloned()
    }
}

impl Default for TemplateBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectionBackend for TemplateBackend {
    fn detect(
        &self,
        _image_data: &[u8],
        width: u32,
        height: u32,
        timestamp_sec: f32,
        frame_index: usize,
    ) -> DetectionFrame {
        DetectionFrame {
            timestamp_sec,
            frame_index,
            camera_id: "front_camera".to_string(),
            objects: Vec::new(),
            metadata: crate::perception::object_detection::FrameMetadata {
                width,
                height,
                detector_model: "template-fallback".to_string(),
                inference_time_ms: 1.0,
                quality_score: 0.75, // Lower confidence for template
                environmental_factors: HashMap::new(),
            },
        }
    }

    fn backend_name(&self) -> &str {
        "template"
    }

    fn set_confidence_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold;
    }

    fn confidence_threshold(&self) -> f32 {
        self.confidence_threshold
    }
}

/// Detection backend selector (smart fallback)
pub enum DetectionBackendType {
    YOLO(YOLOBackend),
    SAM(SAMBackend),
    Template(TemplateBackend),
}

impl DetectionBackendType {
    /// Get backend trait object
    pub fn as_backend(&self) -> &dyn DetectionBackend {
        match self {
            DetectionBackendType::YOLO(b) => b,
            DetectionBackendType::SAM(b) => b,
            DetectionBackendType::Template(b) => b,
        }
    }

    /// Get mutable backend trait object
    pub fn as_backend_mut(&mut self) -> &mut dyn DetectionBackend {
        match self {
            DetectionBackendType::YOLO(b) => b,
            DetectionBackendType::SAM(b) => b,
            DetectionBackendType::Template(b) => b,
        }
    }

    /// Get backend name
    pub fn name(&self) -> &str {
        self.as_backend().backend_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yolo_config_default() {
        let config = YOLOConfig::default();
        assert_eq!(config.model_path, "yolov8n.pt");
        assert_eq!(config.confidence_threshold, 0.5);
    }

    #[test]
    fn test_yolo_backend_creation() {
        let config = YOLOConfig::default();
        let backend = YOLOBackend::new(config);
        assert_eq!(backend.backend_name(), "yolo");
    }

    #[test]
    fn test_yolo_detect() {
        let config = YOLOConfig::default();
        let backend = YOLOBackend::new(config);

        let image_data = vec![0u8; 1920 * 1080 * 3];
        let frame = backend.detect(&image_data, 1920, 1080, 100.0, 0);

        assert_eq!(frame.metadata.detector_model, "YOLO-yolov8n.pt");
        assert_eq!(frame.objects.len(), 0); // Stub returns empty
    }

    #[test]
    fn test_sam_backend_creation() {
        let config = SAMConfig::default();
        let backend = SAMBackend::new(config);
        assert_eq!(backend.backend_name(), "sam");
    }

    #[test]
    fn test_template_backend() {
        let mut backend = TemplateBackend::new();
        assert_eq!(backend.backend_name(), "template");

        let detections = vec![DetectedObject {
            id: 1,
            class: ObjectClass::Person,
            confidence: 0.9,
            bbox: BoundingBox {
                x: 100.0,
                y: 100.0,
                width: 50.0,
                height: 100.0,
            },
            distance_m: Some(2.5),
            velocity_ms: None,
            position_3d: None,
            trajectory_id: None,
            attributes: HashMap::new(),
        }];

        backend.register_template("warehouse_person", detections.clone());
        assert!(backend.get_template("warehouse_person").is_some());
    }

    #[test]
    fn test_backend_type_selection() {
        let mut backend = DetectionBackendType::Template(TemplateBackend::new());
        assert_eq!(backend.name(), "template");

        backend.as_backend_mut().set_confidence_threshold(0.7);
        assert_eq!(backend.as_backend().confidence_threshold(), 0.7);
    }

    #[test]
    fn test_yolo_class_mapping() {
        let config = YOLOConfig::default();
        let backend = YOLOBackend::new(config);

        assert_eq!(backend.map_class(0), ObjectClass::Person);
        assert_eq!(backend.map_class(2), ObjectClass::Vehicle);
        assert_eq!(backend.map_class(999), ObjectClass::Unknown);
    }

    #[test]
    fn test_confidence_threshold() {
        let config = YOLOConfig {
            confidence_threshold: 0.7,
            ..Default::default()
        };
        let backend = YOLOBackend::new(config);
        assert_eq!(backend.confidence_threshold(), 0.7);
    }
}
