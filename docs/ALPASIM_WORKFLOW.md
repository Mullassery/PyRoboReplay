# Alpamayo and AlpaSim Replay Workflow

Status: **Alpamayo ingestion and AlpaSim ASL ingestion implemented.**

PyRoboReplay turns Alpamayo artifacts into trajectory evidence on CPU-only machines. AlpaSim is a separate closed-loop simulator that writes `.asl` protobuf logs; PyRoboReplay now provides an optional, CPU-only boundary adapter for those logs.

## Alpamayo ingestion

Alpamayo inference produces `pred_xyz`, optional `pred_rot`, and Chain-of-Causation text. Save them as JSON or NPZ, then ingest:

```python
from pyroboreplay.alpamayo import (
    build_alpamayo_report,
    load_alpamayo_run,
    report_to_markdown,
    write_alpamayo_report,
)

run = load_alpamayo_run("alpamayo-run.json")
report = build_alpamayo_report(
    run,
    mission=mission,
    ground_truth=ego_future_xyz,
)
write_alpamayo_report(report, "alpamayo-report.md")
print(report_to_markdown(report))
```

Compare trajectories directly:

```python
from pyroboreplay.alpamayo import compare_trajectories

metrics = compare_trajectories(pred_xyz, ground_truth_xyz)
```

The adapter accepts `(T,3)`, `(S,T,3)`, `(B,S,T,3)`, and `(B,sets,S,T,3)` trajectories with `T = 64` and `dt = 0.1 s`. It does not import Torch or Alpamayo and never loads weights.

Synthetic Alpamayo 1.5 and 2 contract fixtures live in `tests/fixtures/alpamayo/`. They exercise the 64-waypoint/10 Hz contract, `model_version` provenance, and Chain-of-Causation ingestion without model weights.

## AlpaSim ASL workflow

Run AlpaSim upstream, in its own Docker Compose environment:

```bash
git clone <ALPASIM_REPO>
cd <ALPASIM_REPO>
docker compose up
```

Each rollout writes a size-delimited protobuf `.asl` log. Read it with AlpaSim's own reader:

```python
from alpasim_grpc.utils.logs import async_read_pb_log

async for entry in async_read_pb_log("rollout.asl"):
    ...
```

Relevant message families:

- `RolloutMetadata`
- `ActorPoses`
- microservice request/response entries

### PyRoboReplay adapter

`pyroboreplay.asl` provides:

- `inspect_asl()`: validates size-delimited framing and counts frames without decoding protobuf.
- `parse_alpasim_entries()`: maps decoded entries into `ASLRollout`, actor poses, provenance, and ground truth.
- `read_asl()`: async ingestion through AlpaSim's own `async_read_pb_log` reader.
- `build_asl_report()` / `write_asl_report()`: JSON or Markdown reports.

```python
import asyncio

from pyroboreplay.asl import (
    build_asl_report,
    inspect_asl,
    read_asl,
    write_asl_report,
)

info = inspect_asl("rollout.asl")
rollout = asyncio.run(read_asl("rollout.asl"))
report = build_asl_report(rollout)
write_asl_report(report, "alpasim-report.json")
```

The upstream `alpasim_grpc` package is imported lazily by `read_asl()`; it is not a hard PyRoboReplay dependency. If the package is unavailable, pass decoded entries to `parse_alpasim_entries()` or run the reader in an environment where AlpaSim is installed. The adapter does not vendor AlpaSim, load model weights, require a GPU, or infer safety outcomes from actor poses alone. Microservice request/response entries are counted but not interpreted yet.

## Validation

CPU-only tests:

```bash
pytest tests -k alpamayo
pytest tests/test_asl_integration.py
```

The ASL tests use synthetic protobuf-like entries and framed bytes; they do not need AlpaSim, gRPC, Docker, or a GPU. Live validation still requires a Linux host with Docker and an AlpaSim rollout. Record a short rollout, verify message counts, timestamps, actor IDs, and ego pose continuity, then inspect the generated JSON report.

## Licensing

- Alpamayo source: Apache-2.0; weights are non-commercial unless separately licensed.
- AlpaSim source: Apache-2.0.
- Do not redistribute Alpamayo weights or vendor AlpaSim's gRPC stack.

Portfolio-level details are in `reports/NVIDIA_INTEGRATION_WORKFLOW.md`.
