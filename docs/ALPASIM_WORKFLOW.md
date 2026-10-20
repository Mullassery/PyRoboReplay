# Alpamayo and AlpaSim Replay Workflow

Status: **Alpamayo ingestion implemented; AlpaSim ASL ingestion planned.**

PyRoboReplay turns Alpamayo artifacts into trajectory evidence on CPU-only machines. AlpaSim is a separate closed-loop simulator that writes `.asl` protobuf logs; its adapter is not implemented yet.

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

### Planned PyRoboReplay adapter

The planned boundary adapter will:

1. Accept an `.asl` path or an iterable of decoded protobuf entries.
2. Extract ego poses, actor poses, timestamps, and rollout metadata.
3. Build the existing Alpamayo-style run/report structures.
4. Keep AlpaSim and gRPC dependencies out of PyRoboReplay.
5. Fail clearly on truncated or unsupported logs.

It will not vendor AlpaSim, infer safety outcomes from actor poses alone, or require a GPU.

## Validation

CPU-only Alpamayo tests:

```bash
pytest tests -k alpamayo
```

AlpaSim validation requires a Linux host with Docker. Record a short rollout, verify message counts, timestamps, actor IDs, and ego pose continuity, then replay the extracted run through the Alpamayo report path.

## Licensing

- Alpamayo source: Apache-2.0; weights are non-commercial unless separately licensed.
- AlpaSim source: Apache-2.0.
- Do not redistribute Alpamayo weights or vendor AlpaSim's gRPC stack.

Portfolio-level details are in `reports/NVIDIA_INTEGRATION_WORKFLOW.md`.
