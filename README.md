# PyRoboReplay

Forensic debugging and mission-replay platform for autonomous robots: replay ROS 2 missions, run causal / root-cause analysis on failures, and perform multispectral (RGB + thermal) forensics. Rust core with Python bindings.

[![CI](https://github.com/Mullassery/PyRoboReplay/actions/workflows/ci.yml/badge.svg)](https://github.com/Mullassery/PyRoboReplay/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/pyroboreplay)](https://pypi.org/project/pyroboreplay/)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue.svg)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

## What it is

PyRoboReplay ingests recorded robot missions (ROS 2 bags) into a unified timeline and answers *why* and *what-if* questions rather than only *where* and *what*: which events preceded a failure, what the perception stack missed, and what the robot should have detected. The heavy lifting is a Rust core (`cdylib` + `rlib`) built with PyO3, exposed to Python through `maturin`. A CLI (`replay`, `analyze`, `compare`, `list`) and a small Python API are the supported entry points today.

## Features

**Exposed via CLI / Python (verified, tested):**
- **Mission replay** — interactive timeline scrubber with keyboard controls (`replay`).
- **Reality-gap analysis** — probabilistic mismatch scoring, severity classification, evidence aggregation, JSON/text output (`analyze --detect-gaps`).
- **Failure detection and diagnosis** — `Mission.detect_failures()`, `Mission.analyze_failure()` with hypotheses, confidence, and prioritized actions.
- **Mission comparison** — side-by-side diff of two bags (`compare`).
- **Topic listing** — inspect available topics in a bag (`list`).
- **Real ROS 2 CDR decoding** — standard `sensor_msgs` / `nav_msgs` / `geometry_msgs` binary layouts are decoded from real message bytes (`src/adapters/cdr.rs`); pose `confidence` is derived from the real position-covariance trace, so localization-loss detection fires on real data.

**Implemented in the Rust core, not yet wired to CLI or Python (library API only):**
- Causal event graphs, multi-factor causality, incident narratives, evidence-quality scoring.
- Persistent world knowledge (entities, locations, facts across missions) and spatial grounding.
- Multi-mission / longitudinal learning.
- Pluggable detection backends (YOLO / SAM / template fallback).
- Retrospective DINO + SAM open-vocabulary object discovery.
- RGB + thermal multispectral fusion and forensic reconstruction.
- Universal temporal fusion and the Nav2 root-cause inference engine (Phases 14-15).

## Who it's for / Use cases

- **Warehouse and logistics fleets** — reconstruct why a run failed, find missed detections and coverage gaps, compare two strategy revisions.
- **Safety and compliance review** — produce auditable, saved JSON findings for an incident (`analyze --detect-gaps --format json --output report.json`).
- **Perception R&D** — compare perception strategies over matched missions and quantify regressions.
- **Field autonomy teams** — investigate localization loss, planner oscillation, and sensor disagreement from recorded missions.

## What it is not

- **Not a real-time monitoring or fleet dashboard.** It works on recorded missions, not live streams.
- **Most advanced modules (Phases 12-15) are not reachable from the CLI or Python.** Use the Rust library directly, or track exposure on the roadmap.
- **Phase 11 "PyTerrainMap integration" is not real interop.** `src/.../pyterrain_bridge.rs` is self-contained, unit-tested terrain-modeling logic conceptually inspired by [PyTerrainMap](https://github.com/Mullassery/PyTerrainMap); there is no Cargo/pip dependency, no shared serialization format, and no parser for PyTerrainMap's output.
- **Not published to crates.io** — build the Rust crate from source if you need it directly.
- **Not a certified or safety-verified system.**

## Project status

Beta (Development Status 4). Current source version is **2.10.1**; the latest PyPI release is **2.10.0**. The CLI and Python API are the tested, supported surface. 843 `cargo test --lib` unit tests pass (re-verified 2026-09-29), plus 7 real-fixture ROS 2 CDR integration tests. `cargo fmt --check` is clean. `cargo clippy --all-targets --all-features -- -D warnings` currently reports ~96 errors, so the CI clippy job runs with `continue-on-error` and does not gate merges. See [ROADMAP_HONEST.md](ROADMAP_HONEST.md) for the implemented/partial/missing breakdown.

## Installation

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

## Usage

```bash
pyroboreplay compare mission_a.bag mission_b.bag        # side-by-side
pyroboreplay list mission.bag                            # topics in a bag
pyroboreplay analyze mission.bag --detect-gaps --detail  # findings to stdout
```

The Python package exports `Mission`, `Event`, `Failure`, `Hypothesis`, `RootCauseAnalysis`, `Action`, `FleetStatistics`, `GeoHotspot` (see `src/pyroboreplay/__init__.py`). The multispectral, retrospective-detection, world-knowledge, and prediction features live in the Rust core and are not yet exposed through these bindings.

## Documentation

- [docs/CLAUDE.md](docs/CLAUDE.md) — product vision and architecture
- [docs/API.md](docs/API.md) — Python and Rust API reference
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — phase-by-phase architecture
- [docs/QUICKSTART.md](docs/QUICKSTART.md) — getting started
- [docs/STATS_DASHBOARD.md](docs/STATS_DASHBOARD.md) — live metrics dashboard (`replay --stats-dashboard`)
- [examples/](examples/) — runnable demos (`cargo run --example root_cause_analysis_demo`)
- [ROADMAP_HONEST.md](ROADMAP_HONEST.md) — implemented vs planned, technical debt
- [CHANGELOG.md](CHANGELOG.md) — notable changes

## Known issues and deferred work

**Blocked / failed:**
- Expose phases 12–15; rerun/MCAP export; clippy gate.

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
