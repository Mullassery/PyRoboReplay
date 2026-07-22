//! Object Detection Layer
//!
//! Identifies and classifies objects in camera frames.
//! Extracts spatial, confidence, and tracking information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Object class taxonomy for robot perception
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectClass {
    Person,
    Vehicle,
    Bicycle,
    Animal,
    TrafficCone,
    Pallet,
    Forklift,
    Machinery,
    Tool,
    StaticObstacle,
    DynamicObstacle,
    Unknown,
}

impl std::fmt::Display for ObjectClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ObjectClass::Person => write!(f, "Person"),
            ObjectClass::Vehicle => write!(f, "Vehicle"),
            ObjectClass::Bicycle => write!(f, "Bicycle"),
            ObjectClass::Animal => write!(f, "Animal"),
            ObjectClass::TrafficCone => write!(f, "TrafficCone"),
            ObjectClass::Pallet => write!(f, "Pallet"),
            ObjectClass::Forklift => write!(f, "Forklift"),
            ObjectClass::Machinery => write!(f, "Machinery"),
            ObjectClass::Tool => write!(f, "Tool"),
            ObjectClass::StaticObstacle => write!(f, "StaticObstacle"),
            ObjectClass::DynamicObstacle => write!(f, "DynamicObstacle"),
            ObjectClass::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Bounding box for detected object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    /// X coordinate (pixels or normalized)
    pub x: f32,

    /// Y coordinate (pixels or normalized)
    pub y: f32,

    /// Width (pixels or normalized)
    pub width: f32,

    /// Height (pixels or normalized)
    pub height: f32,
}

/// Single detected object in a frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedObject {
    /// Unique ID within this frame
    pub id: u32,

    /// Object class/type
    pub class: ObjectClass,

    /// Detection confidence (0.0-1.0)
    pub confidence: f32,

    /// Bounding box in image space
    pub bbox: BoundingBox,

    /// Distance from robot (meters, if available)
    pub distance_m: Option<f32>,

    /// Estimated velocity (m/s, if available)
    pub velocity_ms: Option<f32>,

    /// 3D position relative to robot (x, y, z)
    pub position_3d: Option<(f32, f32, f32)>,

    /// Object trajectory ID (for tracking)
    pub trajectory_id: Option<u32>,

    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// All detections in a single camera frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionFrame {
    /// Frame timestamp (seconds or unix time)
    pub timestamp_sec: f32,

    /// Frame number/index
    pub frame_index: usize,

    /// Camera identifier
    pub camera_id: String,

    /// All detected objects in this frame
    pub objects: Vec<DetectedObject>,

    /// Frame metadata
    pub metadata: FrameMetadata,
}

/// Metadata about a detection frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetadata {
    /// Image width (pixels)
    pub width: u32,

    /// Image height (pixels)
    pub height: u32,

    /// Detection model used
    pub detector_model: String,

    /// Inference time (milliseconds)
    pub inference_time_ms: f32,

    /// Frame quality score (0.0-1.0)
    pub quality_score: f32,

    /// Environmental conditions affecting detection
    pub environmental_factors: HashMap<String, f32>,
}

/// Detects objects in camera frames
pub struct ObjectDetector {
    /// Model name/version
    pub model_id: String,

    /// Confidence threshold (0.0-1.0)
    pub confidence_threshold: f32,

    /// Maximum detections per frame
    pub max_detections: usize,

    /// Supported classes
    pub supported_classes: Vec<ObjectClass>,
}

impl ObjectDetector {
    /// Create detector with defaults
    pub fn new(model_id: &str) -> Self {
        ObjectDetector {
            model_id: model_id.to_string(),
            confidence_threshold: 0.5,
            max_detections: 100,
            supported_classes: vec![
                ObjectClass::Person,
                ObjectClass::Vehicle,
                ObjectClass::Bicycle,
                ObjectClass::Animal,
                ObjectClass::TrafficCone,
                ObjectClass::Pallet,
                ObjectClass::Forklift,
                ObjectClass::Machinery,
                ObjectClass::StaticObstacle,
                ObjectClass::DynamicObstacle,
            ],
        }
    }

    /// Process a frame (stub - actual inference would happen here)
    pub fn detect(&self, frame_index: usize, timestamp_sec: f32) -> DetectionFrame {
        DetectionFrame {
            timestamp_sec,
            frame_index,
            camera_id: "front_camera".to_string(),
            objects: Vec::new(),
            metadata: FrameMetadata {
                width: 1920,
                height: 1080,
                detector_model: self.model_id.clone(),
                inference_time_ms: 45.0,
                quality_score: 0.95,
                environmental_factors: HashMap::new(),
            },
        }
    }

    /// Filter detections by confidence threshold
    pub fn filter_by_confidence(&self, frame: &mut DetectionFrame) {
        frame.objects.retain(|obj| obj.confidence >= self.confidence_threshold);
    }

    /// Limit detections to max_detections (keep highest confidence)
    pub fn limit_detections(&self, frame: &mut DetectionFrame) {
        if frame.objects.len() > self.max_detections {
            frame.objects.sort_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            frame.objects.truncate(self.max_detections);
        }
    }
}

