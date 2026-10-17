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

try:
    from importlib.metadata import PackageNotFoundError, version as _package_version

    try:
        __version__ = _package_version("pyroboreplay")
    except PackageNotFoundError:  # running from a source checkout, not installed
        __version__ = "2.10.1"
except ImportError:  # pragma: no cover - importlib.metadata ships with Python >=3.8
    __version__ = "2.10.1"

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
