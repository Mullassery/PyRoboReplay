//! CLI Output for Consolidated Gap Findings
//!
//! Formats aggregated findings with detector agreement visualization.

use crate::analyzers::aggregation::ConsolidatedFinding;
use std::collections::HashMap;

/// Format consolidated findings with evidence from multiple detectors
pub struct ConsolidatedFormatter {
    detail: bool,
}

impl ConsolidatedFormatter {
    pub fn new(detail: bool) -> Self {
        ConsolidatedFormatter { detail }
    }

    /// Format consolidated findings as text with detector agreement breakdown
    pub fn format_text(&self, consolidated: &[ConsolidatedFinding]) -> String {
        let mut output = String::new();

        if consolidated.is_empty() {
            output.push_str("✅ No reality gaps detected - simulation appears well-calibrated\n");
            return output;
        }

        output.push_str("\n🔍 Consolidated Reality Gap Analysis\n");
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        output.push_str(&format!(
            "Total consolidated findings: {} | Avg detector agreement: {:.1}x\n\n",
            consolidated.len(),
            Self::avg_detector_count(consolidated)
        ));

        for (idx, finding) in consolidated.iter().enumerate() {
            output.push_str(&self.format_consolidated_finding(idx + 1, finding));
        }

        output
    }

    /// Format consolidated findings as JSON
    pub fn format_json(&self, consolidated: &[ConsolidatedFinding]) -> serde_json::Value {
        let findings_json: Vec<_> = consolidated
            .iter()
            .map(|c| {
                let component_names: Vec<String> =
                    c.component_findings.iter().map(|f| f.category.clone()).collect();

                serde_json::json!({
                    "root_cause": c.root_cause,
                    "consolidated_gap_score": format!("{:.0}%", c.consolidated_gap_score * 100.0),
                    "consolidated_confidence": format!("{:.0}%", c.consolidated_confidence * 100.0),
                    "detector_count": c.detector_count,
                    "supporting_detectors": component_names,
                    "explanation": c.explanation,
                    "component_findings": c.component_findings.len(),
                })
            })
            .collect();

        serde_json::json!({
            "consolidated_findings": findings_json,
            "summary": {
                "total_consolidated": consolidated.len(),
                "avg_detectors_per_root_cause": Self::avg_detector_count(consolidated),
                "total_raw_findings": consolidated.iter().map(|c| c.detector_count).sum::<usize>(),
                "redundancy_factor": Self::redundancy_factor(consolidated),
            }
        })
    }

    /// Generate HTML report with detector agreement heatmap
    pub fn format_html(&self, consolidated: &[ConsolidatedFinding], mission_id: &str) -> String {
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PyRoboReplay - Consolidated Gap Analysis</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            border-radius: 8px;
            margin-bottom: 30px;
        }
        .stats {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 15px;
            margin-bottom: 30px;
        }
        .stat-card {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            text-align: center;
        }
        .stat-value {
            font-size: 28px;
            font-weight: bold;
            color: #667eea;
        }
        .stat-label {
            color: #666;
            font-size: 14px;
            margin-top: 8px;
        }
        .consolidated-finding {
            background: white;
            padding: 20px;
            margin-bottom: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            border-left: 4px solid #667eea;
        }
        .root-cause-title {
            font-size: 18px;
            font-weight: bold;
            margin-bottom: 10px;
            color: #333;
        }
        .detector-badges {
            display: flex;
            flex-wrap: wrap;
            gap: 8px;
            margin: 15px 0;
        }
        .detector-badge {
            background: #e8eaf6;
            color: #3f51b5;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 12px;
            font-weight: bold;
        }
        .confidence-bar {
            width: 100%;
            height: 8px;
            background: #eee;
            border-radius: 4px;
            margin: 10px 0;
            overflow: hidden;
        }
        .confidence-fill {
            height: 100%;
            background: linear-gradient(90deg, #667eea, #764ba2);
        }
        .metrics {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 15px;
            margin: 15px 0;
        }
        .metric {
            background: #f8f9fa;
            padding: 10px;
            border-radius: 4px;
            font-size: 13px;
        }
        .metric-label {
            color: #666;
            font-size: 11px;
            text-transform: uppercase;
        }
        .metric-value {
            font-weight: bold;
            color: #333;
            margin-top: 4px;
        }
        .explanation {
            background: #f0f7ff;
            padding: 10px;
            border-left: 3px solid #667eea;
            margin: 10px 0;
            font-size: 13px;
            border-radius: 3px;
        }
        code {
            background: #f4f4f4;
            padding: 2px 6px;
            border-radius: 3px;
            font-size: 12px;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🔍 Consolidated Reality Gap Analysis</h1>
        <p>PyRoboReplay v2.0 - Multi-Detector Evidence Fusion</p>"#
        );

        html.push_str(&format!("        <p><code>{}</code></p>\n", mission_id));
        html.push_str("    </div>\n");

        // Statistics
        html.push_str("    <div class=\"stats\">\n");
        html.push_str(&format!(
            r#"        <div class="stat-card">
            <div class="stat-value">{}</div>
            <div class="stat-label">Consolidated Findings</div>
        </div>
        <div class="stat-card">
            <div class="stat-value">{:.1}x</div>
            <div class="stat-label">Avg Detector Agreement</div>
        </div>
        <div class="stat-card">
            <div class="stat-value">{}</div>
            <div class="stat-label">Total Raw Findings</div>
        </div>
    </div>"#,
            consolidated.len(),
            Self::avg_detector_count(consolidated),
            consolidated.iter().map(|c| c.detector_count).sum::<usize>()
        ));

        // Findings
        for (idx, c) in consolidated.iter().enumerate() {
            let detector_list = c
                .component_findings
                .iter()
                .map(|f| f.category.as_str())
                .collect::<Vec<_>>();

            html.push_str(&format!(
                r#"    <div class="consolidated-finding">
        <div class="root-cause-title">#{}: {}</div>
        <div class="explanation">{}</div>
        <div class="detector-badges">
"#,
                idx + 1,
                c.root_cause,
                c.explanation
            ));

            for detector in detector_list {
                html.push_str(&format!(
                    r#"            <span class="detector-badge">{}</span>
"#,
                    detector
                ));
            }

            html.push_str(&format!(
                r#"        </div>
        <div class="confidence-bar">
            <div class="confidence-fill" style="width: {}%;"></div>
        </div>
        <small style="color: #999;">
            Confidence: {:.0}% | Gap Score: {:.0}% | Detectors: {}
        </small>
        <div class="metrics">
            <div class="metric">
                <div class="metric-label">Gap Score</div>
                <div class="metric-value">{:.0}%</div>
            </div>
            <div class="metric">
                <div class="metric-label">Confidence</div>
                <div class="metric-value">{:.0}%</div>
            </div>
            <div class="metric">
                <div class="metric-label">Detector Count</div>
                <div class="metric-value">{}</div>
            </div>
        </div>
    </div>
"#,
                (c.consolidated_confidence * 100.0) as u32,
                c.consolidated_confidence * 100.0,
                c.consolidated_gap_score * 100.0,
                c.detector_count,
                c.consolidated_gap_score * 100.0,
                c.consolidated_confidence * 100.0,
                c.detector_count
            ));
        }

        html.push_str("</body></html>");
        html
    }

