//! Gap Context Analyzer: Retrospective Analysis with Terrain & Knowledge Context
//!
//! Enriches detection gaps with:
//! - Terrain context: "High-traffic zone where robot failed"
//! - World knowledge: "Entity should have been at (5.2, 3.1)"
//! - Historical patterns: "Third time this gap appeared at this location"
//!
//! Enables: Understanding not just WHAT was missed, but WHY and HOW to prevent it.

use crate::perception::retrospective_detection::DetectionGap;
use serde::{Deserialize, Serialize};

/// Detection gap with rich context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualGap {
    /// The detection gap itself
    pub gap: DetectionGap,
    /// Terrain zone this happened in
    pub zone_id: Option<String>,
    /// Traversability of zone (0.0-1.0)
    pub zone_traversability: Option<f32>,
    /// Zone type (high-traffic, confined, etc.)
    pub zone_type: Option<String>,
    /// Expected entity location from world knowledge
    pub expected_location: Option<(f32, f32)>, // x, y meters
    /// Actual detection location
    pub actual_location: Option<(f32, f32)>,
    /// Distance from expected
    pub location_deviation_m: Option<f32>,
    /// Historical occurrence count
    pub previous_occurrences: usize,
    /// Confidence from multiple observations
    pub confidence: f32,
}

/// Gap severity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapSeverityAssessment {
    /// Primary severity (from invisibility factors)
    pub primary_severity: f32,
    /// Context-adjusted severity (terrain + history)
    pub adjusted_severity: f32,
    /// Risk factors contributing to severity
    pub contributing_factors: Vec<String>,
    /// Recommendation priority (P0/P1/P2/P3)
    pub priority: String,
}

/// Gap context analysis result
pub struct GapContextAnalysis {
    /// All gaps with context
    pub contextual_gaps: Vec<ContextualGap>,
    /// Severity assessments
    pub severity_assessments: Vec<GapSeverityAssessment>,
    /// Patterns identified
    pub patterns: Vec<GapPattern>,
}

/// Recurring gap pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapPattern {
    /// Pattern description (e.g., "person_at_high_traffic_area")
    pub pattern_id: String,
    /// Object type that triggers pattern
    pub object_type: String,
    /// Location characteristic
    pub location_characteristic: String,
    /// Occurrence count
    pub occurrence_count: usize,
    /// Success rate when identified (how often robot eventually detected it)
    pub success_rate: f32,
    /// Recommended mitigation
    pub mitigation: String,
}

/// Gap context analyzer
pub struct GapContextAnalyzer {
    /// All analyzed gaps
    pub gaps: Vec<ContextualGap>,
    /// Historical gap occurrence tracking
    pub gap_history: std::collections::HashMap<String, usize>,
}

