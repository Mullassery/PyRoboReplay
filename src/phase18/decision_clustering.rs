/// Decision Clustering - Group similar decisions into templates

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTemplate {
    pub template_id: String,
    pub name: String,
    pub decision_signature: String,  // Hash of key characteristics
    pub preconditions: Vec<String>,
    pub actions: Vec<String>,
    pub expected_outcomes: Vec<String>,
    pub success_rate: f32,
    pub instances: usize,
}

impl DecisionTemplate {
    pub fn new(name: String) -> Self {
        DecisionTemplate {
            template_id: format!("template_{}", uuid::Uuid::new_v4()),
            name,
            decision_signature: String::new(),
            preconditions: Vec::new(),
            actions: Vec::new(),
            expected_outcomes: Vec::new(),
            success_rate: 0.0,
            instances: 0,
        }
    }

    pub fn fitness(&self) -> f32 {
        // Template fitness: success_rate * prevalence
        let prevalence = (self.instances as f32 / 1000.0).min(1.0);
        self.success_rate * prevalence
    }
}

pub struct ClusterAnalyzer {
    templates: Vec<DecisionTemplate>,
    distance_threshold: f32,
}

#[derive(Debug, Clone)]
struct DecisionSignature {
    preconditions: Vec<String>,
    actions: Vec<String>,
    outcomes: Vec<String>,
}

impl ClusterAnalyzer {
    pub fn new(distance_threshold: f32) -> Self {
        ClusterAnalyzer {
            templates: Vec::new(),
            distance_threshold,
        }
    }

    /// Cluster decisions by similarity
    pub fn cluster_decisions(&self, decisions: &[DecisionData]) -> Vec<Vec<usize>> {
        if decisions.is_empty() {
            return Vec::new();
        }

        let mut clusters: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; decisions.len()];

        for i in 0..decisions.len() {
            if assigned[i] {
                continue;
            }

            let mut cluster = vec![i];
            assigned[i] = true;

            // Find similar decisions
            for j in (i + 1)..decisions.len() {
                if !assigned[j] {
                    let distance = self._decision_distance(&decisions[i], &decisions[j]);
                    if distance < self.distance_threshold {
                        cluster.push(j);
                        assigned[j] = true;
                    }
                }
            }

            clusters.push(cluster);
        }

        clusters
    }

    fn _decision_distance(&self, d1: &DecisionData, d2: &DecisionData) -> f32 {
        let precond_sim = self._vector_similarity(&d1.preconditions, &d2.preconditions);
        let action_sim = self._vector_similarity(&d1.actions, &d2.actions);
        let outcome_sim = self._vector_similarity(&d1.outcomes, &d2.outcomes);

        // Distance: 0 = identical, 1 = completely different
        1.0 - ((precond_sim + action_sim + outcome_sim) / 3.0)
    }

    fn _vector_similarity(&self, v1: &[String], v2: &[String]) -> f32 {
        if v1.is_empty() && v2.is_empty() {
            return 1.0;
        }

        let intersection = v1.iter()
            .filter(|item| v2.contains(item))
            .count();
        let union = v1.len() + v2.len() - intersection;

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Build decision templates from clusters
    pub fn build_templates(&self, decisions: &[DecisionData], clusters: &[Vec<usize>]) -> Vec<DecisionTemplate> {
        let mut templates = Vec::new();

        for cluster in clusters {
            if cluster.is_empty() {
                continue;
            }

            let mut template = DecisionTemplate::new(
                format!("Template_{}", templates.len())
            );

            let cluster_decisions: Vec<_> = cluster.iter()
                .filter_map(|&idx| decisions.get(idx))
                .collect();

            // Find common preconditions
            if !cluster_decisions.is_empty() {
                let first = &cluster_decisions[0];
                template.preconditions = first.preconditions.clone();
                template.actions = first.actions.clone();
                template.expected_outcomes = first.outcomes.clone();
            }

            // Calculate success rate
            let successful = cluster_decisions.iter()
                .filter(|d| d.succeeded)
                .count();
            template.success_rate = successful as f32 / cluster.len() as f32;
            template.instances = cluster.len();

            template.decision_signature = format!("{:?}_{:?}", template.preconditions, template.actions);

            templates.push(template);
        }

        templates
    }

    /// Get cluster statistics
    pub fn get_cluster_stats(&self, clusters: &[Vec<usize>]) -> HashMap<String, f32> {
        let mut stats = HashMap::new();

        stats.insert("num_clusters".to_string(), clusters.len() as f32);

        if !clusters.is_empty() {
            let sizes: Vec<_> = clusters.iter().map(|c| c.len() as f32).collect();
            let avg_cluster_size = sizes.iter().sum::<f32>() / sizes.len() as f32;
            let max_cluster_size = sizes.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let min_cluster_size = sizes.iter().cloned().fold(f32::INFINITY, f32::min);

            stats.insert("avg_cluster_size".to_string(), avg_cluster_size);
            stats.insert("max_cluster_size".to_string(), max_cluster_size);
            stats.insert("min_cluster_size".to_string(), min_cluster_size);
        }

        stats
    }
}

#[derive(Debug, Clone)]
pub struct DecisionData {
    pub decision_id: String,
    pub preconditions: Vec<String>,
    pub actions: Vec<String>,
    pub outcomes: Vec<String>,
    pub succeeded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_template_creation() {
        let template = DecisionTemplate::new("Obstacle Avoidance".to_string());
        assert_eq!(template.name, "Obstacle Avoidance");
        assert_eq!(template.instances, 0);
    }

    #[test]
    fn test_template_fitness() {
        let mut template = DecisionTemplate::new("Test".to_string());
        template.success_rate = 0.8;
        template.instances = 100;

        let fitness = template.fitness();
        assert!(fitness > 0.0);
    }

    #[test]
    fn test_cluster_analyzer_creation() {
        let analyzer = ClusterAnalyzer::new(0.3);
        assert_eq!(analyzer.templates.len(), 0);
    }

    #[test]
    fn test_cluster_decisions() {
        let analyzer = ClusterAnalyzer::new(0.3);
        let decisions = vec![
            DecisionData {
                decision_id: "d1".to_string(),
                preconditions: vec!["obstacle_detected".to_string()],
                actions: vec!["turn_left".to_string()],
                outcomes: vec!["success".to_string()],
                succeeded: true,
            },
            DecisionData {
                decision_id: "d2".to_string(),
                preconditions: vec!["obstacle_detected".to_string()],
                actions: vec!["turn_right".to_string()],
                outcomes: vec!["success".to_string()],
                succeeded: true,
            },
        ];

        let clusters = analyzer.cluster_decisions(&decisions);
        assert!(!clusters.is_empty());
    }

    #[test]
    fn test_build_templates() {
        let analyzer = ClusterAnalyzer::new(0.3);
        let decisions = vec![
            DecisionData {
                decision_id: "d1".to_string(),
                preconditions: vec!["obstacle".to_string()],
                actions: vec!["avoid".to_string()],
                outcomes: vec!["safe".to_string()],
                succeeded: true,
            },
        ];

        let clusters = vec![vec![0]];
        let templates = analyzer.build_templates(&decisions, &clusters);
        assert_eq!(templates.len(), 1);
    }

    #[test]
    fn test_cluster_stats() {
        let analyzer = ClusterAnalyzer::new(0.3);
        let clusters = vec![vec![0, 1], vec![2]];
        let stats = analyzer.get_cluster_stats(&clusters);

        assert_eq!(stats.get("num_clusters"), Some(&2.0));
    }
}
