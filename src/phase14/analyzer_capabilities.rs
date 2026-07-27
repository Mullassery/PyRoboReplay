//! Extended analyzer capabilities registry for Phase 14
//!
//! Manages analyzer metadata, data requirements, modality preferences,
//! and dynamic enablement based on available data sources.

use crate::phase14::timeline_indexing::Modality;
use crate::phase14::modality_adapters::DataSourceType;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerCapabilitiesV2 {
    /// Analyzer identifier
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Required data sources (must be present)
    pub required_sources: Vec<DataSourceType>,

    /// Optional data sources (enhance output if present)
    pub optional_sources: Vec<DataSourceType>,

    /// Required modalities
    pub required_modalities: Vec<Modality>,

    /// Optional modalities
    pub optional_modalities: Vec<Modality>,

    /// Base confidence with only required sources
    pub base_confidence: f32,

    /// Confidence boost per optional source/modality
    pub confidence_boosts: HashMap<String, f32>,

    /// Whether analyzer wants video context
    pub video_context_required: bool,

    /// Whether analyzer wants video processing (YOLO, optical flow, etc.)
    pub video_processing_needed: Vec<VideoProcessingType>,

    /// Minimum confidence threshold to report findings
    pub min_report_confidence: f32,

    /// Performance characteristics
    pub performance: PerformanceProfile,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VideoProcessingType {
    YOLO,
    OpticalFlow,
    LightingAnalysis,
    DepthEstimation,
    SemanticSegmentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Estimated CPU time in ms per 1000 events
    pub estimated_cpu_ms: f32,

    /// Memory usage in MB
    pub estimated_memory_mb: f32,

    /// Whether analyzer is parallelizable
    pub parallelizable: bool,
}

impl AnalyzerCapabilitiesV2 {
    pub fn new(name: String) -> Self {
        AnalyzerCapabilitiesV2 {
            name,
            description: String::new(),
            required_sources: Vec::new(),
            optional_sources: Vec::new(),
            required_modalities: Vec::new(),
            optional_modalities: Vec::new(),
            base_confidence: 0.5,
            confidence_boosts: HashMap::new(),
            video_context_required: false,
            video_processing_needed: Vec::new(),
            min_report_confidence: 0.5,
            performance: PerformanceProfile {
                estimated_cpu_ms: 10.0,
                estimated_memory_mb: 50.0,
                parallelizable: true,
            },
        }
    }

    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    pub fn with_required_source(mut self, source: DataSourceType) -> Self {
        self.required_sources.push(source);
        self
    }

    pub fn with_optional_source(mut self, source: DataSourceType) -> Self {
        self.optional_sources.push(source);
        self
    }

    pub fn with_required_modality(mut self, modality: Modality) -> Self {
        self.required_modalities.push(modality);
        self
    }

    pub fn with_optional_modality(mut self, modality: Modality) -> Self {
        self.optional_modalities.push(modality);
        self
    }

    pub fn with_base_confidence(mut self, conf: f32) -> Self {
        self.base_confidence = conf.clamp(0.0, 1.0);
        self
    }

    pub fn with_video_context(mut self) -> Self {
        self.video_context_required = true;
        self
    }

    pub fn with_video_processing(mut self, vpt: VideoProcessingType) -> Self {
        self.video_processing_needed.push(vpt);
        self
    }

    pub fn add_confidence_boost(mut self, key: String, boost: f32) -> Self {
        self.confidence_boosts.insert(key, boost.clamp(0.0, 0.5));
        self
    }

    /// Compute final confidence based on available sources
    pub fn compute_confidence(&self, available_sources: &[DataSourceType]) -> f32 {
        let mut confidence = self.base_confidence;

        for optional in &self.optional_sources {
            if available_sources.contains(optional) {
                if let Some(boost) = self.confidence_boosts.get(&format!("{:?}", optional)) {
                    confidence += boost;
                }
            }
        }

        confidence.clamp(0.0, 1.0)
    }

    /// Check if analyzer can run with available sources
    pub fn can_run_with(&self, available_sources: &[DataSourceType]) -> bool {
        self.required_sources.iter()
            .all(|req| available_sources.contains(req))
    }

    /// Get list of missing required sources
    pub fn missing_sources(&self, available_sources: &[DataSourceType]) -> Vec<DataSourceType> {
        self.required_sources.iter()
            .filter(|req| !available_sources.contains(req))
            .copied()
            .collect()
    }

    /// Get suggested improvements
    pub fn improvement_suggestions(&self, available_sources: &[DataSourceType]) -> Vec<String> {
        let mut suggestions = Vec::new();

        for optional in &self.optional_sources {
            if !available_sources.contains(optional) {
                let boost = self.confidence_boosts.get(&format!("{:?}", optional))
                    .unwrap_or(&0.1);
                suggestions.push(format!(
                    "Provide {:?} for +{:.0}% confidence improvement",
                    optional, boost * 100.0
                ));
            }
        }

        suggestions
    }
}

/// Registry of all available analyzers
pub struct AnalyzerRegistry {
    analyzers: HashMap<String, AnalyzerCapabilitiesV2>,
}

impl AnalyzerRegistry {
    pub fn new() -> Self {
        AnalyzerRegistry {
            analyzers: HashMap::new(),
        }
    }

    pub fn register(&mut self, capability: AnalyzerCapabilitiesV2) {
        self.analyzers.insert(capability.name.clone(), capability);
    }

    /// Get analyzers enabled for given sources
    pub fn enabled_for(&self, available_sources: &[DataSourceType]) -> Vec<&AnalyzerCapabilitiesV2> {
        self.analyzers.values()
            .filter(|cap| cap.can_run_with(available_sources))
            .collect()
    }

    /// Get all registered analyzers
    pub fn all(&self) -> Vec<&AnalyzerCapabilitiesV2> {
        self.analyzers.values().collect()
    }

    /// Get analyzer by name
    pub fn get(&self, name: &str) -> Option<&AnalyzerCapabilitiesV2> {
        self.analyzers.get(name)
    }

    /// Get analyzers sorted by confidence for given sources
    pub fn sorted_by_confidence(
        &self,
        available_sources: &[DataSourceType],
    ) -> Vec<(&AnalyzerCapabilitiesV2, f32)> {
        let mut analyzers: Vec<_> = self.enabled_for(available_sources)
            .into_iter()
            .map(|cap| {
                let conf = cap.compute_confidence(available_sources);
                (cap, conf)
            })
            .collect();

        analyzers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        analyzers
    }
}

impl Default for AnalyzerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-configured analysis capability sets
pub struct AnalysisCapability;

impl AnalysisCapability {
    /// Localization analyzer: AMCL divergence, odometry drift, sensor degradation
    pub fn localization() -> AnalyzerCapabilitiesV2 {
        AnalyzerCapabilitiesV2::new("Localization".to_string())
            .with_description("Detects AMCL divergence, odometry drift, sensor degradation".to_string())
            .with_required_source(DataSourceType::RosBag)
            .with_optional_source(DataSourceType::LinuxLogs)
            .with_optional_source(DataSourceType::Video)
            .with_required_modality(Modality::RosBag)
            .with_optional_modality(Modality::LinuxLogs)
            .with_optional_modality(Modality::Video)
            .with_base_confidence(0.70)
            .add_confidence_boost("LinuxLogs".to_string(), 0.15)
            .add_confidence_boost("Video".to_string(), 0.20)
    }

    /// Planner analyzer: oscillation, deadlock, replanning frequency
    pub fn planner() -> AnalyzerCapabilitiesV2 {
        AnalyzerCapabilitiesV2::new("Planner".to_string())
            .with_description("Detects planner oscillation, deadlock, excessive replanning".to_string())
            .with_required_source(DataSourceType::RosBag)
            .with_optional_source(DataSourceType::Nav2Export)
            .with_required_modality(Modality::RosBag)
            .with_base_confidence(0.75)
            .add_confidence_boost("Nav2Export".to_string(), 0.20)
    }

    /// Costmap analyzer: inflation, layer conflicts, false blockages
    pub fn costmap() -> AnalyzerCapabilitiesV2 {
        AnalyzerCapabilitiesV2::new("Costmap".to_string())
            .with_description("Detects costmap inflation issues and layer conflicts".to_string())
            .with_required_source(DataSourceType::RosBag)
            .with_required_source(DataSourceType::Nav2Export)
            .with_optional_source(DataSourceType::Video)
            .with_base_confidence(0.80)
            .add_confidence_boost("Video".to_string(), 0.15)
            .with_video_context()
    }

    /// Dynamic obstacle analyzer: human/obstacle prediction gaps
    pub fn dynamic_obstacles() -> AnalyzerCapabilitiesV2 {
        AnalyzerCapabilitiesV2::new("DynamicObstacles".to_string())
            .with_description("Detects dynamic obstacle conflicts and prediction failures".to_string())
            .with_required_source(DataSourceType::RosBag)
            .with_optional_source(DataSourceType::Video)
            .with_optional_source(DataSourceType::PointCloud)
            .with_base_confidence(0.65)
            .add_confidence_boost("Video".to_string(), 0.25)
            .add_confidence_boost("PointCloud".to_string(), 0.15)
            .with_video_context()
            .with_video_processing(VideoProcessingType::YOLO)
    }

    /// Semantic gap analyzer: where occupancy grids fail
    pub fn semantic_gaps() -> AnalyzerCapabilitiesV2 {
        AnalyzerCapabilitiesV2::new("SemanticGaps".to_string())
            .with_description("Identifies semantic navigation limitations of occupancy grids".to_string())
            .with_required_source(DataSourceType::RosBag)
            .with_required_source(DataSourceType::Video)
            .with_optional_source(DataSourceType::Annotation)
            .with_base_confidence(0.72)
            .add_confidence_boost("Annotation".to_string(), 0.25)
            .with_video_context()
            .with_video_processing(VideoProcessingType::SemanticSegmentation)
    }

    /// Environmental context analyzer: multi-floor, scale, warehouse, etc.
    pub fn environmental_context() -> AnalyzerCapabilitiesV2 {
        AnalyzerCapabilitiesV2::new("EnvironmentalContext".to_string())
            .with_description("Analyzes environment type and scale implications".to_string())
            .with_required_source(DataSourceType::RosBag)
            .with_optional_source(DataSourceType::Video)
            .with_optional_source(DataSourceType::EnvironmentMap)
            .with_base_confidence(0.68)
            .add_confidence_boost("Video".to_string(), 0.20)
            .add_confidence_boost("EnvironmentMap".to_string(), 0.25)
    }

    /// Controller analyzer: stability and tracking error
    pub fn controller() -> AnalyzerCapabilitiesV2 {
        AnalyzerCapabilitiesV2::new("Controller".to_string())
            .with_description("Detects controller instability and tracking errors".to_string())
            .with_required_source(DataSourceType::RosBag)
            .with_optional_source(DataSourceType::LinuxLogs)
            .with_base_confidence(0.70)
            .add_confidence_boost("LinuxLogs".to_string(), 0.15)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_capabilities_creation() {
        let cap = AnalyzerCapabilitiesV2::new("test".to_string());
        assert_eq!(cap.name, "test");
        assert_eq!(cap.base_confidence, 0.5);
    }

    #[test]
    fn test_compute_confidence() {
        let cap = AnalyzerCapabilitiesV2::new("test".to_string())
            .with_base_confidence(0.70)
            .with_optional_source(DataSourceType::Video)
            .add_confidence_boost("Video".to_string(), 0.20);

        let with_video = vec![
            DataSourceType::RosBag,
            DataSourceType::Video,
        ];

        let conf = cap.compute_confidence(&with_video);
        assert!(conf > 0.70);
    }

    #[test]
    fn test_can_run_with_required_sources() {
        let cap = AnalyzerCapabilitiesV2::new("test".to_string())
            .with_required_source(DataSourceType::RosBag)
            .with_required_source(DataSourceType::Nav2Export);

        let sufficient = vec![DataSourceType::RosBag, DataSourceType::Nav2Export];
        assert!(cap.can_run_with(&sufficient));

        let insufficient = vec![DataSourceType::RosBag];
        assert!(!cap.can_run_with(&insufficient));
    }

    #[test]
    fn test_missing_sources() {
        let cap = AnalyzerCapabilitiesV2::new("test".to_string())
            .with_required_source(DataSourceType::RosBag)
            .with_required_source(DataSourceType::Nav2Export);

        let available = vec![DataSourceType::RosBag];
        let missing = cap.missing_sources(&available);
        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&DataSourceType::Nav2Export));
    }

    #[test]
    fn test_analyzer_registry() {
        let mut registry = AnalyzerRegistry::new();
        let cap = AnalysisCapability::localization();
        registry.register(cap);

        assert!(registry.get("Localization").is_some());
    }

    #[test]
    fn test_enabled_analyzers() {
        let mut registry = AnalyzerRegistry::new();
        registry.register(AnalysisCapability::localization());
        registry.register(AnalysisCapability::costmap());

        let sources = vec![DataSourceType::RosBag];
        let enabled = registry.enabled_for(&sources);

        assert_eq!(enabled.len(), 1);  // Only localization runs with just ROS bag
    }

    #[test]
    fn test_sorted_by_confidence() {
        let mut registry = AnalyzerRegistry::new();
        registry.register(AnalysisCapability::localization());
        registry.register(AnalysisCapability::dynamic_obstacles());

        let sources = vec![DataSourceType::RosBag, DataSourceType::Video];
        let sorted = registry.sorted_by_confidence(&sources);

        assert!(!sorted.is_empty());
        // Dynamic obstacles should have higher confidence with video
        if sorted.len() > 1 {
            assert!(sorted[0].1 >= sorted[1].1);
        }
    }
}