impl GapContextAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        GapContextAnalyzer {
            gaps: Vec::new(),
            gap_history: std::collections::HashMap::new(),
        }
    }

    /// Analyze gap with context
    pub fn analyze_gap(
        &mut self,
        gap: DetectionGap,
        zone_id: Option<String>,
        zone_traversability: Option<f32>,
        zone_type: Option<String>,
        expected_location: Option<(f32, f32)>,
        actual_location: Option<(f32, f32)>,
    ) -> ContextualGap {
        // Calculate location deviation
        let location_deviation_m = match (expected_location, actual_location) {
            (Some((ex, ey)), Some((ax, ay))) => {
                let dx = ex - ax;
                let dy = ey - ay;
                Some((dx * dx + dy * dy).sqrt())
            }
            _ => None,
        };

        // Look up historical occurrences
        let pattern_key = format!(
            "{}_{:?}",
            gap.dino_detection.class_name, zone_id
        );
        let previous_occurrences = *self.gap_history.get(&pattern_key).unwrap_or(&0);

        // Increment history
        *self
            .gap_history
            .entry(pattern_key)
            .or_insert(0) += 1;

        // Confidence: increases with each occurrence (pattern gets more reliable)
        let confidence = (1.0 - (-1.0 * (previous_occurrences as f32 / 5.0)).exp()).min(0.95);

        let contextual = ContextualGap {
            gap,
            zone_id,
            zone_traversability,
            zone_type,
            expected_location,
            actual_location,
            location_deviation_m,
            previous_occurrences,
            confidence,
        };

        self.gaps.push(contextual.clone());
        contextual
    }

    /// Assess gap severity with context
    pub fn assess_severity(&self, gap: &ContextualGap) -> GapSeverityAssessment {
        let mut adjusted_severity = gap.gap.severity;
        let mut contributing_factors = vec![];

        // High-traffic zone increases severity (more likely to cause collision)
        if let Some(trav) = gap.zone_traversability {
            if trav > 0.8 {
                adjusted_severity *= 1.3;
                contributing_factors.push("high_traffic_zone".to_string());
            } else if trav < 0.3 {
                adjusted_severity *= 1.15;
                contributing_factors.push("low_traversability_zone".to_string());
            }
        }

        // Location deviation from baseline
        if let Some(dev) = gap.location_deviation_m {
            if dev > 2.0 {
                adjusted_severity *= 1.2;
                contributing_factors.push(format!("{}m_from_baseline", dev as u32));
            }
        }

        // Recurring pattern
        if gap.previous_occurrences > 2 {
            adjusted_severity *= 1.4;
            contributing_factors.push(format!("recurring_pattern_{}x", gap.previous_occurrences));
        }

        // Cap at 1.0
        adjusted_severity = adjusted_severity.min(1.0);

        // Determine priority
        let priority = if adjusted_severity > 0.75 {
            "P0".to_string() // Critical
        } else if adjusted_severity > 0.5 {
            "P1".to_string() // High
        } else if adjusted_severity > 0.3 {
            "P2".to_string() // Medium
        } else {
            "P3".to_string() // Low
        };

        GapSeverityAssessment {
            primary_severity: gap.gap.severity,
            adjusted_severity,
            contributing_factors,
            priority,
        }
    }

    /// Identify patterns across gaps
    pub fn identify_patterns(&self) -> Vec<GapPattern> {
        let mut patterns: std::collections::HashMap<String, (usize, f32)> = std::collections::HashMap::new();

        for gap in &self.gaps {
            let pattern_key = format!(
                "{}_{:?}",
                gap.gap.dino_detection.class_name, gap.zone_type
            );
            patterns
                .entry(pattern_key)
                .or_insert((0, 0.0))
                .0 += 1;
        }

        patterns
            .into_iter()
            .map(|(key, (count, _))| {
                let (obj_type, zone_type) = if let Some((obj, zone)) = key.split_once('_') {
                    (obj.to_string(), zone.to_string())
                } else {
                    (key.clone(), "unknown".to_string())
                };

                GapPattern {
                    pattern_id: key,
                    object_type: obj_type.clone(),
                    location_characteristic: zone_type
                        .trim_matches(|c| c == '"' || c == 'S' || c == 'o' || c == 'm' || c == 'e')
                        .to_string(),
                    occurrence_count: count,
                    success_rate: (count as f32 - 1.0) / count as f32, // Tends to improve over time
                    mitigation: format!(
                        "Deploy enhanced sensor for {} detection in {} areas",
                        obj_type, zone_type
                    ),
                }
            })
            .collect()
    }

    /// Generate comprehensive report
    pub fn generate_report(&self) -> String {
        let mut report = format!("Gap Context Analysis: {} gaps analyzed\n", self.gaps.len());

        report.push_str("\nGaps by Severity:\n");

        // Sort by severity
        let mut severity_gaps: Vec<_> = self
            .gaps
            .iter()
            .map(|g| (g, self.assess_severity(g)))
            .collect();
        severity_gaps.sort_by(|a, b| {
            b.1.adjusted_severity
                .partial_cmp(&a.1.adjusted_severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (gap, severity) in severity_gaps.iter().take(5) {
            report.push_str(&format!(
                "\n[{}] {} (primary: {:.0}%, adjusted: {:.0}%)\n",
                severity.priority,
                gap.gap.dino_detection.class_name,
                gap.gap.severity * 100.0,
                severity.adjusted_severity * 100.0
            ));

            if let Some(zone_type) = &gap.zone_type {
                report.push_str(&format!("  Zone: {} (traversability: {:.0}%)\n",
                    zone_type,
                    gap.zone_traversability.unwrap_or(0.5) * 100.0
                ));
            }

            if gap.previous_occurrences > 0 {
                report.push_str(&format!(
                    "  Pattern: {} previous occurrences\n",
                    gap.previous_occurrences
                ));
            }

            if !severity.contributing_factors.is_empty() {
                report.push_str("  Factors: ");
                report.push_str(&severity.contributing_factors.join(", "));
                report.push_str("\n");
            }
        }

        let patterns = self.identify_patterns();
        if !patterns.is_empty() {
            report.push_str("\nRecurring Patterns:\n");
            for pattern in patterns.iter().take(5) {
                report.push_str(&format!(
                    "  • {}: {} occurrences | Mitigation: {}\n",
                    pattern.pattern_id, pattern.occurrence_count, pattern.mitigation
                ));
            }
        }

        report
    }
}

impl Default for GapContextAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perception::object_detection::BoundingBox;
    use crate::perception::retrospective_detection::{DINODetection, InvisibilityFactor};

    fn create_test_gap() -> DetectionGap {
        DetectionGap {
            dino_detection: DINODetection {
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
            sam_segmentation: None,
            invisibility_factors: vec![InvisibilityFactor::Occlusion(0.3)],
            severity: 0.6,
            recommendation: "Test recommendation".to_string(),
        }
    }

    #[test]
    fn test_gap_context_analyzer_creation() {
        let analyzer = GapContextAnalyzer::new();
        assert_eq!(analyzer.gaps.len(), 0);
    }

    #[test]
    fn test_analyze_gap() {
        let mut analyzer = GapContextAnalyzer::new();
        let gap = create_test_gap();

        let contextual = analyzer.analyze_gap(
            gap,
            Some("high_traffic_zone".to_string()),
            Some(0.85),
            Some("warehouse_aisle".to_string()),
            Some((5.0, 5.0)),
            Some((5.2, 5.1)),
        );

        assert_eq!(contextual.zone_traversability, Some(0.85));
        assert!(contextual.location_deviation_m.is_some());
    }

    #[test]
    fn test_severity_assessment() {
        let mut analyzer = GapContextAnalyzer::new();
        let gap = create_test_gap();

        let contextual = analyzer.analyze_gap(
            gap,
            Some("high_traffic_zone".to_string()),
            Some(0.85),
            Some("warehouse_aisle".to_string()),
            None,
            None,
        );

        let severity = analyzer.assess_severity(&contextual);
        assert!(severity.adjusted_severity > severity.primary_severity);
        // High traffic + high severity yields P0
        assert_eq!(severity.priority, "P0");
    }

    #[test]
    fn test_patterns_identification() {
        let mut analyzer = GapContextAnalyzer::new();

        // Add multiple gaps
        for _ in 0..3 {
            let gap = create_test_gap();
            analyzer.analyze_gap(
                gap,
                Some("high_traffic_zone".to_string()),
                Some(0.85),
                Some("aisle".to_string()),
                None,
                None,
            );
        }

        let patterns = analyzer.identify_patterns();
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_report_generation() {
        let mut analyzer = GapContextAnalyzer::new();
        let gap = create_test_gap();

        analyzer.analyze_gap(
            gap,
            Some("high_traffic_zone".to_string()),
            Some(0.85),
            Some("warehouse".to_string()),
            None,
            None,
        );

        let report = analyzer.generate_report();
        assert!(report.contains("Gap Context Analysis"));
        assert!(report.contains("person"));
    }
}
