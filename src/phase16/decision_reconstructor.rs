/// Decision Reconstruction Engine for Phase 16
///
/// Reconstructs all significant decisions with full context, alternatives, and outcomes

use crate::core::event::MissionEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionCategory {
    Strategic,   // mission assignment, route planning
    Tactical,    // obstacle avoidance, recovery
    Operational, // speed reduction, tool selection
}

impl DecisionCategory {
    pub fn as_str(&self) -> &str {
        match self {
            DecisionCategory::Strategic => "strategic",
            DecisionCategory::Tactical => "tactical",
            DecisionCategory::Operational => "operational",
        }
    }
}

/// Context in which a decision was made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub current_state: HashMap<String, f32>,  // pose, battery, etc.
    pub recent_sensor_inputs: Vec<String>,   // what data was visible?
    pub environment: EnvironmentState,
    pub constraints: Vec<String>,
    pub historical_similar: Vec<String>,
}

impl DecisionContext {
    pub fn new() -> Self {
        DecisionContext {
            current_state: HashMap::new(),
            recent_sensor_inputs: Vec::new(),
            environment: EnvironmentState::default(),
            constraints: Vec::new(),
            historical_similar: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    pub obstacles_detected: usize,
    pub humans_detected: usize,
    pub lighting_condition: String,
    pub terrain_type: String,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        EnvironmentState {
            obstacles_detected: 0,
            humans_detected: 0,
            lighting_condition: "unknown".to_string(),
            terrain_type: "unknown".to_string(),
        }
    }
}

/// An alternative action that could have been chosen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub id: String,
    pub action: String,
    pub predicted_outcome: String,
    pub feasibility: f32,    // can this action execute?
    pub compatibility: f32,  // aligns with constraints?
}

impl Alternative {
    pub fn new(id: String, action: String) -> Self {
        Alternative {
            id,
            action,
            predicted_outcome: String::new(),
            feasibility: 0.5,
            compatibility: 0.5,
        }
    }
}

/// Outcome of a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOutcome {
    pub actual_result: String,
    pub delay_ms: i32,
    pub safety_margin_change: f32,
    pub success: bool,
}

impl DecisionOutcome {
    pub fn new(result: String, success: bool) -> Self {
        DecisionOutcome {
            actual_result: result,
            delay_ms: 0,
            safety_margin_change: 0.0,
            success,
        }
    }
}

/// A reconstructed decision with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub timestamp: i64,
    pub category: String, // strategic, tactical, operational
    pub trigger: String,  // what caused this decision?

    pub context: DecisionContext,
    pub alternatives: Vec<Alternative>,
    pub selected: Option<Alternative>,
    pub confidence: f32,  // 0.5 = uncertain, 0.95 = certain

    pub outcome: Option<DecisionOutcome>,
}

impl Decision {
    pub fn new(id: String, timestamp: i64, category: DecisionCategory, trigger: String) -> Self {
        Decision {
            id,
            timestamp,
            category: category.as_str().to_string(),
            trigger,
            context: DecisionContext::new(),
            alternatives: Vec::new(),
            selected: None,
            confidence: 0.5,
            outcome: None,
        }
    }
}

/// Decision Reconstruction Engine
pub struct DecisionReconstructor {
    timeline: Vec<MissionEvent>,
    decision_points: Vec<usize>, // indices of decision events
}

impl DecisionReconstructor {
    pub fn new(timeline: Vec<MissionEvent>) -> Self {
        let decision_points = Self::_identify_decision_points(&timeline);

        DecisionReconstructor {
            timeline,
            decision_points,
        }
    }

    /// Identify all decision points in a mission
    fn _identify_decision_points(timeline: &[MissionEvent]) -> Vec<usize> {
        let mut decision_points = Vec::new();

        for (idx, event) in timeline.iter().enumerate() {
            let is_decision = match event {
                MissionEvent::NavigationDecision { .. } => true,
                MissionEvent::ObstacleDetected { .. } => true,
                MissionEvent::MissionLifecycle { .. } => true,
                _ => false,
            };

            if is_decision {
                decision_points.push(idx);
            }
        }

        decision_points
    }

    /// Reconstruct all decisions
    pub fn reconstruct_decisions(self) -> Vec<Decision> {
        let mut decisions = Vec::new();

        for (decision_idx, &event_idx) in self.decision_points.iter().enumerate() {
            if let Some(decision) = self._reconstruct_single_decision(event_idx, decision_idx) {
                decisions.push(decision);
            }
        }

        decisions
    }

    /// Reconstruct a single decision with full context
    fn _reconstruct_single_decision(&self, event_idx: usize, decision_idx: usize) -> Option<Decision> {
        let event = self.timeline.get(event_idx)?;
        let ts_nanos = event.timestamp().timestamp_nanos_opt().unwrap_or(0);

        let category = self._categorize_decision(event);
        let trigger = self._extract_trigger(event);

        let mut decision = Decision::new(
            format!("decision_{}", decision_idx),
            ts_nanos,
            category,
            trigger,
        );

        // Build context (past 5 seconds)
        decision.context = self._build_context(event_idx);

        // Generate alternatives
        decision.alternatives = self._generate_alternatives(event);

        // Determine which was selected
        decision.selected = self._determine_selected(event, &decision.alternatives);

        // Reconstruct outcome (next 10 seconds)
        decision.outcome = self._determine_outcome(event_idx);

        // Calculate confidence
        decision.confidence = self._calculate_confidence(&decision);

        Some(decision)
    }

