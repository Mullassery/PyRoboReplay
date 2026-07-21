use crate::core::causality::CausalLink;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Coordination event between multiple robots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationEvent {
    /// Timestamp of coordination event
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Involved robot IDs
    pub robots: Vec<String>,
    /// Event type (rendezvous, handoff, avoidance, formation, etc.)
    pub event_type: String,
    /// Associated location
    pub location: (f64, f64, f64),
    /// Distance between robots at event (meters)
    pub inter_robot_distance: f64,
    /// Confidence in coordination event (0.0-1.0)
    pub confidence: f32,
}

/// Inter-robot communication pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationLink {
    /// Source robot ID
    pub from_robot: String,
    /// Target robot ID
    pub to_robot: String,
    /// Communication type (command, status, map_share, etc.)
    pub comm_type: String,
    /// First communication at timestamp
    pub first_seen: chrono::DateTime<chrono::Utc>,
    /// Last communication at timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Number of communication events
    pub event_count: usize,
    /// Bandwidth estimate (bytes/second, if available)
    pub bandwidth: Option<f32>,
}

/// Fleet topology snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetSnapshot {
    /// Timestamp of snapshot
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Robot positions and states
    pub robots: HashMap<String, RobotState>,
    /// Pairwise distances between robots
    pub pairwise_distances: HashMap<(String, String), f64>,
    /// Current fleet formation type
    pub formation: String,
    /// Centroid of fleet
    pub centroid: (f64, f64, f64),
    /// Spread (max inter-robot distance)
    pub spread: f64,
}

/// Snapshot of individual robot state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotState {
    /// Robot ID
    pub robot_id: String,
    /// Position
    pub position: (f64, f64, f64),
    /// Velocity
    pub velocity: (f64, f64, f64),
    /// Heading (radians)
    pub heading: f64,
    /// Battery level (0.0-1.0, if available)
    pub battery: Option<f32>,
    /// Status (active, idle, charging, failed, etc.)
    pub status: String,
}

/// Causal link between robots (e.g., robot A's decision caused robot B's action)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterRobotCausalLink {
    /// Source robot ID
    pub source_robot: String,
    /// Target robot ID
    pub target_robot: String,
    /// Source event
    pub source_event_idx: usize,
    /// Target event
    pub target_event_idx: usize,
    /// Causal relationship type (leader_follower, handoff, collision_avoidance, etc.)
    pub relationship_type: String,
    /// Time lag between events (milliseconds)
    pub time_lag_ms: i64,
    /// Confidence in causal link (0.0-1.0)
    pub confidence: f32,
    /// Physical distance between robots at link time (meters)
    pub physical_distance: f64,
}

/// Coordination pattern (recurring sequence of inter-robot events)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationPattern {
    /// Pattern ID
    pub id: String,
    /// Robots involved
    pub robots: Vec<String>,
    /// Pattern type (rendezvous, handoff_chain, synchronization, etc.)
    pub pattern_type: String,
    /// Number of times pattern occurred
    pub occurrence_count: usize,
    /// Average time between pattern repeats (milliseconds)
    pub avg_repeat_interval: Option<u64>,
    /// Pattern efficiency metric (0.0-1.0)
    pub efficiency: f32,
}

/// Multi-robot coordination analyzer
pub struct MultiRobotCoordinationAnalyzer {
    /// Coordination events
    coordination_events: Vec<CoordinationEvent>,
    /// Communication links between robots
    communication_links: HashMap<(String, String), CommunicationLink>,
    /// Fleet snapshots over time
    fleet_snapshots: Vec<FleetSnapshot>,
    /// Inter-robot causal links
    inter_robot_causality: Vec<InterRobotCausalLink>,
    /// Detected patterns
    patterns: Vec<CoordinationPattern>,
    /// Robot positions over time (for distance calculations)
    robot_trajectories: HashMap<String, Vec<(chrono::DateTime<chrono::Utc>, (f64, f64, f64))>>,
}

