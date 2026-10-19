"""
PyRoboReplay: Python package

Rust-powered extension via PyO3/maturin. The compiled extension module is
built from src/lib.rs's `#[pymodule] fn _core` (see `[tool.maturin]` in
pyproject.toml) and re-exported here so `from pyroboreplay import Mission`
works directly, matching the classes `src/lib.rs`'s `#[pymodule]` actually
registers: Mission, Event, Failure, Hypothesis, RootCauseAnalysis, Action,
FleetStatistics, GeoHotspot.
"""

from .alpamayo import (
    AlpamayoInputError,
    AlpamayoRecord,
    AlpamayoTrajectory,
    build_alpamayo_report,
    compare_trajectories,
    load_alpamayo_run,
    load_alpamayo_runs,
    parse_alpamayo_output,
    report_to_markdown,
    write_alpamayo_report,
)
from .mcap_export import McapUnavailable, export_mcap
from .rerun_export import RerunUnavailable, export_mission

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
        __version__ = "2.10.2"
except ImportError:  # pragma: no cover - importlib.metadata ships with Python >=3.8
    __version__ = "2.10.2"

__all__ = [
    "Mission",
    "Event",
    "Failure",
    "Hypothesis",
    "RootCauseAnalysis",
    "Action",
    "FleetStatistics",
    "GeoHotspot",
    "RerunUnavailable",
    "export_mission",
    "export_mcap",
    "McapUnavailable",
    "AlpamayoInputError",
    "AlpamayoRecord",
    "AlpamayoTrajectory",
    "build_alpamayo_report",
    "compare_trajectories",
    "load_alpamayo_run",
    "load_alpamayo_runs",
    "parse_alpamayo_output",
    "report_to_markdown",
    "write_alpamayo_report",
]
