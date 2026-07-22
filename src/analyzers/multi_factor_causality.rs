//! Multi-Factor Causal Inference
//!
//! Combines gaps + drift + quality into unified causal chains.
//! Shows how multiple phenomena interact to cause failures.

use crate::analyzers::gap_to_causal::GapCausalEvent;
use crate::analyzers::drift_detection::DriftStats;
use crate::analyzers::quality_confidence::QualityMetadata;
use crate::analyzers::RealityGapFinding;
use std::collections::HashMap;

/// Complete causal chain combining gaps, drift, and quality factors
#[derive(Debug, Clone)]
pub struct MultiFactorCausalChain {
    /// Unique identifier for this chain
    pub chain_id: String,

    /// The final failure or outcome
    pub ultimate_effect: String,

    /// Root environmental condition that triggered everything
    pub root_environmental_cause: String,

    /// All events in the chain (chronological order)
    pub events: Vec<ChainEvent>,

    /// Overall confidence in this causal chain (0.0-1.0)
    pub chain_confidence: f32,

    /// Contributing factors (gaps, drift, quality issues)
    pub contributing_factors: Vec<ContributingFactor>,

    /// Predicted severity of ultimate effect
    pub predicted_severity: String,

    /// Recommended intervention points
    pub intervention_points: Vec<InterventionPoint>,
}

/// Individual event in a causal chain
#[derive(Debug, Clone)]
pub struct ChainEvent {
    pub timestamp_sec: f32,
    pub event_type: String, // "gap", "drift", "quality", "environmental"
    pub description: String,
    pub confidence: f32,
    pub downstream_effects: Vec<String>,
}

/// A factor contributing to the overall causal chain
#[derive(Debug, Clone)]
pub struct ContributingFactor {
    pub factor_type: String, // "gap", "drift", "quality"
    pub name: String,
    pub magnitude: f32,
    pub confidence: f32,
}

/// Where to intervene to break the causal chain
#[derive(Debug, Clone)]
pub struct InterventionPoint {
    pub location_in_chain: usize, // Index into events
    pub intervention_type: String, // "compensate", "prevent", "fallback"
    pub recommended_action: String,
    pub effectiveness_score: f32,
}

/// Constructs multi-factor causal chains
pub struct MultiFactorInferenceEngine;

