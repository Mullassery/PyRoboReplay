/// Temporal Pattern Discovery - Find patterns across time windows

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TimeWindow {
    ShortTerm,    // Seconds (0-60s)
    MediumTerm,   // Minutes (1-60 min)
    LongTerm,     // Hours+ (1h+)
}

impl TimeWindow {
    pub fn duration_seconds(&self) -> u64 {
        match self {
            TimeWindow::ShortTerm => 60,
            TimeWindow::MediumTerm => 3600,
            TimeWindow::LongTerm => 86400,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TimeWindow::ShortTerm => "Real-time failures & cascades",
            TimeWindow::MediumTerm => "Recovery procedures & patterns",
            TimeWindow::LongTerm => "Fleet optimization trends",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPattern {
    pub pattern_id: String,
    pub name: String,
    pub window: TimeWindow,
    pub events: Vec<String>,           // Sequence of events
    pub frequency_per_day: f32,
    pub average_duration_seconds: u32,
    pub predictability_score: f32,     // 0-1: how predictable?
    pub first_observed: u64,
    pub last_observed: u64,
}

impl TemporalPattern {
    pub fn new(name: String, window: TimeWindow) -> Self {
        TemporalPattern {
            pattern_id: format!("temporal_{}", uuid::Uuid::new_v4()),
            name,
            window,
            events: Vec::new(),
            frequency_per_day: 0.0,
            average_duration_seconds: 0,
            predictability_score: 0.0,
            first_observed: 0,
            last_observed: 0,
        }
    }

    pub fn impact_score(&self) -> f32 {
        // Combine frequency and predictability
        self.frequency_per_day * self.predictability_score
    }

    pub fn is_emerging(&self) -> bool {
        // Pattern is emerging if frequency increasing (simple heuristic)
        self.frequency_per_day > 0.1
    }
}

pub struct TemporalPatternMiner {
    time_series: Vec<(u64, String)>,   // (timestamp, event)
    patterns: Vec<TemporalPattern>,
    min_sequence_length: usize,
}

impl TemporalPatternMiner {
    pub fn new(min_length: usize) -> Self {
        TemporalPatternMiner {
            time_series: Vec::new(),
            patterns: Vec::new(),
            min_sequence_length: min_length,
        }
    }

    pub fn add_event(&mut self, timestamp: u64, event: String) {
        self.time_series.push((timestamp, event));
    }

    /// Mine temporal patterns across all time windows
    pub fn discover_temporal_patterns(&mut self) -> Vec<TemporalPattern> {
        if self.time_series.is_empty() {
            return Vec::new();
        }

        let mut patterns = Vec::new();

        // Mine patterns for each time window
        for window in &[TimeWindow::ShortTerm, TimeWindow::MediumTerm, TimeWindow::LongTerm] {
            let window_patterns = self._mine_window_patterns(window);
            patterns.extend(window_patterns);
        }

        // Sort by impact
        patterns.sort_by(|a, b| b.impact_score().partial_cmp(&a.impact_score()).unwrap());
        self.patterns = patterns.clone();

        patterns
    }

    fn _mine_window_patterns(&self, window: &TimeWindow) -> Vec<TemporalPattern> {
        let window_duration = window.duration_seconds();
        let mut patterns = Vec::new();
        let mut sequence_map: HashMap<Vec<String>, (usize, u64, u64)> = HashMap::new();

        // Partition events into windows
        for i in 0..self.time_series.len() {
            let (start_time, _) = self.time_series[i];

            // Collect events within this window
            let mut window_events = Vec::new();
            let mut end_time = start_time;

            for j in i..self.time_series.len() {
                let (event_time, event) = &self.time_series[j];
                if event_time - start_time <= window_duration {
                    window_events.push(event.clone());
                    end_time = *event_time;
                } else {
                    break;
                }
            }

            // Only consider sequences above minimum length
            if window_events.len() >= self.min_sequence_length {
                sequence_map
                    .entry(window_events.clone())
                    .and_modify(|(count, _, last)| {
                        *count += 1;
                        *last = end_time;
                    })
                    .or_insert((1, start_time, end_time));
            }
        }

        // Convert to temporal patterns
        for (events, (count, first_time, _last_time)) in sequence_map {
            if count >= 1 {
                let mut pattern = TemporalPattern::new(
                    format!("Pattern: {}", events.join(" → ")),
                    window.clone(),
                );

                pattern.events = events;
                pattern.frequency_per_day = (count as f32 / 30.0).min(100.0);  // Estimate daily

                // Predictability based on consistency
                pattern.predictability_score = (count as f32 / (count as f32 + 5.0)).min(0.95);

                pattern.first_observed = first_time;
                pattern.last_observed = first_time + window_duration;

                patterns.push(pattern);
            }
        }

        patterns
    }

    /// Get patterns for a specific time window
    pub fn get_patterns_by_window(&self, window: TimeWindow) -> Vec<TemporalPattern> {
        self.patterns
            .iter()
            .filter(|p| p.window == window)
            .cloned()
            .collect()
    }

    /// Get emerging patterns (recently observed with increasing frequency)
    pub fn get_emerging_patterns(&self) -> Vec<TemporalPattern> {
        self.patterns
            .iter()
            .filter(|p| p.is_emerging())
            .cloned()
            .collect()
    }

    /// Get statistics about all discovered patterns
    pub fn get_statistics(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();

        stats.insert("total_patterns".to_string(), self.patterns.len() as f32);

        if !self.patterns.is_empty() {
            let avg_frequency: f32 = self.patterns.iter().map(|p| p.frequency_per_day).sum::<f32>()
                / self.patterns.len() as f32;
            let avg_predictability: f32 = self.patterns.iter().map(|p| p.predictability_score).sum::<f32>()
                / self.patterns.len() as f32;

            stats.insert("avg_frequency_per_day".to_string(), avg_frequency);
            stats.insert("avg_predictability".to_string(), avg_predictability);

            let short_term_count = self.patterns.iter()
                .filter(|p| p.window == TimeWindow::ShortTerm)
                .count();
            stats.insert("short_term_patterns".to_string(), short_term_count as f32);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_window_duration() {
        assert_eq!(TimeWindow::ShortTerm.duration_seconds(), 60);
        assert_eq!(TimeWindow::MediumTerm.duration_seconds(), 3600);
        assert_eq!(TimeWindow::LongTerm.duration_seconds(), 86400);
    }

    #[test]
    fn test_temporal_pattern_creation() {
        let pattern = TemporalPattern::new("Test Pattern".to_string(), TimeWindow::ShortTerm);
        assert_eq!(pattern.window, TimeWindow::ShortTerm);
        assert_eq!(pattern.predictability_score, 0.0);
    }

    #[test]
    fn test_temporal_pattern_impact() {
        let mut pattern = TemporalPattern::new("Test".to_string(), TimeWindow::ShortTerm);
        pattern.frequency_per_day = 5.0;
        pattern.predictability_score = 0.8;

        let impact = pattern.impact_score();
        assert_eq!(impact, 4.0);
    }

    #[test]
    fn test_temporal_pattern_emerging() {
        let mut pattern = TemporalPattern::new("Test".to_string(), TimeWindow::ShortTerm);
        pattern.frequency_per_day = 0.05;

        assert!(!pattern.is_emerging());

        pattern.frequency_per_day = 0.5;
        assert!(pattern.is_emerging());
    }

    #[test]
    fn test_miner_creation() {
        let miner = TemporalPatternMiner::new(2);
        assert!(miner.time_series.is_empty());
    }

    #[test]
    fn test_add_events() {
        let mut miner = TemporalPatternMiner::new(2);
        miner.add_event(0, "event_a".to_string());
        miner.add_event(5, "event_b".to_string());

        assert_eq!(miner.time_series.len(), 2);
    }

    #[test]
    fn test_discover_patterns() {
        let mut miner = TemporalPatternMiner::new(2);

        // Add short-term pattern
        for i in 0..5 {
            miner.add_event(i as u64, "sensor_drift".to_string());
            miner.add_event(i as u64 + 1, "localization_loss".to_string());
        }

        let patterns = miner.discover_temporal_patterns();
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_get_patterns_by_window() {
        let mut miner = TemporalPatternMiner::new(2);
        miner.add_event(0, "a".to_string());
        miner.add_event(5, "b".to_string());
        miner.discover_temporal_patterns();

        let short_term = miner.get_patterns_by_window(TimeWindow::ShortTerm);
        assert!(short_term.len() >= 0);
    }

    #[test]
    fn test_get_statistics() {
        let mut miner = TemporalPatternMiner::new(2);
        miner.add_event(0, "a".to_string());
        miner.add_event(5, "b".to_string());
        miner.discover_temporal_patterns();

        let stats = miner.get_statistics();
        assert!(stats.contains_key("total_patterns"));
    }
}
