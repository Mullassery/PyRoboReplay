use crate::core::root_cause::RootCauseAnalysis;
use crate::streaming::channel::StreamEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Single occurrence of a pattern in a mission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionOccurrence {
    pub mission_id: String,
    pub event_idx: usize,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub confidence: f32,
}

/// Pattern learned from missions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionPattern {
    pub pattern_id: Uuid,
    pub pattern_type: String,
    pub occurrences: Vec<MissionOccurrence>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub frequency: f32,
    pub avg_confidence: f32,
}

/// Pattern match result with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_id: Uuid,
    pub pattern_type: String,
    pub confidence: f32,
    pub recommended_action: String,
    pub matched_event_types: Vec<String>,
}

/// Library of learned patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternLibrary {
    pub patterns: HashMap<Uuid, MissionPattern>,
    pub missions_analyzed: usize,
}

impl PatternLibrary {
    /// Create new pattern library
    pub fn new() -> Self {
        PatternLibrary {
            patterns: HashMap::new(),
            missions_analyzed: 0,
        }
    }

    /// Add or update pattern
    pub fn add_pattern(&mut self, pattern: MissionPattern) {
        self.patterns.insert(pattern.pattern_id, pattern);
    }

    /// Retrieve pattern by ID
    pub fn get_pattern(&self, pattern_id: &Uuid) -> Option<&MissionPattern> {
        self.patterns.get(pattern_id)
    }

    /// Get all patterns
    pub fn all_patterns(&self) -> Vec<&MissionPattern> {
        self.patterns.values().collect()
    }

    /// Get patterns by type
    pub fn patterns_by_type(&self, pattern_type: &str) -> Vec<&MissionPattern> {
        self.patterns
            .values()
            .filter(|p| p.pattern_type == pattern_type)
            .collect()
    }

    /// Get most frequent patterns
    pub fn most_frequent(&self, n: usize) -> Vec<&MissionPattern> {
        let mut patterns: Vec<_> = self.patterns.values().collect();
        patterns.sort_by(|a, b| b.frequency.partial_cmp(&a.frequency).unwrap_or(std::cmp::Ordering::Equal));
        patterns.into_iter().take(n).collect()
    }

    /// Serialize for storage
    pub fn serialize_for_storage(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    /// Load from storage
    pub fn load_from_storage(data: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(data)?)
    }
}

impl Default for PatternLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Cross-mission pattern analyzer
#[derive(Debug, Clone)]
pub struct CrossMissionAnalyzer {
    pub library: PatternLibrary,
    pub missions_seen: usize,
}

