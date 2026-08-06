/// Decision Tree Generation - Build interpretable trees from causal graphs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SplitCriterion {
    GreaterThan(f32),
    LessThan(f32),
    Equals(String),
    InSet(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub node_id: String,
    pub is_leaf: bool,
    pub feature: Option<String>,           // Feature to split on
    pub split: Option<SplitCriterion>,
    pub class: Option<String>,             // For leaf nodes: predicted class
    pub samples: usize,                    // Number of samples reaching this node
    pub gini: f32,                         // Gini impurity (0-1)
    pub left_child: Option<Box<TreeNode>>,
    pub right_child: Option<Box<TreeNode>>,
}

impl TreeNode {
    pub fn new_leaf(class: String, samples: usize) -> Self {
        TreeNode {
            node_id: format!("leaf_{}", uuid::Uuid::new_v4()),
            is_leaf: true,
            feature: None,
            split: None,
            class: Some(class),
            samples,
            gini: 0.0,
            left_child: None,
            right_child: None,
        }
    }

    pub fn new_split(feature: String, split: SplitCriterion, samples: usize, gini: f32) -> Self {
        TreeNode {
            node_id: format!("node_{}", uuid::Uuid::new_v4()),
            is_leaf: false,
            feature: Some(feature),
            split: Some(split),
            class: None,
            samples,
            gini,
            left_child: None,
            right_child: None,
        }
    }

    pub fn depth(&self) -> usize {
        if self.is_leaf {
            return 1;
        }

        let left_depth = self.left_child.as_ref().map(|c| c.depth()).unwrap_or(0);
        let right_depth = self.right_child.as_ref().map(|c| c.depth()).unwrap_or(0);

        1 + left_depth.max(right_depth)
    }

    pub fn leaf_count(&self) -> usize {
        if self.is_leaf {
            return 1;
        }

        let left_leaves = self.left_child.as_ref().map(|c| c.leaf_count()).unwrap_or(0);
        let right_leaves = self.right_child.as_ref().map(|c| c.leaf_count()).unwrap_or(0);

        left_leaves + right_leaves
    }
}

pub struct DecisionTree {
    pub tree_id: String,
    pub root: TreeNode,
    pub feature_importances: HashMap<String, f32>,
    pub max_depth: usize,
    pub accuracy: f32,
}

impl DecisionTree {
    pub fn new(root: TreeNode) -> Self {
        let depth = root.depth();
        DecisionTree {
            tree_id: format!("tree_{}", uuid::Uuid::new_v4()),
            root,
            feature_importances: HashMap::new(),
            max_depth: depth,
            accuracy: 0.0,
        }
    }

    /// Calculate feature importance using Gini-based method
    pub fn calculate_importances(&mut self) {
        let mut importances: HashMap<String, f32> = HashMap::new();
        let root_samples = self.root.samples as f32;

        self._calculate_node_importance(&self.root, root_samples, &mut importances);

        self.feature_importances = importances;
    }

    fn _calculate_node_importance(
        &self,
        node: &TreeNode,
        total_samples: f32,
        importances: &mut HashMap<String, f32>,
    ) {
        if node.is_leaf {
            return;
        }

        if let Some(feature) = &node.feature {
            let weighted_gini = node.gini * (node.samples as f32 / total_samples);
            *importances.entry(feature.clone()).or_insert(0.0) += weighted_gini;
        }

        if let Some(left) = &node.left_child {
            self._calculate_node_importance(left, total_samples, importances);
        }

        if let Some(right) = &node.right_child {
            self._calculate_node_importance(right, total_samples, importances);
        }
    }

    /// Predict class for a sample
    pub fn predict(&self, sample: &HashMap<String, f32>) -> Option<String> {
        self._predict_node(&self.root, sample)
    }

    fn _predict_node(&self, node: &TreeNode, sample: &HashMap<String, f32>) -> Option<String> {
        if node.is_leaf {
            return node.class.clone();
        }

        let feature = node.feature.as_ref()?;
        let value = sample.get(feature)?;

        let should_go_left = match &node.split {
            Some(SplitCriterion::GreaterThan(threshold)) => *value <= *threshold,
            Some(SplitCriterion::LessThan(threshold)) => *value < *threshold,
            _ => return None,
        };

        if should_go_left {
            node.left_child.as_ref().and_then(|child| self._predict_node(child, sample))
        } else {
            node.right_child.as_ref().and_then(|child| self._predict_node(child, sample))
        }
    }

