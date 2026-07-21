use crate::core::event::MissionEvent;
use crate::core::{CausalHypothesis, CausalQuery};
use serde::{Deserialize, Serialize};

/// ASCII visualization for causal analysis
pub struct CausalViz;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalFlowChart {
    /// ASCII representation of the causal flow
    pub diagram: String,
    /// Statistics about the flow
    pub stats: FlowChartStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowChartStats {
    pub total_chains: usize,
    pub avg_chain_length: f32,
    pub avg_confidence: f32,
    pub longest_time_gap_ms: i64,
    pub shortest_time_gap_ms: i64,
}

impl CausalViz {
    /// Generate ASCII flowchart for query results
    pub fn render_query(query: &CausalQuery, events: &[MissionEvent]) -> CausalFlowChart {
        let mut lines = Vec::new();

        lines.push("╔════════════════════════════════════════════════════════════════╗".to_string());
        lines.push(format!(
            "║ Causal Analysis: Event #{} - {} │",
            query.target_event_idx,
            events
                .get(query.target_event_idx)
                .map(|e| e.event_type())
                .unwrap_or("?"),
        ));
        lines.push("╠════════════════════════════════════════════════════════════════╣".to_string());

        if query.hypotheses.is_empty() {
            lines.push("║ No causal relationships found                                      ║".to_string());
        } else {
            for (rank, hypothesis) in query.hypotheses.iter().enumerate() {
                lines.push(String::new());
                lines.push(format!(
                    "║ Hypothesis {} [Confidence: {:.0}%]",
                    rank + 1,
                    hypothesis.confidence * 100.0
                ));

                // Draw the causal chain as ASCII flowchart
                let chain_viz = Self::_render_chain(
                    &hypothesis.chain.event_chain,
                    events,
                    hypothesis.confidence,
                );
                for line in chain_viz.split('\n') {
                    if !line.is_empty() {
                        lines.push(format!("║ {}", line));
                    }
                }

                // Add metadata
                lines.push(format!(
                    "║   └─ {} ({}ms total)",
                    hypothesis.explanation, hypothesis.total_time_gap_ms
                ));
            }
        }

        lines.push("╚════════════════════════════════════════════════════════════════╝".to_string());

        let diagram = lines.join("\n");
        let stats = Self::_compute_stats(query);

        CausalFlowChart { diagram, stats }
    }

    fn _render_chain(event_indices: &[usize], events: &[MissionEvent], confidence: f32) -> String {
        if event_indices.is_empty() {
            return String::new();
        }

        let mut result = Vec::new();
        let confidence_level = Self::_confidence_to_level(confidence);

        for (i, &event_idx) in event_indices.iter().enumerate() {
            let event_type = events
                .get(event_idx)
                .map(|e| e.event_type())
                .unwrap_or("?");

            let event_box = Self::_create_event_box(event_type, confidence_level);
            result.push(event_box);

            if i < event_indices.len() - 1 {
                result.push("      │".to_string());
                result.push("      ↓".to_string());
                result.push("      │".to_string());
            }
        }

        result.join("\n")
    }

    fn _create_event_box(event_type: &str, confidence_level: char) -> String {
        let box_char = match confidence_level {
            '█' => '█', // Very high confidence
            '▓' => '▓', // High confidence
            '▒' => '▒', // Medium confidence
            '░' => '░', // Low confidence
            _ => '·',   // Very low confidence
        };

        format!("  {}{} {}", box_char, box_char, event_type)
    }

    fn _confidence_to_level(confidence: f32) -> char {
        match (confidence * 10.0) as u32 {
            9..=10 => '█',
            7..=8 => '▓',
            5..=6 => '▒',
            3..=4 => '░',
            _ => '·',
        }
    }

