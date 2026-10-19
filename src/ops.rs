/// JSON operation bridge for the Phase 16-20 libraries.
///
/// These phases are implemented as Rust library APIs with rich, Rust-native
/// input types. Rather than guessing at hand-written typed Python wrappers,
/// this module exposes a small allowlisted registry of real, self-contained
/// operations that can be driven with `serde_json` documents.
use crate::core::event::MissionEvent;
use crate::phase16::causal_builder::CausalGraphBuilderV2;
use crate::phase16::graph_validator::CausalGraphValidator;
use crate::phase17::counterfactual::{CounterfactualAnalyzer, CounterfactualQuery};
use crate::phase18::fleet_learning::FleetLearner;
use crate::phase19::temporal_patterns::TemporalPatternMiner;
use crate::phase19::trend_detector::TrendDetector;
use crate::phase20::decision_tree::TreeBuilder;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};

/// Error returned by the advanced operation bridge.
#[derive(Debug)]
pub enum OpError {
    UnknownOp(String),
    InvalidPayload(String),
    Execution(String),
}

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpError::UnknownOp(message) => write!(f, "{message}"),
            OpError::InvalidPayload(message) => write!(f, "{message}"),
            OpError::Execution(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for OpError {}

type OpHandler = fn(Value) -> Result<Value, OpError>;

/// Metadata and handler for one JSON operation.
pub struct OpSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: fn() -> Value,
    pub handler: OpHandler,
}

const OP_SPECS: &[OpSpec] = &[
    OpSpec {
        name: "phase16.causal_graph",
        description: "Build a Phase 16 causal graph from MissionEvent JSON and validate it (DAG, confidence calibration, conflicts). Currently emits temporal-proximity and decision-trigger edges; the module's other three detectors are placeholders.",
        schema: causal_graph_schema,
        handler: causal_graph_op,
    },
    OpSpec {
        name: "phase17.counterfactual",
        description: "Run a Phase 17 counterfactual query against a causal graph and report the alternative outcome and affected events.",
        schema: counterfactual_schema,
        handler: counterfactual_op,
    },
    OpSpec {
        name: "phase18.fleet_learning",
        description: "Build a Phase 18 fleet leaderboard and optimization tips from per-robot mission results.",
        schema: fleet_learning_schema,
        handler: fleet_learning_op,
    },
    OpSpec {
        name: "phase19.temporal_patterns",
        description: "Discover Phase 19 temporal patterns across short-, medium-, and long-term windows from timestamped events.",
        schema: temporal_patterns_schema,
        handler: temporal_patterns_op,
    },
    OpSpec {
        name: "phase19.trend_detection",
        description: "Fit a Phase 19 linear trend to timestamped metric points, report significance, and optionally predict a future value.",
        schema: trend_detection_schema,
        handler: trend_detection_op,
    },
    OpSpec {
        name: "phase20.decision_tree",
        description: "Build a Phase 20 decision tree from labeled feature rows, calculate feature importances, and optionally predict samples.",
        schema: decision_tree_schema,
        handler: decision_tree_op,
    },
];

/// List the allowlisted operations as `(name, description, schema_json)`.
pub fn ops() -> Vec<(String, String, String)> {
    OP_SPECS
        .iter()
        .map(|spec| {
            (
                spec.name.to_string(),
                spec.description.to_string(),
                (spec.schema)().to_string(),
            )
        })
        .collect()
}

/// Execute one allowlisted operation and return its JSON result as a string.
pub fn run_op_json(op: &str, payload_json: &str) -> Result<String, OpError> {
    let spec = OP_SPECS
        .iter()
        .find(|spec| spec.name == op)
        .ok_or_else(|| {
            let available = OP_SPECS
                .iter()
                .map(|spec| spec.name)
                .collect::<Vec<_>>()
                .join(", ");
            OpError::UnknownOp(format!("unknown op {op:?}; available ops: {available}"))
        })?;

    let payload: Value = serde_json::from_str(payload_json)
        .map_err(|e| OpError::InvalidPayload(format!("malformed JSON payload: {e}")))?;

    if !payload.is_object() {
        return Err(OpError::InvalidPayload(
            "payload must be a JSON object".to_string(),
        ));
    }

    let result = (spec.handler)(payload)?;
    serde_json::to_string(&result)
        .map_err(|e| OpError::Execution(format!("failed to serialize operation result: {e}")))
}

