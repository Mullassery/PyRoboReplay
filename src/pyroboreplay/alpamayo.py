"""Alpamayo run ingestion and forensic trajectory comparison.

NVIDIA Alpamayo produces ego trajectories plus Chain-of-Causation (CoC)
reasoning traces. This module turns those research-model outputs into
PyRoboReplay evidence: predicted-vs-actual trajectory metrics, reasoning
text, and correlations with failures already detected in a mission.

The integration is intentionally a boundary adapter:

* It does not import ``torch`` or ``alpamayo_r1`` and never loads model
  weights, so it works on CPU-only replay machines and in CI.
* It does not vendor, fork, or relicense upstream code. Alpamayo inference
  is Apache-2.0; Alpamayo 1 weights are research/non-commercial.
* Inputs can be a mapping, a ``.json``/``.npz`` artifact produced by an
  offline inference job, or a duck-typed object exposing ``pred_xyz`` and
  ``extra`` (as in the upstream notebook).

Typical use::

    from pyroboreplay.alpamayo import (
        build_alpamayo_report,
        load_alpamayo_run,
        write_alpamayo_report,
    )

    run = load_alpamayo_run("run.json")
    report = build_alpamayo_report(run, mission=mission, ground_truth=ego_future_xyz)
    write_alpamayo_report(report, "alpamayo-report.md")
"""

from __future__ import annotations

import json
from collections.abc import Mapping
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import numpy as np

REPORT_SCHEMA_VERSION = "1.0"
DEFAULT_HORIZON = 64
DEFAULT_DT = 0.1


class AlpamayoInputError(ValueError):
    """Raised when an Alpamayo artifact or trajectory shape is invalid."""


def _as_numpy(value: Any, *, name: str) -> np.ndarray | None:
    if value is None:
        return None
    if hasattr(value, "detach"):
        value = value.detach()
    if hasattr(value, "cpu"):
        value = value.cpu()
    if hasattr(value, "numpy"):
        value = value.numpy()
    try:
        return np.asarray(value, dtype=np.float64)
    except (TypeError, ValueError) as exc:
        raise AlpamayoInputError(
            f"{name} cannot be converted to a numeric array"
        ) from exc


def _normalize_trajectories(
    value: Any,
    *,
    batch_index: int = 0,
    coordinate_dims: tuple[int, ...] = (3,),
) -> np.ndarray:
    array = _as_numpy(value, name="pred_xyz")
    if array is None:
        raise AlpamayoInputError(
            "an Alpamayo artifact must contain pred_xyz/trajectories"
        )

    if array.ndim == 2:
        array = array[None, ...]
    elif array.ndim == 3:
        pass
    elif array.ndim == 4:
        if not 0 <= batch_index < array.shape[0]:
            raise AlpamayoInputError(
                f"batch_index {batch_index} is out of range for batch size {array.shape[0]}"
            )
        array = array[batch_index]
    elif array.ndim == 5:
        if not 0 <= batch_index < array.shape[0]:
            raise AlpamayoInputError(
                f"batch_index {batch_index} is out of range for batch size {array.shape[0]}"
            )
        batch = array[batch_index]
        array = batch.reshape((-1,) + batch.shape[-2:])
    else:
        raise AlpamayoInputError(
            "pred_xyz must have shape (T,3), (S,T,3), (B,S,T,3), or "
            f"(B,sets,S,T,3); got {array.shape}"
        )

    if array.ndim != 3 or array.shape[-1] not in coordinate_dims:
        raise AlpamayoInputError(
            f"pred_xyz must resolve to (samples, horizon, {coordinate_dims}); "
            f"got {array.shape}"
        )
    if array.shape[0] < 1 or array.shape[1] < 1:
        raise AlpamayoInputError(
            "pred_xyz must contain at least one sample and waypoint"
        )
    if not np.all(np.isfinite(array)):
        raise AlpamayoInputError("pred_xyz contains non-finite values")
    return array