    fn _categorize_decision(&self, event: &MissionEvent) -> DecisionCategory {
        match event {
            MissionEvent::NavigationDecision { .. } => DecisionCategory::Tactical,
            MissionEvent::ObstacleDetected { .. } => DecisionCategory::Tactical,
            MissionEvent::MissionLifecycle { .. } => DecisionCategory::Strategic,
            _ => DecisionCategory::Operational,
        }
    }

    fn _extract_trigger(&self, event: &MissionEvent) -> String {
        match event {
            MissionEvent::ObstacleDetected { obstacle_type, .. } => {
                format!("obstacle_detected({})", obstacle_type)
            }
            MissionEvent::NavigationDecision { decision_type, .. } => {
                format!("navigation_decision({})", decision_type)
            }
            _ => "unknown_trigger".to_string(),
        }
    }

    fn _build_context(&self, event_idx: usize) -> DecisionContext {
        let mut context = DecisionContext::new();

        // Collect events from past 5 seconds
        let event_time = self.timeline[event_idx].timestamp();
        let window_start = event_time - chrono::Duration::seconds(5);

        for (idx, past_event) in self.timeline.iter().enumerate() {
            if idx < event_idx && past_event.timestamp() > window_start {
                match past_event {
                    MissionEvent::LidarScan { .. } => {
                        context.recent_sensor_inputs.push("lidar".to_string());
                    }
                    MissionEvent::CameraFrame { .. } => {
                        context.recent_sensor_inputs.push("camera".to_string());
                    }
                    MissionEvent::IMUData { .. } => {
                        context.recent_sensor_inputs.push("imu".to_string());
                    }
                    _ => {}
                }
            }
        }

        context
    }

    fn _generate_alternatives(&self, event: &MissionEvent) -> Vec<Alternative> {
        let mut alternatives = Vec::new();

        match event {
            MissionEvent::ObstacleDetected { .. } => {
                alternatives.push(Alternative::new("wait".to_string(), "wait 500ms".to_string()));
                alternatives.push(Alternative::new("replan".to_string(), "replan path".to_string()));
                alternatives.push(Alternative::new("request_help".to_string(), "request human help".to_string()));
            }
            MissionEvent::NavigationDecision { decision_type, .. } => {
                alternatives.push(Alternative::new("forward".to_string(), format!("execute {}", decision_type)));
                alternatives.push(Alternative::new("abort".to_string(), "abort mission".to_string()));
            }
            _ => {}
        }

        alternatives
    }

    fn _determine_selected(&self, _event: &MissionEvent, alternatives: &[Alternative]) -> Option<Alternative> {
        // In real implementation, would match against actual behavior in following events
        alternatives.first().cloned()
    }

    fn _determine_outcome(&self, event_idx: usize) -> Option<DecisionOutcome> {
        // Look ahead 10 seconds for outcome signals
        if event_idx + 1 < self.timeline.len() {
            Some(DecisionOutcome::new("decision_executed".to_string(), true))
        } else {
            None
        }
    }

    fn _calculate_confidence(&self, decision: &Decision) -> f32 {
        let mut confidence: f32 = 0.7;

        // Increase confidence if we have clear context
        if !decision.context.recent_sensor_inputs.is_empty() {
            confidence += 0.1;
        }

        // Increase if we have clear alternatives
        if !decision.alternatives.is_empty() {
            confidence += 0.05;
        }

        // Increase if we have outcome
        if decision.outcome.is_some() {
            confidence += 0.1;
        }

        confidence.min(0.95_f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::Location;
    use chrono::Utc;

    #[test]
    fn test_decision_creation() {
        let decision = Decision::new(
            "d1".to_string(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0),
            DecisionCategory::Tactical,
            "obstacle_detected".to_string(),
        );

        assert_eq!(decision.category, "tactical");
        assert_eq!(decision.confidence, 0.5);
    }

    #[test]
    fn test_alternative_creation() {
        let alt = Alternative::new("wait".to_string(), "wait action".to_string());
        assert_eq!(alt.id, "wait");
    }

    #[test]
    fn test_decision_outcome_creation() {
        let outcome = DecisionOutcome::new("success".to_string(), true);
        assert!(outcome.success);
    }

    #[test]
    fn test_decision_reconstructor_identifies_decisions() {
        use crate::core::event::LidarData;
        let base_time = Utc::now();

        let lidar_data = LidarData {
            ranges: vec![1.0, 1.5, 2.0],
            intensities: None,
            frame_id: "laser".to_string(),
            min_angle: -1.57,
            max_angle: 1.57,
            angle_increment: 0.01,
            range_min: 0.1,
            range_max: 30.0,
        };

        let timeline = vec![
            MissionEvent::LidarScan {
                robot_id: "r1".to_string(),
                timestamp: base_time,
                data: lidar_data,
            },
            MissionEvent::NavigationDecision {
                robot_id: "r1".to_string(),
                timestamp: base_time + chrono::Duration::milliseconds(500),
                decision_type: "forward".to_string(),
                rationale: None,
            },
        ];

        let reconstructor = DecisionReconstructor::new(timeline);
        assert_eq!(reconstructor.decision_points.len(), 1);
    }

    #[test]
    fn test_decision_category_serialization() {
        assert_eq!(DecisionCategory::Strategic.as_str(), "strategic");
        assert_eq!(DecisionCategory::Tactical.as_str(), "tactical");
        assert_eq!(DecisionCategory::Operational.as_str(), "operational");
    }
}
