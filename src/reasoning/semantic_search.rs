//! Semantic Search Engine for Replay Sessions
//!
//! Allows searching over mission history using natural language queries.
//!
//! Examples:
//! - "Show all collisions caused by pedestrians"
//! - "Find instances where optical detection failed"
//! - "Show near-collisions with vehicles"
//! - "List all perception failures in warehouse missions"

use std::collections::HashMap;

/// Semantic search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Mission ID that matches
    pub mission_id: String,

    /// Timestamp within mission
    pub timestamp_sec: f32,

    /// What matched
    pub match_description: String,

    /// Relevance score (0.0-1.0)
    pub relevance: f32,

    /// Event details
    pub metadata: HashMap<String, String>,
}

/// Search query
#[derive(Debug, Clone)]
pub struct SemanticQuery {
    /// Natural language query
    pub query: String,

    /// Keywords to match
    pub keywords: Vec<String>,

    /// Filters (optional)
    pub filters: QueryFilters,
}

/// Query filters
#[derive(Debug, Clone)]
pub struct QueryFilters {
    /// Filter by mission outcome
    pub outcome: Option<String>, // "collision", "success", "failure"

    /// Filter by robot type
    pub robot_type: Option<String>,

    /// Time range (seconds)
    pub time_range: Option<(f32, f32)>,

    /// Minimum severity
    pub min_severity: Option<f32>,

    /// Objects involved
    pub objects: Vec<String>, // "pedestrian", "vehicle", etc.
}

impl Default for QueryFilters {
    fn default() -> Self {
        QueryFilters {
            outcome: None,
            robot_type: None,
            time_range: None,
            min_severity: None,
            objects: Vec::new(),
        }
    }
}

/// Semantic search engine
pub struct SemanticSearchEngine {
    // In production, this would build an index of:
    // - Vectorized mission summaries (using embeddings)
    // - Keyword indices
    // - Event metadata
    // - Temporal indices
    indexed_missions: Vec<IndexedMission>,
}

/// Indexed mission data
#[derive(Debug, Clone)]
pub struct IndexedMission {
    pub mission_id: String,
    pub keywords: Vec<String>,
    pub events: Vec<IndexedEvent>,
    pub metadata: MissionMetadata,
}

/// Indexed event
#[derive(Debug, Clone)]
pub struct IndexedEvent {
    pub timestamp_sec: f32,
    pub event_type: String,
    pub keywords: Vec<String>,
    pub severity: f32,
}

/// Mission metadata
#[derive(Debug, Clone)]
pub struct MissionMetadata {
    pub outcome: String,
    pub robot_type: String,
    pub objects_involved: Vec<String>,
    pub perception_failures: usize,
    pub gap_count: usize,
}

impl SemanticSearchEngine {
    /// Create new search engine
    pub fn new() -> Self {
        SemanticSearchEngine {
            indexed_missions: Vec::new(),
        }
    }

    /// Index a mission for searching
    pub fn index_mission(&mut self, mission: &MissionForIndexing) {
        let keywords = Self::extract_keywords(mission);
        let events = Self::extract_events(mission);
        let metadata = Self::extract_metadata(mission);

        let indexed = IndexedMission {
            mission_id: mission.mission_id.clone(),
            keywords,
            events,
            metadata,
        };

        self.indexed_missions.push(indexed);
    }

    /// Extract keywords from mission
    fn extract_keywords(mission: &MissionForIndexing) -> Vec<String> {
        let mut keywords = Vec::new();

        // Add outcome
        keywords.push(mission.outcome.clone());

        // Add robot type
        keywords.push(mission.robot_type.clone());

        // Add objects involved
        for obj in &mission.objects_involved {
            keywords.push(obj.clone());
        }

        // Add gap types
        for gap in &mission.gaps {
            keywords.push(gap.clone());
        }

        // Add failure modes
        if mission.had_collision {
            keywords.push("collision".to_string());
        }
        if mission.had_near_miss {
            keywords.push("near_collision".to_string());
        }
        if mission.had_perception_failure {
            keywords.push("perception_failure".to_string());
        }

        keywords
    }

    /// Extract events from mission
    fn extract_events(mission: &MissionForIndexing) -> Vec<IndexedEvent> {
        let mut events = Vec::new();

        for event in &mission.events {
            events.push(IndexedEvent {
                timestamp_sec: event.0,
                event_type: event.1.clone(),
                keywords: vec![event.1.clone()],
                severity: event.2,
            });
        }

        events
    }

    /// Extract metadata from mission
    fn extract_metadata(mission: &MissionForIndexing) -> MissionMetadata {
        MissionMetadata {
            outcome: mission.outcome.clone(),
            robot_type: mission.robot_type.clone(),
            objects_involved: mission.objects_involved.clone(),
            perception_failures: mission.perception_failures,
            gap_count: mission.gaps.len(),
        }
    }