impl MultiRobotCoordinationAnalyzer {
    /// Create new multi-robot coordination analyzer
    pub fn new() -> Self {
        MultiRobotCoordinationAnalyzer {
            coordination_events: Vec::new(),
            communication_links: HashMap::new(),
            fleet_snapshots: Vec::new(),
            inter_robot_causality: Vec::new(),
            patterns: Vec::new(),
            robot_trajectories: HashMap::new(),
        }
    }

    /// Add coordination event
    pub fn add_coordination_event(&mut self, event: CoordinationEvent) {
        self.coordination_events.push(event);
    }

    /// Add communication link between robots
    pub fn add_communication_link(&mut self, link: CommunicationLink) {
        let key = (link.from_robot.clone(), link.to_robot.clone());
        self.communication_links.insert(key, link);
    }

    /// Add fleet snapshot
    pub fn add_fleet_snapshot(&mut self, snapshot: FleetSnapshot) {
        // Update robot trajectories
        for (robot_id, state) in &snapshot.robots {
            self.robot_trajectories
                .entry(robot_id.clone())
                .or_insert_with(Vec::new)
                .push((snapshot.timestamp, state.position));
        }

        self.fleet_snapshots.push(snapshot);
    }

    /// Add inter-robot causal link
    pub fn add_inter_robot_link(&mut self, link: InterRobotCausalLink) {
        self.inter_robot_causality.push(link);
    }

    /// Get active robots in fleet
    pub fn active_robots(&self) -> Vec<String> {
        let mut robots = std::collections::HashSet::new();
        for event in &self.coordination_events {
            for robot in &event.robots {
                robots.insert(robot.clone());
            }
        }
        robots.into_iter().collect()
    }

    /// Get coordination events between specific robots
    pub fn events_between(&self, robot_a: &str, robot_b: &str) -> Vec<&CoordinationEvent> {
        self.coordination_events
            .iter()
            .filter(|event| {
                (event.robots.contains(&robot_a.to_string())
                    && event.robots.contains(&robot_b.to_string()))
                    || event.robots.contains(&format!("{}-{}", robot_a, robot_b))
            })
            .collect()
    }

    /// Query inter-robot causal links
    pub fn query_inter_robot_causality(
        &self,
        source_robot: &str,
        target_robot: &str,
    ) -> Vec<&InterRobotCausalLink> {
        self.inter_robot_causality
            .iter()
            .filter(|link| link.source_robot == source_robot && link.target_robot == target_robot)
            .collect()
    }

