"""AlpaSim ASL log ingestion.

AlpaSim writes size-delimited protobuf ``.asl`` logs. This module keeps the
AlpaSim/gRPC dependency optional: :func:`inspect_asl` validates framing with
only the standard library, :func:`parse_alpasim_entries` maps decoded
protobuf-like objects, and :func:`read_asl` imports the upstream reader only
when called.
"""

from __future__ import annotations

import json
from collections import defaultdict
from collections.abc import AsyncIterator, Iterable, Mapping, Sequence
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import numpy as np

REPORT_SCHEMA_VERSION = "1.0"
DEFAULT_MAX_FRAME_BYTES = 256 * 1024 * 1024
_KNOWN_ENTRY_FIELDS = (
    "rollout_metadata",
    "metadata",
    "actor_poses",
    "driver_return",
    "controller_return",
    "render_return",
    "traffic_return",
    "physics_return",
    "video_model_return",
)


class ASLInputError(ValueError):
    """Raised when an ASL log or decoded AlpaSim entry is invalid."""


@dataclass(frozen=True)
class ASLActorPose:
    """One actor pose at a rollout timestamp."""

    timestamp_us: int
    actor_id: str
    x: float
    y: float
    z: float
    qx: float = 0.0
    qy: float = 0.0
    qz: float = 0.0
    qw: float = 1.0
    actor_label: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "timestamp_us": self.timestamp_us,
            "actor_id": self.actor_id,
            "actor_label": self.actor_label,
            "position": {"x": self.x, "y": self.y, "z": self.z},
            "orientation": {"qx": self.qx, "qy": self.qy, "qz": self.qz, "qw": self.qw},
        }


@dataclass
class ASLRollout:
    """Decoded rollout metadata, actor poses, and ground truth."""

    session_uuid: str | None = None
    scene_id: str | None = None
    start_timestamp_us: int | None = None
    control_timestep_us: int | None = None
    version_ids: dict[str, str] = field(default_factory=dict)
    ground_truth_trajectory: np.ndarray | None = None
    actor_poses: list[ASLActorPose] = field(default_factory=list)
    entry_counts: dict[str, int] = field(default_factory=dict)
    unknown_entries: int = 0
    frame_count: int = 0

    def actor_tracks(self) -> dict[str, list[ASLActorPose]]:
        tracks: dict[str, list[ASLActorPose]] = defaultdict(list)
        for pose in self.actor_poses:
            tracks[pose.actor_id].append(pose)
        for poses in tracks.values():
            poses.sort(key=lambda pose: pose.timestamp_us)
        return dict(tracks)


@dataclass(frozen=True)
class ASLFrameInfo:
    """Structural inspection of an ASL file without protobuf decoding."""

    path: str
    frame_count: int
    payload_bytes: int
    max_frame_bytes: int


def _attr(obj: Any, *names: str, default: Any = None) -> Any:
    if isinstance(obj, Mapping):
        for name in names:
            if name in obj:
                return obj[name]
        return default
    for name in names:
        if hasattr(obj, name):
            return getattr(obj, name)
    return default


