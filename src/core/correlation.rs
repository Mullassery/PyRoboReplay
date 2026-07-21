use crate::core::event::MissionEvent;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a correlation between two events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventCorrelation {
    /// Index of first event
    pub event_a_idx: usize,
    /// Index of second event
    pub event_b_idx: usize,
    /// Time gap between events (milliseconds)
    pub time_gap_ms: i64,
    /// Event type of A
    pub event_a_type: String,
    /// Event type of B
    pub event_b_type: String,
    /// Correlation strength (0.0-1.0)
    pub correlation_strength: f32,
    /// Whether this is an anomaly
    pub is_anomaly: bool,
}

/// Statistics about event correlations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationStats {
    pub total_correlations: usize,
    pub anomalies_detected: usize,
    pub avg_correlation_strength: f32,
    pub strongest_correlation: f32,
    pub weakest_correlation: f32,
}

/// Analyzes temporal correlations in mission events
pub struct CorrelationAnalyzer {
    /// Time window for correlation analysis (milliseconds)
    correlation_window_ms: i64,
    /// Anomaly detection threshold (0.0-1.0)
    anomaly_threshold: f32,
}

impl CorrelationAnalyzer {
    pub fn new() -> Self {
        CorrelationAnalyzer {
            correlation_window_ms: 1000, // 1 second default
            anomaly_threshold: 0.85,     // High correlation = potential anomaly
        }
    }

    pub fn with_window(mut self, window_ms: i64) -> Self {
        self.correlation_window_ms = window_ms.max(100);
        self
    }