    /// Detect coordination patterns
    pub fn detect_patterns(&mut self, min_occurrences: usize) {
        // Simple pattern detection: group same event sequences
        let mut pattern_map: HashMap<String, Vec<&CoordinationEvent>> = HashMap::new();

        for event in &self.coordination_events {
            let pattern_key = format!("{:?}", event.robots); // Simplified pattern key
            pattern_map
                .entry(pattern_key)
                .or_insert_with(Vec::new)
                .push(event);
        }

        // Convert to CoordinationPattern
        for (pattern_key, events) in pattern_map {
            if events.len() >= min_occurrences {
                let mut robots: Vec<String> = pattern_key
                    .split(',')
                    .map(|s| s.trim_matches(|c| c == '[' || c == ']' || c == '"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                robots.sort();

                let pattern_type = if events.len() >= 3 {
                    "synchronization".to_string()
                } else if events.len() == 2 {
                    "pairwise".to_string()
                } else {
                    "single".to_string()
                };

                // Calculate repeat interval
                let avg_repeat_interval = if events.len() > 1 {
                    let mut intervals = Vec::new();
                    for i in 0..events.len() - 1 {
                        let interval = events[i + 1]
                            .timestamp
                            .signed_duration_since(events[i].timestamp)
                            .num_milliseconds() as u64;
                        intervals.push(interval);
                    }
                    Some(intervals.iter().sum::<u64>() / intervals.len() as u64)
                } else {
                    None
                };

                // Calculate efficiency (average confidence of events)
                let efficiency =
                    events.iter().map(|e| e.confidence).sum::<f32>() / events.len() as f32;

                self.patterns.push(CoordinationPattern {
                    id: format!("pattern_{}", self.patterns.len()),
                    robots,
                    pattern_type,
                    occurrence_count: events.len(),
                    avg_repeat_interval,
                    efficiency,
                });
            }
        }
    }

    /// Get fleet centroid at timestamp
    pub fn fleet_centroid(&self, timestamp: chrono::DateTime<chrono::Utc>) -> Option<(f64, f64, f64)> {
        let snapshot = self
            .fleet_snapshots
            .iter()
            .rev()
            .find(|s| s.timestamp <= timestamp)?;

        if snapshot.robots.is_empty() {
            return None;
        }

        let (sum_x, sum_y, sum_z) = snapshot.robots.values().fold(
            (0.0, 0.0, 0.0),
            |(sx, sy, sz), state| (sx + state.position.0, sy + state.position.1, sz + state.position.2),
        );

        Some((
            sum_x / snapshot.robots.len() as f64,
            sum_y / snapshot.robots.len() as f64,
            sum_z / snapshot.robots.len() as f64,
        ))
    }

    /// Get inter-robot distance at timestamp for specific pair
    pub fn pairwise_distance(
        &self,
        robot_a: &str,
        robot_b: &str,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Option<f64> {
        let snapshot = self
            .fleet_snapshots
            .iter()
            .rev()
            .find(|s| s.timestamp <= timestamp)?;

        let state_a = snapshot.robots.get(robot_a)?;
        let state_b = snapshot.robots.get(robot_b)?;

        let dx = state_a.position.0 - state_b.position.0;
        let dy = state_a.position.1 - state_b.position.1;
        let dz = state_a.position.2 - state_b.position.2;

        Some((dx * dx + dy * dy + dz * dz).sqrt())
    }

    /// Compute multi-robot coordination statistics
    pub fn compute_stats(&self) -> MultiRobotCoordinationStats {
        let robot_count = self.active_robots().len();
        let avg_fleet_spread = self
            .fleet_snapshots
            .iter()
            .map(|s| s.spread)
            .sum::<f64>()
            / self.fleet_snapshots.len().max(1) as f64;

        let avg_coord_confidence = self
            .coordination_events
            .iter()
            .map(|e| e.confidence)
            .sum::<f32>()
            / self.coordination_events.len().max(1) as f32;

        let inter_robot_links = self.inter_robot_causality.len();
        let patterns_detected = self.patterns.len();

        MultiRobotCoordinationStats {
            robot_count,
            coordination_events: self.coordination_events.len(),
            communication_links: self.communication_links.len(),
            avg_fleet_spread,
            avg_coordination_confidence: avg_coord_confidence,
            inter_robot_causal_links: inter_robot_links,
            patterns_detected,
        }
    }

    /// Get coordination patterns
    pub fn patterns(&self) -> &[CoordinationPattern] {
        &self.patterns
    }

    /// Get inter-robot causality links
    pub fn causality_links(&self) -> &[InterRobotCausalLink] {
        &self.inter_robot_causality
    }

    /// Get coordination events
    pub fn coordination_events(&self) -> &[CoordinationEvent] {
        &self.coordination_events
    }
}

impl Default for MultiRobotCoordinationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for multi-robot coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRobotCoordinationStats {
    pub robot_count: usize,
    pub coordination_events: usize,
    pub communication_links: usize,
    pub avg_fleet_spread: f64,
    pub avg_coordination_confidence: f32,
    pub inter_robot_causal_links: usize,
    pub patterns_detected: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = MultiRobotCoordinationAnalyzer::new();
        assert_eq!(analyzer.active_robots().len(), 0);
    }

    #[test]
    fn test_add_coordination_event() {
        let mut analyzer = MultiRobotCoordinationAnalyzer::new();
        let event = CoordinationEvent {
            timestamp: chrono::Utc::now(),
            robots: vec!["robot_1".to_string(), "robot_2".to_string()],
            event_type: "rendezvous".to_string(),
            location: (1.0, 2.0, 0.0),
            inter_robot_distance: 0.5,
            confidence: 0.95,
        };

        analyzer.add_coordination_event(event);
        assert_eq!(analyzer.coordination_events.len(), 1);
    }

    #[test]
    fn test_active_robots() {
        let mut analyzer = MultiRobotCoordinationAnalyzer::new();
        let event = CoordinationEvent {
            timestamp: chrono::Utc::now(),
            robots: vec!["robot_1".to_string(), "robot_2".to_string()],
            event_type: "rendezvous".to_string(),
            location: (1.0, 2.0, 0.0),
            inter_robot_distance: 0.5,
            confidence: 0.95,
        };

        analyzer.add_coordination_event(event);
        assert_eq!(analyzer.active_robots().len(), 2);
    }

    #[test]
    fn test_events_between() {
        let mut analyzer = MultiRobotCoordinationAnalyzer::new();
        let event = CoordinationEvent {
            timestamp: chrono::Utc::now(),
            robots: vec!["robot_1".to_string(), "robot_2".to_string()],
            event_type: "handoff".to_string(),
            location: (1.0, 2.0, 0.0),
            inter_robot_distance: 0.5,
            confidence: 0.90,
        };

        analyzer.add_coordination_event(event);
        let events = analyzer.events_between("robot_1", "robot_2");
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_add_fleet_snapshot() {
        let mut analyzer = MultiRobotCoordinationAnalyzer::new();
        let mut robots = HashMap::new();
        robots.insert(
            "robot_1".to_string(),
            RobotState {
                robot_id: "robot_1".to_string(),
                position: (0.0, 0.0, 0.0),
                velocity: (1.0, 0.0, 0.0),
                heading: 0.0,
                battery: Some(0.8),
                status: "active".to_string(),
            },
        );

        let snapshot = FleetSnapshot {
            timestamp: chrono::Utc::now(),
            robots,
            pairwise_distances: HashMap::new(),
            formation: "line".to_string(),
            centroid: (0.0, 0.0, 0.0),
            spread: 0.0,
        };

        analyzer.add_fleet_snapshot(snapshot);
        assert_eq!(analyzer.fleet_snapshots.len(), 1);
    }

    #[test]
    fn test_add_inter_robot_link() {
        let mut analyzer = MultiRobotCoordinationAnalyzer::new();
        let link = InterRobotCausalLink {
            source_robot: "robot_1".to_string(),
            target_robot: "robot_2".to_string(),
            source_event_idx: 0,
            target_event_idx: 1,
            relationship_type: "leader_follower".to_string(),
            time_lag_ms: 500,
            confidence: 0.85,
            physical_distance: 2.0,
        };

        analyzer.add_inter_robot_link(link);
        assert_eq!(analyzer.inter_robot_causality.len(), 1);
    }

    #[test]
    fn test_detect_patterns() {
        let mut analyzer = MultiRobotCoordinationAnalyzer::new();

        for _ in 0..3 {
            let event = CoordinationEvent {
                timestamp: chrono::Utc::now(),
                robots: vec!["robot_1".to_string(), "robot_2".to_string()],
                event_type: "sync".to_string(),
                location: (0.0, 0.0, 0.0),
                inter_robot_distance: 1.0,
                confidence: 0.9,
            };
            analyzer.add_coordination_event(event);
        }

        analyzer.detect_patterns(2); // Min 2 occurrences
        assert!(analyzer.patterns.len() > 0);
    }

    #[test]
    fn test_compute_stats() {
        let mut analyzer = MultiRobotCoordinationAnalyzer::new();
        let event = CoordinationEvent {
            timestamp: chrono::Utc::now(),
            robots: vec!["robot_1".to_string(), "robot_2".to_string()],
            event_type: "rendezvous".to_string(),
            location: (0.0, 0.0, 0.0),
            inter_robot_distance: 0.5,
            confidence: 0.95,
        };

        analyzer.add_coordination_event(event);
        let stats = analyzer.compute_stats();

        assert_eq!(stats.robot_count, 2);
        assert_eq!(stats.coordination_events, 1);
    }
}
