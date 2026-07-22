//! Knowledge Graph for Entity Relationships
//!
//! Tracks relationships between entities, locations, and events
//! enabling rich contextual queries.

use std::collections::HashMap;

/// Knowledge graph for environment relationships
#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    /// Entity relationships
    pub relationships: HashMap<String, Vec<Relationship>>,
}

/// Relationship between two entities or locations
#[derive(Debug, Clone)]
pub struct Relationship {
    pub from_id: String,
    pub to_id: String,
    pub relationship_type: String, // "near", "blocks", "depends_on", "contains", etc.
    pub strength: f32, // 0.0-1.0
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        KnowledgeGraph {
            relationships: HashMap::new(),
        }
    }

    pub fn add_relationship(&mut self, relationship: Relationship) {
        self.relationships
            .entry(relationship.from_id.clone())
            .or_insert_with(Vec::new)
            .push(relationship);
    }

    pub fn query_relationships(&self, entity_id: &str) -> Vec<&Relationship> {
        self.relationships
            .get(entity_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_graph_creation() {
        let graph = KnowledgeGraph::new();
        assert!(graph.relationships.is_empty());
    }

    #[test]
    fn test_add_relationship() {
        let mut graph = KnowledgeGraph::new();

        graph.add_relationship(Relationship {
            from_id: "pallet_42".to_string(),
            to_id: "aisle_3".to_string(),
            relationship_type: "located_at".to_string(),
            strength: 0.95,
        });

        let rels = graph.query_relationships("pallet_42");
        assert_eq!(rels.len(), 1);
    }
}