fn parse_payload<T: for<'de> Deserialize<'de>>(payload: Value) -> Result<T, OpError> {
    serde_json::from_value(payload)
        .map_err(|e| OpError::InvalidPayload(format!("invalid payload for op: {e}")))
}

fn causal_graph_schema() -> Value {
    json!({
        "type": "object",
        "required": ["events"],
        "properties": {
            "events": {
                "type": "array",
                "description": "MissionEvent values in serde's externally-tagged form, e.g. {\"NavigationDecision\": {\"robot_id\": \"r1\", \"timestamp\": \"2026-01-01T00:00:00Z\", \"decision_type\": \"stop\", \"rationale\": null}}"
            }
        },
        "additionalProperties": false
    })
}

#[derive(Debug, Deserialize)]
struct CausalGraphPayload {
    events: Vec<MissionEvent>,
}

fn causal_graph_op(payload: Value) -> Result<Value, OpError> {
    let input: CausalGraphPayload = parse_payload(payload)?;
    let graph = CausalGraphBuilderV2::new(input.events).build();
    let validation = CausalGraphValidator::validate(&graph);
    Ok(json!({
        "graph": graph,
        "validation": validation,
    }))
}

fn counterfactual_schema() -> Value {
    json!({
        "type": "object",
        "required": ["graph", "baseline_outcome", "query"],
        "properties": {
            "graph": {
                "type": "object",
                "description": "Phase 16 CausalGraphV2 document (the `graph` object returned by phase16.causal_graph)."
            },
            "baseline_outcome": {"type": "string"},
            "query": {
                "type": "object",
                "required": ["query_type", "target_node_id"],
                "properties": {
                    "query_type": {
                        "type": "string",
                        "enum": ["remove_node", "replace_node", "modify_edge_weight", "remove_edge", "parallel_decisions"]
                    },
                    "target_node_id": {"type": "string"},
                    "parameter": {"type": ["string", "null"]},
                    "timestamp": {"type": "integer"}
                },
                "additionalProperties": false
            }
        },
        "additionalProperties": false
    })
}

#[derive(Debug, Deserialize)]
struct CounterfactualPayload {
    graph: crate::phase16::causal_builder::CausalGraphV2,
    baseline_outcome: String,
    query: CounterfactualQueryInput,
}

#[derive(Debug, Deserialize)]
struct CounterfactualQueryInput {
    query_type: String,
    target_node_id: String,
    #[serde(default)]
    parameter: Option<String>,
    #[serde(default)]
    timestamp: i64,
}

fn counterfactual_op(payload: Value) -> Result<Value, OpError> {
    let input: CounterfactualPayload = parse_payload(payload)?;
    let query = CounterfactualQuery {
        query_type: input.query.query_type,
        target_node_id: input.query.target_node_id,
        parameter: input.query.parameter,
        timestamp: input.query.timestamp,
    };
    let analyzer = CounterfactualAnalyzer::new(input.graph, input.baseline_outcome, Vec::new());
    let result = analyzer.analyze_counterfactual(query);
    serde_json::to_value(result)
        .map_err(|e| OpError::Execution(format!("failed to serialize counterfactual result: {e}")))
}

fn fleet_learning_schema() -> Value {
    json!({
        "type": "object",
        "required": ["robots"],
        "properties": {
            "robots": {
                "type": "object",
                "description": "Map of robot_id to mission result rows.",
                "additionalProperties": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["succeeded", "time_ms"],
                        "properties": {
                            "succeeded": {"type": "boolean"},
                            "time_ms": {"type": "integer", "minimum": 0}
                        },
                        "additionalProperties": false
                    }
                }
            },
            "top_tips": {"type": "integer", "minimum": 0, "default": 10}
        },
        "additionalProperties": false
    })
}

#[derive(Debug, Deserialize)]
struct FleetLearningPayload {
    robots: BTreeMap<String, Vec<FleetMission>>,
    #[serde(default = "default_top_tips")]
    top_tips: usize,
}

#[derive(Debug, Deserialize)]
struct FleetMission {
    succeeded: bool,
    time_ms: u32,
}

fn default_top_tips() -> usize {
    10
}

fn fleet_learning_op(payload: Value) -> Result<Value, OpError> {
    let input: FleetLearningPayload = parse_payload(payload)?;
    let mut learner = FleetLearner::new();
    for (robot_id, missions) in input.robots {
        learner.add_robot(robot_id.clone());
        for mission in missions {
            learner.add_mission_result(&robot_id, mission.succeeded, mission.time_ms);
        }
    }

    let leaderboard = learner.generate_leaderboard();
    let mut tips = learner.generate_optimization_tips(&leaderboard);
    tips.truncate(input.top_tips);

    Ok(json!({
        "leaderboard": leaderboard,
        "optimization_tips": tips,
        "statistics": learner.get_fleet_stats(),
    }))
}