    /// Generate summary statistics visualization
    pub fn render_summary(query: &CausalQuery) -> String {
        if query.hypotheses.is_empty() {
            return "No causal relationships found".to_string();
        }

        let stats = Self::_compute_stats(query);

        format!(
            "╔══════════════════════════════════════════════════╗\n\
             ║            Causal Analysis Summary               ║\n\
             ╠══════════════════════════════════════════════════╣\n\
             ║ Total Causal Chains:    {:<27} ║\n\
             ║ Average Chain Length:   {:<27.1} ║\n\
             ║ Average Confidence:     {:<26.0}% ║\n\
             ║ Min Time Gap:           {:<23} ms ║\n\
             ║ Max Time Gap:           {:<23} ms ║\n\
             ╚══════════════════════════════════════════════════╝",
            stats.total_chains,
            stats.avg_chain_length,
            stats.avg_confidence * 100.0,
            stats.shortest_time_gap_ms,
            stats.longest_time_gap_ms
        )
    }

    fn _compute_stats(query: &CausalQuery) -> FlowChartStats {
        if query.hypotheses.is_empty() {
            return FlowChartStats {
                total_chains: 0,
                avg_chain_length: 0.0,
                avg_confidence: 0.0,
                longest_time_gap_ms: 0,
                shortest_time_gap_ms: 0,
            };
        }

        let total_chains = query.hypotheses.len();
        let avg_chain_length = query
            .hypotheses
            .iter()
            .map(|h| h.chain.length() as f32)
            .sum::<f32>()
            / total_chains as f32;
        let avg_confidence = query
            .hypotheses
            .iter()
            .map(|h| h.confidence)
            .sum::<f32>()
            / total_chains as f32;

        let longest_time_gap = query
            .hypotheses
            .iter()
            .map(|h| h.total_time_gap_ms)
            .max()
            .unwrap_or(0);

        let shortest_time_gap = query
            .hypotheses
            .iter()
            .map(|h| h.total_time_gap_ms)
            .min()
            .unwrap_or(0);

        FlowChartStats {
            total_chains,
            avg_chain_length,
            avg_confidence,
            longest_time_gap_ms: longest_time_gap,
            shortest_time_gap_ms: shortest_time_gap,
        }
    }

    /// Render comparison of multiple hypotheses side-by-side
    pub fn render_comparison(hypotheses: &[CausalHypothesis], events: &[MissionEvent]) -> String {
        let mut lines = Vec::new();

        lines.push("╔════════════════════════════════════════════════════════════════╗".to_string());
        lines.push("║            Causal Hypothesis Comparison                       ║".to_string());
        lines.push("╠════════════════════════════════════════════════════════════════╣".to_string());

        for (rank, hypothesis) in hypotheses.iter().enumerate() {
            lines.push(String::new());
            lines.push(format!(
                "║ Path {}:  Confidence {:.0}%  │  Chain length: {}",
                rank + 1,
                hypothesis.confidence * 100.0,
                hypothesis.chain.length()
            ));

            // Render chain compactly
            let chain_str = hypothesis
                .chain
                .event_chain
                .iter()
                .map(|&idx| {
                    events
                        .get(idx)
                        .map(|e| Self::_event_short_name(e.event_type()))
                        .unwrap_or("?".to_string())
                })
                .collect::<Vec<_>>()
                .join(" → ");

            lines.push(format!("║    {}", chain_str));
        }

        lines.push("╚════════════════════════════════════════════════════════════════╝".to_string());

        lines.join("\n")
    }

    fn _event_short_name(event_type: &str) -> String {
        match event_type {
            "lidar_scan" => "LIDAR".to_string(),
            "camera_frame" => "CAM".to_string(),
            "imu_data" => "IMU".to_string(),
            "odometry_update" => "ODOM".to_string(),
            "costmap_update" => "MAP".to_string(),
            "robot_pose" => "POSE".to_string(),
            "navigation_decision" => "NAV".to_string(),
            "obstacle_detected" => "OBST".to_string(),
            "communication_event" => "COMM".to_string(),
            "coordination_event" => "COORD".to_string(),
            "environmental_change" => "ENV".to_string(),
            "mission_lifecycle" => "LIFE".to_string(),
            _ => event_type[..4.min(event_type.len())].to_uppercase(),
        }
    }

