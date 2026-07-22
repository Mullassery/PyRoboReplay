//! Detection Orchestrator: Multi-Backend Fallback Strategy
//!
//! Coordinates detection backends with automatic fallback:
//! 1. Try YOLO (fast, requires model)
//! 2. Fall back to SAM (slower, zero-shot)
//! 3. Fall back to Template (offline, hardcoded)
//!
//! Enables graceful degradation without deployment complexity.

use crate::perception::detection_backends::{
    DetectionBackend, DetectionBackendType, TemplateBackend, YOLOBackend, YOLOConfig,
};
use crate::perception::object_detection::DetectionFrame;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Detection provider strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DetectionStrategy {
    /// Try YOLO first, fall back to SAM
    YOLOPrimary,
    /// Try SAM first (for zero-shot), fall back to YOLO
    SAMPrimary,
    /// Use template fallback only
    TemplateOnly,
    /// Try all in order
    Cascade,
}

/// Detection orchestrator with fallback
pub struct DetectionOrchestrator {
    /// Primary backend
    primary: Option<DetectionBackendType>,
    /// Secondary backend
    secondary: Option<DetectionBackendType>,
    /// Fallback template backend
    fallback: TemplateBackend,
    /// Strategy
    strategy: DetectionStrategy,
    /// Track successful backend usage
    successful_detections_by_backend: std::collections::HashMap<String, usize>,
    /// Track inference times
    inference_times: Vec<f32>,
}

impl DetectionOrchestrator {
    /// Create orchestrator with YOLO primary
    pub fn new_yolo_primary() -> Self {
        let yolo_config = YOLOConfig::default();
        let yolo_backend = YOLOBackend::new(yolo_config);

        DetectionOrchestrator {
            primary: Some(DetectionBackendType::YOLO(yolo_backend)),
            secondary: None,
            fallback: TemplateBackend::new(),
            strategy: DetectionStrategy::YOLOPrimary,
            successful_detections_by_backend: std::collections::HashMap::new(),
            inference_times: Vec::new(),
        }
    }

    /// Create orchestrator with SAM primary
    pub fn new_sam_primary() -> Self {
        let sam_config = crate::perception::detection_backends::SAMConfig::default();
        let sam_backend =
            crate::perception::detection_backends::SAMBackend::new(sam_config);

        DetectionOrchestrator {
            primary: Some(DetectionBackendType::SAM(sam_backend)),
            secondary: None,
            fallback: TemplateBackend::new(),
            strategy: DetectionStrategy::SAMPrimary,
            successful_detections_by_backend: std::collections::HashMap::new(),
            inference_times: Vec::new(),
        }
    }

    /// Create orchestrator with template fallback only
    pub fn new_template_only() -> Self {
        DetectionOrchestrator {
            primary: None,
            secondary: None,
            fallback: TemplateBackend::new(),
            strategy: DetectionStrategy::TemplateOnly,
            successful_detections_by_backend: std::collections::HashMap::new(),
            inference_times: Vec::new(),
        }
    }

    /// Set primary backend
    pub fn with_primary(mut self, backend: DetectionBackendType) -> Self {
        self.primary = Some(backend);
        self
    }

    /// Set secondary backend
    pub fn with_secondary(mut self, backend: DetectionBackendType) -> Self {
        self.secondary = Some(backend);
        self
    }

    /// Register template scenario
    pub fn register_template(
        &mut self,
        scenario_id: &str,
        detections: Vec<crate::perception::object_detection::DetectedObject>,
    ) {
        self.fallback.register_template(scenario_id, detections);
    }

    /// Detect with fallback strategy
    pub fn detect_with_fallback(
        &mut self,
        image_data: &[u8],
        width: u32,
        height: u32,
        timestamp_sec: f32,
        frame_index: usize,
    ) -> DetectionFrame {
        let start = Instant::now();

        let result = match self.strategy {
            DetectionStrategy::YOLOPrimary => self.try_yolo_then_fallback(
                image_data,
                width,
                height,
                timestamp_sec,
                frame_index,
            ),
            DetectionStrategy::SAMPrimary => self.try_sam_then_fallback(
                image_data,
                width,
                height,
                timestamp_sec,
                frame_index,
            ),
            DetectionStrategy::TemplateOnly => self.fallback.detect(
                image_data,
                width,
                height,
                timestamp_sec,
                frame_index,
            ),
            DetectionStrategy::Cascade => self.try_cascade(
                image_data,
                width,
                height,
                timestamp_sec,
                frame_index,
            ),
        };

        let elapsed = start.elapsed().as_secs_f32() * 1000.0;
        self.inference_times.push(elapsed);

        let backend_name = result.metadata.detector_model.clone();
        *self
            .successful_detections_by_backend
            .entry(backend_name)
            .or_insert(0) += 1;

        result
    }

    fn try_yolo_then_fallback(
        &mut self,
        image_data: &[u8],
        width: u32,
        height: u32,
        timestamp_sec: f32,
        frame_index: usize,
    ) -> DetectionFrame {
        if let Some(backend) = &self.primary {
            let frame = backend
                .as_backend()
                .detect(image_data, width, height, timestamp_sec, frame_index);
            if !frame.objects.is_empty() {
                return frame;
            }
        }

        if let Some(backend) = &self.secondary {
            let frame = backend
                .as_backend()
                .detect(image_data, width, height, timestamp_sec, frame_index);
            if !frame.objects.is_empty() {
                return frame;
            }
        }

        // Fall back to template
        self.fallback.detect(image_data, width, height, timestamp_sec, frame_index)
    }

