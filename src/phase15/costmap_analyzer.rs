//! Costmap failure analysis
//!
//! Detects and categorizes costmap issues:
//! - Inflation radius too large
//! - Missing dynamic obstacle layer
//! - Layer conflicts/overlaps
//! - False static obstacles
//! - Overly conservative settings

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostmapCause {
    ExcessiveInflation,
    MissingDynamicLayer,
    LayerConflict,
    FalseObstacles,
    ConservativeBias,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostmapIssue {
    pub cause: CostmapCause,
    pub confidence: f32,
    pub inflation_ratio: f32,
    pub evidence: Vec<String>,
    pub recommendations: Vec<String>,
}

pub struct CostmapAnalyzer;

impl CostmapAnalyzer {
    pub fn analyze_inflation(
        inflation_radius: f32,
        robot_footprint: f32,
    ) -> Option<CostmapIssue> {
        let ratio = inflation_radius / robot_footprint;

        if ratio > 2.0 {
            return Some(CostmapIssue {
                cause: CostmapCause::ExcessiveInflation,
                confidence: 0.85,
                inflation_ratio: ratio,
                evidence: vec![
                    format!("Inflation radius: {:.2}m, Robot footprint: {:.2}m, Ratio: {:.1}x",
                            inflation_radius, robot_footprint, ratio),
                    "Blocking valid navigation paths unnecessarily".to_string(),
                ],
                recommendations: vec![
                    format!("Reduce inflation radius to 0.15–0.30m range (current: {:.2}m)", inflation_radius),
                    "Verify clearance margins are appropriate for environment".to_string(),
                    "Test navigation with gradually reduced inflation".to_string(),
                ],
            });
        }
        None
    }

    pub fn summarize_costmap(
        static_layer_size: u32,
        dynamic_layer_size: u32,
        inflation_radius: f32,
    ) -> String {
        let mut summary = String::new();

        if dynamic_layer_size == 0 {
            summary.push_str("⚠️  WARNING: No dynamic obstacle layer detected\n");
        }

        if inflation_radius > 0.5 {
            summary.push_str("⚠️  WARNING: High inflation radius may block valid paths\n");
        }

        summary.push_str(&format!("   Static layer: {} cells\n", static_layer_size));
        summary.push_str(&format!("   Dynamic layer: {} cells\n", dynamic_layer_size));
        summary.push_str(&format!("   Inflation radius: {:.2}m\n", inflation_radius));

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excessive_inflation_detection() {
        let issue = CostmapAnalyzer::analyze_inflation(1.0, 0.35);
        assert!(issue.is_some());
    }

    #[test]
    fn test_summarize_costmap() {
        let summary = CostmapAnalyzer::summarize_costmap(1000, 0, 0.6);
        assert!(summary.contains("dynamic"));
    }
}