/// Statistics about detections across multiple frames
#[derive(Debug, Clone)]
pub struct DetectionStatistics {
    /// Total frames processed
    pub total_frames: usize,

    /// Total objects detected
    pub total_detections: usize,

    /// Detection rate by class
    pub detections_by_class: HashMap<ObjectClass, usize>,

    /// Average confidence by class
    pub avg_confidence_by_class: HashMap<ObjectClass, f32>,

    /// Frames with no detections
    pub empty_frames: usize,

    /// Detection rate (detections / frames)
    pub detection_rate: f32,
}

impl DetectionStatistics {
    /// Compute statistics from frames
    pub fn from_frames(frames: &[DetectionFrame]) -> Self {
        let total_frames = frames.len();
        let mut total_detections = 0;
        let mut detections_by_class: HashMap<ObjectClass, usize> = HashMap::new();
        let mut confidence_sums: HashMap<ObjectClass, f32> = HashMap::new();
        let mut confidence_counts: HashMap<ObjectClass, usize> = HashMap::new();
        let mut empty_frames = 0;

        for frame in frames {
            if frame.objects.is_empty() {
                empty_frames += 1;
            }

            for obj in &frame.objects {
                total_detections += 1;
                *detections_by_class.entry(obj.class).or_insert(0) += 1;
                *confidence_sums.entry(obj.class).or_insert(0.0) += obj.confidence;
                *confidence_counts.entry(obj.class).or_insert(0) += 1;
            }
        }

        let mut avg_confidence_by_class = HashMap::new();
        for (class, sum) in confidence_sums {
            if let Some(count) = confidence_counts.get(&class) {
                avg_confidence_by_class.insert(class, sum / *count as f32);
            }
        }

        let detection_rate = if total_frames > 0 {
            total_detections as f32 / total_frames as f32
        } else {
            0.0
        };

        DetectionStatistics {
            total_frames,
            total_detections,
            detections_by_class,
            avg_confidence_by_class,
            empty_frames,
            detection_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_detector_creation() {
        let detector = ObjectDetector::new("yolov8");
        assert_eq!(detector.model_id, "yolov8");
        assert!(detector.supported_classes.len() > 0);
    }

    #[test]
    fn test_detection_frame_creation() {
        let frame = DetectionFrame {
            timestamp_sec: 100.5,
            frame_index: 42,
            camera_id: "front".to_string(),
            objects: vec![DetectedObject {
                id: 1,
                class: ObjectClass::Person,
                confidence: 0.95,
                bbox: BoundingBox {
                    x: 100.0,
                    y: 200.0,
                    width: 50.0,
                    height: 100.0,
                },
                distance_m: Some(2.5),
                velocity_ms: Some(1.2),
                position_3d: Some((2.0, 0.5, 0.0)),
                trajectory_id: Some(1),
                attributes: HashMap::new(),
            }],
            metadata: FrameMetadata {
                width: 1920,
                height: 1080,
                detector_model: "yolov8".to_string(),
                inference_time_ms: 45.0,
                quality_score: 0.95,
                environmental_factors: HashMap::new(),
            },
        };

        assert_eq!(frame.objects.len(), 1);
        assert_eq!(frame.objects[0].class, ObjectClass::Person);
    }

    #[test]
    fn test_detection_statistics() {
        let frames = vec![
            DetectionFrame {
                timestamp_sec: 0.0,
                frame_index: 0,
                camera_id: "front".to_string(),
                objects: vec![
                    DetectedObject {
                        id: 1,
                        class: ObjectClass::Person,
                        confidence: 0.95,
                        bbox: BoundingBox {
                            x: 100.0,
                            y: 200.0,
                            width: 50.0,
                            height: 100.0,
                        },
                        distance_m: None,
                        velocity_ms: None,
                        position_3d: None,
                        trajectory_id: None,
                        attributes: HashMap::new(),
                    },
                    DetectedObject {
                        id: 2,
                        class: ObjectClass::Vehicle,
                        confidence: 0.87,
                        bbox: BoundingBox {
                            x: 500.0,
                            y: 300.0,
                            width: 200.0,
                            height: 150.0,
                        },
                        distance_m: None,
                        velocity_ms: None,
                        position_3d: None,
                        trajectory_id: None,
                        attributes: HashMap::new(),
                    },
                ],
                metadata: FrameMetadata {
                    width: 1920,
                    height: 1080,
                    detector_model: "yolov8".to_string(),
                    inference_time_ms: 45.0,
                    quality_score: 0.95,
                    environmental_factors: HashMap::new(),
                },
            },
            DetectionFrame {
                timestamp_sec: 1.0,
                frame_index: 1,
                camera_id: "front".to_string(),
                objects: vec![],
                metadata: FrameMetadata {
                    width: 1920,
                    height: 1080,
                    detector_model: "yolov8".to_string(),
                    inference_time_ms: 45.0,
                    quality_score: 0.95,
                    environmental_factors: HashMap::new(),
                },
            },
        ];

        let stats = DetectionStatistics::from_frames(&frames);

        assert_eq!(stats.total_frames, 2);
        assert_eq!(stats.total_detections, 2);
        assert_eq!(stats.empty_frames, 1);
        assert_eq!(stats.detection_rate, 1.0); // 2 detections / 2 frames
    }
}
