/// Window Analyzer - Statistical analysis of time windows

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowStatistics {
    pub window_id: String,
    pub start_time: u64,
    pub end_time: u64,
    pub event_count: usize,
    pub unique_events: usize,
    pub failure_rate: f32,
    pub average_event_interval_ms: u32,
    pub entropy: f32,                  // Information entropy of events
    pub anomaly_score: f32,
}

impl WindowStatistics {
    pub fn new(window_id: String, start_time: u64, end_time: u64) -> Self {
        WindowStatistics {
            window_id,
            start_time,
            end_time,
            event_count: 0,
            unique_events: 0,
            failure_rate: 0.0,
            average_event_interval_ms: 0,
            entropy: 0.0,
            anomaly_score: 0.0,
        }
    }

    pub fn duration_seconds(&self) -> u64 {
        self.end_time - self.start_time
    }

    pub fn is_anomalous(&self) -> bool {
        self.anomaly_score > 0.7
    }
}

pub struct WindowAnalyzer {
    windows: Vec<WindowStatistics>,
    event_history: Vec<(u64, String)>,
}

impl WindowAnalyzer {
    pub fn new() -> Self {
        WindowAnalyzer {
            windows: Vec::new(),
            event_history: Vec::new(),
        }
    }

    pub fn add_event(&mut self, timestamp: u64, event: String) {
        self.event_history.push((timestamp, event));
    }

    /// Analyze a specific time window
    pub fn analyze_window(&mut self, start_time: u64, end_time: u64) -> WindowStatistics {
        let window_id = format!("window_{}_{}", start_time, end_time);
        let mut stats = WindowStatistics::new(window_id, start_time, end_time);

        // Filter events in window
        let window_events: Vec<_> = self.event_history
            .iter()
            .filter(|(t, _)| *t >= start_time && *t <= end_time)
            .collect();

        stats.event_count = window_events.len();

        // Count unique events
        let mut event_types: HashMap<String, usize> = HashMap::new();
        for (_, event) in &window_events {
            *event_types.entry(event.clone()).or_insert(0) += 1;
        }
        stats.unique_events = event_types.len();

        // Calculate failure rate
        let failures = window_events.iter()
            .filter(|(_, e)| e.contains("failure") || e.contains("error"))
            .count();
        stats.failure_rate = if window_events.is_empty() {
            0.0
        } else {
            failures as f32 / window_events.len() as f32
        };

        // Calculate average event interval
        if window_events.len() > 1 {
            let mut intervals = Vec::new();
            for i in 0..window_events.len() - 1 {
                let interval = window_events[i + 1].0 - window_events[i].0;
                intervals.push(interval);
            }
            let avg_interval = intervals.iter().sum::<u64>() / intervals.len() as u64;
            stats.average_event_interval_ms = (avg_interval * 1000) as u32;
        }

        // Calculate entropy (information theory metric)
        stats.entropy = self._calculate_entropy(&event_types, window_events.len());

        // Calculate anomaly score
        stats.anomaly_score = self._calculate_anomaly_score(&stats);

        self.windows.push(stats.clone());
        stats
    }

