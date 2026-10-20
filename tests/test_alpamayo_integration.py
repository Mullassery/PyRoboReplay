"""Alpamayo integration tests use synthetic artifacts; no model or GPU is used."""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

import numpy as np
import pytest

MODULE_PATH = Path(__file__).parent.parent / "src" / "pyroboreplay" / "alpamayo.py"
spec = importlib.util.spec_from_file_location("alpamayo_integration", MODULE_PATH)
mod = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = mod
spec.loader.exec_module(mod)


def _trajectories(num_samples=2, horizon=64):
    base = np.zeros((num_samples, horizon, 3), dtype=np.float64)
    base[:, :, 0] = np.arange(horizon) * 0.5
    base[:, :, 1] = np.arange(num_samples)[:, None] * 0.1
    return base


class _Failure:
    def __init__(self, timestamp, failure_type="near_collision", severity="high"):
        self._timestamp = timestamp
        self._type = failure_type
        self._severity = severity

    def get_timestamp(self):
        return self._timestamp

    def get_failure_type(self):
        return self._type

    def get_severity(self):
        return self._severity

    def get_description(self):
        return "obstacle intrusion"

    def get_confidence(self):
        return 0.91


class _Mission:
    def mission_id(self):
        return "m-1"

    def name(self):
        return "demo mission"

    def detect_failures(self):
        return [_Failure(100.0)]


def test_parse_nested_output_and_reasoning():
    payload = {
        "pred_xyz": _trajectories()[None, None, ...],
        "pred_rot": np.broadcast_to(np.eye(3), (1, 1, 2, 64, 3, 3)),
        "cot": [["Nudge left for cones.", "Yield to the cyclist."]],
        "timestamp": 100.5,
        "clip_id": "clip-1",
    }

    record = mod.parse_alpamayo_output(payload)

    assert record.num_samples == 2
    assert record.best().xyz.shape == (64, 3)
    assert record.trajectories[0].reasoning == "Nudge left for cones."
    assert record.trajectories[1].reasoning == "Yield to the cyclist."
    assert record.timestamp == 100.5
    assert record.metadata["clip_id"] == "clip-1"


def test_load_json_npz_and_jsonl(tmp_path):
    payload = {
        "pred_xyz": _trajectories(num_samples=1).tolist(),
        "reasoning": ["Keep lane."],
        "timestamp": "2026-01-01T00:00:00Z",
    }
    json_path = tmp_path / "run.json"
    json_path.write_text(json.dumps(payload))

    record = mod.load_alpamayo_run(json_path)
    assert record.num_samples == 1
    assert record.reasoning == ("Keep lane.",)

    npz_path = tmp_path / "run.npz"
    np.savez(
        npz_path,
        pred_xyz=_trajectories(num_samples=1),
        cot=np.array(["Keep lane."]),
    )
    record = mod.load_alpamayo_run(npz_path)
    assert record.reasoning == ("Keep lane.",)

    jsonl_path = tmp_path / "runs.jsonl"
    jsonl_path.write_text(json.dumps(payload) + "\n" + json.dumps(payload))
    records = mod.load_alpamayo_runs(jsonl_path)
    assert len(records) == 2


def test_compare_trajectories_reports_best_sample():
    predicted = np.zeros((2, 3, 2))
    predicted[0] = [[1, 0], [2, 0], [3, 0]]
    predicted[1] = [[0, 0], [0, 0], [0, 0]]
    ground_truth = np.zeros((3, 2))

    metrics = mod.compare_trajectories(predicted, ground_truth)

    assert metrics["best_sample"] == 1
    assert metrics["min_ade"] == 0.0
    assert metrics["min_fde"] == 0.0
    assert metrics["ade"][0] == pytest.approx(2.0)
    assert metrics["horizon"] == 3


def test_report_correlates_reasoning_with_failures():
    record = mod.parse_alpamayo_output(
        {
            "pred_xyz": _trajectories(num_samples=2),
            "cot": ["Nudge left.", "Brake."],
            "timestamp": 100.5,
        }
    )
    ground_truth = _trajectories(num_samples=1)[0]

    report = mod.build_alpamayo_report(
        [record], mission=_Mission(), ground_truth=ground_truth
    )

    assert report["schema_version"] == mod.REPORT_SCHEMA_VERSION
    assert report["summary"]["records"] == 1
    assert report["summary"]["comparisons"] == 1
    assert report["summary"]["reasoning_traces"] == 2
    assert report["summary"]["correlated_records"] == 1
    assert report["records"][0]["correlated_failures"][0]["type"] == "near_collision"
    assert report["records"][0]["comparison"]["best_sample"] == 0


def test_report_does_not_correlate_failures_outside_tolerance():
    record = mod.parse_alpamayo_output(
        {"pred_xyz": _trajectories(num_samples=1), "timestamp": 110.0}
    )

    report = mod.build_alpamayo_report(
        record, mission=_Mission(), ground_truth=_trajectories(num_samples=1)[0]
    )

    assert report["summary"]["correlated_records"] == 0


def test_write_markdown_and_json(tmp_path):
    record = mod.parse_alpamayo_output(
        {
            "pred_xyz": _trajectories(num_samples=1),
            "reasoning": ["Keep lane."],
            "timestamp": 100.1,
        }
    )
    report = mod.build_alpamayo_report(
        record, mission=_Mission(), ground_truth=_trajectories(num_samples=1)[0]
    )

    md_path = tmp_path / "report.md"
    assert mod.write_alpamayo_report(report, md_path) == str(md_path)
    text = md_path.read_text()
    assert "# Alpamayo forensic report" in text
    assert "Keep lane." in text
    assert "minADE" in text

    json_path = tmp_path / "report.json"
    assert mod.write_alpamayo_report(report, json_path) == str(json_path)
    loaded = json.loads(json_path.read_text())
    assert loaded["summary"]["records"] == 1


def test_rejects_non_finite_and_bad_shapes():
    bad = _trajectories(num_samples=1)
    bad[0, 0, 0] = np.nan
    with pytest.raises(mod.AlpamayoInputError, match="non-finite"):
        mod.parse_alpamayo_output({"pred_xyz": bad})

    with pytest.raises(mod.AlpamayoInputError, match="pred_xyz"):
        mod.parse_alpamayo_output({"pred_xyz": np.zeros((64, 4))})

    with pytest.raises(mod.AlpamayoInputError, match="expected 64 waypoints"):
        mod.parse_alpamayo_output({"pred_xyz": np.zeros((10, 3))})


FIXTURE_DIR = Path(__file__).parent / "fixtures" / "alpamayo"


@pytest.mark.parametrize(
    ("filename", "model_version"),
    [
        ("alpamayo_1_5_synthetic.json", "alpamayo-1.5"),
        ("alpamayo_2_synthetic.json", "alpamayo-2"),
    ],
)
def test_load_model_version_fixtures_preserve_metadata(filename, model_version):
    path = FIXTURE_DIR / filename
    payload = json.loads(path.read_text())

    record = mod.load_alpamayo_run(path)
    report = mod.build_alpamayo_report(record)

    assert record.num_samples == 2
    assert record.best().xyz.shape == (64, 3)
    assert record.metadata["model_version"] == model_version
    assert record.metadata["model"] == payload["model"]
    assert record.metadata["clip_id"] == payload["clip_id"]
    assert record.reasoning == tuple(payload["cot"])
    assert report["records"][0]["metadata"]["model_version"] == model_version