impl CrossMissionAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        CrossMissionAnalyzer {
            library: PatternLibrary::new(),
            missions_seen: 0,
        }
    }

    /// Learn patterns from mission analysis
    pub fn learn_from_mission(
        &mut self,
        mission_id: &str,
        analysis: &RootCauseAnalysis,
    ) -> Vec<MissionPattern> {
        let mut patterns = Vec::new();

        // Extract patterns from hypotheses
        for hypothesis in &analysis.hypotheses {
            let pattern_type = hypothesis.root_event_type.clone();
            let confidence = hypothesis.confidence;

            let occurrence = MissionOccurrence {
                mission_id: mission_id.to_string(),
                event_idx: 0,
                event_type: pattern_type.clone(),
                timestamp: Utc::now(),
                confidence,
            };

            // Check if pattern already exists
            let pattern_id = match self.library.patterns_by_type(&pattern_type).first() {
                Some(existing) => existing.pattern_id,
                None => Uuid::new_v4(),
            };

            let pattern = if let Some(existing) = self.library.patterns.get_mut(&pattern_id) {
                // Update existing pattern
                existing.occurrences.push(occurrence.clone());
                existing.last_seen = Utc::now();
                existing.frequency += 1.0;

                let total_confidence: f32 = existing.occurrences.iter().map(|o| o.confidence).sum();
                existing.avg_confidence = total_confidence / existing.occurrences.len() as f32;

                existing.clone()
            } else {
                // Create new pattern
                MissionPattern {
                    pattern_id,
                    pattern_type,
                    occurrences: vec![occurrence],
                    first_seen: Utc::now(),
                    last_seen: Utc::now(),
                    frequency: 1.0,
                    avg_confidence: confidence,
                }
            };

            self.library.add_pattern(pattern.clone());
            patterns.push(pattern);
        }

        self.missions_seen += 1;
        patterns
    }

    /// Predict failures based on current events
    pub fn predict_failure(&self, current_events: &[StreamEvent]) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        // Extract event types from current stream
        let event_types: Vec<String> = current_events.iter().map(|e| e.event_type.clone()).collect();

        // Score against all patterns
        for pattern in self.library.all_patterns() {
            let confidence = self.score_event_sequence_against_pattern(&event_types, pattern);

            if confidence > 0.3 {
                matches.push(PatternMatch {
                    pattern_id: pattern.pattern_id,
                    pattern_type: pattern.pattern_type.clone(),
                    confidence,
                    recommended_action: format!("Monitor {} pattern", pattern.pattern_type),
                    matched_event_types: event_types.clone(),
                });
            }
        }

        // Sort by confidence
        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// Score event sequence against pattern
    fn score_event_sequence_against_pattern(&self, event_types: &[String], pattern: &MissionPattern) -> f32 {
        let pattern_event_types: Vec<&String> = pattern.occurrences.iter().map(|o| &o.event_type).collect();

        // Count matching event types in last 10 events
        let recent_events = event_types.iter().rev().take(10);
        let matches = recent_events.filter(|e| pattern_event_types.contains(e)).count();

        let score = (matches as f32 / pattern_event_types.len().max(1) as f32).min(1.0);
        score * pattern.avg_confidence
    }
}