def _text(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    name = getattr(value, "name", None)
    if isinstance(name, str):
        return name
    return str(value)


def _int(value: Any) -> int | None:
    if value is None:
        return None
    try:
        return int(value)
    except (TypeError, ValueError):
        return None


def _float(value: Any, default: float = 0.0) -> float:
    try:
        return float(value)
    except (TypeError, ValueError):
        return default


def _sequence(value: Any) -> list[Any]:
    if value is None:
        return []
    if isinstance(value, Mapping):
        return [value]
    if isinstance(value, (str, bytes, bytearray)):
        return [value]
    try:
        return list(value)
    except TypeError:
        return [value]


def _is_sequence(value: Any) -> bool:
    if isinstance(value, (str, bytes, bytearray, Mapping)):
        return False
    return isinstance(value, Sequence) or (
        hasattr(value, "__iter__") and not hasattr(value, "DESCRIPTOR")
    )


def _pose_values(pose: Any) -> tuple[float, float, float, float, float, float, float]:
    if pose is None:
        return 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0

    if isinstance(pose, Sequence) and not isinstance(pose, (str, bytes, bytearray)):
        values = [_float(item) for item in pose]
        if len(values) >= 7:
            return tuple(values[:7])  # type: ignore[return-value]
        if len(values) == 3:
            return values[0], values[1], values[2], 0.0, 0.0, 0.0, 1.0

    nested = _attr(pose, "pose", "transform")
    if nested is not None and nested is not pose:
        return _pose_values(nested)

    position = _attr(pose, "position", "translation", "xyz")
    if position is not None:
        values = _sequence(position)
        x = _float(_attr(position, "x", default=values[0] if len(values) > 0 else 0.0))
        y = _float(_attr(position, "y", default=values[1] if len(values) > 1 else 0.0))
        z = _float(_attr(position, "z", default=values[2] if len(values) > 2 else 0.0))
    else:
        x = _float(_attr(pose, "x", "tx"))
        y = _float(_attr(pose, "y", "ty"))
        z = _float(_attr(pose, "z", "tz"))

    orientation = _attr(pose, "orientation", "rotation", "quaternion")
    if orientation is not None:
        qx = _float(_attr(orientation, "qx", "x"))
        qy = _float(_attr(orientation, "qy", "y"))
        qz = _float(_attr(orientation, "qz", "z"))
        qw = _float(_attr(orientation, "qw", "w"), 1.0)
    else:
        qx = _float(_attr(pose, "qx"))
        qy = _float(_attr(pose, "qy"))
        qz = _float(_attr(pose, "qz"))
        qw = _float(_attr(pose, "qw"), 1.0)
    return x, y, z, qx, qy, qz, qw


def _trajectory(value: Any) -> np.ndarray | None:
    if value is None:
        return None
    inner = _attr(value, "poses", "trajectory", "points", "trajectory_points")
    if inner is not None and inner is not value:
        value = inner

    try:
        array = np.asarray(value, dtype=np.float64)
    except (TypeError, ValueError):
        array = None
    if array is not None and array.ndim == 2:
        if array.shape[1] >= 3:
            return array[:, :3].copy()
        if array.shape[1] == 2:
            return array.copy()
    if array is not None and array.ndim == 3 and array.shape[1:] == (4, 4):
        return array[:, :3, 3].copy()

    points = []
    for point in _sequence(value):
        x, y, z, *_ = _pose_values(point)
        points.append((x, y, z))
    if not points:
        return None
    return np.asarray(points, dtype=np.float64)


def _version_ids(value: Any) -> dict[str, str]:
    if value is None:
        return {}
    if isinstance(value, Mapping):
        return {str(key): str(item) for key, item in value.items()}
    if isinstance(value, Sequence) and not isinstance(value, (str, bytes, bytearray)):
        result = {}
        for item in value:
            name = _text(_attr(item, "name", "id", "component", "key"))
            version = _text(_attr(item, "version", "value", "commit", "hash"))
            if name:
                result[name] = version or ""
        return result
    if hasattr(value, "items"):
        return {str(key): str(item) for key, item in value.items()}
    return {}


def _active_field(entry: Any) -> tuple[str | None, Any]:
    if hasattr(entry, "WhichOneof"):
        descriptor = getattr(entry, "DESCRIPTOR", None)
        descriptor_oneofs = [
            oneof.name for oneof in getattr(descriptor, "oneofs", [])
        ]
        oneofs = list(dict.fromkeys(("log_entry", "entry", "payload", *descriptor_oneofs)))
        for oneof in oneofs:
            try:
                chosen = entry.WhichOneof(oneof)
            except ValueError:
                chosen = None
            if chosen:
                return chosen, getattr(entry, chosen)

    if hasattr(entry, "ListFields"):
        try:
            fields = entry.ListFields()
        except (TypeError, ValueError):
            fields = []
        if len(fields) == 1:
            return fields[0][0].name, fields[0][1]

    if _attr(entry, "session_metadata", "session") is not None:
        return "rollout_metadata", entry

    actors = _attr(entry, "actor_poses", "poses")
    if _is_sequence(actors) and _attr(entry, "timestamp_us", "timestamp", "time_us") is not None:
        return "actor_poses", entry

    for name in _KNOWN_ENTRY_FIELDS:
        value = _attr(entry, name)
        if value is None:
            continue
        if name == "actor_poses" and _is_sequence(value):
            continue
        return name, value
    return None, entry


def _apply_metadata(rollout: ASLRollout, payload: Any) -> None:
    session = _attr(payload, "session_metadata", "session")
    if session is None:
        session = payload

    rollout.session_uuid = _text(_attr(session, "session_uuid", "session_id"))
    rollout.scene_id = _text(_attr(session, "scene_id", "scene_uuid"))
    rollout.start_timestamp_us = _int(
        _attr(session, "start_timestamp_us", "start_time_us", "start_timestamp")
    )
    rollout.control_timestep_us = _int(
        _attr(session, "control_timestep_us", "control_timestep", "timestep_us")
    )
    rollout.version_ids.update(
        _version_ids(_attr(payload, "version_ids", "versions", "version_info"))
    )
    ground_truth = _attr(
        payload,
        "ego_rig_recorded_ground_truth_trajectory",
        "ego_ground_truth_trajectory",
        "ground_truth_trajectory",
        "ground_truth",
    )
    trajectory = _trajectory(ground_truth)
    if trajectory is not None:
        rollout.ground_truth_trajectory = trajectory


def _apply_actor_poses(rollout: ASLRollout, payload: Any) -> None:
    timestamp_us = _int(_attr(payload, "timestamp_us", "timestamp", "time_us"))
    actors = _attr(payload, "actor_poses", "poses", "actors")
    for actor in _sequence(actors):
        actor_timestamp = _int(
            _attr(actor, "timestamp_us", "timestamp", "time_us")
        )
        if actor_timestamp is None:
            actor_timestamp = timestamp_us
        if actor_timestamp is None:
            raise ASLInputError("ActorPoses entry is missing a timestamp")

        actor_id = _text(_attr(actor, "actor_id", "id", "name"))
        if not actor_id:
            raise ASLInputError("ActorPoses entry is missing actor_id")

        pose = _attr(actor, "actor_pose", "pose", "pose_3d", "transform", default=actor)
        x, y, z, qx, qy, qz, qw = _pose_values(pose)
        label = _text(_attr(actor, "actor_label", "label", "class_name", "type"))
        rollout.actor_poses.append(
            ASLActorPose(
                timestamp_us=actor_timestamp,
                actor_id=actor_id,
                x=x,
                y=y,
                z=z,
                qx=qx,
                qy=qy,
                qz=qz,
                qw=qw,
                actor_label=label,
            )
        )


def parse_alpasim_entries(entries: Iterable[Any]) -> ASLRollout:
    """Map decoded AlpaSim protobuf entries into an :class:`ASLRollout`."""
    rollout = ASLRollout()
    counts: dict[str, int] = defaultdict(int)
    for entry in entries:
        rollout.frame_count += 1
        kind, payload = _active_field(entry)
        if kind is None:
            rollout.unknown_entries += 1
            continue
        counts[kind] += 1
        normalized = kind.lower()
        if normalized in ("rollout_metadata", "metadata"):
            _apply_metadata(rollout, payload)
        elif normalized == "actor_poses":
            _apply_actor_poses(rollout, payload)
    rollout.entry_counts = dict(counts)
    return rollout


def _read_varint(handle) -> tuple[int, int] | None:
    value = 0
    shift = 0
    for index in range(10):
        byte = handle.read(1)
        if not byte:
            if index == 0:
                return None
            raise ASLInputError("truncated varint length prefix")
        part = byte[0]
        value |= (part & 0x7F) << shift
        if not part & 0x80:
            return value, index + 1
        shift += 7
    raise ASLInputError("varint length prefix is too long")


def iter_asl_frames(
    path: str | Path, *, max_frame_bytes: int = DEFAULT_MAX_FRAME_BYTES
) -> Iterable[bytes]:
    """Yield raw size-delimited protobuf frames from an ASL file."""
    with Path(path).open("rb") as handle:
        while True:
            prefix = _read_varint(handle)
            if prefix is None:
                return
            size, _ = prefix
            if size < 0 or size > max_frame_bytes:
                raise ASLInputError(
                    f"invalid ASL frame size {size} (max {max_frame_bytes})"
                )
            payload = handle.read(size)
            if len(payload) != size:
                raise ASLInputError(
                    f"truncated ASL frame: expected {size} bytes, got {len(payload)}"
                )
            yield payload


def inspect_asl(
    path: str | Path, *, max_frame_bytes: int = DEFAULT_MAX_FRAME_BYTES
) -> ASLFrameInfo:
    """Validate ASL framing and return frame counts without decoding protobuf."""
    frame_count = 0
    payload_bytes = 0
    largest = 0
    for payload in iter_asl_frames(path, max_frame_bytes=max_frame_bytes):
        frame_count += 1
        payload_bytes += len(payload)
        largest = max(largest, len(payload))
    return ASLFrameInfo(
        path=str(path),
        frame_count=frame_count,
        payload_bytes=payload_bytes,
        max_frame_bytes=largest,
    )


async def iter_alpasim_entries(path: str | Path) -> AsyncIterator[Any]:
    """Yield decoded entries using the optional upstream AlpaSim package."""
    try:
        from alpasim_grpc.utils.logs import async_read_pb_log
    except ImportError as exc:
        raise ASLInputError(
            "AlpaSim ASL decoding requires the optional alpasim-grpc package; "
            "install it in a separate environment or pass decoded entries to "
            "parse_alpasim_entries()"
        ) from exc

    async for entry in async_read_pb_log(str(path)):
        yield entry


async def read_asl(path: str | Path) -> ASLRollout:
    """Read and map an ASL file with the upstream AlpaSim reader."""
    entries = [entry async for entry in iter_alpasim_entries(path)]
    return parse_alpasim_entries(entries)


def build_asl_report(rollout: ASLRollout) -> dict[str, Any]:
    """Build a JSON-serializable rollout report."""
    tracks = rollout.actor_tracks()
    actors = []
    for actor_id, poses in sorted(tracks.items()):
        labels = sorted({pose.actor_label for pose in poses if pose.actor_label})
        actors.append(
            {
                "actor_id": actor_id,
                "labels": labels,
                "samples": len(poses),
                "first_timestamp_us": poses[0].timestamp_us,
                "last_timestamp_us": poses[-1].timestamp_us,
                "first_position": {
                    "x": poses[0].x,
                    "y": poses[0].y,
                    "z": poses[0].z,
                },
                "last_position": {
                    "x": poses[-1].x,
                    "y": poses[-1].y,
                    "z": poses[-1].z,
                },
            }
        )

    ground_truth = rollout.ground_truth_trajectory
    gt_summary = None
    if ground_truth is not None and len(ground_truth):
        gt_summary = {
            "points": len(ground_truth),
            "dimensions": int(ground_truth.shape[1]),
            "first": ground_truth[0].tolist(),
            "last": ground_truth[-1].tolist(),
        }

    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "generated_at": datetime.now(tz=timezone.utc)
        .isoformat()
        .replace("+00:00", "Z"),
        "session": {
            "session_uuid": rollout.session_uuid,
            "scene_id": rollout.scene_id,
            "start_timestamp_us": rollout.start_timestamp_us,
            "control_timestep_us": rollout.control_timestep_us,
        },
        "version_ids": dict(rollout.version_ids),
        "summary": {
            "frames": rollout.frame_count,
            "actor_poses": len(rollout.actor_poses),
            "actors": len(actors),
            "unknown_entries": rollout.unknown_entries,
            "entry_counts": dict(rollout.entry_counts),
        },
        "actors": actors,
        "ground_truth": gt_summary,
    }


