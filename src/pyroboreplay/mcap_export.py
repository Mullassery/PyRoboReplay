"""Optional MCAP export for interoperability.

MCAP (MIT) is the indexed robotics log container used by Foxglove and Rerun.
This adapter logs mission events/failures as JSON messages, preserving
timestamps and topics without bundling a viewer or format fork.

Install::

    pip install "pyroboreplay[mcap]"
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


class McapUnavailable(RuntimeError):
    """Raised when the mcap package is not installed."""


def _require_mcap():
    try:
        from mcap.writer import Writer
    except ImportError as exc:
        raise McapUnavailable(
            "mcap is not installed. Install with "
            '`pip install "pyroboreplay[mcap]"`.'
        ) from exc
    return Writer


def _to_ns(value: Any) -> int:
    if isinstance(value, datetime):
        dt = value if value.tzinfo else value.replace(tzinfo=timezone.utc)
        return int(dt.timestamp() * 1_000_000_000)
    if isinstance(value, (int, float)):
        return int(float(value) * 1_000_000_000)
    if isinstance(value, str):
        try:
            dt = datetime.fromisoformat(value.replace("Z", "+00:00"))
        except ValueError:
            return 0
        return int(dt.timestamp() * 1_000_000_000)
    return 0


def export_mcap(mission: Any, path: str | Path) -> str:
    """Write mission events and failures to an MCAP file."""
    Writer = _require_mcap()
    path = Path(path)
    schema_id = 0
    channel_ids: dict[str, int] = {}
    with path.open("wb") as stream:
        writer = Writer(stream)
        writer.start(profile="pyroboreplay", library="pyroboreplay")
        schema_id = writer.register_schema(
            name="pyroboreplay.Event",
            encoding="jsonschema",
            data=json.dumps(
                {
                    "type": "object",
                    "properties": {
                        "event_type": {"type": "string"},
                        "robot_id": {"type": ["string", "null"]},
                        "sensor_type": {"type": ["string", "null"]},
                    },
                }
            ).encode("utf-8"),
        )

        def channel(topic: str) -> int:
            if topic not in channel_ids:
                channel_ids[topic] = writer.register_channel(
                    topic=topic, message_encoding="json", schema_id=schema_id
                )
            return channel_ids[topic]

        for event in mission.get_all_events():
            sensor = event.get_sensor_type() or "other"
            topic = f"/events/{sensor}"
            payload = {
                "event_type": event.get_event_type(),
                "robot_id": event.get_robot_id(),
                "sensor_type": event.get_sensor_type(),
            }
            ts = _to_ns(event.get_timestamp())
            writer.add_message(
                channel_id=channel(topic),
                log_time=ts,
                publish_time=ts,
                data=json.dumps(payload).encode("utf-8"),
            )

        for failure in mission.detect_failures():
            payload = {
                "failure_type": failure.get_failure_type(),
                "severity": failure.get_severity(),
                "description": failure.get_description(),
                "confidence": failure.get_confidence(),
            }
            ts = _to_ns(failure.get_timestamp())
            writer.add_message(
                channel_id=channel("/failures"),
                log_time=ts,
                publish_time=ts,
                data=json.dumps(payload).encode("utf-8"),
            )
        writer.finish()
    return str(path)


__all__ = ["McapUnavailable", "export_mcap"]
