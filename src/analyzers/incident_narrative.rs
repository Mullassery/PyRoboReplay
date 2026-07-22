//! Incident Narrative Generation from Reality Gaps
//!
//! Transforms raw gaps + causal chains into human-readable incident narratives
//! explaining what happened, why it happened, and what to do about it.

use crate::analyzers::gap_to_causal::GapCausalEvent;
use crate::analyzers::multi_factor_causality::MultiFactorCausalChain;
use crate::analyzers::RealityGapFinding;
use std::collections::HashMap;

/// Complete incident narrative: what happened, why, and what to do
#[derive(Debug, Clone)]
pub struct IncidentNarrative {
    /// Unique incident identifier
    pub incident_id: String,

    /// Mission where this occurred
    pub mission_id: String,

    /// When did the incident start (seconds)
    pub start_time_sec: f32,

    /// When did the incident end (seconds)
    pub end_time_sec: f32,

    /// Executive summary (1 sentence)
    pub executive_summary: String,

    /// What happened: chronological description
    pub what_happened: String,

    /// Why it happened: causal chain explanation
    pub why_it_happened: String,

    /// The impact: what was the consequence
    pub impact_description: String,

    /// Contributing factors with explanations
    pub contributing_factors_explained: Vec<FactorExplanation>,

    /// What should be done to prevent recurrence
    pub recommended_actions: Vec<RecommendedAction>,

    /// Risk if not addressed
    pub escalation_risk: String,

    /// Evidence supporting this narrative
    pub supporting_evidence: Vec<String>,

    /// Confidence in this narrative (0.0-1.0)
    pub narrative_confidence: f32,
}

/// Explanation of a contributing factor
#[derive(Debug, Clone)]
pub struct FactorExplanation {
    /// The factor type (gap, drift, environmental, etc.)
    pub factor_type: String,

    /// The factor name
    pub name: String,

    /// Plain English explanation of what it means
    pub explanation: String,

    /// How critical is this factor to the incident
    pub criticality: String, // "root_cause", "major_contributor", "minor_factor"
}

/// Recommended action to prevent recurrence
#[derive(Debug, Clone)]
pub struct RecommendedAction {
    /// Short action title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// How to implement it
    pub implementation: String,

    /// Expected effectiveness (0.0-1.0)
    pub effectiveness: f32,

    /// Priority: "critical", "high", "medium", "low"
    pub priority: String,

    /// Estimated effort: "trivial", "easy", "moderate", "hard"
    pub effort: String,
}

/// Generates human-readable incident narratives
pub struct IncidentNarrativeGenerator;

impl IncidentNarrativeGenerator {
    /// Generate narrative from causal chain
    pub fn from_causal_chain(
        chain: &MultiFactorCausalChain,
        gaps: &[RealityGapFinding],
    ) -> IncidentNarrative {
        let start_time = chain.events.first().map(|e| e.timestamp_sec).unwrap_or(0.0);
        let end_time = chain
            .events
            .last()
            .map(|e| e.timestamp_sec)
            .unwrap_or(0.0);

        let executive_summary =
            Self::generate_executive_summary(&chain.ultimate_effect, &chain.root_environmental_cause);

        let what_happened = Self::generate_what_happened(&chain.events);
        let why_it_happened = Self::generate_why(&chain, gaps);
        let impact_description = Self::generate_impact(&chain.ultimate_effect);

        let contributing_factors_explained =
            Self::explain_contributing_factors(&chain.contributing_factors);

        let recommended_actions = Self::generate_recommendations(&chain);
        let escalation_risk = Self::assess_escalation_risk(&chain.predicted_severity);

        let supporting_evidence = chain
            .events
            .iter()
            .map(|e| {
                format!(
                    "At t={:.2}s: {} (confidence: {:.0}%)",
                    e.timestamp_sec,
                    e.description,
                    e.confidence * 100.0
                )
            })
            .collect();

        IncidentNarrative {
            incident_id: chain.chain_id.clone(),
            mission_id: "unknown".to_string(),
            start_time_sec: start_time,
            end_time_sec: end_time,
            executive_summary,
            what_happened,
            why_it_happened,
            impact_description,
            contributing_factors_explained,
            recommended_actions,
            escalation_risk,
            supporting_evidence,
            narrative_confidence: chain.chain_confidence,
        }
    }

    /// Generate executive summary
    fn generate_executive_summary(ultimate_effect: &str, root_cause: &str) -> String {
        format!(
            "{}; triggered by {}.",
            ultimate_effect.trim_end_matches('.'),
            root_cause.to_lowercase()
        )
    }

