"""MCAP export tests read the file back with the mcap reader."""

from __future__ import annotations

import pytest

pytest.importorskip("mcap")

from mcap.reader import make_reader

from pyroboreplay.mcap_export import export_mcap


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

    def get_failure_type(self):
        return "near_collision"

    def get_severity(self):
        return "high"

    def get_description(self):
        return "obstacle too close"

    def get_confidence(self):
        return 0.91


class _Mission:
    def get_all_events(self):
        return [
            _Event("lidar_scan", "2026-01-01T00:00:00Z"),
            _Event("camera_frame", 1.0, sensor="camera"),
        ]

    def detect_failures(self):
        return [_Failure()]


def test_export_mcap_roundtrip(tmp_path):
    path = tmp_path / "mission.mcap"
    assert export_mcap(_Mission(), path) == str(path)
    with path.open("rb") as stream:
        reader = make_reader(stream)
        summary = reader.get_summary()
        assert summary is not None
        assert summary.statistics.message_count == 3
        topics = {channel.topic for channel in summary.channels.values()}
        assert "/events/lidar" in topics
        assert "/events/camera" in topics
        assert "/failures" in topics