    fn _calculate_entropy(&self, event_counts: &HashMap<String, usize>, total: usize) -> f32 {
        if total == 0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for count in event_counts.values() {
            let p = *count as f32 / total as f32;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    fn _calculate_anomaly_score(&self, stats: &WindowStatistics) -> f32 {
        // Higher anomaly if: high failure rate + high entropy + unusual event count
        let failure_factor = stats.failure_rate * 2.0;  // Weight failures heavily
        let entropy_factor = (stats.entropy / 8.0).min(1.0);  // Normalize to 0-1
        let event_factor = if stats.event_count > 100 { 0.5 } else { 0.0 };

        ((failure_factor + entropy_factor + event_factor) / 3.0).min(1.0)
    }

    /// Get windows exceeding anomaly threshold
    pub fn get_anomalous_windows(&self, threshold: f32) -> Vec<WindowStatistics> {
        self.windows
            .iter()
            .filter(|w| w.anomaly_score > threshold)
            .cloned()
            .collect()
    }

    /// Compare two windows
    pub fn compare_windows(&self, w1_id: &str, w2_id: &str) -> Option<WindowComparison> {
        let window1 = self.windows.iter().find(|w| w.window_id == w1_id)?;
        let window2 = self.windows.iter().find(|w| w.window_id == w2_id)?;

        let failure_rate_diff = (window2.failure_rate - window1.failure_rate).abs();
        let event_count_ratio = window2.event_count as f32 / (window1.event_count.max(1)) as f32;
        let entropy_diff = (window2.entropy - window1.entropy).abs();

        let similarity = 1.0 - (failure_rate_diff + entropy_diff / 8.0).min(1.0);

        Some(WindowComparison {
            window1_id: w1_id.to_string(),
            window2_id: w2_id.to_string(),
            similarity,
            event_count_ratio,
            failure_rate_diff,
        })
    }

    /// Get window statistics
    pub fn get_statistics(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();

        stats.insert("total_windows".to_string(), self.windows.len() as f32);

        if !self.windows.is_empty() {
            let avg_events: f32 = self.windows.iter().map(|w| w.event_count as f32).sum::<f32>()
                / self.windows.len() as f32;
            let avg_failure_rate: f32 = self.windows.iter().map(|w| w.failure_rate).sum::<f32>()
                / self.windows.len() as f32;
            let anomalous_count = self.windows.iter()
                .filter(|w| w.is_anomalous())
                .count();

            stats.insert("avg_events_per_window".to_string(), avg_events);
            stats.insert("avg_failure_rate".to_string(), avg_failure_rate);
            stats.insert("anomalous_windows".to_string(), anomalous_count as f32);
        }

        stats
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowComparison {
    pub window1_id: String,
    pub window2_id: String,
    pub similarity: f32,           // 0-1: how similar?
    pub event_count_ratio: f32,
    pub failure_rate_diff: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_statistics_creation() {
        let stats = WindowStatistics::new("w1".to_string(), 0, 100);
        assert_eq!(stats.duration_seconds(), 100);
        assert_eq!(stats.event_count, 0);
    }

    #[test]
    fn test_window_is_anomalous() {
        let mut stats = WindowStatistics::new("w1".to_string(), 0, 100);
        stats.anomaly_score = 0.5;
        assert!(!stats.is_anomalous());

        stats.anomaly_score = 0.9;
        assert!(stats.is_anomalous());
    }

    #[test]
    fn test_window_analyzer_creation() {
        let analyzer = WindowAnalyzer::new();
        assert!(analyzer.windows.is_empty());
    }

    #[test]
    fn test_add_event() {
        let mut analyzer = WindowAnalyzer::new();
        analyzer.add_event(0, "event_a".to_string());
        analyzer.add_event(10, "event_b".to_string());

        assert_eq!(analyzer.event_history.len(), 2);
    }

    #[test]
    fn test_analyze_window() {
        let mut analyzer = WindowAnalyzer::new();
        analyzer.add_event(5, "start".to_string());
        analyzer.add_event(10, "event_a".to_string());
        analyzer.add_event(20, "event_b".to_string());
        analyzer.add_event(50, "end".to_string());

        let stats = analyzer.analyze_window(0, 100);
        assert_eq!(stats.event_count, 4);
    }

    #[test]
    fn test_analyze_window_failure_rate() {
        let mut analyzer = WindowAnalyzer::new();
        analyzer.add_event(0, "success".to_string());
        analyzer.add_event(10, "failure".to_string());

        let stats = analyzer.analyze_window(0, 100);
        assert_eq!(stats.failure_rate, 0.5);
    }

    #[test]
    fn test_get_anomalous_windows() {
        let mut analyzer = WindowAnalyzer::new();
        analyzer.add_event(0, "event".to_string());
        let mut stats = analyzer.analyze_window(0, 100);
        stats.anomaly_score = 0.9;
        analyzer.windows.push(stats);

        let anomalous = analyzer.get_anomalous_windows(0.7);
        assert!(!anomalous.is_empty());
    }

    #[test]
    fn test_compare_windows() {
        let mut analyzer = WindowAnalyzer::new();
        analyzer.add_event(0, "a".to_string());
        let stats1 = analyzer.analyze_window(0, 100);

        analyzer.add_event(200, "b".to_string());
        let stats2 = analyzer.analyze_window(100, 200);

        let comparison = analyzer.compare_windows(&stats1.window_id, &stats2.window_id);
        assert!(comparison.is_some());
    }

    #[test]
    fn test_window_statistics() {
        let mut analyzer = WindowAnalyzer::new();
        analyzer.add_event(0, "a".to_string());
        analyzer.analyze_window(0, 100);

        let stats = analyzer.get_statistics();
        assert!(stats.contains_key("total_windows"));
    }
}