    /// Generate chronological "what happened" description
    fn generate_what_happened(events: &[crate::analyzers::multi_factor_causality::ChainEvent]) -> String {
        if events.is_empty() {
            return "No events recorded.".to_string();
        }

        let mut description = "Sequence of events:\n".to_string();
        for (i, event) in events.iter().enumerate() {
            description.push_str(&format!(
                "{}. At t={:.1}s: {} ({}% confidence)\n",
                i + 1,
                event.timestamp_sec,
                event.description,
                (event.confidence * 100.0) as u32
            ));
        }
        description
    }

    /// Generate causal explanation
    fn generate_why(chain: &MultiFactorCausalChain, gaps: &[RealityGapFinding]) -> String {
        let mut explanation = String::new();

        explanation.push_str("This incident resulted from a cascade of failures:\n\n");

        explanation.push_str(&format!(
            "Root environmental cause: {}\n\n",
            chain.root_environmental_cause
        ));

        explanation.push_str("Gap types contributing to this incident:\n");
        for gap in gaps.iter().take(3) {
            explanation.push_str(&format!("- {}: {}\n", gap.category, gap.description));
        }

        explanation.push_str("\nThe chain of causality:\n");
        for (i, event) in chain.events.iter().enumerate() {
            if i < chain.events.len() - 1 {
                explanation.push_str(&format!("  {} → ", event.description));
                if i % 2 == 1 {
                    explanation.push('\n');
                }
            } else {
                explanation.push_str(&format!("{}", event.description));
            }
        }
        explanation.push_str(&format!("\n\nFinal outcome: {}", chain.ultimate_effect));

        explanation
    }

    /// Generate impact description
    fn generate_impact(ultimate_effect: &str) -> String {
        format!(
            "Result: {}. This could lead to mission failure, safety violations, or unplanned downtime.",
            ultimate_effect
        )
    }

    /// Explain each contributing factor
    fn explain_contributing_factors(
        factors: &[crate::analyzers::multi_factor_causality::ContributingFactor],
    ) -> Vec<FactorExplanation> {
        factors
            .iter()
            .map(|factor| {
                let (explanation, criticality) = match factor.factor_type.as_str() {
                    "gap" => {
                        let crit = if factor.confidence > 0.8 {
                            "root_cause"
                        } else if factor.confidence > 0.6 {
                            "major_contributor"
                        } else {
                            "minor_factor"
                        };
                        (
                            format!(
                                "{} was detected with {:.0}% confidence. \
                             This is a sim-to-real gap: the simulation doesn't model this phenomenon accurately.",
                                factor.name, factor.confidence * 100.0
                            ),
                            crit.to_string(),
                        )
                    }
                    "drift" => (
                        format!(
                            "{} showed statistical drift of {:.2} standard deviations. \
                         This indicates the metric is moving outside normal operating range.",
                            factor.name, factor.magnitude
                        ),
                        "major_contributor".to_string(),
                    ),
                    "quality" => (
                        format!(
                            "{} quality degraded to {:.0}%. \
                         This reduces confidence in sensor data and decision-making.",
                            factor.name, factor.confidence * 100.0
                        ),
                        "minor_factor".to_string(),
                    ),
                    "environmental" => (
                        format!(
                            "{} was present in the environment. \
                         These conditions accelerated the failure cascade.",
                            factor.name
                        ),
                        "major_contributor".to_string(),
                    ),
                    _ => (
                        format!(
                            "Unknown factor type: {}. Magnitude: {:.2}",
                            factor.factor_type, factor.magnitude
                        ),
                        "unknown".to_string(),
                    ),
                };
                FactorExplanation {
                    factor_type: factor.factor_type.clone(),
                    name: factor.name.clone(),
                    explanation,
                    criticality,
                }
            })
            .collect()
    }

    /// Generate actionable recommendations
    fn generate_recommendations(
        chain: &MultiFactorCausalChain,
    ) -> Vec<RecommendedAction> {
        let mut actions = Vec::new();

        for (i, intervention) in chain.intervention_points.iter().enumerate().take(3) {
            let (priority, effort) = match chain.predicted_severity.as_str() {
                "Critical" => {
                    if i == 0 {
                        ("critical", "moderate")
                    } else {
                        ("high", "easy")
                    }
                }
                "High" => ("high", if i == 0 { "moderate" } else { "easy" }),
                _ => ("medium", "easy"),
            };

            actions.push(RecommendedAction {
                title: intervention.recommended_action.clone(),
                description: format!(
                    "This is a {} intervention at the {} point of the failure cascade.",
                    intervention.intervention_type,
                    if i == 0 {
                        "initial"
                    } else if i == chain.intervention_points.len() - 1 {
                        "final"
                    } else {
                        "intermediate"
                    }
                ),
                implementation: Self::generate_implementation_steps(&intervention.intervention_type),
                effectiveness: intervention.effectiveness_score,
                priority: priority.to_string(),
                effort: effort.to_string(),
            });
        }

        actions
    }