    /// Search missions
    pub fn search(&self, query: &SemanticQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for mission in &self.indexed_missions {
            // Apply filters
            if !Self::passes_filters(mission, &query.filters) {
                continue;
            }

            // Score relevance
            let mut relevance = 0.0;

            // Keyword matching
            for keyword in &query.keywords {
                if mission.keywords.contains(keyword) {
                    relevance += 0.3;
                }
            }

            // Event matching
            for event in &mission.events {
                for keyword in &query.keywords {
                    if event.keywords.contains(keyword) {
                        relevance += 0.2 * event.severity;
                        results.push(SearchResult {
                            mission_id: mission.mission_id.clone(),
                            timestamp_sec: event.timestamp_sec,
                            match_description: format!(
                                "{} at t={:.1}s",
                                event.event_type, event.timestamp_sec
                            ),
                            relevance: (relevance / 10.0).min(1.0),
                            metadata: {
                                let mut map = HashMap::new();
                                map.insert("event_type".to_string(), event.event_type.clone());
                                map.insert("severity".to_string(), event.severity.to_string());
                                map
                            },
                        });
                    }
                }
            }
        }

        // Sort by relevance
        results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Check if mission passes filters
    fn passes_filters(mission: &IndexedMission, filters: &QueryFilters) -> bool {
        if let Some(outcome) = &filters.outcome {
            if mission.metadata.outcome != *outcome {
                return false;
            }
        }

        if let Some(robot_type) = &filters.robot_type {
            if mission.metadata.robot_type != *robot_type {
                return false;
            }
        }

        if let Some(min_severity) = filters.min_severity {
            let has_high_severity_event = mission
                .events
                .iter()
                .any(|e| e.severity >= min_severity);
            if !has_high_severity_event {
                return false;
            }
        }

        if !filters.objects.is_empty() {
            let has_object = filters
                .objects
                .iter()
                .any(|obj| mission.metadata.objects_involved.contains(obj));
            if !has_object {
                return false;
            }
        }

        true
    }
}

impl Default for SemanticSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Mission data for indexing
#[derive(Debug, Clone)]
pub struct MissionForIndexing {
    pub mission_id: String,
    pub outcome: String, // "collision", "success", "partial_failure"
    pub robot_type: String,
    pub objects_involved: Vec<String>,
    pub gaps: Vec<String>,
    pub events: Vec<(f32, String, f32)>, // (timestamp, event_type, severity)
    pub had_collision: bool,
    pub had_near_miss: bool,
    pub had_perception_failure: bool,
    pub perception_failures: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_mission() -> MissionForIndexing {
        MissionForIndexing {
            mission_id: "mission_001".to_string(),
            outcome: "collision".to_string(),
            robot_type: "mobile_robot".to_string(),
            objects_involved: vec!["pedestrian".to_string(), "pallet".to_string()],
            gaps: vec!["optical_contamination".to_string()],
            events: vec![(100.0, "collision".to_string(), 1.0)],
            had_collision: true,
            had_near_miss: false,
            had_perception_failure: true,
            perception_failures: 1,
        }
    }

    #[test]
    fn test_semantic_search_engine_creation() {
        let engine = SemanticSearchEngine::new();
        assert_eq!(engine.indexed_missions.len(), 0);
    }

    #[test]
    fn test_mission_indexing() {
        let mut engine = SemanticSearchEngine::new();
        let mission = create_test_mission();

        engine.index_mission(&mission);

        assert_eq!(engine.indexed_missions.len(), 1);
        assert!(engine.indexed_missions[0].keywords.contains(&"collision".to_string()));
    }

    #[test]
    fn test_collision_search() {
        let mut engine = SemanticSearchEngine::new();
        let mission = create_test_mission();

        engine.index_mission(&mission);

        let query = SemanticQuery {
            query: "Show all collisions".to_string(),
            keywords: vec!["collision".to_string()],
            filters: QueryFilters::default(),
        };

        let results = engine.search(&query);

        assert!(!results.is_empty());
    }

    #[test]
    fn test_filtered_search() {
        let mut engine = SemanticSearchEngine::new();
        let mission = create_test_mission();

        engine.index_mission(&mission);

        let mut filters = QueryFilters::default();
        filters.outcome = Some("collision".to_string());
        filters.objects = vec!["pedestrian".to_string()];

        let query = SemanticQuery {
            query: "Collisions with pedestrians".to_string(),
            keywords: vec!["collision".to_string(), "pedestrian".to_string()],
            filters,
        };

        let results = engine.search(&query);

        assert!(!results.is_empty());
    }
}