fn temporal_patterns_schema() -> Value {
    json!({
        "type": "object",
        "required": ["events"],
        "properties": {
            "events": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["timestamp", "event"],
                    "properties": {
                        "timestamp": {"type": "integer", "minimum": 0},
                        "event": {"type": "string"}
                    },
                    "additionalProperties": false
                }
            },
            "min_sequence_length": {"type": "integer", "minimum": 1, "default": 2}
        },
        "additionalProperties": false
    })
}

#[derive(Debug, Deserialize)]
struct TemporalPatternsPayload {
    events: Vec<TemporalEvent>,
    #[serde(default = "default_min_sequence_length")]
    min_sequence_length: usize,
}

#[derive(Debug, Deserialize)]
struct TemporalEvent {
    timestamp: u64,
    event: String,
}

fn default_min_sequence_length() -> usize {
    2
}

fn temporal_patterns_op(payload: Value) -> Result<Value, OpError> {
    let input: TemporalPatternsPayload = parse_payload(payload)?;
    let mut miner = TemporalPatternMiner::new(input.min_sequence_length);
    for event in input.events {
        miner.add_event(event.timestamp, event.event);
    }

    let patterns = miner.discover_temporal_patterns();
    let emerging = miner.get_emerging_patterns();
    let statistics = miner.get_statistics();

    Ok(json!({
        "patterns": patterns,
        "emerging_patterns": emerging,
        "statistics": statistics,
    }))
}

fn trend_detection_schema() -> Value {
    json!({
        "type": "object",
        "required": ["metric_name", "points"],
        "properties": {
            "metric_name": {"type": "string"},
            "min_window_size": {"type": "integer", "minimum": 1, "default": 2},
            "points": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["timestamp", "value"],
                    "properties": {
                        "timestamp": {"type": "integer", "minimum": 0},
                        "value": {"type": "number"}
                    },
                    "additionalProperties": false
                }
            },
            "predict_hours": {"type": ["number", "null"], "description": "If set, call Predict after detecting the trend."}
        },
        "additionalProperties": false
    })
}

#[derive(Debug, Deserialize)]
struct TrendDetectionPayload {
    metric_name: String,
    #[serde(default = "default_min_window_size")]
    min_window_size: usize,
    points: Vec<MetricPoint>,
    #[serde(default)]
    predict_hours: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct MetricPoint {
    timestamp: u64,
    value: f32,
}

fn default_min_window_size() -> usize {
    2
}

fn trend_detection_op(payload: Value) -> Result<Value, OpError> {
    let input: TrendDetectionPayload = parse_payload(payload)?;
    let mut detector = TrendDetector::new(input.min_window_size);
    for point in input.points {
        detector.add_data_point(point.timestamp, point.value);
    }

    let trend = detector.detect_trends(input.metric_name);
    let prediction = match &trend {
        Some(_) => input.predict_hours.and_then(|h| detector.predict_value(h)),
        None => None,
    };

    Ok(json!({
        "trend": trend,
        "significant_trends": detector.get_significant_trends(),
        "prediction": prediction,
        "statistics": detector.get_statistics(),
    }))
}

fn decision_tree_schema() -> Value {
    json!({
        "type": "object",
        "required": ["training_data"],
        "properties": {
            "training_data": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["features", "class"],
                    "properties": {
                        "features": {
                            "type": "object",
                            "additionalProperties": {"type": "number"}
                        },
                        "class": {"type": "string"}
                    },
                    "additionalProperties": false
                }
            },
            "max_depth": {"type": "integer", "minimum": 1, "default": 10},
            "min_samples_split": {"type": "integer", "minimum": 1, "default": 2},
            "predict": {
                "type": "array",
                "description": "Optional feature maps to classify with the trained tree.",
                "items": {
                    "type": "object",
                    "additionalProperties": {"type": "number"}
                }
            }
        },
        "additionalProperties": false
    })
}

#[derive(Debug, Deserialize)]
struct DecisionTreePayload {
    training_data: Vec<TrainingRow>,
    #[serde(default = "default_max_depth")]
    max_depth: usize,
    #[serde(default = "default_min_samples_split")]
    min_samples_split: usize,
    #[serde(default)]
    predict: Vec<HashMap<String, f32>>,
}

