"""Optional Rerun export for mission timelines.

Rerun (Apache-2.0) is the common multimodal robotics viewer. This adapter logs
the mission event timeline and detected failures into a `.rrd` recording or a
live Rerun viewer. PyRoboReplay does not bundle a viewer or fork Rerun.

Install::

    pip install "pyroboreplay[rerun]"
"""

from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional


class RerunUnavailable(RuntimeError):
    """Raised when Rerun is not installed."""


def _require_rerun():
    try:
        import rerun as rr
    except ImportError as exc:
        raise RerunUnavailable(
            "Rerun is not installed. Install the optional viewer with "
            '`pip install "pyroboreplay[rerun]"` (or `pip install rerun-sdk`).'
        ) from exc
    return rr


def _parse_timestamp(value: Any) -> Optional[datetime]:
    if isinstance(value, datetime):
        return value if value.tzinfo else value.replace(tzinfo=timezone.utc)
    if isinstance(value, (int, float)):
        return datetime.fromtimestamp(float(value), tz=timezone.utc)
    if isinstance(value, str):
        try:
            parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
        except ValueError:
            return None
        return parsed if parsed.tzinfo else parsed.replace(tzinfo=timezone.utc)
    return None


def export_mission(
    mission: Any,
    path: str | Path | None = None,
    *,
    application_id: Optional[str] = None,
    spawn: bool = False,
) -> Optional[str]:
    """Export a mission timeline and failures to Rerun.

    Args:
        mission: A PyRoboReplay ``Mission`` (or a compatible object exposing
            ``mission_id()``, ``name()``, ``get_all_events()`` and
            ``detect_failures()``).
        path: Optional ``.rrd`` output path. If omitted and ``spawn=False``,
            a recording is initialized but not saved unless the caller later
            calls ``rr.save``; prefer passing a path for CLI use.
        application_id: Rerun application id; defaults to the mission id.
        spawn: Launch the live Rerun viewer (requires a desktop environment).

    Returns:
        The output path as a string, or None when no path was given.
    """
    rr = _require_rerun()
    mission_id = str(mission.mission_id())
    name = str(mission.name())
    rr.init(application_id or f"pyroboreplay/{mission_id}", spawn=spawn)
    if path is not None:
        rr.save(str(path))

    summary = (
        f"# {name}\n\n"
        f"- mission id: `{mission_id}`\n"
        f"- events: {mission.event_count()}\n"
        f"- duration: {mission.duration_seconds()} s\n"
    )
    rr.log("mission/summary", rr.TextDocument(summary), static=True)

    for event in mission.get_all_events():
        timestamp = _parse_timestamp(event.get_timestamp())
        if timestamp is not None:
            rr.set_time("time", timestamp=timestamp)
        sensor = event.get_sensor_type() or "other"
        line = f"{event.get_event_type()}"
        if event.get_robot_id():
            line += f" robot={event.get_robot_id()}"
        rr.log(f"events/{sensor}", rr.TextLog(line))

    for failure in mission.detect_failures():
        timestamp = _parse_timestamp(failure.get_timestamp())
        if timestamp is not None:
            rr.set_time("time", timestamp=timestamp)
        rr.log(
            "failures",
            rr.TextLog(
                f"[{failure.get_severity()}] {failure.get_failure_type()}: "
                f"{failure.get_description()} (confidence={failure.get_confidence():.2f})",
                level="ERROR" if failure.get_severity().lower() in ("critical", "high") else "WARN",
            ),
        )

    return str(path) if path is not None else None


__all__ = ["RerunUnavailable", "export_mission"]