def report_to_markdown(report: Mapping[str, Any]) -> str:
    """Render an ASL report as Markdown."""
    session = report.get("session") or {}
    summary = report.get("summary") or {}
    lines = [
        "# AlpaSim rollout report",
        "",
        f"**Session:** `{session.get('session_uuid') or 'n/a'}`",
        f"**Scene:** `{session.get('scene_id') or 'n/a'}`",
        "",
        "## Summary",
        "",
        f"- Frames: {summary.get('frames', 0)}",
        f"- Actor poses: {summary.get('actor_poses', 0)}",
        f"- Actors: {summary.get('actors', 0)}",
        f"- Unknown entries: {summary.get('unknown_entries', 0)}",
        "",
        "## Actors",
        "",
    ]
    for actor in report.get("actors", []):
        labels = ", ".join(actor.get("labels") or []) or "unknown"
        lines.append(
            f"- `{actor['actor_id']}` ({labels}): {actor['samples']} samples, "
            f"{actor['first_timestamp_us']}–{actor['last_timestamp_us']} µs"
        )
    ground_truth = report.get("ground_truth")
    if ground_truth:
        lines.extend(
            [
                "",
                "## Ground truth",
                "",
                f"- Points: {ground_truth['points']}",
                f"- Dimensions: {ground_truth['dimensions']}",
            ]
        )
    return "\n".join(lines).rstrip() + "\n"


def write_asl_report(
    report: Mapping[str, Any], path: str | Path, *, format: str | None = None
) -> str:
    """Write an ASL report as JSON or Markdown."""
    output = Path(path)
    chosen = (format or output.suffix.lstrip(".") or "json").lower()
    if chosen in ("md", "markdown"):
        output.write_text(report_to_markdown(report), encoding="utf-8")
    elif chosen == "json":
        output.write_text(json.dumps(report, indent=2), encoding="utf-8")
    else:
        raise ASLInputError(
            f"unsupported report format {chosen!r}; use 'json' or 'markdown'"
        )
    return str(output)


__all__ = [
    "REPORT_SCHEMA_VERSION",
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
    "report_to_markdown",
    "write_asl_report",
]
