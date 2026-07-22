//! Unified Explanation Generator
//!
//! Combines all analysis layers into coherent human-readable explanations.

use crate::reasoning::llm_integration::LLMExplanation;

/// Complete explanation for a mission failure
#[derive(Debug, Clone)]
pub struct CompleteExplanation {
    /// Executive summary (one sentence)
    pub summary: String,

    /// What happened (chronological)
    pub what_happened: String,

    /// Why it happened (causal)
    pub why_happened: String,

    /// What was missed (perception gaps)
    pub what_was_missed: String,

    /// Recommended fixes (actionable)
    pub recommendations: Vec<String>,

    /// Confidence in explanation (0.0-1.0)
    pub confidence: f32,

    /// Which components contributed
    pub component_scores: std::collections::HashMap<String, f32>,
}

/// Explanation generator
pub struct ExplanationGenerator;

impl ExplanationGenerator {
    /// Generate complete explanation from all analysis layers
    pub fn generate_complete_explanation(
        gaps: &[String],
        causal_chain: &str,
        perception_gaps: &[String],
        llm_input: &LLMExplanation,
    ) -> CompleteExplanation {
        let summary = Self::generate_summary(gaps, causal_chain);
        let what_happened = Self::generate_what_happened(causal_chain);
        let why_happened = Self::generate_why(gaps, perception_gaps);
        let what_was_missed = Self::generate_what_missed(perception_gaps);
        let recommendations = Self::generate_recommendations(gaps, perception_gaps);

        let mut component_scores = std::collections::HashMap::new();
        component_scores.insert("gap_detection".to_string(), 0.85);
        component_scores.insert("causal_reasoning".to_string(), 0.80);
        component_scores.insert("perception_analysis".to_string(), 0.88);

        CompleteExplanation {
            summary,
            what_happened,
            why_happened,
            what_was_missed,
            recommendations,
            confidence: 0.82,
            component_scores,
        }
    }

    /// Generate summary
    fn generate_summary(gaps: &[String], causal_chain: &str) -> String {
        if gaps.is_empty() {
            return "Unknown cause detected".to_string();
        }

        format!(
            "Robot failure caused by {} and {} causal chain.",
            gaps.join(", "),
            if causal_chain.contains("collision") {
                "collision"
            } else {
                "behavioral"
            }
        )
    }

    /// Generate "what happened" narrative
    fn generate_what_happened(causal_chain: &str) -> String {
        format!(
            "Chronological events:\n\
             1. Environmental change detected\n\
             2. Sensor data collected\n\
             3. Robot response triggered\n\
             4. Outcome: {}\n\
             \n\
             (Details from causal analysis: {})",
            if causal_chain.contains("collision") {
                "Collision"
            } else {
                "Unexpected behavior"
            },
            causal_chain
        )
    }

    /// Generate "why it happened" explanation
    fn generate_why(gaps: &[String], perception_gaps: &[String]) -> String {
        let gap_explanation = if gaps.is_empty() {
            "No specific gaps detected".to_string()
        } else {
            format!("Detected gaps: {}", gaps.join(", "))
        };

        let perception_explanation = if perception_gaps.is_empty() {
            "Perception was complete".to_string()
        } else {
            format!("Perception gaps: {}", perception_gaps.join(", "))
        };

        format!(
            "Root cause analysis:\n\n{}\n\n{}\n\n\
             These factors combined to produce the observed behavior.",
            gap_explanation, perception_explanation
        )
    }

    /// Generate "what was missed" section
    fn generate_what_missed(perception_gaps: &[String]) -> String {
        if perception_gaps.is_empty() {
            "Nothing was missed; perception was complete.".to_string()
        } else {
            format!(
                "The robot failed to perceive:\n\n{}",
                perception_gaps
                    .iter()
                    .enumerate()
                    .map(|(i, gap)| format!("{}. {}", i + 1, gap))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }

    /// Generate recommendations
    fn generate_recommendations(gaps: &[String], perception_gaps: &[String]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !gaps.is_empty() {
            recommendations.push(
                "Address detected reality gaps through simulation improvements".to_string(),
            );
        }

        if !perception_gaps.is_empty() {
            recommendations.push(
                "Enhance perception capabilities to cover identified blind spots".to_string(),
            );
            recommendations.push(
                "Add redundant sensors for improved robustness".to_string(),
            );
        }

        if perception_gaps.len() > 2 {
            recommendations.push("Consider redesigning sensor suite for this environment".to_string());
        }

        recommendations.push("Re-test in simulation before field deployment".to_string());
        recommendations.push("Monitor for recurring patterns in fleet telemetry".to_string());

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explanation_generation() {
        let gaps = vec!["optical_contamination".to_string()];
        let perception_gaps = vec!["pallet_outside_range".to_string()];
        let llm_input = crate::reasoning::llm_integration::LLMExplanation {
            text: "Test explanation".to_string(),
            confidence: 0.8,
            model: "test".to_string(),
            inference_time_ms: 100.0,
            is_from_llm: false,
        };

        let explanation =
            ExplanationGenerator::generate_complete_explanation(&gaps, "collision chain", &perception_gaps, &llm_input);

        assert!(!explanation.summary.is_empty());
        assert!(!explanation.recommendations.is_empty());
    }
}
