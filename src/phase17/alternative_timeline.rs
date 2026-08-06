/// Alternative Timeline Generation - Compare actual vs. counterfactual timelines

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeTimeline {
    pub timeline_id: String,
    pub scenario_name: String,
    pub divergence_point: usize,     // Index where timeline diverges
    pub divergence_reason: String,   // What changed at divergence point
    pub events: Vec<TimelineEvent>,
    pub final_outcome: String,
    pub duration_ms: i32,
    pub estimated_success_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub event_index: usize,
    pub event_type: String,
    pub timestamp_ms: i32,
    pub is_divergent: bool,  // True if different from actual timeline
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineComparison {
    pub actual_timeline: AlternativeTimeline,
    pub alternative_timeline: AlternativeTimeline,
    pub divergence_points: Vec<usize>,
    pub total_divergent_events: usize,
    pub outcome_similarity: f32,  // 0-1, where 1 = identical outcomes
    pub insights: Vec<String>,
}

pub struct AlternativeTimelineGenerator {
    baseline_timeline: Vec<TimelineEvent>,
    baseline_outcome: String,
}

impl AlternativeTimelineGenerator {
    pub fn new(baseline_outcome: String) -> Self {
        AlternativeTimelineGenerator {
            baseline_timeline: Vec::new(),
            baseline_outcome,
        }
    }

    pub fn add_event(&mut self, event: TimelineEvent) {
        self.baseline_timeline.push(event);
    }

    /// Generate an alternative timeline by modifying an event at divergence_point
    pub fn generate_alternative(
        &self,
        divergence_point: usize,
        modification: &str,
        alternative_outcome: String,
    ) -> AlternativeTimeline {
        let mut alt_events = self.baseline_timeline.clone();

        // Mark divergence
        if let Some(event) = alt_events.get_mut(divergence_point) {
            event.is_divergent = true;
        }

        // Cascade changes after divergence point
        let divergent_count = self._cascade_changes(&mut alt_events, divergence_point);

        AlternativeTimeline {
            timeline_id: format!("alt_{}_{}", divergence_point, modification.replace(" ", "_")),
            scenario_name: format!("Alternative: {}", modification),
            divergence_point,
            divergence_reason: modification.to_string(),
            events: alt_events,
            final_outcome: alternative_outcome,
            duration_ms: self._calculate_duration(&self.baseline_timeline),
            estimated_success_rate: self._estimate_success_rate(divergence_point),
        }
    }

    /// Compare actual timeline with alternative
    pub fn compare_timelines(
        &self,
        actual: AlternativeTimeline,
        alternative: AlternativeTimeline,
    ) -> TimelineComparison {
        let divergence_points = self._find_divergence_points(&actual.events, &alternative.events);

        let outcome_similarity = if actual.final_outcome == alternative.final_outcome {
            1.0
        } else if actual.final_outcome.contains("success") && alternative.final_outcome.contains("failure") {
            0.0
        } else {
            0.5
        };

        let mut insights = Vec::new();

        // Generate insights
        let div_count = divergence_points.len();
        if div_count == 0 {
            insights.push("Timelines are identical despite modification".to_string());
        } else if div_count < 5 {
            insights.push(format!(
                "Only {} events differ - modification had limited impact",
                div_count
            ));
        } else {
            insights.push(format!(
                "{} events differ - modification significantly altered mission",
                div_count
            ));
        }

        // Outcome insight
        if actual.final_outcome != alternative.final_outcome {
            insights.push(format!(
                "Different outcomes: actual='{}', alternative='{}'",
                actual.final_outcome, alternative.final_outcome
            ));
        }

        // Duration impact
        if actual.duration_ms != alternative.duration_ms {
            let diff = (alternative.duration_ms - actual.duration_ms) as f32 / actual.duration_ms as f32;
            insights.push(format!(
                "Duration impact: {:.1}% {}",
                diff.abs() * 100.0,
                if diff > 0.0 { "slower" } else { "faster" }
            ));
        }

        TimelineComparison {
            actual_timeline: actual,
            alternative_timeline: alternative,
            divergence_points,
            total_divergent_events: div_count,
            outcome_similarity,
            insights,
        }
    }

    fn _cascade_changes(&self, events: &mut [TimelineEvent], divergence_point: usize) -> usize {
        let mut count = 0;

        for i in (divergence_point + 1)..events.len() {
            // Probabilistically mark subsequent events as divergent
            if i % 3 == divergence_point % 3 {
                events[i].is_divergent = true;
                count += 1;
            }
        }

        count
    }

    fn _find_divergence_points(
        &self,
        actual: &[TimelineEvent],
        alternative: &[TimelineEvent],
    ) -> Vec<usize> {
        let mut divergences = Vec::new();

        for (i, (a, b)) in actual.iter().zip(alternative.iter()).enumerate() {
            if a.event_type != b.event_type || a.is_divergent != b.is_divergent {
                divergences.push(i);
            }
        }

        divergences
    }

    fn _calculate_duration(&self, events: &[TimelineEvent]) -> i32 {
        if events.is_empty() {
            0
        } else {
            events.last().map(|e| e.timestamp_ms).unwrap_or(0)
        }
    }

    fn _estimate_success_rate(&self, divergence_point: usize) -> f32 {
        // Early divergence = more time to recover = higher success
        let recovery_factor = 1.0 - (divergence_point as f32 / 100.0).min(1.0);
        0.5 + (recovery_factor * 0.4)  // Range 0.5 to 0.9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_event_creation() {
        let event = TimelineEvent {
            event_index: 0,
            event_type: "decision".to_string(),
            timestamp_ms: 1000,
            is_divergent: false,
            confidence: 0.95,
        };

        assert_eq!(event.event_index, 0);
        assert!(!event.is_divergent);
    }

    #[test]
    fn test_timeline_generator() {
        let generator = AlternativeTimelineGenerator::new("success".to_string());
        assert_eq!(generator.baseline_outcome, "success");
    }

    #[test]
    fn test_alternative_generation() {
        let mut generator = AlternativeTimelineGenerator::new("success".to_string());

        generator.add_event(TimelineEvent {
            event_index: 0,
            event_type: "start".to_string(),
            timestamp_ms: 0,
            is_divergent: false,
            confidence: 1.0,
        });

        let alt = generator.generate_alternative(0, "Different planner", "partial_success".to_string());
        assert_eq!(alt.scenario_name, "Alternative: Different planner");
        assert!(alt.events.len() > 0);
    }

    #[test]
    fn test_timeline_comparison() {
        let actual = AlternativeTimeline {
            timeline_id: "actual".to_string(),
            scenario_name: "Actual".to_string(),
            divergence_point: 0,
            divergence_reason: "baseline".to_string(),
            events: vec![],
            final_outcome: "success".to_string(),
            duration_ms: 5000,
            estimated_success_rate: 0.95,
        };

        let alternative = AlternativeTimeline {
            timeline_id: "alt".to_string(),
            scenario_name: "Alternative".to_string(),
            divergence_point: 5,
            divergence_reason: "different decision".to_string(),
            events: vec![],
            final_outcome: "failure".to_string(),
            duration_ms: 3000,
            estimated_success_rate: 0.60,
        };

        let generator = AlternativeTimelineGenerator::new("success".to_string());
        let comparison = generator.compare_timelines(actual, alternative);

        assert!(comparison.insights.len() > 0);
        assert_eq!(comparison.outcome_similarity, 0.0);  // Different outcomes
    }
}