    /// Generate implementation steps for an intervention
    fn generate_implementation_steps(intervention_type: &str) -> String {
        match intervention_type {
            "prevent" => {
                "1. Identify root environmental cause\n\
                 2. Apply protective measure (coating, shielding, or design change)\n\
                 3. Validate effectiveness in simulation\n\
                 4. Test in controlled environment\n\
                 5. Deploy to fleet with monitoring"
                    .to_string()
            }
            "compensate" => {
                "1. Add redundant sensing or fallback algorithm\n\
                 2. Tune thresholds for early detection\n\
                 3. Test failover mechanism\n\
                 4. Monitor for effectiveness\n\
                 5. Adjust confidence thresholds as needed"
                    .to_string()
            }
            "fallback" => {
                "1. Define safe fallback state or behavior\n\
                 2. Implement automatic detection and transition\n\
                 3. Add safety constraints to fallback mode\n\
                 4. Test graceful degradation\n\
                 5. Communicate degraded behavior to operators"
                    .to_string()
            }
            _ => "Implement appropriate intervention strategy.".to_string(),
        }
    }

    /// Assess escalation risk
    fn assess_escalation_risk(severity: &str) -> String {
        match severity {
            "Critical" => {
                "CRITICAL ESCALATION RISK: This incident requires immediate attention. \
                 If not addressed, expect repeated failures and potential safety incidents."
                    .to_string()
            }
            "High" => {
                "HIGH ESCALATION RISK: This gap will likely recur under similar conditions. \
                 Recommend priority implementation of preventive measures."
                    .to_string()
            }
            "Medium" => {
                "MODERATE ESCALATION RISK: This gap may recur intermittently. \
                 Plan mitigation within current development cycle."
                    .to_string()
            }
            "Low" => {
                "LOW ESCALATION RISK: This is a rare edge case. \
                 Monitor and consider for future hardening efforts."
                    .to_string()
            }
            _ => "Unknown severity level.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_chain() -> MultiFactorCausalChain {
        use crate::analyzers::multi_factor_causality::{ChainEvent, ContributingFactor, InterventionPoint};

        MultiFactorCausalChain {
            chain_id: "test_chain".to_string(),
            ultimate_effect: "Undetected obstacles → Collision".to_string(),
            root_environmental_cause: "Rain (80%) + Humidity (90%)".to_string(),
            events: vec![
                ChainEvent {
                    timestamp_sec: 0.0,
                    event_type: "environmental".to_string(),
                    description: "Rain detected".to_string(),
                    confidence: 0.85,
                    downstream_effects: vec!["Water accumulates".to_string()],
                },
                ChainEvent {
                    timestamp_sec: 5.0,
                    event_type: "gap".to_string(),
                    description: "Optical contamination detected".to_string(),
                    confidence: 0.78,
                    downstream_effects: vec!["Image degraded".to_string()],
                },
            ],
            chain_confidence: 0.70,
            contributing_factors: vec![
                ContributingFactor {
                    factor_type: "gap".to_string(),
                    name: "Optical Contamination".to_string(),
                    magnitude: 0.7,
                    confidence: 0.78,
                },
                ContributingFactor {
                    factor_type: "environmental".to_string(),
                    name: "Rain".to_string(),
                    magnitude: 0.8,
                    confidence: 0.85,
                },
            ],
            predicted_severity: "Critical".to_string(),
            intervention_points: vec![
                InterventionPoint {
                    location_in_chain: 0,
                    intervention_type: "prevent".to_string(),
                    recommended_action: "Apply hydrophobic coating".to_string(),
                    effectiveness_score: 0.95,
                },
            ],
        }
    }

    #[test]
    fn test_narrative_generation() {
        let chain = create_test_chain();
        let gaps = vec![];

        let narrative = IncidentNarrativeGenerator::from_causal_chain(&chain, &gaps);

        assert!(!narrative.executive_summary.is_empty());
        assert!(!narrative.what_happened.is_empty());
        assert!(!narrative.why_it_happened.is_empty());
        assert!(!narrative.impact_description.is_empty());
    }

    #[test]
    fn test_factor_explanations() {
        let chain = create_test_chain();
        let gaps = vec![];

        let narrative = IncidentNarrativeGenerator::from_causal_chain(&chain, &gaps);

        assert_eq!(narrative.contributing_factors_explained.len(), 2);
        assert!(narrative.contributing_factors_explained[0].explanation.len() > 0);
    }

    #[test]
    fn test_recommendations_priority() {
        let chain = create_test_chain();
        let gaps = vec![];

        let narrative = IncidentNarrativeGenerator::from_causal_chain(&chain, &gaps);

        assert!(!narrative.recommended_actions.is_empty());
        // Critical severity should have critical priority for first action
        if narrative.recommended_actions.len() > 0 {
            assert_eq!(narrative.recommended_actions[0].priority, "critical");
        }
    }
}