def _normalize_rotations(value: Any, *, batch_index: int = 0) -> np.ndarray | None:
    array = _as_numpy(value, name="pred_rot")
    if array is None:
        return None

    if array.ndim == 3 and array.shape[-1] == 9:
        array = array.reshape(array.shape[0], array.shape[1], 3, 3)
    elif array.ndim == 3 and array.shape[-2:] == (3, 3):
        array = array[None, ...]
    elif array.ndim == 4 and array.shape[-2:] == (3, 3):
        pass
    elif array.ndim in (5, 6) and array.shape[-2:] == (3, 3):
        if not 0 <= batch_index < array.shape[0]:
            raise AlpamayoInputError(
                f"batch_index {batch_index} is out of range for batch size {array.shape[0]}"
            )
        batch = array[batch_index]
        array = batch.reshape((-1,) + batch.shape[-3:])
    else:
        raise AlpamayoInputError(
            "pred_rot must have shape (T,3,3), (S,T,3,3), (B,S,T,3,3), or "
            f"(B,sets,S,T,3,3); got {array.shape}"
        )

    if array.ndim != 4 or array.shape[-2:] != (3, 3):
        raise AlpamayoInputError(
            f"pred_rot must resolve to (samples, horizon, 3, 3); got {array.shape}"
        )
    if not np.all(np.isfinite(array)):
        raise AlpamayoInputError("pred_rot contains non-finite values")
    return array


