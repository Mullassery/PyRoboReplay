"""
PyRoboReplay: Python package

Rust-powered extension via PyO3/maturin. The compiled extension module is
built from src/lib.rs's `#[pymodule] fn _core` (see `[tool.maturin]` in
pyproject.toml) and re-exported here so `from pyroboreplay import Mission`
works directly, matching the classes `src/lib.rs`'s `#[pymodule]` actually
registers: Mission, Event, Failure, Hypothesis, RootCauseAnalysis, Action,
FleetStatistics, GeoHotspot. Phase 16-20 Rust libraries are reachable through
the JSON operation bridge (`ops()` and `run_op()`); they are not typed Python
classes.
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
from .asl import (
    ASLActorPose,
    ASLFrameInfo,
    ASLInputError,
    ASLRollout,
    build_asl_report,
    inspect_asl,
    iter_alpasim_entries,
    iter_asl_frames,
    parse_alpasim_entries,
    read_asl,
    report_to_markdown as asl_report_to_markdown,
    write_asl_report,
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
    analyze_detection_gaps,
    analyze_nav2_diagnostics,
    build_navigation_session,
    fuse_rgb_thermal,
    load_pyterrainmap_grid,
    ops,
    run_op,
)

try:
    from importlib.metadata import PackageNotFoundError, version as _package_version

    try:
        __version__ = _package_version("pyroboreplay")
    except PackageNotFoundError:  # running from a source checkout, not installed
        __version__ = "2.13.0"
except ImportError:  # pragma: no cover - importlib.metadata ships with Python >=3.8
    __version__ = "2.13.0"

__all__ = [
    "Mission",
    "Event",
    "Failure",
    "Hypothesis",
    "RootCauseAnalysis",
    "Action",
    "FleetStatistics",
    "GeoHotspot",
    "load_pyterrainmap_grid",
    "analyze_detection_gaps",
    "fuse_rgb_thermal",
    "analyze_nav2_diagnostics",
    "build_navigation_session",
    "run_op",
    "ops",
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
    "ASLActorPose",
    "ASLFrameInfo",
    "ASLInputError",
    "ASLRollout",
    "build_asl_report",
    "inspect_asl",
    "iter_alpasim_entries",
    "iter_asl_frames",
    "parse_alpasim_entries",
    "read_asl",
    "asl_report_to_markdown",
    "write_asl_report",
]
