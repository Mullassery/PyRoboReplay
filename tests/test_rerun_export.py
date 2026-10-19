"""Rerun exporter tests use a duck-typed mission; no ROS bag is required."""

from __future__ import annotations

import importlib.util
from pathlib import Path

import pytest

pytest.importorskip("rerun")

MODULE_PATH = Path(__file__).parent.parent / "src" / "pyroboreplay" / "rerun_export.py"
spec = importlib.util.spec_from_file_location("rerun_export", MODULE_PATH)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)


class _Event:
    def __init__(self, kind, ts, sensor="lidar", robot="r1"):
        self._kind = kind
        self._ts = ts
        self._sensor = sensor
        self._robot = robot

    def get_event_type(self):
        return self._kind

    def get_timestamp(self):
        return self._ts

    def get_sensor_type(self):
        return self._sensor

    def get_robot_id(self):
        return self._robot


class _Failure:
    def get_timestamp(self):
        return 2.0

    def get_severity(self):
        return "high"

    def get_failure_type(self):
        return "near_collision"

    def get_description(self):
        return "obstacle too close"

    def get_confidence(self):
        return 0.91


class _Mission:
    def mission_id(self):
        return "m-1"

    def name(self):
        return "demo mission"

    def event_count(self):
        return 2

    def duration_seconds(self):
        return 2

    def get_all_events(self):
        return [
            _Event("lidar_scan", "2026-01-01T00:00:00Z"),
            _Event("camera_frame", 1.0, sensor="camera"),
        ]

    def detect_failures(self):
        return [_Failure()]


def test_export_writes_rrd(tmp_path):
    out = tmp_path / "mission.rrd"
    assert mod.export_mission(_Mission(), out) == str(out)
    assert out.is_file()
    assert out.stat().st_size > 0


def test_missing_rerun_is_actionable(monkeypatch, tmp_path):
    monkeypatch.setitem(__import__("sys").modules, "rerun", None)
    with pytest.raises(mod.RerunUnavailable, match="pyroboreplay\\[rerun\\]"):
        mod.export_mission(_Mission(), tmp_path / "x.rrd")