def _flatten_reasoning(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    if isinstance(value, Mapping):
        for key in ("text", "content", "reasoning", "cot", "value"):
            if key in value:
                return _flatten_reasoning(value[key])
        return []
    if isinstance(value, np.ndarray):
        value = value.tolist()
    if isinstance(value, (list, tuple)):
        traces: list[str] = []
        for item in value:
            traces.extend(_flatten_reasoning(item))
        return traces
    return []


def _reasoning_from_extra(extra: Any) -> list[str]:
    if isinstance(extra, Mapping):
        source: Any = None
        for key in ("cot", "reasoning", "reasoning_traces", "chain_of_causation"):
            if key in extra:
                source = extra[key]
                break
    else:
        source = extra
    return _flatten_reasoning(source)


def _timestamp_seconds(value: Any) -> float | None:
    if value is None:
        return None
    if isinstance(value, datetime):
        return value.timestamp()
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        try:
            return datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp()
        except ValueError:
            try:
                return float(value)
            except ValueError:
                return None
    return None


def _timestamp_iso(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, str):
        return value
    seconds = _timestamp_seconds(value)
    if seconds is None:
        return None
    return (
        datetime.fromtimestamp(seconds, tz=timezone.utc)
        .isoformat()
        .replace("+00:00", "Z")
    )


@dataclass(frozen=True)
class AlpamayoTrajectory:
    """One predicted ego trajectory and its reasoning trace."""

    xyz: np.ndarray
    rotation: np.ndarray | None
    reasoning: str | None
    sample_index: int


@dataclass(frozen=True)
class AlpamayoRecord:
    """A normalized Alpamayo inference run."""

    trajectories: tuple[AlpamayoTrajectory, ...]
    reasoning: tuple[str, ...] = ()
    timestamp: Any = None
    metadata: Mapping[str, Any] = field(default_factory=dict)

    @property
    def num_samples(self) -> int:
        return len(self.trajectories)

    def best(self) -> AlpamayoTrajectory:
        return self.trajectories[0]


def _record_from_mapping(
    data: Mapping[str, Any],
    *,
    batch_index: int,
    horizon: int | None,
    timestamp: Any,
    metadata: Mapping[str, Any] | None,
) -> AlpamayoRecord:
    pred_xyz = None
    for key in ("pred_xyz", "trajectories", "trajectory", "predicted_xyz", "xyz"):
        if key in data:
            pred_xyz = data[key]
            break
    if pred_xyz is None:
        raise AlpamayoInputError("Alpamayo artifact is missing pred_xyz/trajectories")

    pred_rot = None
    for key in ("pred_rot", "rotations", "rotation"):
        if key in data:
            pred_rot = data[key]
            break

    extra = data.get("extra")
    if extra is None:
        for key in ("cot", "reasoning", "reasoning_traces", "chain_of_causation"):
            if key in data:
                extra = data[key]
                break

    record_timestamp = timestamp
    if record_timestamp is None:
        for key in ("timestamp", "time", "t"):
            if key in data:
                record_timestamp = data[key]
                break

    record_metadata: dict[str, Any] = {"source": "alpamayo"}
    if metadata:
        record_metadata.update(metadata)
    for key in ("clip_id", "scene_id", "model", "model_version", "run_id"):
        if key in data:
            record_metadata[key] = data[key]

    return _build_record(
        pred_xyz,
        pred_rot,
        extra,
        batch_index=batch_index,
        horizon=horizon,
        timestamp=record_timestamp,
        metadata=record_metadata,
    )


def _build_record(
    pred_xyz: Any,
    pred_rot: Any = None,
    extra: Any = None,
    *,
    batch_index: int = 0,
    horizon: int | None = DEFAULT_HORIZON,
    timestamp: Any = None,
    metadata: Mapping[str, Any] | None = None,
) -> AlpamayoRecord:
    trajectories = _normalize_trajectories(pred_xyz, batch_index=batch_index)
    rotations = _normalize_rotations(pred_rot, batch_index=batch_index)

    if rotations is not None and rotations.shape[:2] != trajectories.shape[:2]:
        raise AlpamayoInputError(
            "pred_rot and pred_xyz must have matching sample and horizon dimensions; "
            f"got {rotations.shape[:2]} and {trajectories.shape[:2]}"
        )
    if horizon is not None and trajectories.shape[1] != horizon:
        raise AlpamayoInputError(
            f"expected {horizon} waypoints, got {trajectories.shape[1]}"
        )

    traces = _reasoning_from_extra(extra)
    if len(traces) < trajectories.shape[0]:
        per_sample = traces + [None] * (trajectories.shape[0] - len(traces))
    else:
        per_sample = traces[: trajectories.shape[0]]

    samples = tuple(
        AlpamayoTrajectory(
            xyz=trajectories[index].copy(),
            rotation=None if rotations is None else rotations[index].copy(),
            reasoning=per_sample[index],
            sample_index=index,
        )
        for index in range(trajectories.shape[0])
    )

    record_metadata: dict[str, Any] = {
        "horizon": int(trajectories.shape[1]),
        "dt": DEFAULT_DT,
        "num_samples": int(trajectories.shape[0]),
    }
    if metadata:
        record_metadata.update(metadata)

    return AlpamayoRecord(
        trajectories=samples,
        reasoning=tuple(traces),
        timestamp=timestamp,
        metadata=record_metadata,
    )


def parse_alpamayo_output(
    data: Any,
    *,
    batch_index: int = 0,
    horizon: int | None = DEFAULT_HORIZON,
    timestamp: Any = None,
    metadata: Mapping[str, Any] | None = None,
) -> AlpamayoRecord:
    """Normalize an Alpamayo inference result.

    ``data`` may be a mapping, a ``.json``/``.npz`` path, or an object with
    ``pred_xyz`` and optional ``pred_rot``/``extra`` attributes.
    """
    if isinstance(data, (str, Path)):
        path = Path(data)
        if not path.is_file():
            raise FileNotFoundError(f"Alpamayo artifact not found: {path}")
        if path.suffix == ".npz":
            with np.load(path, allow_pickle=False) as loaded:
                payload = {key: loaded[key] for key in loaded.files}
        elif path.suffix == ".npy":
            payload = {"pred_xyz": np.load(path, allow_pickle=False)}
        elif path.suffix in (".json", ".jsonl"):
            if path.suffix == ".jsonl":
                lines = [line for line in path.read_text().splitlines() if line.strip()]
                if len(lines) != 1:
                    raise AlpamayoInputError(
                        "JSONL input must contain exactly one record; use load_alpamayo_runs "
                        "for multi-record files"
                    )
                payload = json.loads(lines[0])
            else:
                payload = json.loads(path.read_text())
        else:
            raise AlpamayoInputError(
                f"unsupported Alpamayo artifact type {path.suffix!r}; use .json, .jsonl, .npz, or .npy"
            )
        return parse_alpamayo_output(
            payload,
            batch_index=batch_index,
            horizon=horizon,
            timestamp=timestamp,
            metadata=metadata,
        )

    if isinstance(data, Mapping):
        return _record_from_mapping(
            data,
            batch_index=batch_index,
            horizon=horizon,
            timestamp=timestamp,
            metadata=metadata,
        )

    pred_xyz = getattr(data, "pred_xyz", None)
    if pred_xyz is not None:
        extra = getattr(data, "extra", None)
        if extra is None:
            extra = getattr(data, "cot", None)
        return _build_record(
            pred_xyz,
            getattr(data, "pred_rot", None),
            extra,
            batch_index=batch_index,
            horizon=horizon,
            timestamp=timestamp,
            metadata=metadata,
        )

    raise AlpamayoInputError(
        "unsupported Alpamayo input; expected a mapping, path, or object "
        "with pred_xyz/pred_rot/extra"
    )


def load_alpamayo_run(
    source: Any,
    *,
    batch_index: int = 0,
    horizon: int | None = DEFAULT_HORIZON,
    timestamp: Any = None,
    metadata: Mapping[str, Any] | None = None,
) -> AlpamayoRecord:
    """Alias for :func:`parse_alpamayo_output` with a task-oriented name."""
    return parse_alpamayo_output(
        source,
        batch_index=batch_index,
        horizon=horizon,
        timestamp=timestamp,
        metadata=metadata,
    )


def load_alpamayo_runs(source: Any, **kwargs: Any) -> list[AlpamayoRecord]:
    """Load one or many Alpamayo records from a JSON list/JSONL/NPZ collection."""
    if isinstance(source, (str, Path)):
        path = Path(source)
        if path.suffix == ".jsonl":
            return [
                parse_alpamayo_output(json.loads(line), **kwargs)
                for line in path.read_text().splitlines()
                if line.strip()
            ]
        if path.suffix == ".json":
            payload = json.loads(path.read_text())
            if isinstance(payload, list):
                return [parse_alpamayo_output(item, **kwargs) for item in payload]
            return [parse_alpamayo_output(payload, **kwargs)]
        if path.suffix == ".npz":
            return [parse_alpamayo_output(path, **kwargs)]

    if isinstance(source, (list, tuple)):
        return [parse_alpamayo_output(item, **kwargs) for item in source]
    return [parse_alpamayo_output(source, **kwargs)]


def compare_trajectories(predicted: Any, ground_truth: Any) -> dict[str, Any]:
    """Compute minADE/minFDE and per-sample errors for Alpamayo trajectories."""
    pred = _normalize_trajectories(predicted, coordinate_dims=(2, 3))
    target = _as_numpy(ground_truth, name="ground_truth")
    if target is None:
        raise AlpamayoInputError("ground_truth is required")
    if target.ndim != 2 or target.shape[1] not in (2, 3):
        raise AlpamayoInputError(
            f"ground_truth must have shape (T,2) or (T,3); got {target.shape}"
        )
    if target.shape[1] > pred.shape[-1]:
        raise AlpamayoInputError(
            "ground_truth has more dimensions than predicted trajectories"
        )
    if not np.all(np.isfinite(target)):
        raise AlpamayoInputError("ground_truth contains non-finite values")

    horizon = min(pred.shape[1], target.shape[0])
    if horizon < 1:
        raise AlpamayoInputError("trajectories must share at least one waypoint")

    predicted_xyz = pred[:, :horizon, : target.shape[1]]
    target_xyz = target[:horizon]
    per_timestep = np.linalg.norm(predicted_xyz - target_xyz[None, :, :], axis=-1)
    ade = per_timestep.mean(axis=1)
    fde = per_timestep[:, -1]
    best = int(np.argmin(ade))

    return {
        "min_ade": float(ade[best]),
        "min_fde": float(fde[best]),
        "best_sample": best,
        "ade": [float(value) for value in ade],
        "fde": [float(value) for value in fde],
        "per_timestep": per_timestep.tolist(),
        "horizon": int(horizon),
    }


def _failure_summary(failure: Any) -> dict[str, Any]:
    def call(name: str, default: Any = None) -> Any:
        getter = getattr(failure, name, None)
        if getter is None:
            return default
        try:
            return getter()
        except TypeError:
            return getter

    timestamp = call("get_timestamp")
    return {
        "timestamp": _timestamp_iso(timestamp),
        "timestamp_seconds": _timestamp_seconds(timestamp),
        "type": call("get_failure_type", "unknown"),
        "severity": call("get_severity", "unknown"),
        "description": call("get_description", ""),
        "confidence": call("get_confidence"),
    }


def build_alpamayo_report(
    runs: Any,
    *,
    mission: Any = None,
    ground_truth: Any = None,
    failure_tolerance_s: float = 1.0,
) -> dict[str, Any]:
    """Build a JSON-serializable forensic report from Alpamayo runs.

    Args:
        runs: One record, a list of records, or raw inputs accepted by
            :func:`load_alpamayo_runs`.
        mission: Optional PyRoboReplay ``Mission`` (or compatible object) used
            for mission metadata and failure correlation.
        ground_truth: Optional ``(T,2)``/``(T,3)`` actual ego trajectory. When
            supplied, every run gets minADE/minFDE metrics.
        failure_tolerance_s: Match a reasoning trace to a failure within this
            many seconds of the run timestamp.
    """
    if isinstance(runs, AlpamayoRecord):
        records = [runs]
    elif isinstance(runs, (list, tuple)) and all(
        isinstance(item, AlpamayoRecord) for item in runs
    ):
        records = list(runs)
    else:
        records = load_alpamayo_runs(runs)

    failures = []
    mission_summary = None
    if mission is not None:
        mission_id = getattr(mission, "mission_id", None)
        mission_name = getattr(mission, "name", None)
        mission_summary = {
            "id": mission_id() if callable(mission_id) else None,
            "name": mission_name() if callable(mission_name) else None,
        }
        detect = getattr(mission, "detect_failures", None)
        if callable(detect):
            failures = [_failure_summary(failure) for failure in detect()]

    record_reports = []
    comparison_count = 0
    reasoning_count = 0
    correlated_count = 0
    ade_values = []
    fde_values = []

    for record in records:
        comparison = None
        if ground_truth is not None:
            comparison = compare_trajectories(
                np.stack([sample.xyz for sample in record.trajectories]),
                ground_truth,
            )
            comparison_count += 1
            ade_values.append(comparison["min_ade"])
            fde_values.append(comparison["min_fde"])

        reasoning_count += len(record.reasoning)
        record_seconds = _timestamp_seconds(record.timestamp)
        correlated = []
        if record_seconds is not None:
            for failure in failures:
                failure_seconds = failure.get("timestamp_seconds")
                if failure_seconds is None:
                    continue
                if abs(record_seconds - failure_seconds) <= failure_tolerance_s:
                    correlated.append(failure)

        if correlated:
            correlated_count += 1

        record_reports.append(
            {
                "timestamp": _timestamp_iso(record.timestamp),
                "timestamp_seconds": record_seconds,
                "metadata": dict(record.metadata),
                "num_samples": record.num_samples,
                "reasoning": list(record.reasoning),
                "comparison": comparison,
                "correlated_failures": correlated,
            }
        )

    summary = {
        "records": len(record_reports),
        "comparisons": comparison_count,
        "reasoning_traces": reasoning_count,
        "correlated_records": correlated_count,
        "mean_min_ade": float(np.mean(ade_values)) if ade_values else None,
        "mean_min_fde": float(np.mean(fde_values)) if fde_values else None,
    }

    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "generated_at": datetime.now(tz=timezone.utc)
        .isoformat()
        .replace("+00:00", "Z"),
        "mission": mission_summary,
        "summary": summary,
        "records": record_reports,
        "failures": failures,
    }


