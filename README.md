# PyRoboReplay

Forensic debugging and mission-replay platform for autonomous robots: replay ROS 2 missions, run causal / root-cause analysis on failures, and perform multispectral (RGB + thermal) forensics. Rust core with Python bindings.

[![CI](https://github.com/Mullassery/PyRoboReplay/actions/workflows/ci.yml/badge.svg)](https://github.com/Mullassery/PyRoboReplay/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/pyroboreplay)](https://pypi.org/project/pyroboreplay/)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue.svg)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

## What it is

PyRoboReplay ingests recorded robot missions (ROS 2 bags) into a unified timeline and answers *why* and *what-if* questions rather than only *where* and *what*: which events preceded a failure, what the perception stack missed, and what the robot should have detected. The heavy lifting is a Rust core (`cdylib` + `rlib`) built with PyO3, exposed to Python through `maturin`. The supported entry points are the Python API and a Rust CLI with `replay`, `compare`, `analyze`, `list`, `terrain`, `forensics`, `nav2`, `session`, and `export`.

## Features

**Exposed via CLI / Python (verified, tested):**
- **Mission replay** — interactive timeline scrubber with keyboard controls (`replay`).
- **Reality-gap analysis** — probabilistic mismatch scoring, severity classification, evidence aggregation, JSON/text output (`analyze --detect-gaps`).
- **Failure detection and diagnosis** — `Mission.detect_failures()`, `Mission.analyze_failure()` with hypotheses, confidence, and prioritized actions.
- **Mission comparison** — side-by-side diff of two bags (`compare`).
- **Topic listing** — inspect available topics in a bag (`list`).
- **Real ROS 2 CDR decoding** — standard `sensor_msgs` / `nav_msgs` / `geometry_msgs` binary layouts are decoded from real message bytes (`src/adapters/cdr.rs`); pose `confidence` is derived from the real position-covariance trace, so localization-loss detection fires on real data.
- **Retrospective detection gaps (Phase 12)** — ingest externally generated DINO detections and compare them with the robot's YOLO detections (`forensics --dino`, `analyze_detection_gaps()`).
- **RGB + thermal fusion (Phase 13)** — ingest RGB detections and a thermal frame, then fuse them (`forensics --rgb --thermal`, `fuse_rgb_thermal()`).
- **Navigation sessions (Phase 14)** — build a unified timeline from a recorded ROS 2 bag plus optional Linux logs and Nav2 exports (`session`, `build_navigation_session()`).
- **Nav2 root-cause heuristics (Phase 15)** — run localization/planner/costmap/obstacle/semantic analyzers and classification on a measurements document (`nav2`, `analyze_nav2_diagnostics()`).
- **Recorded-mission exports** — native MCAP (`export --format mcap`) plus optional Rerun `.rrd` export through the Python API or `pyroboreplay-export`.
- **PyTerrainMap grid interop** — read the documented `pyterrainmap-gridmap` v1 JSON export directly (`terrain`, `load_pyterrainmap_grid()`).
- **Phase 16-20 JSON operation bridge** — six real Rust operations (causal graph, counterfactual query, fleet learning, temporal patterns, trend detection, decision tree) via `ops()` / `run_op()`; see [Advanced operations](docs/ADVANCED_OPS.md).

**Implemented in the Rust core, not yet wired to CLI or Python (library API only):**
- Persistent world knowledge (entities, locations, facts across missions) and spatial grounding.
- Multi-mission / longitudinal learning.
- Pluggable detection backends (YOLO / SAM / template fallback). Phase 12 accepts external DINO detections; it does not run a model itself.

## Who it's for / Use cases

- **Warehouse and logistics fleets** — reconstruct why a run failed, find missed detections and coverage gaps, compare two strategy revisions.
- **Safety and compliance review** — produce auditable, saved JSON findings for an incident (`analyze --detect-gaps --format json --output report.json`).
- **Perception R&D** — compare perception strategies over matched missions and quantify regressions.
- **Field autonomy teams** — investigate localization loss, planner oscillation, and sensor disagreement from recorded missions.

## What it is not

- **Not a real-time monitoring or fleet dashboard.** It works on recorded missions, not live streams. The `streaming` modules are internal building blocks, not a supported live fleet product.
- **Phases 16-20 are not typed Python APIs.** Six implemented Rust operations are reachable through the JSON-only `ops()` / `run_op()` bridge ([Advanced operations](docs/ADVANCED_OPS.md)); the rest of those modules, including decision reconstruction, clustering, scenario analysis, and rule extraction, remain Rust-library-only.
- **Phase 12 does not run DINO/SAM models.** It ingests detections produced by an external open-vocabulary run and performs gap analysis; the in-crate `run_dino_detection` method remains a deterministic test stub.
- **PyTerrainMap interop is file-based, not a runtime dependency.** PyRoboReplay reads the documented `pyterrainmap-gridmap` v1 JSON export; there is still no Cargo/pip dependency, H3 store integration, or ROS bridge. `src/core/pyterrain_bridge.rs` remains self-contained terrain-modeling logic.
- **Not published to crates.io** — build the Rust crate from source if you need it directly.
- **Not a certified or safety-verified system.**

## Project status

Beta (Development Status 4). Current source version is **2.12.1**; the latest PyPI release is also **2.12.1** (published 2026-10-20) and includes the Phase 16-20 operation bridge. The CLI and Python API are the tested, supported surface. 869 `cargo test --lib` unit tests pass plus public-surface Python tests (including 10 for the Phase 16-20 bridge) and 7 real-fixture ROS 2 CDR tests. `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings` are clean (re-verified 2026-10-20). See [ROADMAP_HONEST.md](ROADMAP_HONEST.md) for the implemented/partial/missing breakdown.

## Installation

> **Packaging:** the 2.12.x wheel ships both the `pyroboreplay` and
> `pyroboreplay-export` console scripts and declares `numpy`, so installing
> the package is enough for both the Python API and the CLI.

> **Wheel coverage:** PyPI 2.12.x publishes a macOS arm64 wheel plus an sdist;
> the sdist builds on other platforms. Release CI is configured to build and
> smoke-test Linux x86_64/aarch64, macOS arm64/x86_64, and Windows x86_64 for
> Python 3.10–3.13, and those wheels are produced when the release workflow's
> upload job runs. See the [platform matrix](docs/INSTALL.md#platform-matrix).

```bash
pip install pyroboreplay

# or with uv
uv pip install pyroboreplay

# verify
pyroboreplay --version
```

Requires Python 3.10+. From source (needs a Rust toolchain):

```bash
git clone https://github.com/Mullassery/PyRoboReplay.git
cd pyroboreplay
pip install maturin
maturin develop --release      # install the Python package
# or, for the Rust binary/crate only:
cargo build --release
```

## Quick start

```bash
# Interactive timeline scrubber
pyroboreplay replay mission.bag

# Reality-gap analysis, saved as JSON
pyroboreplay analyze mission.bag --detect-gaps --detail --format json --output investigation.json
```

```python
from pyroboreplay import Mission

mission = Mission.from_ros_bag("warehouse.bag")

failures = mission.detect_failures()
print(f"Failures detected: {len(failures)}")

analysis = mission.analyze_failure(timestamp=1234567890.0)
print(f"Root cause: {analysis.get_primary_hypothesis()}")
print(f"Confidence: {analysis.get_diagnostic_confidence():.0%}")

for action in mission.recommend_actions(timestamp=1234567890.0):
    print(f"[{action.get_priority()}] {action.get_description()}")
```

Interactive replay keys: **Space** play/pause, **n / →** next event, **p / ←** previous event, **↑ / ↓** speed, **?** help, **q / Esc** quit.

## MCAP export

Export a mission timeline to the indexed MCAP robotics log format. The native
CLI needs no extra dependency; the Python helper needs `mcap`.

```bash
pyroboreplay export mission.db3 mission.mcap --format mcap
```

```bash
pip install "pyroboreplay[mcap]"
```

```python
from pyroboreplay import Mission, export_mcap

mission = Mission.from_ros_bag("mission.db3")
export_mcap(mission, "mission.mcap")
```

Events are written to `/events/<sensor>` and failures to `/failures` as JSON
messages with their original timestamps, so Foxglove/Rerun and other MCAP
readers can open the recording. MCAP (MIT) is an optional dependency.

## Rerun mission playback

Export a mission timeline and detected failures to a Rerun recording:

```bash
pip install "pyroboreplay[rerun]"
```

```bash
pyroboreplay-export mission.db3 mission.rrd --format rerun
```

```python
from pyroboreplay import Mission, export_mission

mission = Mission.from_ros_bag("mission.db3")
export_mission(mission, "mission.rrd")
# rerun mission.rrd
```

Events are logged under `events/<sensor>` on a `time` timeline; failures are
logged under `failures` with severity/confidence text. Rerun is an optional
Apache-2.0 viewer dependency, not a vendored component.

## Alpamayo forensic reports

Turn an offline Alpamayo run (`.json`, `.jsonl`, `.npz`, or a mapping) into a
trajectory and reasoning report. Run timestamps are matched against failures
already detected in the mission:

```python
from pyroboreplay import (
    build_alpamayo_report,
    load_alpamayo_run,
    write_alpamayo_report,
)

run = load_alpamayo_run("alpamayo-run.json")
report = build_alpamayo_report(run, mission=mission, ground_truth=ego_future_xyz)
write_alpamayo_report(report, "alpamayo-report.md")
```

The report includes minADE/minFDE per run, per-sample reasoning traces, and
correlated failures. No `torch` or model weights are required; generate the
artifacts with the external Alpamayo inference stack.

## Usage

```bash
pyroboreplay compare mission_a.bag mission_b.bag        # side-by-side
pyroboreplay list mission.bag                            # topics in a bag
pyroboreplay analyze mission.bag --detect-gaps --detail  # findings to stdout
pyroboreplay export mission.bag mission.mcap --format mcap

# Phase 12/13: ingest external detections and a thermal frame
pyroboreplay forensics --yolo detections.json --dino dino.json \
  --rgb rgb.json --thermal thermal.json --format json

# Phase 15: Nav2 root-cause heuristics from a measurements document
pyroboreplay nav2 nav2-measurements.json --format json

# Phase 14: unified session from recorded sources
pyroboreplay session mission.bag --logs system.log --log-type syslog

# PyTerrainMap grid-map JSON export
pyroboreplay terrain terrain-grid.json --format json
```

The Python package exports `Mission`, `Event`, `Failure`, `Hypothesis`, `RootCauseAnalysis`, `Action`, `FleetStatistics`, `GeoHotspot`, plus `load_pyterrainmap_grid`, `analyze_detection_gaps`, `fuse_rgb_thermal`, `analyze_nav2_diagnostics`, `build_navigation_session`, and the Phase 16-20 JSON bridge (`ops`, `run_op`; see [Advanced operations](docs/ADVANCED_OPS.md)). World-knowledge and typed advanced-analysis APIs still live in the Rust core only.

## Documentation

- [Documentation index](docs/README.md) — start here
- [Install](docs/INSTALL.md) — Python API vs CLI and platform matrix
- [Advanced operations](docs/ADVANCED_OPS.md) — Phase 16-20 JSON bridge
- [Project status](docs/STATUS.md) — current gaps and fixes
- [Alternatives](docs/ALTERNATIVES.md) — Foxglove/Rerun comparison
- [docs/CLAUDE.md](docs/CLAUDE.md) — product vision and architecture
- [docs/API.md](docs/API.md) — Python and Rust API reference
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — phase-by-phase architecture
- [docs/QUICKSTART.md](docs/QUICKSTART.md) — getting started
- [docs/STATS_DASHBOARD.md](docs/STATS_DASHBOARD.md) — live metrics dashboard (`replay --stats-dashboard`)
- [examples/](examples/) — runnable demos (`cargo run --example root_cause_analysis_demo`)
- [ROADMAP_HONEST.md](ROADMAP_HONEST.md) — implemented vs planned, technical debt
- [CHANGELOG.md](CHANGELOG.md) — notable changes

## Known issues and deferred work

**Shipped:**
- Phases 12-15 exposed through the CLI and Python (`forensics`, `session`, `nav2`).
- Six Phase 16-20 operations exposed through the JSON `ops()` / `run_op()` bridge (not typed Python APIs).
- MCAP export in the native CLI; Rerun export via `pyroboreplay-export`.
- Clippy is clean and gates CI (`-D warnings`).
- PyTerrainMap `pyterrainmap-gridmap` v1 JSON reader.

**Blocked / deferred:**
- Typed Python APIs for the remaining Phase 16-20 library types.
- Full PyTerrainMap runtime integration (H3 store, ROS bridge).

**Rejected / deferred:**
- Live fleet dashboard.

Full shipped/blocked/rejected detail: [`QUEUE_STATUS.md`](QUEUE_STATUS.md).

## Contributing

Contributions welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) for setup, conventions, and PR guidelines. Bugs and ideas: [GitHub Issues](https://github.com/Mullassery/PyRoboReplay/issues); questions: [GitHub Discussions](https://github.com/Mullassery/PyRoboReplay/discussions).

## Part of the robotics & autonomy family

PyRoboReplay is one of six complementary, composable tools in the Mullassery **robotics & autonomy** family. Together they form a single stack: **perceive → map → simulate → learn → debug → coordinate the fleet**.

| Tool | Role | What it does |
|---|---|---|
| [PyRoboVision](https://github.com/Mullassery/PyRoboVision) | Perception | Multi-object tracker (Kalman filter + Hungarian algorithm) and trajectory predictor for robot perception. |
| [PyTerrainMap](https://github.com/Mullassery/PyTerrainMap) | Mapping / SLAM | Fuses multi-robot terrain observations into a spatial-temporal store, with H3 spatial indexing and SLAM. |
| [PyRoboSimulator](https://github.com/Mullassery/PyRoboSimulator) | Simulation | World simulator for autonomous systems, with a FastAPI backend. |
| [PyRoboFrames](https://github.com/Mullassery/PyRoboFrames) | Learning data | Rust-backed ML dataloader for robot-learning datasets (native LeRobot format, Apple Silicon hardware video decode). |
| [PyRoboReplay](https://github.com/Mullassery/PyRoboReplay) | Mission forensics | **This project** — forensic-debugging and causal-analysis engine for autonomous robot mission replay. |
| [SyncFleet](https://github.com/Mullassery/SyncFleet) | Fleet coordination | Rust + SQLite, peer-to-peer state sync for robotics fleets — mutual-TLS 1.3, version-vector replication, mDNS discovery; conflict resolution on the roadmap. |

Each works standalone, but they're complementary by design. [Browse all projects →](https://github.com/Mullassery?tab=repositories)

## License

Apache License 2.0 — see [LICENSE](LICENSE).
