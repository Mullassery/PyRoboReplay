"""
PyRoboReplay: Python package

Rust-powered extension via PyO3/maturin. The compiled extension module is
built from src/lib.rs's `#[pymodule] fn _core` (see `[tool.maturin]` in
pyproject.toml) and re-exported here so `from pyroboreplay import Mission`
works directly, matching the classes `src/lib.rs`'s `#[pymodule]` actually
registers: Mission, Event, Failure, Hypothesis, RootCauseAnalysis, Action,
FleetStatistics, GeoHotspot.
"""

from ._core import (  # noqa: F401
    Mission,
    Event,
    Failure,
    Hypothesis,
    RootCauseAnalysis,
    Action,
    FleetStatistics,
    GeoHotspot,
)

__version__ = "2.10.0"
__all__ = [
    "Mission",
    "Event",
    "Failure",
    "Hypothesis",
    "RootCauseAnalysis",
    "Action",
    "FleetStatistics",
    "GeoHotspot",
]
