"""Tests for AlpaSim ASL framing, entry mapping, and reporting."""

from __future__ import annotations

import asyncio
import json
from types import SimpleNamespace

import pytest

from pyroboreplay.asl import (
    ASLInputError,
    build_asl_report,
    inspect_asl,
    parse_alpasim_entries,
    read_asl,
    report_to_markdown,
    write_asl_report,
)


def _varint(value: int) -> bytes:
    out = bytearray()
    while True:
        part = value & 0x7F
        value >>= 7
        out.append(part | 0x80 if value else part)
        if not value:
            return bytes(out)


def _metadata_entry():
    session = SimpleNamespace(
        session_uuid="session-1",
        scene_id="scene-1",
        start_timestamp_us=1_000_000,
        control_timestep_us=100_000,
    )
    metadata = SimpleNamespace(
        session_metadata=session,
        version_ids=[
            SimpleNamespace(name="driver", version="v1"),
            SimpleNamespace(name="renderer", version="v2"),
        ],
        ego_rig_recorded_ground_truth_trajectory=[
            [0.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
        ],
    )
    return SimpleNamespace(rollout_metadata=metadata)


def _actor_poses_entry():
    ego = SimpleNamespace(
        actor_id="EGO",
        actor_label="ego",
        actor_pose=SimpleNamespace(
            position=SimpleNamespace(x=1.0, y=2.0, z=3.0),
            orientation=SimpleNamespace(x=0.0, y=0.0, z=0.0, w=1.0),
        ),
    )
    actor = SimpleNamespace(actor_id=7, actor_pose=[4.0, 5.0, 6.0, 0.0, 0.0, 0.0, 1.0])
    return SimpleNamespace(
        actor_poses=SimpleNamespace(timestamp_us=1_100_000, actor_poses=[ego, actor])
    )


def test_parse_metadata_and_actor_poses():
    rollout = parse_alpasim_entries(
        [_metadata_entry(), _actor_poses_entry(), object()]
    )

    assert rollout.frame_count == 3
    assert rollout.session_uuid == "session-1"
    assert rollout.scene_id == "scene-1"
    assert rollout.start_timestamp_us == 1_000_000
    assert rollout.control_timestep_us == 100_000
    assert rollout.version_ids == {"driver": "v1", "renderer": "v2"}
    assert rollout.ground_truth_trajectory is not None
    assert rollout.ground_truth_trajectory.shape == (2, 3)
    assert len(rollout.actor_poses) == 2
    assert rollout.actor_poses[0].actor_id == "EGO"
    assert rollout.actor_poses[0].actor_label == "ego"
    assert rollout.actor_poses[0].timestamp_us == 1_100_000
    assert rollout.actor_poses[1].actor_id == "7"
    assert rollout.actor_poses[1].x == 4.0
    assert rollout.unknown_entries == 1
    assert rollout.entry_counts == {"rollout_metadata": 1, "actor_poses": 1}


def test_parse_oneof_wrapper_entry():
    metadata = _metadata_entry().rollout_metadata
    entry = SimpleNamespace(rollout_metadata=metadata)
    entry.DESCRIPTOR = SimpleNamespace(oneofs=[SimpleNamespace(name="message")])
    entry.WhichOneof = lambda name: (
        "rollout_metadata" if name == "message" else None
    )

    rollout = parse_alpasim_entries([entry])

    assert rollout.session_uuid == "session-1"
    assert rollout.entry_counts == {"rollout_metadata": 1}


def test_parse_direct_actor_poses_message():
    direct = SimpleNamespace(
        timestamp_us=2_000_000,
        actor_poses=[
            SimpleNamespace(
                actor_id="EGO",
                actor_pose=SimpleNamespace(
                    position=SimpleNamespace(x=1.0, y=0.0, z=0.0),
                    orientation=SimpleNamespace(x=0.0, y=0.0, z=0.0, w=1.0),
                ),
            )
        ],
    )

    rollout = parse_alpasim_entries([direct])

    assert len(rollout.actor_poses) == 1
    assert rollout.actor_poses[0].actor_id == "EGO"
    assert rollout.actor_poses[0].timestamp_us == 2_000_000


def test_actor_pose_without_timestamp_fails():
    entry = SimpleNamespace(
        actor_poses=SimpleNamespace(
            actor_poses=[SimpleNamespace(actor_id="EGO", actor_pose=[0, 0, 0])]
        )
    )

    with pytest.raises(ASLInputError):
        parse_alpasim_entries([entry])


def test_build_report_summarizes_actors_and_ground_truth():
    rollout = parse_alpasim_entries([_metadata_entry(), _actor_poses_entry()])

    report = build_asl_report(rollout)

    assert report["schema_version"] == "1.0"
    assert report["session"]["session_uuid"] == "session-1"
    assert report["summary"]["actors"] == 2
    assert report["summary"]["actor_poses"] == 2
    assert report["ground_truth"]["points"] == 2
    assert [actor["actor_id"] for actor in report["actors"]] == ["7", "EGO"]
    assert report["actors"][1]["labels"] == ["ego"]


def test_write_report_json_and_markdown(tmp_path):
    report = build_asl_report(
        parse_alpasim_entries([_metadata_entry(), _actor_poses_entry()])
    )

    json_path = tmp_path / "rollout.json"
    assert write_asl_report(report, json_path) == str(json_path)
    loaded = json.loads(json_path.read_text())
    assert loaded["summary"]["frames"] == 2

    md_path = tmp_path / "rollout.md"
    assert write_asl_report(report, md_path) == str(md_path)
    text = md_path.read_text()
    assert "# AlpaSim rollout report" in text
    assert "`EGO`" in text
    assert report_to_markdown(report) == text


def test_inspect_asl_counts_frames_and_bytes(tmp_path):
    path = tmp_path / "rollout.asl"
    frames = [b"abc", b"defg"]
    path.write_bytes(b"".join(_varint(len(frame)) + frame for frame in frames))

    info = inspect_asl(path)

    assert info.frame_count == 2
    assert info.payload_bytes == 7
    assert info.max_frame_bytes == 4


def test_inspect_asl_rejects_truncated_frame(tmp_path):
    path = tmp_path / "truncated.asl"
    path.write_bytes(_varint(10) + b"abc")

    with pytest.raises(ASLInputError, match="truncated ASL frame"):
        inspect_asl(path)


def test_inspect_asl_rejects_oversized_frame(tmp_path):
    path = tmp_path / "oversized.asl"
    path.write_bytes(_varint(100) + b"x" * 100)

    with pytest.raises(ASLInputError, match="invalid ASL frame size"):
        inspect_asl(path, max_frame_bytes=10)


def test_read_asl_requires_optional_upstream_package(tmp_path):
    try:
        import alpasim_grpc  # noqa: F401
    except ImportError:
        path = tmp_path / "empty.asl"
        path.write_bytes(b"")
        with pytest.raises(ASLInputError, match="alpasim-grpc"):
            asyncio.run(read_asl(path))


def test_inspect_asl_many_frames(tmp_path):
    path = tmp_path / "many.asl"
    with path.open("wb") as handle:
        for index in range(5000):
            payload = f"frame-{index}".encode()
            handle.write(_varint(len(payload)) + payload)

    info = inspect_asl(path)

    assert info.frame_count == 5000
    assert info.max_frame_bytes == len("frame-4999")
    assert info.payload_bytes > 0
