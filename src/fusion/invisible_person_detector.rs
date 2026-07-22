//! Invisible Person Detection
//!
//! Identifies scenarios where people were present but difficult to detect
//! using RGB imagery alone. Uses thermal evidence and contextual reasoning.

use crate::fusion::thermal_model::{ThermalFrame, ThermalSource};
use serde::{Deserialize, Serialize};

/// Scenario where person is difficult to detect via RGB
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InvisiblePersonScenario {
    LowLight,
    NightTime,
    Backlit,
    Glare,
    Shadows,
    PartialOcclusion,
    DenseVegetation,
    Smoke,
    Fog,
    Rain,
    Dust,
    CamouflagedClothing,
    ColorBlending,
    SmallDistant,
    EnteringFrame,
    Stationary,
    PartiallyHidden,
    Unknown,
}

impl std::fmt::Display for InvisiblePersonScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InvisiblePersonScenario::LowLight => write!(f, "Low Light"),
            InvisiblePersonScenario::NightTime => write!(f, "Night Time"),
            InvisiblePersonScenario::Backlit => write!(f, "Backlit"),
            InvisiblePersonScenario::Glare => write!(f, "Glare"),
            InvisiblePersonScenario::Shadows => write!(f, "Shadows"),
            InvisiblePersonScenario::PartialOcclusion => write!(f, "Partial Occlusion"),
            InvisiblePersonScenario::DenseVegetation => write!(f, "Dense Vegetation"),
            InvisiblePersonScenario::Smoke => write!(f, "Smoke"),
            InvisiblePersonScenario::Fog => write!(f, "Fog"),
            InvisiblePersonScenario::Rain => write!(f, "Rain"),
            InvisiblePersonScenario::Dust => write!(f, "Dust"),
            InvisiblePersonScenario::CamouflagedClothing => write!(f, "Camouflaged Clothing"),
            InvisiblePersonScenario::ColorBlending => write!(f, "Color Blending"),
            InvisiblePersonScenario::SmallDistant => write!(f, "Small Distant"),
            InvisiblePersonScenario::EnteringFrame => write!(f, "Entering Frame"),
            InvisiblePersonScenario::Stationary => write!(f, "Stationary"),
            InvisiblePersonScenario::PartiallyHidden => write!(f, "Partially Hidden"),
            InvisiblePersonScenario::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Invisible person detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvisiblePersonDetection {
    /// Scenario type
    pub scenario: InvisiblePersonScenario,
    /// Estimated location (x, y)
    pub location: (f32, f32),
    /// Confidence in detection (0.0-1.0)
    pub confidence: f32,
    /// Timestamp when detected thermally
    pub first_detectable_timestamp: f32,
    /// Duration visible in thermal
    pub thermal_visible_duration_sec: f32,
    /// Duration likely visible to RGB
    pub rgb_visible_duration_sec: f32,
    /// Thermal evidence supporting detection
    pub thermal_evidence: String,
    /// RGB detection probability (if RGB had been perfect)
    pub rgb_detection_probability: f32,
    /// Would fusion have detected earlier?
    pub fusion_would_improve: bool,
    /// Improvement potential
    pub improvement_potential_sec: f32,
}

/// Invisible person detector
pub struct InvisiblePersonDetector {
    /// Detected invisible persons
    pub detections: Vec<InvisiblePersonDetection>,
    /// Scenario occurrence counts
    pub scenario_counts: std::collections::HashMap<InvisiblePersonScenario, usize>,
}

impl InvisiblePersonDetector {
    /// Create new detector
    pub fn new() -> Self {
        InvisiblePersonDetector {
            detections: Vec::new(),
            scenario_counts: std::collections::HashMap::new(),
        }
    }

    /// Analyze thermal frame for invisible persons
    pub fn analyze_thermal_frame(
        &mut self,
        thermal: &ThermalFrame,
        timestamp: f32,
        ambient_light_estimate: f32, // 0.0 (dark) to 1.0 (bright)
        occlusion_percentage: f32,
    ) {
        let hotspots = thermal.detect_hotspots(8.0);

        for hotspot in hotspots {
            if hotspot.estimate_source() != ThermalSource::Human
                && hotspot.estimate_source() != ThermalSource::HumanPartial
            {
                continue; // Not human-like thermal signature
            }

            // Determine scenario
            let scenario = if ambient_light_estimate < 0.1 {
                InvisiblePersonScenario::NightTime
            } else if ambient_light_estimate < 0.3 {
                InvisiblePersonScenario::LowLight
            } else if occlusion_percentage > 0.5 {
                InvisiblePersonScenario::PartialOcclusion
            } else if occlusion_percentage > 0.2 {
                InvisiblePersonScenario::Shadows
            } else {
                InvisiblePersonScenario::Unknown
            };

            let rgb_detection_prob = match scenario {
                InvisiblePersonScenario::NightTime => 0.1,
                InvisiblePersonScenario::LowLight => 0.3,
                InvisiblePersonScenario::Shadows => 0.5,
                InvisiblePersonScenario::PartialOcclusion => 0.6,
                _ => 0.7,
            };

            let confidence = hotspot.source_confidence();

            *self.scenario_counts.entry(scenario.clone()).or_insert(0) += 1;

            let detection = InvisiblePersonDetection {
                scenario,
                location: (hotspot.center_x, hotspot.center_y),
                confidence,
                first_detectable_timestamp: timestamp,
                thermal_visible_duration_sec: 1.0, // Stub
                rgb_visible_duration_sec: 0.1,     // Very brief
                thermal_evidence: format!(
                    "Thermal hotspot: {:.1}K avg, {} pixels",
                    hotspot.avg_temp_k, hotspot.pixel_count
                ),
                rgb_detection_probability: rgb_detection_prob,
                fusion_would_improve: true,
                improvement_potential_sec: (1.0 - rgb_detection_prob) * 2.0,
            };

            self.detections.push(detection);
        }
    }