    fn format_consolidated_finding(
        &self,
        number: usize,
        consolidated: &ConsolidatedFinding,
    ) -> String {
        let mut output = format!(
            "🎯 Consolidated Gap #{}: {}\n",
            number, consolidated.root_cause
        );

        output.push_str(&format!(
            "   Explanation: {}\n",
            consolidated.explanation
        ));

        let detectors: Vec<&str> = consolidated
            .component_findings
            .iter()
            .map(|f| f.category.as_str())
            .collect();
        output.push_str(&format!(
            "   Supporting Detectors ({}): {}\n",
            detectors.len(),
            detectors.join(", ")
        ));

        output.push_str(&format!(
            "   Consolidated Confidence: {:.0}% (boosted from avg {:.0}%)\n",
            consolidated.consolidated_confidence * 100.0,
            consolidated.component_findings.iter().map(|f| f.confidence).sum::<f32>()
                / consolidated.component_findings.len() as f32
                * 100.0
        ));

        output.push_str(&format!(
            "   Reality Gap Score: {:.0}%\n",
            consolidated.consolidated_gap_score * 100.0
        ));

        if self.detail {
            output.push_str(&format!(
                "   Components ({}): {}\n",
                consolidated.component_findings.len(),
                consolidated
                    .component_findings
                    .iter()
                    .map(|f| format!("{} ({:.0}% conf)", f.finding_type, f.confidence * 100.0))
                    .collect::<Vec<_>>()
                    .join(" → ")
            ));
        }

        output.push_str("\n");
        output
    }

    fn avg_detector_count(consolidated: &[ConsolidatedFinding]) -> f32 {
        if consolidated.is_empty() {
            return 1.0;
        }
        consolidated.iter().map(|c| c.detector_count as f32).sum::<f32>()
            / consolidated.len() as f32
    }

    fn redundancy_factor(consolidated: &[ConsolidatedFinding]) -> f32 {
        if consolidated.is_empty() {
            return 1.0;
        }
        let total_components: usize = consolidated.iter().map(|c| c.detector_count).sum();
        let root_cause_count = consolidated.len();
        (total_components as f32) / (root_cause_count as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_creation() {
        let _formatter = ConsolidatedFormatter::new(false);
    }

    #[test]
    fn test_empty_consolidated_findings() {
        let formatter = ConsolidatedFormatter::new(false);
        let output = formatter.format_text(&[]);
        assert!(output.contains("No reality gaps detected"));
    }

    #[test]
    fn test_json_format_empty() {
        let formatter = ConsolidatedFormatter::new(false);
        let json = formatter.format_json(&[]);
        assert_eq!(json["summary"]["total_consolidated"], 0);
    }

    #[test]
    fn test_html_generation() {
        let formatter = ConsolidatedFormatter::new(false);
        let html = formatter.format_html(&[], "test_mission");
        assert!(html.contains("<!DOCTYPE html"));
        assert!(html.contains("test_mission"));
        assert!(html.contains("Consolidated Reality Gap Analysis"));
    }
}
