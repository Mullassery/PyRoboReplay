//! Public forensics API for Phase 12 and Phase 13 analyses.
//!
//! These helpers are intentionally data-in/data-out: retrospective DINO runs
//! and thermal capture happen outside this crate, and the resulting artifacts
//! are ingested as JSON. Nothing here fabricates detections.

use crate::fusion::rgb_thermal_fusion::{
    FusedDetection, FusionStatistics, RGBThermalFusionEngine, SensorDisagreement,
    ThermalOnlyDetection,
};
use crate::fusion::thermal_model::ThermalFrame;
use crate::perception::object_detection::DetectedObject;
use crate::perception::retrospective_detection::{
    DINOConfig, DINODetection, DetectionGap, RetrospectiveDetectionEngine,
    RetrospectiveDetectionStats,
};
use serde::{Deserialize, Serialize};

/// Result of comparing externally generated DINO detections against the
/// robot's own YOLO detections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionGapAnalysis {
    pub stats: RetrospectiveDetectionStats,
    pub gaps: Vec<DetectionGap>,
}

/// Result of fusing externally generated RGB detections with a thermal frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionAnalysis {
    pub stats: FusionStatistics,
    pub fused_detections: Vec<FusedDetection>,
    pub thermal_only: Vec<ThermalOnlyDetection>,
    pub disagreements: Vec<SensorDisagreement>,
}

/// Analyze detection gaps using DINO detections produced outside this crate.
pub fn analyze_detection_gaps(
    yolo_detections: Vec<DetectedObject>,
    dino_detections: Vec<DINODetection>,
) -> DetectionGapAnalysis {
    let mut engine = RetrospectiveDetectionEngine::new(DINOConfig::default());
    engine.load_yolo_detections(yolo_detections);
    engine.load_dino_detections(dino_detections);
    engine.analyze_gaps();

    DetectionGapAnalysis {
        stats: engine.get_stats(),
        gaps: engine.gaps.clone(),
    }
}

/// Fuse RGB detections with a thermal frame.
pub fn fuse_rgb_thermal(
    rgb_detections: Vec<DetectedObject>,
    thermal_frame: ThermalFrame,
) -> FusionAnalysis {
    let mut engine = RGBThermalFusionEngine::new();
    engine.load_rgb_detections(rgb_detections);
    engine.load_thermal_frame(thermal_frame);
    engine.fuse();

    FusionAnalysis {
        stats: engine.get_statistics(),
        fused_detections: engine.fused_detections.clone(),
        thermal_only: engine.thermal_only.clone(),
        disagreements: engine.disagreements.clone(),
    }
}

/// Render a compact human-readable summary of a gap analysis.
pub fn gap_analysis_text(analysis: &DetectionGapAnalysis) -> String {
    let mut text = format!(
        "Detection gaps: {} of {} retrospective detections ({} critical)\n",
        analysis.stats.detection_gaps, analysis.stats.dino_detections, analysis.stats.critical_gaps
    );
    for (idx, gap) in analysis.gaps.iter().enumerate() {
        text.push_str(&format!(
            "  {}. {} ({:.0}% confidence, severity {:.0}%)\n     {}\n",
            idx + 1,
            gap.dino_detection.class_name,
            gap.dino_detection.confidence * 100.0,
            gap.severity * 100.0,
            gap.recommendation
        ));
    }
    text
}

/// Render a compact human-readable summary of a fusion analysis.
pub fn fusion_analysis_text(analysis: &FusionAnalysis) -> String {
    format!(
        "RGB/thermal fusion: {} RGB detections, {} thermal-only, {} fused\n\
         RGB miss rate: {:.1}% | confidence improvement: {:+.1} pp\n",
        analysis.stats.rgb_detections,
        analysis.stats.thermal_only_detections,
        analysis.stats.fused_detections,
        analysis.stats.rgb_miss_rate * 100.0,
        analysis.stats.confidence_improvement * 100.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fusion::thermal_model::{ThermalCameraConfig, ThermalFrame};
    use crate::perception::object_detection::{BoundingBox, ObjectClass};
    use std::collections::HashMap;

    fn rgb(id: u32, class: ObjectClass, x: f32, y: f32) -> DetectedObject {
        DetectedObject {
            id,
            class,
            confidence: 0.8,
            bbox: BoundingBox {
                x,
                y,
                width: 20.0,
                height: 20.0,
            },
            distance_m: Some(2.0),
            velocity_ms: None,
            position_3d: None,
            trajectory_id: None,
            attributes: HashMap::new(),
        }
    }

    #[test]
    fn gap_analysis_uses_loaded_dino_detections() {
        let analysis = analyze_detection_gaps(
            vec![rgb(1, ObjectClass::Person, 10.0, 10.0)],
            vec![DINODetection {
                class_name: "person".to_string(),
                bbox: BoundingBox {
                    x: 10.0,
                    y: 10.0,
                    width: 20.0,
                    height: 20.0,
                },
                confidence: 0.9,
                distance_m: Some(2.0),
            }],
        );
        assert_eq!(analysis.stats.dino_detections, 1);
        assert_eq!(analysis.stats.detection_gaps, 0);
    }

    #[test]
    fn gap_analysis_flags_missed_object() {
        let analysis = analyze_detection_gaps(
            vec![rgb(1, ObjectClass::Person, 10.0, 10.0)],
            vec![DINODetection {
                class_name: "pallet".to_string(),
                bbox: BoundingBox {
                    x: 500.0,
                    y: 400.0,
                    width: 80.0,
                    height: 40.0,
                },
                confidence: 0.85,
                distance_m: Some(5.0),
            }],
        );
        assert_eq!(analysis.stats.detection_gaps, 1);
        assert!(!gap_analysis_text(&analysis).is_empty());
    }

    #[test]
    fn fusion_handles_thermal_only_hotspot() {
        let mut thermal_data = vec![300.0; 64 * 64];
        for y in 10..20 {
            for x in 10..20 {
                thermal_data[y * 64 + x] = 310.0;
            }
        }
        let frame = ThermalFrame {
            timestamp_sec: 1.0,
            frame_index: 0,
            thermal_data,
            width: 64,
            height: 64,
            config: ThermalCameraConfig::default(),
        };
        let analysis = fuse_rgb_thermal(Vec::new(), frame);
        assert_eq!(analysis.stats.rgb_detections, 0);
        assert_eq!(analysis.stats.thermal_only_detections, 1);
        assert!(!fusion_analysis_text(&analysis).is_empty());
    }
}