impl MultiFactorInferenceEngine {
    /// Build complete causal chains from gaps, drift, and quality
    pub fn construct_chains(
        gaps: &[RealityGapFinding],
        drifts: &[DriftStats],
        quality_context: &HashMap<String, QualityMetadata>,
        environmental_conditions: &HashMap<String, f32>,
    ) -> Vec<MultiFactorCausalChain> {
        let mut chains = Vec::new();

        // Pattern 1: Optical contamination + detection drift + poor quality
        if let Some(optical_gap) = gaps.iter().find(|g| g.category.contains("Optical")) {
            if let Some(detect_drift) = drifts.iter().find(|d| d.metric.contains("detection")) {
                let chain = Self::optical_contamination_chain(
                    optical_gap,
                    detect_drift,
                    quality_context,
                    environmental_conditions,
                );
                chains.push(chain);
            }
        }

        // Pattern 2: Thermal + CPU throttle + latency
        if let Some(thermal_gap) = gaps.iter().find(|g| g.category.contains("Thermal")) {
            if let Some(latency_drift) = drifts.iter().find(|d| d.metric.contains("latency")) {
                let chain = Self::thermal_throttle_chain(
                    thermal_gap,
                    latency_drift,
                    quality_context,
                    environmental_conditions,
                );
                chains.push(chain);
            }
        }

        // Pattern 3: Clock drift + localization + mechanical
        if let Some(clock_gap) = gaps.iter().find(|g| g.category.contains("Clock")) {
            if let Some(mech_gap) = gaps.iter().find(|g| g.category.contains("Mechanical")) {
                let chain = Self::timing_navigation_chain(
                    clock_gap,
                    mech_gap,
                    quality_context,
                );
                chains.push(chain);
            }
        }

        // Sort by chain confidence
        chains.sort_by(|a, b| {
            b.chain_confidence
                .partial_cmp(&a.chain_confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        chains
    }

    /// Optical contamination chain with environment + quality factors
    fn optical_contamination_chain(
        gap: &RealityGapFinding,
        drift: &DriftStats,
        quality: &HashMap<String, QualityMetadata>,
        environment: &HashMap<String, f32>,
    ) -> MultiFactorCausalChain {
        let rain_probability = environment.get("rain_probability").copied().unwrap_or(0.0);
        let humidity = environment.get("humidity").copied().unwrap_or(0.5);

        let mut events = vec![
            ChainEvent {
                timestamp_sec: 0.0,
                event_type: "environmental".to_string(),
                description: format!("Rain/humidity detected (rain: {:.0}%, humidity: {:.0}%)",
                    rain_probability * 100.0, humidity * 100.0),
                confidence: rain_probability * 0.9 + humidity * 0.3,
                downstream_effects: vec!["Water accumulates on optics".to_string()],
            },
            ChainEvent {
                timestamp_sec: gap.detection_time_sec.unwrap_or(0.0),
                event_type: "gap".to_string(),
                description: "Optical contamination detected".to_string(),
                confidence: gap.confidence * 0.92,
                downstream_effects: vec!["Image quality degraded".to_string()],
            },
            ChainEvent {
                timestamp_sec: gap.detection_time_sec.unwrap_or(0.0) + 0.05,
                event_type: "drift".to_string(),
                description: format!("Detection confidence drifting {:.1}σ", drift.drift_sigma),
                confidence: drift.confidence * gap.confidence,
                downstream_effects: vec!["Object detection fails".to_string()],
            },
            ChainEvent {
                timestamp_sec: gap.detection_time_sec.unwrap_or(0.0) + 0.15,
                event_type: "quality".to_string(),
                description: "Data quality metrics degrade".to_string(),
                confidence: quality
                    .get("camera")
                    .map(|q| 1.0 - q.overall_quality)
                    .unwrap_or(0.5),
                downstream_effects: vec!["System confidence in perception drops".to_string()],
            },
        ];

        let chain_confidence = (gap.confidence * drift.confidence * 0.85).min(1.0);

        MultiFactorCausalChain {
            chain_id: format!("optical_contamination_{:?}", gap.detection_time_sec),
            ultimate_effect: "Undetected obstacles → Collision".to_string(),
            root_environmental_cause: format!(
                "Rain ({:.0}%) + Humidity ({:.0}%)",
                rain_probability * 100.0,
                humidity * 100.0
            ),
            events,
            chain_confidence,
            contributing_factors: vec![
                ContributingFactor {
                    factor_type: "gap".to_string(),
                    name: "Optical Contamination".to_string(),
                    magnitude: gap.reality_gap_score,
                    confidence: gap.confidence,
                },
                ContributingFactor {
                    factor_type: "drift".to_string(),
                    name: "Detection Confidence Drift".to_string(),
                    magnitude: drift.drift_sigma,
                    confidence: drift.confidence,
                },
                ContributingFactor {
                    factor_type: "environmental".to_string(),
                    name: "Rain/Humidity".to_string(),
                    magnitude: rain_probability.max(humidity),
                    confidence: rain_probability * 0.9 + humidity * 0.3,
                },
            ],
            predicted_severity: "Critical".to_string(),
            intervention_points: vec![
                InterventionPoint {
                    location_in_chain: 0,
                    intervention_type: "prevent".to_string(),
                    recommended_action: "Apply hydrophobic coating to optics".to_string(),
                    effectiveness_score: 0.95,
                },
                InterventionPoint {
                    location_in_chain: 1,
                    intervention_type: "compensate".to_string(),
                    recommended_action: "Use optical flow as fallback detection".to_string(),
                    effectiveness_score: 0.75,
                },
                InterventionPoint {
                    location_in_chain: 2,
                    intervention_type: "fallback".to_string(),
                    recommended_action: "Increase safety margins until weather clears".to_string(),
                    effectiveness_score: 0.85,
                },
            ],
        }
    }

    /// Thermal throttling chain
    fn thermal_throttle_chain(
        gap: &RealityGapFinding,
        drift: &DriftStats,
        quality: &HashMap<String, QualityMetadata>,
        environment: &HashMap<String, f32>,
    ) -> MultiFactorCausalChain {
        let temperature = environment.get("temperature_c").copied().unwrap_or(25.0);

        let events = vec![
            ChainEvent {
                timestamp_sec: 0.0,
                event_type: "environmental".to_string(),
                description: format!("High ambient temperature: {:.0}°C", temperature),
                confidence: if temperature > 45.0 { 0.9 } else { 0.5 },
                downstream_effects: vec!["Motor/CPU heat accumulation".to_string()],
            },
            ChainEvent {
                timestamp_sec: gap.detection_time_sec.unwrap_or(0.0),
                event_type: "gap".to_string(),
                description: "Thermal accumulation detected".to_string(),
                confidence: gap.confidence,
                downstream_effects: vec!["Thermal throttling initiated".to_string()],
            },
            ChainEvent {
                timestamp_sec: gap.detection_time_sec.unwrap_or(0.0) + 0.05,
                event_type: "drift".to_string(),
                description: format!("Latency drifting {:.1}σ", drift.drift_sigma),
                confidence: drift.confidence,
                downstream_effects: vec!["Real-time guarantees broken".to_string()],
            },
        ];

        let chain_confidence = (gap.confidence * drift.confidence * 0.80).min(1.0);

        MultiFactorCausalChain {
            chain_id: format!("thermal_throttle_{:?}", gap.detection_time_sec),
            ultimate_effect: "Late collision avoidance → Collision".to_string(),
            root_environmental_cause: format!("High temperature: {:.0}°C", temperature),
            events,
            chain_confidence,
            contributing_factors: vec![
                ContributingFactor {
                    factor_type: "gap".to_string(),
                    name: "Thermal Effects".to_string(),
                    magnitude: gap.reality_gap_score,
                    confidence: gap.confidence,
                },
                ContributingFactor {
                    factor_type: "drift".to_string(),
                    name: "Latency Drift".to_string(),
                    magnitude: drift.drift_sigma,
                    confidence: drift.confidence,
                },
                ContributingFactor {
                    factor_type: "environmental".to_string(),
                    name: "Ambient Temperature".to_string(),
                    magnitude: temperature / 100.0,
                    confidence: if temperature > 45.0 { 0.9 } else { 0.5 },
                },
            ],
            predicted_severity: "High".to_string(),
            intervention_points: vec![
                InterventionPoint {
                    location_in_chain: 0,
                    intervention_type: "prevent".to_string(),
                    recommended_action: "Improve cooling system or reduce mission duration".to_string(),
                    effectiveness_score: 0.9,
                },
                InterventionPoint {
                    location_in_chain: 1,
                    intervention_type: "compensate".to_string(),
                    recommended_action: "Reduce CPU load before throttling occurs".to_string(),
                    effectiveness_score: 0.7,
                },
            ],
        }
    }

    /// Timing + navigation chain
    fn timing_navigation_chain(
        clock_gap: &RealityGapFinding,
        mech_gap: &RealityGapFinding,
        _quality: &HashMap<String, QualityMetadata>,
    ) -> MultiFactorCausalChain {
        let events = vec![
            ChainEvent {
                timestamp_sec: clock_gap.detection_time_sec.unwrap_or(0.0),
                event_type: "gap".to_string(),
                description: "Clock synchronization lost".to_string(),
                confidence: clock_gap.confidence,
                downstream_effects: vec!["Sensor fusion fails".to_string()],
            },
            ChainEvent {
                timestamp_sec: mech_gap.detection_time_sec.unwrap_or(0.0),
                event_type: "gap".to_string(),
                description: "Mechanical degradation detected".to_string(),
                confidence: mech_gap.confidence,
                downstream_effects: vec!["Motion estimates unreliable".to_string()],
            },
            ChainEvent {
                timestamp_sec: (clock_gap.detection_time_sec.unwrap_or(0.0)
                    + mech_gap.detection_time_sec.unwrap_or(0.0))
                    / 2.0,
                event_type: "gap".to_string(),
                description: "Localization divergence".to_string(),
                confidence: (clock_gap.confidence * mech_gap.confidence).min(1.0),
                downstream_effects: vec!["Navigation commands in wrong reference frame".to_string()],
            },
        ];

        let chain_confidence = (clock_gap.confidence * mech_gap.confidence * 0.75).min(1.0);

        MultiFactorCausalChain {
            chain_id: format!(
                "timing_navigation_{:?}",
                clock_gap.detection_time_sec.unwrap_or(0.0)
            ),
            ultimate_effect: "Navigation failure → Position error → Collision".to_string(),
            root_environmental_cause: "Multi-sensor desynchronization".to_string(),
            events,
            chain_confidence,
            contributing_factors: vec![
                ContributingFactor {
                    factor_type: "gap".to_string(),
                    name: "Clock Drift".to_string(),
                    magnitude: clock_gap.reality_gap_score,
                    confidence: clock_gap.confidence,
                },
                ContributingFactor {
                    factor_type: "gap".to_string(),
                    name: "Mechanical Degradation".to_string(),
                    magnitude: mech_gap.reality_gap_score,
                    confidence: mech_gap.confidence,
                },
            ],
            predicted_severity: "High".to_string(),
            intervention_points: vec![
                InterventionPoint {
                    location_in_chain: 0,
                    intervention_type: "prevent".to_string(),
                    recommended_action: "Implement PTP clock synchronization".to_string(),
                    effectiveness_score: 0.95,
                },
                InterventionPoint {
                    location_in_chain: 1,
                    intervention_type: "compensate".to_string(),
                    recommended_action: "Reduce reliance on precise timing for fusion".to_string(),
                    effectiveness_score: 0.6,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_gap(category: &str, confidence: f32, detection_time: f32) -> RealityGapFinding {
        RealityGapFinding {
            domain: crate::analyzers::RealityDomain::Physical,
            category: category.to_string(),
            finding_type: format!("Test {}", category),
            severity: crate::analyzers::Severity::Medium,
            confidence,
            reality_gap_score: 0.7,
            description: "Test gap".to_string(),
            evidence: vec![],
            metrics: HashMap::new(),
            sim_recreation_suggestion: "Test".to_string(),
            remediation: "Test".to_string(),
            detection_time_sec: Some(detection_time),
        }
    }

    fn create_test_drift(metric: &str, sigma: f32) -> DriftStats {
        DriftStats {
            drift_sigma: sigma,
            drift_direction: 1.0,
            confidence: 0.85,
            metric: metric.to_string(),
            drift_type: "trend".to_string(),
        }
    }

    #[test]
    fn test_optical_contamination_chain() {
        let gaps = vec![create_test_gap("Optical Contamination", 0.78, 100.0)];
        let drifts = vec![create_test_drift("detection_confidence", 2.5)];
        let quality = HashMap::new();
        let mut environment = HashMap::new();
        environment.insert("rain_probability".to_string(), 0.8);
        environment.insert("humidity".to_string(), 0.9);

        let chains = MultiFactorInferenceEngine::construct_chains(&gaps, &drifts, &quality, &environment);

        assert!(chains.len() > 0);
        assert!(chains[0].chain_confidence > 0.5);
        assert_eq!(chains[0].predicted_severity, "Critical");
    }

    #[test]
    fn test_thermal_chain() {
        let gaps = vec![create_test_gap("Thermal Effects", 0.82, 50.0)];
        let drifts = vec![create_test_drift("latency", 1.8)];
        let quality = HashMap::new();
        let mut environment = HashMap::new();
        environment.insert("temperature_c".to_string(), 48.0);

        let chains = MultiFactorInferenceEngine::construct_chains(&gaps, &drifts, &quality, &environment);

        assert!(chains.len() > 0);
        assert_eq!(chains[0].predicted_severity, "High");
    }

    #[test]
    fn test_intervention_points() {
        let gaps = vec![
            create_test_gap("Clock Drift", 0.9, 10.0),
            create_test_gap("Mechanical Degradation", 0.8, 20.0),
        ];
        let drifts = vec![];
        let quality = HashMap::new();
        let environment = HashMap::new();

        let chains = MultiFactorInferenceEngine::construct_chains(&gaps, &drifts, &quality, &environment);

        assert!(chains.len() > 0);
        assert!(chains[0].intervention_points.len() > 0);

        // Verify intervention effectiveness scores
        for intervention in &chains[0].intervention_points {
            assert!(intervention.effectiveness_score > 0.0);
            assert!(intervention.effectiveness_score <= 1.0);
        }
    }
}
