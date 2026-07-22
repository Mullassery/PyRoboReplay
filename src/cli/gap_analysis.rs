//! CLI for Reality Gap Analysis
//!
//! Handles output formatting for gap detection results.

use crate::analyzers::{RealityGapFinding, Severity};
use std::collections::HashMap;

/// Format gap findings for display
pub struct GapFormatter {
    detail: bool,
}

impl GapFormatter {
    pub fn new(detail: bool) -> Self {
        GapFormatter { detail }
    }

    /// Format findings as text (human-readable, terminal-friendly)
    pub fn format_text(&self, findings: &[RealityGapFinding]) -> String {
        let mut output = String::new();

        if findings.is_empty() {
            output.push_str("✅ No reality gaps detected - simulation appears well-calibrated\n");
            return output;
        }

        // Summary
        let critical = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let high = findings.iter().filter(|f| f.severity == Severity::High).count();
        let medium = findings.iter().filter(|f| f.severity == Severity::Medium).count();
        let low = findings.iter().filter(|f| f.severity == Severity::Low).count();

        output.push_str("\n🔍 Reality Gap Analysis Summary\n");
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        output.push_str(&format!(
            "Total findings: {} | 🔴 Critical: {} | 🟠 High: {} | 🟡 Medium: {} | 🔵 Low: {}\n\n",
            findings.len(),
            critical,
            high,
            medium,
            low
        ));

        // Detailed findings
        for (idx, finding) in findings.iter().enumerate() {
            output.push_str(&self.format_finding(idx + 1, finding));
        }

        output
    }