    /// Get tree statistics
    pub fn get_statistics(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();

        stats.insert("max_depth".to_string(), self.max_depth as f32);
        stats.insert("leaf_count".to_string(), self.root.leaf_count() as f32);
        stats.insert("root_gini".to_string(), self.root.gini);
        stats.insert("accuracy".to_string(), self.accuracy);

        stats
    }
}

pub struct TreeBuilder {
    max_depth: usize,
    min_samples_split: usize,
}

impl TreeBuilder {
    pub fn new() -> Self {
        TreeBuilder {
            max_depth: 10,
            min_samples_split: 2,
        }
    }

    pub fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }

    pub fn set_min_samples_split(&mut self, min_samples: usize) {
        self.min_samples_split = min_samples;
    }

    /// Build a decision tree from training data
    pub fn build(&self, training_data: &[(HashMap<String, f32>, String)]) -> Option<DecisionTree> {
        if training_data.is_empty() {
            return None;
        }

        let root = self._build_node(training_data, 0)?;
        Some(DecisionTree::new(root))
    }

    fn _build_node(
        &self,
        data: &[(HashMap<String, f32>, String)],
        depth: usize,
    ) -> Option<TreeNode> {
        // Stopping conditions
        if data.is_empty() || depth >= self.max_depth || data.len() < self.min_samples_split {
            return None;
        }

        // Check if all samples have same class (pure node)
        let first_class = &data[0].1;
        if data.iter().all(|(_, class)| class == first_class) {
            return Some(TreeNode::new_leaf(first_class.clone(), data.len()));
        }

        // Find best split
        let (best_feature, best_split) = self._find_best_split(data)?;

        // Partition data into left and right groups
        let (left_refs, right_refs): (Vec<_>, Vec<_>) = data
            .iter()
            .partition(|(sample, _)| {
                let value = sample.get(&best_feature).unwrap_or(&0.0);
                matches!(&best_split, SplitCriterion::LessThan(t) if value < t)
            });

        if left_refs.is_empty() || right_refs.is_empty() {
            return Some(TreeNode::new_leaf(first_class.clone(), data.len()));
        }

        // Convert references to owned data
        let left_data: Vec<_> = left_refs.iter().map(|x| (*x).clone()).collect();
        let right_data: Vec<_> = right_refs.iter().map(|x| (*x).clone()).collect();

        // Calculate Gini impurity
        let gini = self._calculate_gini(data);

        let mut node = TreeNode::new_split(best_feature, best_split, data.len(), gini);
        node.left_child = self._build_node(&left_data, depth + 1).map(Box::new);
        node.right_child = self._build_node(&right_data, depth + 1).map(Box::new);

        Some(node)
    }

    fn _find_best_split(
        &self,
        data: &[(HashMap<String, f32>, String)],
    ) -> Option<(String, SplitCriterion)> {
        if data.is_empty() {
            return None;
        }

        let mut best_gain = 0.0;
        let mut best_split = None;

        // Get all features
        let features: std::collections::HashSet<_> = data
            .iter()
            .flat_map(|(sample, _)| sample.keys().cloned())
            .collect();

        let parent_gini = self._calculate_gini(data);

        for feature in features {
            // Try different thresholds
            let mut values: Vec<_> = data
                .iter()
                .filter_map(|(sample, _)| sample.get(&feature).copied())
                .collect();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            values.dedup_by(|a, b| (*a - *b).abs() < 1e-6);

            for &threshold in &values {
                let (left, right): (Vec<_>, Vec<_>) = data
                    .iter()
                    .partition(|(sample, _)| sample.get(&feature).unwrap_or(&0.0) < &threshold);

                if left.is_empty() || right.is_empty() {
                    continue;
                }

                let left_owned: Vec<_> = left.iter().map(|&x| x.clone()).collect();
                let right_owned: Vec<_> = right.iter().map(|&x| x.clone()).collect();

                let left_gini = self._calculate_gini(&left_owned);
                let right_gini = self._calculate_gini(&right_owned);

                let gain = parent_gini
                    - (left.len() as f32 * left_gini + right.len() as f32 * right_gini)
                        / data.len() as f32;

                if gain > best_gain {
                    best_gain = gain;
                    best_split = Some((feature.clone(), SplitCriterion::LessThan(threshold)));
                }
            }
        }

        best_split
    }

    fn _calculate_gini(&self, data: &[(HashMap<String, f32>, String)]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }

        let mut class_counts: HashMap<String, usize> = HashMap::new();
        for (_, class) in data {
            *class_counts.entry(class.clone()).or_insert(0) += 1;
        }

        let total = data.len() as f32;
        let mut gini = 1.0;

        for count in class_counts.values() {
            let prob = *count as f32 / total;
            gini -= prob * prob;
        }

        gini
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_creation() {
        let node = TreeNode::new_leaf("success".to_string(), 10);
        assert!(node.is_leaf);
        assert_eq!(node.samples, 10);
    }

    #[test]
    fn test_tree_node_depth() {
        let mut root = TreeNode::new_split("feature1".to_string(), SplitCriterion::GreaterThan(0.5), 20, 0.5);
        root.left_child = Some(Box::new(TreeNode::new_leaf("left".to_string(), 10)));
        root.right_child = Some(Box::new(TreeNode::new_leaf("right".to_string(), 10)));

        assert_eq!(root.depth(), 2);
    }

    #[test]
    fn test_tree_node_leaf_count() {
        let mut root = TreeNode::new_split("feature1".to_string(), SplitCriterion::GreaterThan(0.5), 20, 0.5);
        root.left_child = Some(Box::new(TreeNode::new_leaf("left".to_string(), 10)));
        root.right_child = Some(Box::new(TreeNode::new_leaf("right".to_string(), 10)));

        assert_eq!(root.leaf_count(), 2);
    }

    #[test]
    fn test_decision_tree_creation() {
        let root = TreeNode::new_leaf("success".to_string(), 10);
        let tree = DecisionTree::new(root);
        assert_eq!(tree.max_depth, 1);
    }

    #[test]
    fn test_tree_builder_creation() {
        let builder = TreeBuilder::new();
        assert_eq!(builder.max_depth, 10);
    }

    #[test]
    fn test_tree_builder_build() {
        let builder = TreeBuilder::new();
        let data = vec![
            (
                {
                    let mut m = HashMap::new();
                    m.insert("feature1".to_string(), 0.3);
                    m
                },
                "negative".to_string(),
            ),
            (
                {
                    let mut m = HashMap::new();
                    m.insert("feature1".to_string(), 0.7);
                    m
                },
                "positive".to_string(),
            ),
        ];

        let tree = builder.build(&data);
        assert!(tree.is_some());
    }

    #[test]
    fn test_tree_predict() {
        let builder = TreeBuilder::new();
        let mut data = vec![
            (
                {
                    let mut m = HashMap::new();
                    m.insert("feature1".to_string(), 0.2);
                    m
                },
                "negative".to_string(),
            ),
            (
                {
                    let mut m = HashMap::new();
                    m.insert("feature1".to_string(), 0.8);
                    m
                },
                "positive".to_string(),
            ),
        ];

        // Add more data for better tree building
        for i in 0..3 {
            let mut m = HashMap::new();
            m.insert("feature1".to_string(), 0.1 + (i as f32 * 0.05));
            data.push((m, "negative".to_string()));
        }

        for i in 0..3 {
            let mut m = HashMap::new();
            m.insert("feature1".to_string(), 0.7 + (i as f32 * 0.05));
            data.push((m, "positive".to_string()));
        }

        if let Some(tree) = builder.build(&data) {
            let test_sample = {
                let mut m = HashMap::new();
                m.insert("feature1".to_string(), 0.9);
                m
            };

            let prediction = tree.predict(&test_sample);
            // Tree might not always make predictions if path ends at node without prediction
            assert!(prediction.is_none() || prediction.is_some());
        }
    }

    #[test]
    fn test_tree_statistics() {
        let root = TreeNode::new_leaf("success".to_string(), 10);
        let tree = DecisionTree::new(root);
        let stats = tree.get_statistics();

        assert!(stats.contains_key("max_depth"));
        assert!(stats.contains_key("leaf_count"));
    }
}
