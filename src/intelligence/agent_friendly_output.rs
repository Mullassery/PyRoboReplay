//! Agent-Friendly Output Format
//!
//! Converts replay analysis into structured format suitable for
//! AI agents to reason over without reprocessing raw sensor data.

use serde::{Deserialize, Serialize};

/// Event as understood by AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub timestamp: f32,
    pub event_type: String,
    pub confidence: f32,
    pub details: std::collections::HashMap<String, serde_json::Value>,
}

/// Complete mission as agent-friendly summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMission {
    pub mission_id: String,
    pub duration_sec: f32,
    pub key_events: Vec<AgentEvent>,
    pub perception_gaps: Vec<String>,
    pub root_causes: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Generates agent-friendly output
pub struct AgentOutputGenerator;

impl AgentOutputGenerator {
    /// Convert analysis to agent-friendly format
    pub fn generate_agent_summary(
        mission_id: &str,
        duration_sec: f32,
    ) -> AgentMission {
        AgentMission {
            mission_id: mission_id.to_string(),
            duration_sec,
            key_events: vec![],
            perception_gaps: vec![],
            root_causes: vec![],
            recommendations: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_summary_generation() {
        let summary = AgentOutputGenerator::generate_agent_summary("mission_1", 300.0);
        assert_eq!(summary.mission_id, "mission_1");
        assert_eq!(summary.duration_sec, 300.0);
    }
}
