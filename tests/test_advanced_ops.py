"""Public-surface tests for the Phase 16-20 JSON operation bridge."""

from __future__ import annotations

import json

import pytest

pytest.importorskip("pyroboreplay")

import pyroboreplay  # noqa: E402


def _run(op: str, payload: dict) -> dict:
    return json.loads(pyroboreplay.run_op(op, json.dumps(payload)))


def test_ops_registry_lists_allowlisted_operations():
    entries = pyroboreplay.ops()
    assert entries
    assert "run_op" in pyroboreplay.__all__
    assert "ops" in pyroboreplay.__all__
    assert "Phase 16-20" in (pyroboreplay.run_op.__doc__ or "")

    names = {name for name, _, _ in entries}
    assert names == {
        "phase16.causal_graph",
        "phase17.counterfactual",
        "phase18.fleet_learning",
        "phase19.temporal_patterns",
        "phase19.trend_detection",
        "phase20.decision_tree",
    }
    for name, description, schema in entries:
        assert isinstance(name, str) and name
        assert isinstance(description, str) and description
        schema_doc = json.loads(schema)
        assert schema_doc["type"] == "object"


def test_phase16_causal_graph():
    events = [
        {
            "ObstacleDetected": {
                "robot_id": "r1",
                "timestamp": "2026-01-01T00:00:00Z",
                "location": {"x": 1.0, "y": 2.0, "z": 0.0},
                "obstacle_type": "box",
                "confidence": 0.9,
            }
        },
        {
            "NavigationDecision": {
                "robot_id": "r1",
                "timestamp": "2026-01-01T00:00:00.500Z",
                "decision_type": "stop",
                "rationale": None,
            }
        },
        {
            "NavigationDecision": {
                "robot_id": "r1",
                "timestamp": "2026-01-01T00:00:01Z",
                "decision_type": "replan",
                "rationale": None,
            }
        },
    ]
    payload = _run("phase16.causal_graph", {"events": events})
    assert len(payload["graph"]["vertices"]) == 3
    assert len(payload["graph"]["edges"]) >= 2
    assert payload["validation"]["vertex_count"] == 3
    assert payload["validation"]["is_valid_dag"] is True


def test_phase17_counterfactual():
    graph = {
        "vertices": [
            {
                "id": "event_0",
                "vertex_type": "sensor_reading",
                "timestamp_ns": 0,
                "attributes": {},
                "confidence": 0.9,
            },
            {
                "id": "event_1",
                "vertex_type": "sensor_reading",
                "timestamp_ns": 1,
                "attributes": {},
                "confidence": 0.9,
            },
        ],
        "edges": [
            {
                "source_id": "event_0",
                "target_id": "event_1",
                "edge_type": "causal",
                "confidence": 0.8,
                "time_gap_ms": 10,
                "evidence": [],
            }
        ],
        "timestamp_ns": 0,
    }
    payload = _run(
        "phase17.counterfactual",
        {
            "graph": graph,
            "baseline_outcome": "Mission success",
            "query": {
                "query_type": "remove_node",
                "target_node_id": "event_0",
            },
        },
    )
    assert payload["affected_events"] == [1]
    assert payload["original_outcome"] == "Mission success"
    assert payload["outcome_change_magnitude"] > 0


def test_phase18_fleet_learning():
    payload = _run(
        "phase18.fleet_learning",
        {
            "robots": {
                "r1": [
                    {"succeeded": True, "time_ms": 1000},
                    {"succeeded": True, "time_ms": 1200},
                ],
                "r2": [
                    {"succeeded": False, "time_ms": 4000},
                    {"succeeded": False, "time_ms": 4500},
                ],
            }
        },
    )
    assert payload["leaderboard"][0]["robot_id"] == "r1"
    assert payload["leaderboard"][0]["rank"] == 1
    assert payload["optimization_tips"]
    assert payload["statistics"]["num_robots"] == 2.0


def test_phase19_temporal_patterns():
    payload = _run(
        "phase19.temporal_patterns",
        {
            "events": [
                {"timestamp": 0, "event": "sensor_drift"},
                {"timestamp": 1, "event": "localization_loss"},
                {"timestamp": 2, "event": "sensor_drift"},
                {"timestamp": 3, "event": "localization_loss"},
            ],
            "min_sequence_length": 2,
        },
    )
    assert payload["patterns"]
    assert payload["statistics"]["total_patterns"] > 0


def test_phase19_trend_detection():
    payload = _run(
        "phase19.trend_detection",
        {
            "metric_name": "success_rate",
            "points": [
                {"timestamp": 0, "value": 50.0},
                {"timestamp": 100, "value": 55.0},
                {"timestamp": 200, "value": 60.0},
                {"timestamp": 300, "value": 65.0},
            ],
            "predict_hours": 1.0,
        },
    )
    assert payload["trend"]["trend_type"] == "Improving"
    assert payload["trend"]["slope"] > 0
    assert payload["prediction"] > 65.0
    assert payload["statistics"]["total_trends"] == 1.0


def test_phase20_decision_tree():
    payload = _run(
        "phase20.decision_tree",
        {
            "training_data": [
                {"features": {"distance": 0.2, "battery": 0.9}, "class": "success"},
                {"features": {"distance": 0.3, "battery": 0.8}, "class": "success"},
                {"features": {"distance": 5.0, "battery": 0.2}, "class": "failure"},
                {"features": {"distance": 6.0, "battery": 0.1}, "class": "failure"},
            ],
            "predict": [
                {"distance": 0.4, "battery": 0.9},
                {"distance": 8.0, "battery": 0.1},
            ],
        },
    )
    assert payload["tree"]["root"]["is_leaf"] is False
    assert payload["predictions"] == ["success", "failure"]
    assert payload["training_accuracy"] > 0.9
    assert payload["feature_importances"]


def test_unknown_op_reports_available_operations():
    with pytest.raises(ValueError) as excinfo:
        pyroboreplay.run_op("phase99.nope", "{}")
    message = str(excinfo.value)
    assert "unknown op" in message
    assert "phase16.causal_graph" in message


def test_malformed_json_is_a_value_error():
    with pytest.raises(ValueError) as excinfo:
        pyroboreplay.run_op("phase16.causal_graph", "{not json")
    assert "malformed JSON" in str(excinfo.value)


def test_non_object_payload_is_a_value_error():
    with pytest.raises(ValueError) as excinfo:
        pyroboreplay.run_op("phase18.fleet_learning", "[]")
    assert "JSON object" in str(excinfo.value)