    fn try_sam_then_fallback(
        &mut self,
        image_data: &[u8],
        width: u32,
        height: u32,
        timestamp_sec: f32,
        frame_index: usize,
    ) -> DetectionFrame {
        if let Some(backend) = &self.primary {
            let frame = backend
                .as_backend()
                .detect(image_data, width, height, timestamp_sec, frame_index);
            if !frame.objects.is_empty() {
                return frame;
            }
        }

        // SAM primary should fall back to YOLO/template
        if let Some(backend) = &self.secondary {
            let frame = backend
                .as_backend()
                .detect(image_data, width, height, timestamp_sec, frame_index);
            if !frame.objects.is_empty() {
                return frame;
            }
        }

        // Fall back to template
        self.fallback.detect(image_data, width, height, timestamp_sec, frame_index)
    }

    fn try_cascade(
        &mut self,
        image_data: &[u8],
        width: u32,
        height: u32,
        timestamp_sec: f32,
        frame_index: usize,
    ) -> DetectionFrame {
        // Try primary
        if let Some(backend) = &self.primary {
            let frame = backend
                .as_backend()
                .detect(image_data, width, height, timestamp_sec, frame_index);
            if !frame.objects.is_empty() {
                return frame;
            }
        }

        // Try secondary
        if let Some(backend) = &self.secondary {
            let frame = backend
                .as_backend()
                .detect(image_data, width, height, timestamp_sec, frame_index);
            if !frame.objects.is_empty() {
                return frame;
            }
        }

        // Fall back to template
        self.fallback.detect(image_data, width, height, timestamp_sec, frame_index)
    }

    /// Get detection statistics
    pub fn get_stats(&self) -> DetectionOrchestrationStats {
        let total_inferences = self.inference_times.len();
        let avg_inference_time = if total_inferences > 0 {
            self.inference_times.iter().sum::<f32>() / total_inferences as f32
        } else {
            0.0
        };

        let max_inference_time = self
            .inference_times
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);

        DetectionOrchestrationStats {
            total_inferences,
            avg_inference_time_ms: avg_inference_time,
            max_inference_time_ms: max_inference_time,
            detections_by_backend: self.successful_detections_by_backend.clone(),
            strategy: self.strategy.clone(),
        }
    }

    /// Get backend usage summary
    pub fn backend_summary(&self) -> String {
        let mut summary = format!("Strategy: {:?}\n", self.strategy);
        summary.push_str("Backend usage:\n");

        for (backend, count) in &self.successful_detections_by_backend {
            summary.push_str(&format!("  {}: {} detections\n", backend, count));
        }

        summary.push_str(&format!(
            "Avg inference: {:.1}ms\n",
            self.get_stats().avg_inference_time_ms
        ));

        summary
    }
}

/// Orchestration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionOrchestrationStats {
    /// Total inferences executed
    pub total_inferences: usize,
    /// Average inference time (milliseconds)
    pub avg_inference_time_ms: f32,
    /// Maximum inference time (milliseconds)
    pub max_inference_time_ms: f32,
    /// Detections by backend
    pub detections_by_backend: std::collections::HashMap<String, usize>,
    /// Strategy used
    pub strategy: DetectionStrategy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_yolo_primary() {
        let orchestrator = DetectionOrchestrator::new_yolo_primary();
        assert_eq!(orchestrator.strategy, DetectionStrategy::YOLOPrimary);
    }

    #[test]
    fn test_orchestrator_sam_primary() {
        let orchestrator = DetectionOrchestrator::new_sam_primary();
        assert_eq!(orchestrator.strategy, DetectionStrategy::SAMPrimary);
    }

    #[test]
    fn test_orchestrator_template_only() {
        let orchestrator = DetectionOrchestrator::new_template_only();
        assert_eq!(orchestrator.strategy, DetectionStrategy::TemplateOnly);
    }

    #[test]
    fn test_detect_with_fallback() {
        let mut orchestrator = DetectionOrchestrator::new_template_only();
        let image_data = vec![0u8; 1920 * 1080 * 3];

        let frame = orchestrator.detect_with_fallback(&image_data, 1920, 1080, 100.0, 0);
        assert_eq!(frame.metadata.detector_model, "template-fallback");
    }

    #[test]
    fn test_statistics_tracking() {
        let mut orchestrator = DetectionOrchestrator::new_template_only();
        let image_data = vec![0u8; 1920 * 1080 * 3];

        for i in 0..5 {
            orchestrator.detect_with_fallback(
                &image_data,
                1920,
                1080,
                100.0 + (i as f32),
                i,
            );
        }

        let stats = orchestrator.get_stats();
        assert_eq!(stats.total_inferences, 5);
        assert!(stats.avg_inference_time_ms > 0.0);
    }

    #[test]
    fn test_backend_summary() {
        let mut orchestrator = DetectionOrchestrator::new_template_only();
        let image_data = vec![0u8; 1920 * 1080 * 3];

        orchestrator.detect_with_fallback(&image_data, 1920, 1080, 100.0, 0);

        let summary = orchestrator.backend_summary();
        assert!(summary.contains("Strategy"));
        assert!(summary.contains("Backend usage"));
    }

    #[test]
    fn test_cascade_strategy() {
        let orchestrator = DetectionOrchestrator::new_yolo_primary();
        assert_eq!(orchestrator.strategy, DetectionStrategy::YOLOPrimary);

        // Could test cascade with multiple backends
    }
}