impl Default for CrossMissionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_creation() {
        let library = PatternLibrary::new();
        assert_eq!(library.patterns.len(), 0);
        assert_eq!(library.missions_analyzed, 0);
    }

    #[test]
    fn test_add_pattern() {
        let mut library = PatternLibrary::new();
        let pattern = MissionPattern {
            pattern_id: Uuid::new_v4(),
            pattern_type: "obstacle_collision".to_string(),
            occurrences: vec![],
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            frequency: 1.0,
            avg_confidence: 0.95,
        };

        library.add_pattern(pattern.clone());
        assert_eq!(library.patterns.len(), 1);
    }

    #[test]
    fn test_get_pattern() {
        let mut library = PatternLibrary::new();
        let pattern_id = Uuid::new_v4();
        let pattern = MissionPattern {
            pattern_id,
            pattern_type: "sensor_failure".to_string(),
            occurrences: vec![],
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            frequency: 1.0,
            avg_confidence: 0.9,
        };

        library.add_pattern(pattern.clone());
        let retrieved = library.get_pattern(&pattern_id);

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().pattern_type, "sensor_failure");
    }

    #[test]
    fn test_patterns_by_type() {
        let mut library = PatternLibrary::new();

        for i in 0..3 {
            let pattern = MissionPattern {
                pattern_id: Uuid::new_v4(),
                pattern_type: "navigation_deadlock".to_string(),
                occurrences: vec![],
                first_seen: Utc::now(),
                last_seen: Utc::now(),
                frequency: (i + 1) as f32,
                avg_confidence: 0.85,
            };
            library.add_pattern(pattern);
        }

        let deadlock_patterns = library.patterns_by_type("navigation_deadlock");
        assert_eq!(deadlock_patterns.len(), 3);
    }

    #[test]
    fn test_most_frequent() {
        let mut library = PatternLibrary::new();

        let p1 = MissionPattern {
            pattern_id: Uuid::new_v4(),
            pattern_type: "error_1".to_string(),
            occurrences: vec![],
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            frequency: 5.0,
            avg_confidence: 0.9,
        };

        let p2 = MissionPattern {
            pattern_id: Uuid::new_v4(),
            pattern_type: "error_2".to_string(),
            occurrences: vec![],
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            frequency: 10.0,
            avg_confidence: 0.85,
        };

        library.add_pattern(p1);
        library.add_pattern(p2);

        let top = library.most_frequent(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].frequency, 10.0);
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = CrossMissionAnalyzer::new();
        assert_eq!(analyzer.missions_seen, 0);
        assert_eq!(analyzer.library.patterns.len(), 0);
    }

    #[test]
    fn test_library_serialization() {
        let library = PatternLibrary::new();
        let json = library.serialize_for_storage().unwrap();
        let loaded = PatternLibrary::load_from_storage(&json).unwrap();

        assert_eq!(loaded.patterns.len(), 0);
    }

    #[test]
    fn test_pattern_confidence_averaging() {
        let mut library = PatternLibrary::new();
        let pattern_id = Uuid::new_v4();

        let occurrence1 = MissionOccurrence {
            mission_id: "mission_1".to_string(),
            event_idx: 0,
            event_type: "failure".to_string(),
            timestamp: Utc::now(),
            confidence: 0.9,
        };

        let occurrence2 = MissionOccurrence {
            mission_id: "mission_2".to_string(),
            event_idx: 0,
            event_type: "failure".to_string(),
            timestamp: Utc::now(),
            confidence: 0.8,
        };

        let pattern = MissionPattern {
            pattern_id,
            pattern_type: "critical_failure".to_string(),
            occurrences: vec![occurrence1, occurrence2],
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            frequency: 2.0,
            avg_confidence: 0.85,
        };

        library.add_pattern(pattern.clone());
        let retrieved = library.get_pattern(&pattern_id).unwrap();

        assert_eq!(retrieved.avg_confidence, 0.85);
    }

    #[test]
    fn test_predict_failure_empty_patterns() {
        let analyzer = CrossMissionAnalyzer::new();
        let events = vec![];

        let matches = analyzer.predict_failure(&events);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_predict_failure_with_matches() {
        let mut analyzer = CrossMissionAnalyzer::new();

        let occurrence = MissionOccurrence {
            mission_id: "mission_1".to_string(),
            event_idx: 0,
            event_type: "obstacle_detected".to_string(),
            timestamp: Utc::now(),
            confidence: 0.95,
        };

        let pattern = MissionPattern {
            pattern_id: Uuid::new_v4(),
            pattern_type: "collision_risk".to_string(),
            occurrences: vec![occurrence],
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            frequency: 1.0,
            avg_confidence: 0.95,
        };

        analyzer.library.add_pattern(pattern);

        // Create matching event
        let event = StreamEvent {
            event_id: "event_1".to_string(),
            mission_id: "mission_2".to_string(),
            event_type: "obstacle_detected".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("robot_1".to_string()),
            payload: serde_json::json!({}),
            sequence_number: 0,
        };

        let matches = analyzer.predict_failure(&[event]);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_pattern_frequency_increments() {
        let mut library = PatternLibrary::new();
        let pattern_id = Uuid::new_v4();

        let pattern = MissionPattern {
            pattern_id,
            pattern_type: "error".to_string(),
            occurrences: vec![],
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            frequency: 1.0,
            avg_confidence: 0.9,
        };

        library.add_pattern(pattern.clone());

        let mut updated = pattern;
        updated.frequency = 2.0;
        library.add_pattern(updated);

        let retrieved = library.get_pattern(&pattern_id).unwrap();
        assert_eq!(retrieved.frequency, 2.0);
    }

    #[test]
    fn test_all_patterns() {
        let mut library = PatternLibrary::new();

        for i in 0..5 {
            let pattern = MissionPattern {
                pattern_id: Uuid::new_v4(),
                pattern_type: format!("error_{}", i),
                occurrences: vec![],
                first_seen: Utc::now(),
                last_seen: Utc::now(),
                frequency: 1.0,
                avg_confidence: 0.9,
            };
            library.add_pattern(pattern);
        }

        let all = library.all_patterns();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_mission_occurrence_fields() {
        let occurrence = MissionOccurrence {
            mission_id: "mission_1".to_string(),
            event_idx: 42,
            event_type: "test_event".to_string(),
            timestamp: Utc::now(),
            confidence: 0.87,
        };

        assert_eq!(occurrence.mission_id, "mission_1");
        assert_eq!(occurrence.event_idx, 42);
        assert_eq!(occurrence.confidence, 0.87);
    }
}