    /// Get summary of invisible person findings
    pub fn get_summary(&self) -> InvisiblePersonSummary {
        let total_detections = self.detections.len();
        let high_confidence = self
            .detections
            .iter()
            .filter(|d| d.confidence > 0.7)
            .count();
        let fusion_would_improve = self
            .detections
            .iter()
            .filter(|d| d.fusion_would_improve)
            .count();

        let most_common_scenario = self
            .scenario_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(scenario, _)| scenario.clone());

        InvisiblePersonSummary {
            total_invisible_persons: total_detections,
            high_confidence_detections: high_confidence,
            fusion_improvement_potential: fusion_would_improve,
            avg_rgb_detection_probability: if total_detections > 0 {
                self.detections
                    .iter()
                    .map(|d| d.rgb_detection_probability)
                    .sum::<f32>()
                    / total_detections as f32
            } else {
                0.0
            },
            most_common_scenario,
        }
    }

    /// Generate invisible person report
    pub fn generate_report(&self) -> String {
        let summary = self.get_summary();

        let mut report = String::from("INVISIBLE PERSON DETECTION REPORT\n");
        report.push_str("==================================\n\n");

        report.push_str(&format!(
            "Total Invisible Persons Detected: {}\n",
            summary.total_invisible_persons
        ));
        report.push_str(&format!(
            "High Confidence (>70%): {}\n",
            summary.high_confidence_detections
        ));
        report.push_str(&format!(
            "Fusion Could Have Improved Detection: {}\n\n",
            summary.fusion_improvement_potential
        ));

        report.push_str(&format!(
            "Average RGB Detection Probability: {:.0}%\n",
            summary.avg_rgb_detection_probability * 100.0
        ));

        if let Some(scenario) = &summary.most_common_scenario {
            report.push_str(&format!(
                "Most Common Scenario: {}\n",
                scenario
            ));
        }

        report.push_str("\nDETAILED DETECTIONS:\n");
        for (idx, detection) in self.detections.iter().enumerate() {
            report.push_str(&format!(
                "\n{}. {} (confidence {:.0}%)\n",
                idx + 1,
                detection.scenario,
                detection.confidence * 100.0
            ));
            report.push_str(&format!(
                "   Location: ({:.0}, {:.0})\n",
                detection.location.0, detection.location.1
            ));
            report.push_str(&format!(
                "   Thermal Evidence: {}\n",
                detection.thermal_evidence
            ));
            report.push_str(&format!(
                "   RGB Detection Probability: {:.0}%\n",
                detection.rgb_detection_probability * 100.0
            ));
            if detection.fusion_would_improve {
                report.push_str(&format!(
                    "   Fusion Improvement Potential: +{:.1}s detection time\n",
                    detection.improvement_potential_sec
                ));
            }
        }

        report
    }
}

impl Default for InvisiblePersonDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of invisible person findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvisiblePersonSummary {
    pub total_invisible_persons: usize,
    pub high_confidence_detections: usize,
    pub fusion_improvement_potential: usize,
    pub avg_rgb_detection_probability: f32,
    pub most_common_scenario: Option<InvisiblePersonScenario>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invisible_person_detector_creation() {
        let detector = InvisiblePersonDetector::new();
        assert_eq!(detector.detections.len(), 0);
    }

    #[test]
    fn test_scenario_display() {
        let scenario = InvisiblePersonScenario::LowLight;
        assert_eq!(scenario.to_string(), "Low Light");
    }

    #[test]
    fn test_get_summary() {
        let detector = InvisiblePersonDetector::new();
        let summary = detector.get_summary();
        assert_eq!(summary.total_invisible_persons, 0);
    }

    #[test]
    fn test_report_generation() {
        let detector = InvisiblePersonDetector::new();
        let report = detector.generate_report();
        assert!(report.contains("INVISIBLE PERSON DETECTION REPORT"));
    }
}
