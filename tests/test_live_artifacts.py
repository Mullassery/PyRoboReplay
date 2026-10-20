"""Opt-in checks against real Alpamayo and AlpaSim artifacts."""

from __future__ import annotations

import asyncio
import os
from pathlib import Path

import pytest

from pyroboreplay.alpamayo import build_alpamayo_report, load_alpamayo_run
from pyroboreplay.asl import build_asl_report, read_asl

ALPASIM_LOG = os.environ.get("ALPASIM_LOG")
ALPAMAYO_ARTIFACT = os.environ.get("ALPAMAYO_ARTIFACT")


@pytest.mark.skipif(not ALPASIM_LOG, reason="set ALPASIM_LOG to a real .asl file")
def test_live_alpasim_rollout():
    pytest.importorskip("alpasim_grpc")
    path = Path(ALPASIM_LOG)
    assert path.is_file()

    rollout = asyncio.run(read_asl(path))
    report = build_asl_report(rollout)

    assert rollout.frame_count > 0
    assert report["summary"]["frames"] == rollout.frame_count


@pytest.mark.skipif(
    not ALPAMAYO_ARTIFACT, reason="set ALPAMAYO_ARTIFACT to a real run file"
)
def test_live_alpamayo_artifact():
    path = Path(ALPAMAYO_ARTIFACT)
    assert path.is_file()

    record = load_alpamayo_run(path)
    report = build_alpamayo_report(record)

    assert record.num_samples >= 1
    assert record.metadata["horizon"] == 64
    assert all(sample.xyz.shape == (64, 3) for sample in record.trajectories)
    assert report["summary"]["records"] == 1
