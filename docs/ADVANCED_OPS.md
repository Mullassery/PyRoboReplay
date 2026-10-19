# Advanced operations (Phases 16-20)

Phases 16-20 are Rust library modules. They are **not** exposed as typed Python
classes or methods. Instead, the Python extension exposes one JSON operation
bridge so the implemented Rust logic is usable without hand-wrapping every
Rust type:

```python
import json
import pyroboreplay

for name, description, schema_json in pyroboreplay.ops():
    print(name, "-", description)

result = json.loads(
    pyroboreplay.run_op("phase19.trend_detection", json.dumps({
        "metric_name": "success_rate",
        "points": [
            {"timestamp": 0, "value": 50.0},
            {"timestamp": 100, "value": 55.0},
            {"timestamp": 200, "value": 60.0},
            {"timestamp": 300, "value": 65.0},
        ],
        "predict_hours": 1.0,
    }))
)
print(result["trend"]["trend_type"])  # "Improving"
```

`ops()` returns `(name, description, schema_json)` tuples. `schema_json` is a
JSON-Schema-style document describing the payload object. `run_op()` returns a
JSON string. Unknown operation names and malformed/invalid payloads raise
`ValueError`; execution failures raise `RuntimeError`.

This is a deliberately small allowlist, not a claim that every Phase 16-20
library type has a Python API.

## Operations

| Operation | Rust source | What it does |
|---|---|---|
| `phase16.causal_graph` | `phase16/causal_builder.rs`, `graph_validator.rs` | Builds a causal graph from `MissionEvent` values and validates it. |
| `phase17.counterfactual` | `phase17/counterfactual.rs` | Runs a counterfactual query against a causal graph. |
| `phase18.fleet_learning` | `phase18/fleet_learning.rs` | Builds a fleet leaderboard and optimization tips. |
| `phase19.temporal_patterns` | `phase19/temporal_patterns.rs` | Mines short/medium/long-term temporal event sequences. |
| `phase19.trend_detection` | `phase19/trend_detector.rs` | Fits a linear trend and optionally predicts a future value. |
| `phase20.decision_tree` | `phase20/decision_tree.rs` | Builds a decision tree, feature importances, and predictions. |

## `phase16.causal_graph`

Payload (`events` are serde-serialized `MissionEvent` values in externally
tagged form, i.e. `{"VariantName": {...}}`):

```json
{
  "events": [
    {
      "ObstacleDetected": {
        "robot_id": "r1",
        "timestamp": "2026-01-01T00:00:00Z",
        "location": {"x": 1.0, "y": 2.0, "z": 0.0},
        "obstacle_type": "box",
        "confidence": 0.9
      }
    },
    {
      "NavigationDecision": {
        "robot_id": "r1",
        "timestamp": "2026-01-01T00:00:00.500Z",
        "decision_type": "stop",
        "rationale": null
      }
    }
  ]
}
```

Response (trimmed):

```json
{
  "graph": {
    "vertices": [{"id": "event_0", "confidence": 0.9, "...": "..."}],
    "edges": [
      {
        "source_id": "event_0",
        "target_id": "event_1",
        "edge_type": "decision_trigger",
        "confidence": 0.85,
        "time_gap_ms": 500
      }
    ],
    "timestamp_ns": 1792442144559713000
  },
  "validation": {
    "is_valid_dag": true,
    "edge_count": 2,
    "vertex_count": 2,
    "variance_explained": 0.7875,
    "confidence_calibration_score": 0.2,
    "conflicts": [],
    "issues": []
  }
}
```

Scope note: the Phase 16 builder currently emits temporal-proximity and
decision-trigger edges. The magnitude-change, multi-modal, and historical
detector implementations in that module are placeholders and contribute no
edges, so this operation does not claim those detectors work.

## `phase17.counterfactual`

Pass the `graph` object returned by `phase16.causal_graph` (or any valid
`CausalGraphV2` document) and one query:

```json
{
  "graph": {
    "vertices": [
      {"id": "event_0", "vertex_type": "sensor_reading", "timestamp_ns": 0, "attributes": {}, "confidence": 0.9},
      {"id": "event_1", "vertex_type": "sensor_reading", "timestamp_ns": 1, "attributes": {}, "confidence": 0.9}
    ],
    "edges": [
      {"source_id": "event_0", "target_id": "event_1", "edge_type": "causal", "confidence": 0.8, "time_gap_ms": 10, "evidence": []}
    ],
    "timestamp_ns": 0
  },
  "baseline_outcome": "Mission success",
  "query": {
    "query_type": "remove_node",
    "target_node_id": "event_0"
  }
}
```

`query_type` is passed through to the Rust analyzer (`remove_node`,
`replace_node`, `modify_edge_weight`, and so on). Response:

```json
{
  "affected_events": [1],
  "alternative_outcome": "Mission delayed by ~1 events",
  "confidence": 0.83,
  "divergence_point": 1,
  "original_outcome": "Mission success",
  "outcome_change_magnitude": 0.5,
  "query": {"query_type": "remove_node", "target_node_id": "event_0", "parameter": null, "timestamp": 0},
  "summary": "Removing decision event_0 would moderately change the outcome. Result: Mission delayed by ~1 events"
}
```