    /// Format findings as JSON
    pub fn format_json(&self, findings: &[RealityGapFinding]) -> serde_json::Value {
        let findings_json: Vec<_> = findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "domain": f.domain.to_string(),
                    "category": f.category,
                    "finding_type": f.finding_type,
                    "severity": f.severity.to_string(),
                    "confidence": format!("{:.0}%", f.confidence * 100.0),
                    "reality_gap_score": format!("{:.0}%", f.reality_gap_score * 100.0),
                    "description": f.description,
                    "evidence_count": f.evidence.len(),
                    "metrics": f.metrics,
                    "sim_recreation": f.sim_recreation_suggestion,
                    "remediation": f.remediation,
                })
            })
            .collect();

        serde_json::json!({
            "findings": findings_json,
            "summary": {
                "total": findings.len(),
                "critical": findings.iter().filter(|f| f.severity == Severity::Critical).count(),
                "high": findings.iter().filter(|f| f.severity == Severity::High).count(),
                "medium": findings.iter().filter(|f| f.severity == Severity::Medium).count(),
                "low": findings.iter().filter(|f| f.severity == Severity::Low).count(),
            }
        })
    }

    /// Generate HTML report
    pub fn format_html(&self, findings: &[RealityGapFinding], mission_id: &str) -> String {
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PyRoboReplay - Reality Gap Analysis</title>
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
        .summary {
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 15px;
            margin-bottom: 30px;
        }
        .summary-card {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            text-align: center;
        }
        .severity-critical { border-left: 4px solid #dc3545; }
        .severity-high { border-left: 4px solid #fd7e14; }
        .severity-medium { border-left: 4px solid #ffc107; }
        .severity-low { border-left: 4px solid #28a745; }
        .finding {
            background: white;
            padding: 20px;
            margin-bottom: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .finding-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 15px;
            padding-bottom: 15px;
            border-bottom: 1px solid #eee;
        }
        .severity-badge {
            padding: 4px 12px;
            border-radius: 20px;
            font-weight: bold;
            font-size: 12px;
        }
        .badge-critical { background: #dc3545; color: white; }
        .badge-high { background: #fd7e14; color: white; }
        .badge-medium { background: #ffc107; color: #333; }
        .badge-low { background: #28a745; color: white; }
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
            grid-template-columns: repeat(2, 1fr);
            gap: 15px;
            margin: 15px 0;
        }
        .metric {
            background: #f8f9fa;
            padding: 10px;
            border-radius: 4px;
            font-size: 14px;
        }
        .metric-label {
            color: #666;
            font-size: 12px;
            text-transform: uppercase;
        }
        .metric-value {
            font-weight: bold;
            color: #333;
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
        <h1>🔍 Reality Gap Analysis Report</h1>
        <p>PyRoboReplay v2.0 - Sim-to-Real Diagnostics</p>"#
        );

        html.push_str(&format!("        <p><code>{}</code></p>\n", mission_id));
        html.push_str("    </div>\n");

        // Summary cards
        html.push_str("    <div class=\"summary\">\n");
        let critical = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let high = findings.iter().filter(|f| f.severity == Severity::High).count();
        let medium = findings.iter().filter(|f| f.severity == Severity::Medium).count();
        let low = findings.iter().filter(|f| f.severity == Severity::Low).count();

        html.push_str(&format!(
            r#"        <div class="summary-card severity-critical">
            <div style="font-size: 28px; font-weight: bold;">{}</div>
            <div>Critical</div>
        </div>
        <div class="summary-card severity-high">
            <div style="font-size: 28px; font-weight: bold;">{}</div>
            <div>High</div>
        </div>
        <div class="summary-card severity-medium">
            <div style="font-size: 28px; font-weight: bold;">{}</div>
            <div>Medium</div>
        </div>
        <div class="summary-card severity-low">
            <div style="font-size: 28px; font-weight: bold;">{}</div>
            <div>Low</div>
        </div>
    </div>"#,
            critical, high, medium, low
        ));

        // Findings
        for finding in findings {
            html.push_str(&format!(
                r#"    <div class="finding">
        <div class="finding-header">
            <div>
                <h3>{}</h3>
                <p style="margin: 0; color: #666; font-size: 14px;">{} - {}</p>
            </div>
            <div class="severity-badge badge-{}">
                {}
            </div>
        </div>
        <p>{}</p>
        <div class="confidence-bar">
            <div class="confidence-fill" style="width: {}%;"></div>
        </div>
        <small style="color: #999;">Confidence: {:.0}% | Reality Gap Score: {:.0}%</small>"#,
                finding.finding_type,
                finding.domain,
                finding.category,
                finding.severity.to_string().to_lowercase(),
                finding.severity,
                finding.description,
                (finding.confidence * 100.0) as u32,
                finding.confidence * 100.0,
                finding.reality_gap_score * 100.0
            ));

            if !finding.metrics.is_empty() {
                html.push_str("        <div class=\"metrics\">\n");
                for (key, value) in &finding.metrics {
                    html.push_str(&format!(
                        r#"            <div class="metric">
                <div class="metric-label">{}</div>
                <div class="metric-value">{:.2}</div>
            </div>
"#,
                        key.replace('_', " "),
                        value
                    ));
                }
                html.push_str("        </div>\n");
            }

            html.push_str("    </div>\n");
        }

        html.push_str("</body></html>");
        html
    }

    fn format_finding(&self, number: usize, finding: &RealityGapFinding) -> String {
        let severity_icon = match finding.severity {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🔵",
        };

        let mut output = format!(
            "{} Finding #{}: {}\n",
            severity_icon, number, finding.finding_type
        );

        output.push_str(&format!(
            "   Domain: {} | Category: {}\n",
            finding.domain, finding.category
        ));

        output.push_str(&format!(
            "   Severity: {} | Confidence: {:.0}% | Gap Score: {:.0}%\n",
            finding.severity,
            finding.confidence * 100.0,
            finding.reality_gap_score * 100.0
        ));

        output.push_str(&format!("   Description: {}\n", finding.description));

        if self.detail {
            if !finding.evidence.is_empty() {
                output.push_str("   Evidence:\n");
                for ev in &finding.evidence {
                    output.push_str(&format!(
                        "     - {}: {} (confidence: {:.0}%)\n",
                        ev.signal, ev.value, ev.confidence * 100.0
                    ));
                }
            }

            if !finding.metrics.is_empty() {
                output.push_str("   Metrics:\n");
                for (key, value) in &finding.metrics {
                    output.push_str(&format!("     - {}: {:.2}\n", key, value));
                }
            }

            output.push_str(&format!(
                "   Sim Recreation: {}\n",
                finding.sim_recreation_suggestion
            ));
            output.push_str(&format!(
                "   Remediation: {}\n",
                finding.remediation
            ));
        }

        output.push('\n');
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_creation() {
        let _formatter = GapFormatter::new(false);
    }

    #[test]
    fn test_empty_findings() {
        let formatter = GapFormatter::new(false);
        let text = formatter.format_text(&[]);
        assert!(text.contains("No reality gaps"));
    }
}