def report_to_markdown(report: Mapping[str, Any]) -> str:
    """Render an Alpamayo report as a compact Markdown document."""
    summary = report.get("summary", {})
    mission = report.get("mission") or {}
    lines = ["# Alpamayo forensic report", ""]

    if mission:
        lines.append(
            f"**Mission:** {mission.get('name') or 'n/a'} "
            f"(`{mission.get('id') or 'n/a'}`)"
        )
        lines.append("")

    lines.extend(
        [
            "## Summary",
            "",
            f"- Records: {summary.get('records', 0)}",
            f"- Trajectory comparisons: {summary.get('comparisons', 0)}",
            f"- Reasoning traces: {summary.get('reasoning_traces', 0)}",
            f"- Records correlated with failures: {summary.get('correlated_records', 0)}",
        ]
    )
    if summary.get("mean_min_ade") is not None:
        lines.append(f"- Mean minADE: {summary['mean_min_ade']:.3f} m")
    if summary.get("mean_min_fde") is not None:
        lines.append(f"- Mean minFDE: {summary['mean_min_fde']:.3f} m")
    lines.append("")

    for index, record in enumerate(report.get("records", []), start=1):
        lines.append(f"## Run {index}")
        lines.append("")
        if record.get("timestamp"):
            lines.append(f"- Timestamp: `{record['timestamp']}`")
        lines.append(f"- Samples: {record.get('num_samples', 0)}")
        comparison = record.get("comparison")
        if comparison:
            lines.append(
                f"- minADE: {comparison['min_ade']:.3f} m; "
                f"minFDE: {comparison['min_fde']:.3f} m "
                f"(sample {comparison['best_sample']})"
            )
        correlated = record.get("correlated_failures") or []
        if correlated:
            lines.append("- Correlated failures:")
            for failure in correlated:
                lines.append(
                    f"  - {failure.get('type')} ({failure.get('severity')}): "
                    f"{failure.get('description')}"
                )
        reasoning = record.get("reasoning") or []
        if reasoning:
            lines.append("- Reasoning:")
            for trace in reasoning:
                lines.append(f"  - {trace}")
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def write_alpamayo_report(
    report: Mapping[str, Any],
    path: str | Path,
    *,
    format: str | None = None,
) -> str:
    """Write a report as JSON or Markdown and return the output path."""
    output = Path(path)
    chosen = (format or output.suffix.lstrip(".") or "json").lower()
    if chosen in ("md", "markdown"):
        output.write_text(report_to_markdown(report), encoding="utf-8")
    elif chosen == "json":
        output.write_text(json.dumps(report, indent=2), encoding="utf-8")
    else:
        raise AlpamayoInputError(
            f"unsupported report format {chosen!r}; use 'json' or 'markdown'"
        )
    return str(output)


__all__ = [
    "REPORT_SCHEMA_VERSION",
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