    /// Render confidence heatmap for event sequence
    pub fn render_confidence_timeline(query: &CausalQuery, events: &[MissionEvent]) -> String {
        let mut lines = Vec::new();

        lines.push("╔═══════════════════════════════════════════════════════════════╗".to_string());
        lines.push("║           Confidence Timeline Visualization                  ║".to_string());
        lines.push("╠═══════════════════════════════════════════════════════════════╣".to_string());

        let mut confidence_map = vec![0.0; events.len()];

        for hypothesis in &query.hypotheses {
            for &event_idx in &hypothesis.chain.event_chain {
                if event_idx < confidence_map.len() {
                    confidence_map[event_idx] =
                        f32::max(confidence_map[event_idx], hypothesis.confidence);
                }
            }
        }

        for (idx, &confidence) in confidence_map.iter().enumerate() {
            if confidence > 0.0 {
                let bar = Self::_render_confidence_bar(confidence);
                let event_type = events
                    .get(idx)
                    .map(|e| e.event_type())
                    .unwrap_or("?");
                lines.push(format!("║ [{}] {:20} {} {:.0}%", idx, event_type, bar, confidence * 100.0));
            }
        }

        lines.push("╚═══════════════════════════════════════════════════════════════╝".to_string());

        lines.join("\n")
    }

    fn _render_confidence_bar(confidence: f32) -> String {
        let filled = (confidence * 20.0) as usize;
        let empty = 20 - filled;

        format!(
            "[{}{}]",
            "█".repeat(filled),
            "░".repeat(empty)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_to_level() {
        assert_eq!(CausalViz::_confidence_to_level(0.95), '█');
        assert_eq!(CausalViz::_confidence_to_level(0.75), '▓');
        assert_eq!(CausalViz::_confidence_to_level(0.55), '▒');
        assert_eq!(CausalViz::_confidence_to_level(0.35), '░');
        assert_eq!(CausalViz::_confidence_to_level(0.05), '·');
    }

    #[test]
    fn test_event_short_name() {
        assert_eq!(CausalViz::_event_short_name("lidar_scan"), "LIDAR");
        assert_eq!(CausalViz::_event_short_name("navigation_decision"), "NAV");
        assert_eq!(CausalViz::_event_short_name("obstacle_detected"), "OBST");
    }

    #[test]
    fn test_confidence_bar_rendering() {
        let bar_high = CausalViz::_render_confidence_bar(0.9);
        let bar_low = CausalViz::_render_confidence_bar(0.2);

        assert!(bar_high.contains("█"));
        assert!(bar_low.contains("░"));
    }

    #[test]
    fn test_flowchart_creation() {
        use crate::core::{CausalChain, CausalHypothesis};
        use chrono::Utc;

        let base_time = Utc::now();
        let events = vec![
            MissionEvent::LidarScan {
                robot_id: "robot_1".to_string(),
                timestamp: base_time,
                data: crate::core::event::LidarData {
                    ranges: vec![5.0; 360],
                    intensities: None,
                    frame_id: "lidar".to_string(),
                    min_angle: 0.0,
                    max_angle: 6.28,
                    angle_increment: 0.01745,
                    range_min: 0.1,
                    range_max: 10.0,
                },
            },
        ];

        let hypothesis = CausalHypothesis {
            chain: CausalChain::new(vec![0], 0.9),
            confidence: 0.9,
            explanation: "Test".to_string(),
            direct_cause_type: Some("test_type".to_string()),
            total_time_gap_ms: 100,
        };

        let viz = CausalViz::_render_chain(&hypothesis.chain.event_chain, &events, 0.9);
        assert!(viz.contains("lidar_scan"));
    }
}