    pub fn with_anomaly_threshold(mut self, threshold: f32) -> Self {
        self.anomaly_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Analyze all correlations in event sequence
    pub fn analyze(&self, events: &[MissionEvent]) -> Vec<EventCorrelation> {
        let mut correlations = Vec::new();

        for (i, event_a) in events.iter().enumerate() {
            let time_a = event_a.timestamp();

            for (j, event_b) in events.iter().enumerate() {
                if i >= j {
                    continue; // Skip self and duplicates
                }

                let time_b = event_b.timestamp();
                let time_gap = time_b - time_a;

                // Only consider events within correlation window
                if time_gap.num_milliseconds().abs() <= self.correlation_window_ms {
                    if let Some(correlation) = self._compute_correlation(
                        i,
                        event_a,
                        j,
                        event_b,
                        time_gap.num_milliseconds(),
                    ) {
                        correlations.push(correlation);
                    }
                }
            }
        }

        correlations.sort_by(|a, b| {
            b.correlation_strength
                .partial_cmp(&a.correlation_strength)
                .unwrap()
        });

        correlations
    }

    fn _compute_correlation(
        &self,
        idx_a: usize,
        event_a: &MissionEvent,
        idx_b: usize,
        event_b: &MissionEvent,
        time_gap_ms: i64,
    ) -> Option<EventCorrelation> {
        let type_a = event_a.event_type();
        let type_b = event_b.event_type();

        // Heuristic rules for correlation detection
        let (strength, is_anomaly) = match (type_a, type_b) {
            // Sensor → Detection correlations (strong)
            ("lidar_scan", "obstacle_detected") => (0.95, false),
            ("camera_frame", "obstacle_detected") => (0.90, false),

            // Detection → Navigation correlations (strong)
            ("obstacle_detected", "navigation_decision") => (0.92, false),
            ("costmap_update", "navigation_decision") => (0.88, false),

            // Navigation → Motion correlations (moderate)
            ("navigation_decision", "imu_data") => (0.75, false),
            ("navigation_decision", "odometry_update") => (0.85, false),

            // Motion → Sensor feedback loop (moderate)
            ("odometry_update", "lidar_scan") => (0.70, false),
            ("imu_data", "robot_pose") => (0.80, false),

            // Anomalous patterns (detect spikes)
            ("lidar_scan", "lidar_scan") => {
                // Rapid successive lidar scans could indicate scanning anomaly
                if time_gap_ms < 100 {
                    (0.88, true)
                } else {
                    (0.50, false)
                }
            }
            ("obstacle_detected", "obstacle_detected") => {
                // Multiple rapid detections = potential instability
                if time_gap_ms < 200 {
                    (0.90, true)
                } else {
                    (0.55, false)
                }
            }

            // Communication with coordination
            ("communication_event", "coordination_event") => (0.85, false),
            ("coordination_event", "navigation_decision") => (0.80, false),

            // Default weak correlation
            _ => (0.30, false),
        };

        // Apply anomaly threshold
        let is_anomaly = is_anomaly || strength >= self.anomaly_threshold;

        Some(EventCorrelation {
            event_a_idx: idx_a,
            event_b_idx: idx_b,
            time_gap_ms,
            event_a_type: type_a.to_string(),
            event_b_type: type_b.to_string(),
            correlation_strength: strength,
            is_anomaly,
        })
    }

    /// Get statistics about correlations
    pub fn compute_stats(&self, correlations: &[EventCorrelation]) -> CorrelationStats {
        if correlations.is_empty() {
            return CorrelationStats {
                total_correlations: 0,
                anomalies_detected: 0,
                avg_correlation_strength: 0.0,
                strongest_correlation: 0.0,
                weakest_correlation: 0.0,
            };
        }

        let anomalies = correlations.iter().filter(|c| c.is_anomaly).count();
        let avg_strength = correlations
            .iter()
            .map(|c| c.correlation_strength)
            .sum::<f32>()
            / correlations.len() as f32;

        let strongest = correlations
            .iter()
            .map(|c| c.correlation_strength)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let weakest = correlations
            .iter()
            .map(|c| c.correlation_strength)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        CorrelationStats {
            total_correlations: correlations.len(),
            anomalies_detected: anomalies,
            avg_correlation_strength: avg_strength,
            strongest_correlation: strongest,
            weakest_correlation: weakest,
        }
    }

    /// Find anomaly patterns (unusual event sequences)
    pub fn detect_anomaly_patterns(&self, correlations: &[EventCorrelation]) -> Vec<AnomalyPattern> {
        let mut patterns = Vec::new();

        // Group anomalies by event types
        let mut anomaly_map: HashMap<String, Vec<&EventCorrelation>> = HashMap::new();

        for correlation in correlations {
            if correlation.is_anomaly {
                let key = format!(
                    "{}_->_{}",
                    correlation.event_a_type, correlation.event_b_type
                );
                anomaly_map.entry(key).or_insert_with(Vec::new).push(correlation);
            }
        }

        // Convert to patterns
        for (pattern_type, anomalies) in anomaly_map {
            let severity = anomalies
                .iter()
                .map(|a| a.correlation_strength)
                .sum::<f32>()
                / anomalies.len() as f32;

            patterns.push(AnomalyPattern {
                pattern_type,
                count: anomalies.len(),
                avg_severity: severity,
                correlations: anomalies.iter().map(|a| (*a).clone()).collect(),
            });
        }

        // Sort by severity
        patterns.sort_by(|a, b| b.avg_severity.partial_cmp(&a.avg_severity).unwrap());

        patterns
    }

    /// Find correlated event chains (multi-event patterns)
    pub fn find_event_chains(&self, correlations: &[EventCorrelation], min_length: usize) -> Vec<EventChain> {
        let mut chains = Vec::new();

        // Build adjacency graph
        let mut graph: HashMap<usize, Vec<usize>> = HashMap::new();
        for corr in correlations {
            graph
                .entry(corr.event_a_idx)
                .or_insert_with(Vec::new)
                .push(corr.event_b_idx);
        }

        // Find chains via depth-first search
        for &start_idx in graph.keys() {
            let mut visited = std::collections::HashSet::new();
            let chain = self._find_chain(start_idx, &graph, &mut visited, vec![start_idx]);

            if chain.len() >= min_length {
                let correlations_in_chain = self._get_correlations_for_chain(&chain, correlations);
                let avg_strength = correlations_in_chain
                    .iter()
                    .map(|c| c.correlation_strength)
                    .sum::<f32>()
                    / correlations_in_chain.len().max(1) as f32;

                chains.push(EventChain {
                    event_indices: chain,
                    avg_correlation_strength: avg_strength,
                    correlations: correlations_in_chain,
                });
            }
        }

        // Remove duplicate chains
        chains.sort_by_key(|c| {
            (
                std::cmp::Reverse(c.event_indices.len()),
                std::cmp::Reverse((c.avg_correlation_strength * 100.0) as i32),
            )
        });
        chains.dedup_by(|a, b| a.event_indices == b.event_indices);

        chains
    }

    fn _find_chain(
        &self,
        current: usize,
        graph: &HashMap<usize, Vec<usize>>,
        visited: &mut std::collections::HashSet<usize>,
        mut path: Vec<usize>,
    ) -> Vec<usize> {
        visited.insert(current);

        if let Some(neighbors) = graph.get(&current) {
            for &next in neighbors {
                if !visited.contains(&next) {
                    path.push(next);
                    return self._find_chain(next, graph, visited, path);
                }
            }
        }

        path
    }

    fn _get_correlations_for_chain(
        &self,
        chain: &[usize],
        all_correlations: &[EventCorrelation],
    ) -> Vec<EventCorrelation> {
        let mut result = Vec::new();

        for i in 0..chain.len().saturating_sub(1) {
            let curr_idx = chain[i];
            let next_idx = chain[i + 1];

            if let Some(corr) = all_correlations.iter().find(|c| {
                (c.event_a_idx == curr_idx && c.event_b_idx == next_idx)
                    || (c.event_a_idx == next_idx && c.event_b_idx == curr_idx)
            }) {
                result.push(corr.clone());
            }
        }

        result
    }
}

impl Default for CorrelationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an anomaly pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyPattern {
    /// Type of anomaly (e.g., "sensor_spike", "navigation_instability")
    pub pattern_type: String,
    /// How many times this pattern occurred
    pub count: usize,
    /// Average severity (correlation strength)
    pub avg_severity: f32,
    /// Correlations that form this pattern
    pub correlations: Vec<EventCorrelation>,
}

/// Represents a chain of correlated events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChain {
    /// Sequence of event indices
    pub event_indices: Vec<usize>,
    /// Average correlation strength across the chain
    pub avg_correlation_strength: f32,
    /// Correlations in this chain
    pub correlations: Vec<EventCorrelation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = CorrelationAnalyzer::new();
        assert_eq!(analyzer.correlation_window_ms, 1000);
        assert_eq!(analyzer.anomaly_threshold, 0.85);
    }

    #[test]
    fn test_with_window() {
        let analyzer = CorrelationAnalyzer::new().with_window(2000);
        assert_eq!(analyzer.correlation_window_ms, 2000);
    }

    #[test]
    fn test_with_anomaly_threshold() {
        let analyzer = CorrelationAnalyzer::new().with_anomaly_threshold(0.90);
        assert_eq!(analyzer.anomaly_threshold, 0.90);
    }

    #[test]
    fn test_threshold_clamping() {
        let analyzer = CorrelationAnalyzer::new().with_anomaly_threshold(1.5);
        assert_eq!(analyzer.anomaly_threshold, 1.0);
    }

    #[test]
    fn test_correlation_creation() {
        let corr = EventCorrelation {
            event_a_idx: 0,
            event_b_idx: 1,
            time_gap_ms: 500,
            event_a_type: "lidar_scan".to_string(),
            event_b_type: "obstacle_detected".to_string(),
            correlation_strength: 0.95,
            is_anomaly: false,
        };

        assert_eq!(corr.correlation_strength, 0.95);
        assert!(!corr.is_anomaly);
    }

    #[test]
    fn test_correlation_stats() {
        let analyzer = CorrelationAnalyzer::new();
        let correlations = vec![
            EventCorrelation {
                event_a_idx: 0,
                event_b_idx: 1,
                time_gap_ms: 500,
                event_a_type: "a".to_string(),
                event_b_type: "b".to_string(),
                correlation_strength: 0.9,
                is_anomaly: false,
            },
            EventCorrelation {
                event_a_idx: 1,
                event_b_idx: 2,
                time_gap_ms: 500,
                event_a_type: "b".to_string(),
                event_b_type: "c".to_string(),
                correlation_strength: 0.8,
                is_anomaly: false,
            },
        ];

        let stats = analyzer.compute_stats(&correlations);
        assert_eq!(stats.total_correlations, 2);
        assert_eq!(stats.strongest_correlation, 0.9);
        assert_eq!(stats.weakest_correlation, 0.8);
    }

    #[test]
    fn test_anomaly_detection() {
        let correlations = vec![
            EventCorrelation {
                event_a_idx: 0,
                event_b_idx: 1,
                time_gap_ms: 500,
                event_a_type: "a".to_string(),
                event_b_type: "b".to_string(),
                correlation_strength: 0.92,
                is_anomaly: true,
            },
        ];

        let analyzer = CorrelationAnalyzer::new();
        let patterns = analyzer.detect_anomaly_patterns(&correlations);

        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].count, 1);
    }
}