## `phase18.fleet_learning`

Payload: a map of robot IDs to mission results, plus an optional tip limit.

```json
{
  "robots": {
    "r1": [
      {"succeeded": true, "time_ms": 1000},
      {"succeeded": true, "time_ms": 1200}
    ],
    "r2": [
      {"succeeded": false, "time_ms": 4000},
      {"succeeded": false, "time_ms": 4500}
    ]
  },
  "top_tips": 10
}
```

Response (trimmed):

```json
{
  "leaderboard": [
    {"robot_id": "r1", "rank": 1, "success_rate": 1.0, "avg_mission_time_ms": 1100, "efficiency_score": 78.0},
    {"robot_id": "r2", "rank": 2, "success_rate": 0.0, "avg_mission_time_ms": 4250, "efficiency_score": 15.0}
  ],
  "optimization_tips": [
    {
      "title": "Improve success rate for r2",
      "priority": "Critical",
      "potential_improvement": 90.0,
      "affected_robots": ["r2"],
      "category": "Navigation"
    }
  ],
  "statistics": {
    "num_robots": 2.0,
    "fleet_avg_success_rate": 0.5,
    "fleet_avg_mission_time_ms": 2675.0,
    "total_missions": 4.0
  }
}
```

## `phase19.temporal_patterns`

Payload:

```json
{
  "events": [
    {"timestamp": 0, "event": "sensor_drift"},
    {"timestamp": 1, "event": "localization_loss"},
    {"timestamp": 2, "event": "sensor_drift"},
    {"timestamp": 3, "event": "localization_loss"}
  ],
  "min_sequence_length": 2
}
```

Response (trimmed):

```json
{
  "patterns": [
    {
      "name": "Pattern: sensor_drift → localization_loss",
      "window": "ShortTerm",
      "events": ["sensor_drift", "localization_loss"],
      "frequency_per_day": 0.0333,
      "predictability_score": 0.1667
    }
  ],
  "emerging_patterns": [],
  "statistics": {"total_patterns": 9.0, "avg_frequency_per_day": 0.0333, "avg_predictability": 0.1667}
}
```

The exact pattern count depends on the event sequence because the miner scans
all three time windows.

## `phase19.trend_detection`

Payload:

```json
{
  "metric_name": "success_rate",
  "min_window_size": 2,
  "points": [
    {"timestamp": 0, "value": 50.0},
    {"timestamp": 100, "value": 55.0},
    {"timestamp": 200, "value": 60.0},
    {"timestamp": 300, "value": 65.0}
  ],
  "predict_hours": 1.0
}
```

`predict_hours` is optional. Timestamps are seconds; the Rust detector converts
them to hours internally. Response (trimmed):

```json
{
  "trend": {
    "metric_name": "success_rate",
    "trend_type": "Improving",
    "slope": 180.0,
    "confidence": 1.0,
    "duration_seconds": 300,
    "start_value": 50.0,
    "end_value": 65.0,
    "total_change_percent": 30.0
  },
  "significant_trends": [],
  "prediction": 245.0,
  "statistics": {"total_trends": 1.0, "avg_confidence": 1.0, "improving_trends": 1.0, "degrading_trends": 0.0}
}
```

If fewer than `min_window_size` points are supplied, `trend` is `null`.

## `phase20.decision_tree`

Payload:

```json
{
  "training_data": [
    {"features": {"distance": 0.2, "battery": 0.9}, "class": "success"},
    {"features": {"distance": 0.3, "battery": 0.8}, "class": "success"},
    {"features": {"distance": 5.0, "battery": 0.2}, "class": "failure"},
    {"features": {"distance": 6.0, "battery": 0.1}, "class": "failure"}
  ],
  "max_depth": 10,
  "min_samples_split": 2,
  "predict": [
    {"distance": 0.4, "battery": 0.9},
    {"distance": 8.0, "battery": 0.1}
  ]
}
```

`max_depth`, `min_samples_split`, and `predict` are optional. Response
(trimmed):

```json
{
  "tree": {
    "root": {
      "is_leaf": false,
      "feature": "distance",
      "split": {"LessThan": 5.0},
      "samples": 4,
      "left_child": {"is_leaf": true, "class": "success", "samples": 2},
      "right_child": {"is_leaf": true, "class": "failure", "samples": 2}
    },
    "max_depth": 2
  },
  "feature_importances": {"distance": 0.5},
  "statistics": {"max_depth": 2.0, "leaf_count": 2.0, "root_gini": 0.5, "accuracy": 0.0},
  "predictions": ["success", "failure"],
  "training_accuracy": 1.0
}
```

`training_accuracy` is computed by the bridge by re-predicting the supplied
training rows; it is not a held-out evaluation metric.

## Not exposed through this bridge

The allowlist intentionally does not cover every Phase 16-20 type. In
particular, decision reconstruction, alternative timelines, outcome-influence
scoring, decision clustering, pattern mining, window analysis, counterfactual
scenario analysis, and rule extraction remain Rust-library-only. Some of those
input types contain module-private fields, so exposing them would require a
larger Rust API change rather than a JSON mapping.
