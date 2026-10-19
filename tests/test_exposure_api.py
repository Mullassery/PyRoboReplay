"""Public-surface tests for the Phase 12-15 / PyTerrainMap Python bindings."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

pytest.importorskip("pyroboreplay")

import pyroboreplay  # noqa: E402

FIXTURES = Path(__file__).parent / "fixtures"


def test_new_symbols_exported():
    for name in (
        "load_pyterrainmap_grid",
        "analyze_detection_gaps",
        "fuse_rgb_thermal",
        "analyze_nav2_diagnostics",
        "build_navigation_session",
    ):
        assert name in pyroboreplay.__all__
        assert callable(getattr(pyroboreplay, name))


def test_load_pyterrainmap_grid_reads_real_export():
    raw = pyroboreplay.load_pyterrainmap_grid(
        str(FIXTURES / "pyterrainmap-grid.json")
    )
    payload = json.loads(raw)
    assert payload["document"]["format"] == "pyterrainmap-gridmap"
    assert payload["document"]["version"] == 1
    assert payload["summary"]["cells"] >= 1


def test_analyze_detection_gaps_matches_rust_surface():
    yolo = json.dumps(
        [
            {
                "id": 1,
                "class": "Person",
                "confidence": 0.9,
                "bbox": {"x": 0.0, "y": 0.0, "width": 10.0, "height": 10.0},
                "distance_m": 3.0,
                "velocity_ms": None,
                "position_3d": None,
                "trajectory_id": None,
                "attributes": {},
            }
        ]
    )
    payload = json.loads(pyroboreplay.analyze_detection_gaps(yolo, "[]"))
    assert payload["stats"]["yolo_detections"] == 1
    assert payload["stats"]["dino_detections"] == 0
    assert payload["stats"]["detection_gaps"] == 0


def test_analyze_nav2_diagnostics_runs_available_analyzers():
    payload = json.loads(
        pyroboreplay.analyze_nav2_diagnostics(
            json.dumps(
                {
                    "localization": {
                        "particle_spread_before": 0.2,
                        "particle_spread_after": 1.5,
                    },
                    "classification": {
                        "failure_type": "door_crossing",
                        "environment_type": "office",
                        "scale_meters": 10.0,
                    },
                }
            )
        )
    )
    assert len(payload["localization_issues"]) == 1
    assert payload["classification"]["limitation"] == "SemanticNavigationRequired"


def test_build_navigation_session_from_real_bag():
    payload = json.loads(
        pyroboreplay.build_navigation_session(
            str(FIXTURES / "ros2_cdr_fixture.db3")
        )
    )
    assert payload["stats"]["total_events"] > 0
    assert payload["modalities"]
