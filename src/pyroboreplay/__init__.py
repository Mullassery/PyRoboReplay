"""
PyRoboReplay: AI Navigation Reliability Engineer for autonomous robots.

Rust-powered extension via PyO3/maturin for mission replay, root cause analysis, and fleet monitoring.
"""

# Import Rust extension classes
try:
    from . import pyroboreplay as _core

    # Export all Rust classes
    Mission = _core.Mission
    Event = _core.Event
    Failure = _core.Failure
    Hypothesis = _core.Hypothesis
    RootCauseAnalysis = _core.RootCauseAnalysis
    Action = _core.Action
    FleetStatistics = _core.FleetStatistics
    GeoHotspot = _core.GeoHotspot
except (ImportError, AttributeError):
    # Fallback: undefined if Rust extension not built
    Mission = None
    Event = None
    Failure = None
    Hypothesis = None
    RootCauseAnalysis = None
    Action = None
    FleetStatistics = None
    GeoHotspot = None

__version__ = "2.3.0"
__author__ = "Georgi Mammen Mullassery"
__license__ = "MIT"

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
