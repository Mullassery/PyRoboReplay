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

from ._core import (
    Action,
    Event,
    Failure,
    FleetStatistics,
    GeoHotspot,
    Hypothesis,
    Mission,
    RootCauseAnalysis,
    analyze_detection_gaps,
    analyze_nav2_diagnostics,
    build_navigation_session,
    fuse_rgb_thermal,
    load_pyterrainmap_grid,
    ops,
    run_op,
)
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
    write_asl_report,
)
from .asl import (
    report_to_markdown as asl_report_to_markdown,
)
from .mcap_export import McapUnavailable, export_mcap
from .rerun_export import RerunUnavailable, export_mission

try:
    from importlib.metadata import PackageNotFoundError
    from importlib.metadata import version as _package_version

    try:
        __version__ = _package_version("pyroboreplay")
    except PackageNotFoundError:  # running from a source checkout, not installed
        __version__ = "2.13.1"
except ImportError:  # pragma: no cover - importlib.metadata ships with Python >=3.8
    __version__ = "2.13.1"

__all__ = [
    "ASLActorPose",
    "ASLFrameInfo",
    "ASLInputError",
    "ASLRollout",
    "Action",
    "AlpamayoInputError",
    "AlpamayoRecord",
    "AlpamayoTrajectory",
    "Event",
    "Failure",
    "FleetStatistics",
    "GeoHotspot",
    "Hypothesis",
    "McapUnavailable",
    "Mission",
    "RerunUnavailable",
    "RootCauseAnalysis",
    "analyze_detection_gaps",
    "analyze_nav2_diagnostics",
    "asl_report_to_markdown",
    "build_alpamayo_report",
    "build_asl_report",
    "build_navigation_session",
    "compare_trajectories",
    "export_mcap",
    "export_mission",
    "fuse_rgb_thermal",
    "inspect_asl",
    "iter_alpasim_entries",
    "iter_asl_frames",
    "load_alpamayo_run",
    "load_alpamayo_runs",
    "load_pyterrainmap_grid",
    "ops",
    "parse_alpamayo_output",
    "parse_alpasim_entries",
    "read_asl",
    "report_to_markdown",
    "run_op",
    "write_alpamayo_report",
    "write_asl_report",
]