#[derive(Debug, Deserialize)]
struct TrainingRow {
    features: HashMap<String, f32>,
    class: String,
}

fn default_max_depth() -> usize {
    10
}

fn default_min_samples_split() -> usize {
    2
}

fn decision_tree_op(payload: Value) -> Result<Value, OpError> {
    let input: DecisionTreePayload = parse_payload(payload)?;
    if input.training_data.is_empty() {
        return Err(OpError::InvalidPayload(
            "training_data must contain at least one row".to_string(),
        ));
    }

    let mut builder = TreeBuilder::new();
    builder.set_max_depth(input.max_depth);
    builder.set_min_samples_split(input.min_samples_split);

    let training: Vec<(HashMap<String, f32>, String)> = input
        .training_data
        .into_iter()
        .map(|row| (row.features, row.class))
        .collect();

    let mut tree = builder.build(&training).ok_or_else(|| {
        OpError::Execution(
            "decision tree could not be built from the provided training data".to_string(),
        )
    })?;
    tree.calculate_importances();

    let predictions: Vec<Option<String>> = input
        .predict
        .iter()
        .map(|sample| tree.predict(sample))
        .collect();

    let correct = training
        .iter()
        .filter(|(features, expected)| tree.predict(features).as_deref() == Some(expected.as_str()))
        .count();
    let training_accuracy = correct as f32 / training.len() as f32;

    Ok(json!({
        "tree": tree,
        "feature_importances": tree.feature_importances,
        "statistics": tree.get_statistics(),
        "predictions": predictions,
        "training_accuracy": training_accuracy,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn navigation_event(seconds: i64, decision_type: &str) -> MissionEvent {
        MissionEvent::NavigationDecision {
            robot_id: "r1".to_string(),
            timestamp: Utc.timestamp_opt(seconds, 0).unwrap(),
            decision_type: decision_type.to_string(),
            rationale: None,
        }
    }

    fn obstacle_event(seconds: i64, obstacle_type: &str) -> MissionEvent {
        MissionEvent::ObstacleDetected {
            robot_id: "r1".to_string(),
            timestamp: Utc.timestamp_opt(seconds, 0).unwrap(),
            location: crate::core::event::Location {
                x: 1.0,
                y: 2.0,
                z: 0.0,
            },
            obstacle_type: obstacle_type.to_string(),
            confidence: Some(0.9),
        }
    }

    fn run_value(op: &str, payload: Value) -> Value {
        let text = run_op_json(op, &payload.to_string()).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn registry_is_allowlisted_and_documented() {
        let ops = ops();
        assert_eq!(ops.len(), 6);
        assert!(ops
            .iter()
            .any(|(name, _, _)| name == "phase16.causal_graph"));
        assert!(ops
            .iter()
            .any(|(name, _, _)| name == "phase20.decision_tree"));
        for (name, description, schema) in ops {
            assert!(!description.is_empty(), "{name} has no description");
            let parsed: Value = serde_json::from_str(&schema).unwrap();
            assert_eq!(parsed["type"], "object", "{name} schema is not an object");
        }
    }

    #[test]
    fn causal_graph_builds_real_graph_and_validation() {
        let events = vec![
            obstacle_event(100, "box"),
            navigation_event(101, "stop"),
            navigation_event(103, "replan"),
        ];
        let output = run_value("phase16.causal_graph", json!({ "events": events }));

        assert_eq!(output["graph"]["vertices"].as_array().unwrap().len(), 3);
        assert!(output["graph"]["edges"].as_array().unwrap().len() >= 2);
        assert_eq!(output["validation"]["vertex_count"], 3);
        assert_eq!(output["validation"]["is_valid_dag"], true);
    }

    #[test]
    fn counterfactual_removes_a_connected_node() {
        let graph = json!({
            "vertices": [
                {"id": "event_0", "vertex_type": "sensor_reading", "timestamp_ns": 0, "attributes": {}, "confidence": 0.9},
                {"id": "event_1", "vertex_type": "sensor_reading", "timestamp_ns": 1, "attributes": {}, "confidence": 0.9}
            ],
            "edges": [
                {"source_id": "event_0", "target_id": "event_1", "edge_type": "causal", "confidence": 0.8, "time_gap_ms": 10, "evidence": []}
            ],
            "timestamp_ns": 0
        });
        let output = run_value(
            "phase17.counterfactual",
            json!({
                "graph": graph,
                "baseline_outcome": "Mission success",
                "query": {
                    "query_type": "remove_node",
                    "target_node_id": "event_0"
                }
            }),
        );

        assert_eq!(output["affected_events"], json!([1]));
        assert_eq!(output["original_outcome"], "Mission success");
        assert!(output["outcome_change_magnitude"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn fleet_learning_ranks_and_advises() {
        let output = run_value(
            "phase18.fleet_learning",
            json!({
                "robots": {
                    "r1": [
                        {"succeeded": true, "time_ms": 1000},
                        {"succeeded": true, "time_ms": 1200}
                    ],
                    "r2": [
                        {"succeeded": false, "time_ms": 4000},
                        {"succeeded": false, "time_ms": 4500}
                    ]
                }
            }),
        );

        let leaderboard = output["leaderboard"].as_array().unwrap();
        assert_eq!(leaderboard.len(), 2);
        assert_eq!(leaderboard[0]["robot_id"], "r1");
        assert_eq!(leaderboard[0]["rank"], 1);
        assert!(!output["optimization_tips"].as_array().unwrap().is_empty());
        assert_eq!(output["statistics"]["num_robots"], 2.0);
    }

    #[test]
    fn temporal_patterns_finds_repeated_sequences() {
        let events = vec![
            json!({"timestamp": 0, "event": "sensor_drift"}),
            json!({"timestamp": 1, "event": "localization_loss"}),
            json!({"timestamp": 2, "event": "sensor_drift"}),
            json!({"timestamp": 3, "event": "localization_loss"}),
        ];
        let output = run_value(
            "phase19.temporal_patterns",
            json!({"events": events, "min_sequence_length": 2}),
        );

        assert!(!output["patterns"].as_array().unwrap().is_empty());
        assert!(output["statistics"]["total_patterns"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn trend_detection_fits_improving_series() {
        let points = vec![
            json!({"timestamp": 0, "value": 50.0}),
            json!({"timestamp": 100, "value": 55.0}),
            json!({"timestamp": 200, "value": 60.0}),
            json!({"timestamp": 300, "value": 65.0}),
        ];
        let output = run_value(
            "phase19.trend_detection",
            json!({
                "metric_name": "success_rate",
                "points": points,
                "predict_hours": 1.0
            }),
        );

        assert_eq!(output["trend"]["trend_type"], "Improving");
        assert!(output["trend"]["slope"].as_f64().unwrap() > 0.0);
        assert!(output["prediction"].as_f64().unwrap() > 65.0);
        assert_eq!(output["statistics"]["total_trends"], 1.0);
    }

    #[test]
    fn decision_tree_builds_and_predicts() {
        let training = vec![
            json!({"features": {"distance": 0.2, "battery": 0.9}, "class": "success"}),
            json!({"features": {"distance": 0.3, "battery": 0.8}, "class": "success"}),
            json!({"features": {"distance": 5.0, "battery": 0.2}, "class": "failure"}),
            json!({"features": {"distance": 6.0, "battery": 0.1}, "class": "failure"}),
        ];
        let output = run_value(
            "phase20.decision_tree",
            json!({
                "training_data": training,
                "predict": [
                    {"distance": 0.4, "battery": 0.9},
                    {"distance": 8.0, "battery": 0.1}
                ]
            }),
        );

        assert_eq!(output["tree"]["root"]["is_leaf"], false);
        assert_eq!(output["predictions"], json!(["success", "failure"]));
        assert!(output["training_accuracy"].as_f64().unwrap() > 0.9);
        assert!(!output["feature_importances"]
            .as_object()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn unknown_op_is_rejected_with_available_names() {
        let error = run_op_json("phase99.nope", "{}").unwrap_err();
        let message = error.to_string();
        assert!(message.contains("unknown op"));
        assert!(message.contains("phase16.causal_graph"));
    }

    #[test]
    fn malformed_json_is_rejected() {
        let error = run_op_json("phase16.causal_graph", "{not json").unwrap_err();
        assert!(matches!(error, OpError::InvalidPayload(_)));
        assert!(error.to_string().contains("malformed JSON"));
    }

    #[test]
    fn non_object_payload_is_rejected() {
        let error = run_op_json("phase18.fleet_learning", "[]").unwrap_err();
        assert!(matches!(error, OpError::InvalidPayload(_)));
        assert!(error.to_string().contains("JSON object"));
    }
}
