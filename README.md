# PyRoboReplay

Robotics perception and replay engine. RGB+Thermal sensor fusion, invisible person detection, trajectory analysis, multi-modal training datasets. Analyze what happened, why it happened, and how to prevent it.

Includes thermal imaging analysis, motion replay, and causal inference for autonomous systems.

> **Forensic debugging platform for autonomous robot systems.** Replay missions, perform causal analysis, detect hidden objects, fuse multispectral sensors, and reconstruct what really happened—from passive replay to intelligent agent debugging.

[![CI Status](https://github.com/Mullassery/PyRoboReplay/actions/workflows/ci.yml/badge.svg)](https://github.com/Mullassery/PyRoboReplay/actions/workflows/ci.yml)
[![Security Audit](https://github.com/Mullassery/PyRoboReplay/actions/workflows/security.yml/badge.svg)](https://github.com/Mullassery/PyRoboReplay/actions/workflows/security.yml)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10+-blue.svg)](https://www.python.org/)
[![PyPI](https://img.shields.io/badge/PyPI-2.9.2-blue.svg)](https://pypi.org/project/pyroboreplay/)
[![Tests](https://img.shields.io/github/actions/workflow/status/Mullassery/PyRoboReplay/ci.yml?label=tests)](https://github.com/Mullassery/PyRoboReplay/actions)
[![License](https://img.shields.io/badge/License-Proprietary-lightgrey.svg)](LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/mullassery/pyroboreplay?style=social)](https://github.com/Mullassery/PyRoboReplay)

---

## Why PyRoboReplay?

Robotics teams waste **2-16 hours debugging a single mission failure**—jumping between rosbags, logs, dashboards, and manually reconstructing causality.

**Old tools answer "where/what" questions:**
- Where is the robot now?
- What sensor data was captured?

**PyRoboReplay 2.0 answers "why" and "what if" questions:**
- Why did the robot fail? (Root cause analysis + causal graphs)
- What should have been detected? (Retrospective DINO + SAM)
- What was actually there? (RGB + thermal fusion forensics)
- What changed in the environment? (Temporal knowledge + terrain intelligence)
- Will this happen again? (Predictive modeling + pattern detection)
- How do we prevent it? (Recommendations + sensor fusion analysis)

**Result:** Debug 10x faster, fix failures before they happen, understand reality gaps at scale.

---

## What You Get (v2.9.2)

### Phase 1-4: Reality Gap Detection Foundations
Comprehensive detection of perception mismatches between simulation and reality. Identifies where and why robot perception diverged from expectations.

### Phase 5-9: Intelligent Analysis
Causal reasoning engine, multi-factor causality analysis, incident narratives, evidence quality scoring, and LLM-assisted root cause analysis with semantic search.

### Phase 10: Persistent World Knowledge
OKF-inspired temporal knowledge system. Entities persist across missions. Track location history, temporal facts, anomaly records. Enable: "Pallet moved from aisle_3 to aisle_5."

### Phase 10.2: Spatial Grounding
Ground entities in X,Y,Z coordinates with movement vectors and trends. Track "moved 2.3m northeast" not just "moved."

### Phase 10.3: Multi-Mission Learning
Longitudinal reasoning across mission sequences. Predict entity behavior, detect environmental evolution, enable cross-mission pattern detection.

### Phase 7 Enhanced: Pluggable Detection
Swappable detection backends (YOLO for speed, SAM for zero-shot flexibility, template fallback for offline). Automatic fallback chain ensures always-working detection.

### Phase 11: PyTerrainMap Integration + Fleet Learning
Terrain-aware perception. Track zone traversability, assess entity risk by terrain type. Multi-robot fleet consensus on zone difficulty. Anomaly detection at fleet scale.

### Phase 12: Retrospective DINO + SAM Analysis
Open-vocabulary object detection for invisible object discovery. Segment anything model for precise boundaries. Compare YOLO vs DINO to identify perception gaps. Context-aware severity scoring with terrain and historical data.

### Phase 13: Multispectral Sensor Fusion & Forensic Analysis
RGB + thermal/infrared fusion for offline forensic reconstruction. Identify invisible persons in low-light, smoke, fog, shadows, occlusions. Root cause analysis, sensor disagreement detection, recommendations for future systems.

### **Phase 14: Universal Temporal Fusion Foundation** (NEW)
Multi-modal data ingestion for heterogeneous sources: ROS 2 bags, video, Linux system logs, Nav2 exports, point clouds, operator annotations, sensor calibration. Unified timeline with automatic clock synchronization. Handles time model detection (ROS nanoseconds, wall-clock, frame numbers, sequences) and temporal alignment across all modalities.

### **Phase 15: Root Cause Inference Engine** (NEW)
AI-powered navigation failure analysis across 7 dimensions: localization (AMCL divergence, odometry drift), planner (oscillation, deadlock), costmap (inflation, conflicts), dynamic obstacles, semantic gaps, environmental context, controller stability. Distinguishes Nav2 architectural limitations from tuning/environment issues. Generates structured findings with tiered recommendations (tuning/capability/architecture) and confidence scoring (0.0-1.0) based on evidence strength.

---

## Quick Start

### Installation

```bash
pip install pyroboreplay==2.9.2

# or with uv
uv pip install pyroboreplay==2.9.2

# From source
git clone https://github.com/Mullassery/PyRoboReplay.git
cd pyroboreplay
cargo build --release

# Verify installation
pyroboreplay --version
```

Note: PyRoboReplay is published to PyPI (`pip install pyroboreplay`). It is **not** currently published to crates.io — build from source via `cargo build --release` if you want the Rust crate/binary directly.

### Your First Forensic Analysis

The CLI currently exposes four subcommands: `replay`, `analyze`, `compare`, `list`.

```bash
# Interactive timeline scrubber
pyroboreplay replay mission.bag

# Reality-gap analysis with detailed findings, saved to a file
pyroboreplay analyze mission.bag --detect-gaps --detail --format json --output investigation.json

# Compare two missions side-by-side
pyroboreplay compare mission_a.bag mission_b.bag

# List available topics in a bag file
pyroboreplay list mission.bag
```

The RGB+thermal fusion, retrospective DINO/SAM detection, cross-mission learning, and Nav2 root-cause-inference capabilities described above (Phases 12-15) are implemented as internal Rust library modules with dedicated unit test coverage — they are not yet wired up as CLI subcommands or Python bindings. Use the Rust library API (`src/fusion`, `src/perception`, `src/intelligence`, `src/phase14`, `src/phase15`) directly, or track CLI/Python exposure on the [roadmap](ROADMAP.md).

Keyboard shortcuts (interactive replay):
- **Space**: Play/Pause | **n / →**: Next event | **p / ←**: Previous event | **↑ / ↓**: Speed up/down | **?**: Help | **q / Esc**: Quit

### Python API

```python
from pyroboreplay import Mission

# Load mission
mission = Mission.from_ros_bag("warehouse.bag")

# Detected failures
failures = mission.detect_failures()
print(f"Failures detected: {len(failures)}")

# Root cause analysis
analysis = mission.analyze_failure(timestamp=1234567890.0)
print(f"Root cause: {analysis.get_primary_hypothesis()}")
print(f"Confidence: {analysis.get_diagnostic_confidence():.0%}")

# Recommended actions
for action in mission.recommend_actions(timestamp=1234567890.0):
    print(f"[{action.get_priority()}] {action.get_description()}")
```

The Python package currently exposes `Mission`, `Event`, `Failure`, `Hypothesis`, `RootCauseAnalysis`, `Action`, `FleetStatistics`, and `GeoHotspot` (see `src/pyroboreplay/__init__.py`). The RGB+thermal fusion, retrospective object discovery, persistent world knowledge, and next-mission prediction functionality described earlier in this README exists in the Rust core but is not yet exposed through the Python bindings.

---

## Feature Matrix: v0.1 to v2.9.2

| Feature | v0.1 | v0.5 | v0.9 | v1.0 | v2.0 | v2.1 |
|---------|:----:|:----:|:----:|:----:|:----:|:----:|
| Sensor Replay | A | A | A | A | A | A |
| Timeline Queries | - | A | A | A | A | A |
| Causal Analysis | - | - | A | A | A | A |
| Root Cause Diagnosis | - | - | A | A | A | A |
| Cross-Mission Learning | - | - | - | A | A | A |
| Pluggable Detection (YOLO/SAM) | - | - | - | - | A | A |
| **Terrain Intelligence** | - | - | - | - | A | A |
| **Persistent World Knowledge** | - | - | - | - | A | A |
| **Retrospective Object Discovery** | - | - | - | - | A | A |
| **Multispectral Sensor Fusion** | - | - | - | - | A | A |
| **Forensic Investigation Reports** | - | - | - | - | A | A |
| **Fleet Learning & Consensus** | - | - | - | - | A | A |
| **Invisible Person Detection** | - | - | - | - | A | A |
| **Universal Temporal Fusion** | - | - | - | - | - | **A** |
| **Multi-Modal Data Ingestion** | - | - | - | - | - | **A** |
| **Root Cause Inference Engine** | - | - | - | - | - | **A** |
| **Nav2 Limitation Detection** | - | - | - | - | - | **A** |
| **Semantic Gap Analysis** | - | - | - | - | - | **A** |
| **826 Comprehensive Tests** | - | - | - | - | - | **A** |

---

## Real-World Use Cases

### Warehouse Operations
Debug fleet behavior, identify missed detections, optimize coverage.

```bash
# Reality-gap analysis on a warehouse mission
pyroboreplay analyze warehouse.bag --detect-gaps --detail
```

Result: Identify missed detections and coverage gaps. (RGB+thermal invisible-person fusion is a Rust library capability today, not yet a CLI flag — see note above.)

### Precision Agriculture
Verify inspection coverage, detect missed areas, analyze sensor performance.

```bash
# Reality-gap analysis on survey coverage
pyroboreplay analyze rgb_survey.bag --detect-gaps --detail
```

Result: Find coverage gaps in the survey. (Multispectral RGB+thermal fusion is a Rust library capability today, not yet a CLI flag — see note above.)

### Research & Development
Compare perception strategies, analyze fleet behavior, identify sim-to-reality gaps.

```python
exp_a = Mission.from_ros_bag("strategy_v1.bag")
exp_b = Mission.from_ros_bag("strategy_v2.bag")

# Compare detected failures
failures_a = exp_a.detect_failures()
failures_b = exp_b.detect_failures()

improvement = len(failures_a) - len(failures_b)
print(f"v2 fixes {improvement} issues vs v1")
```

Result: Data-driven strategy selection, quantified improvements.

### Safety & Compliance
Verify robot didn't miss people, generate forensic reports, audit sensor performance.

```bash
# Reality-gap analysis with full findings, saved for audit
pyroboreplay analyze operation.bag --detect-gaps --detail --format json --output compliance_report.json
```

Result: Auditable incident investigation. (A dedicated forensic-report CLI command is not yet implemented; `analyze --detect-gaps` is today's closest equivalent — see note above.)

---

## Architecture: 13 Integrated Phases

```
Mission Data Input (ROS 2 Bag / Gazebo / Simulation)
 |
 v
Phases 1-4: Reality Gap Detection
 |-- Probabilistic gap scoring
 |-- Severity classification
 |-- Historical findings database
 |-- Evidence aggregation
 |
 v
Phases 5-9: Intelligent Analysis
 |-- Causal event graphs
 |-- Multi-factor causality
 |-- Incident narratives
 |-- Evidence quality scoring
 |-- LLM-assisted root cause analysis
 |-- Semantic search
 |
 v
Phases 10-11: Temporal Knowledge + Terrain Intelligence
 |-- Persistent world model (entities, locations, facts)
 |-- Spatial grounding (x,y,z coordinates)
 |-- Multi-mission learning (longitudinal reasoning)
 |-- Terrain zones and traversability
 |-- Fleet learning (multi-robot consensus)
 |
 v
Phase 7 Enhanced: Pluggable Detection
 |-- YOLO backend (real-time)
 |-- SAM backend (zero-shot)
 |-- Template fallback (offline)
 |-- Orchestrator (automatic fallback)
 |
 v
Phase 12: Retrospective DINO + SAM
 |-- Open-vocabulary object detection
 |-- Segment anything model
 |-- Invisible object discovery
 |-- Context-aware gap analysis
 |-- Recommendations engine
 |
 v
Phase 13: Multispectral Sensor Fusion
 |-- Thermal imaging model
 |-- RGB-Thermal fusion engine
 |-- Invisible person detection (17 scenarios)
 |-- Forensic report generation
 |-- Root cause analysis
 |
 v
Output: Forensic Reports, Recommendations, Predictions
```

**Key Innovation:** Each phase builds on prior layers. Real-time detection (Phase 7) feeds offline analysis (Phases 12-13). Offline findings improve world knowledge (Phase 10). World knowledge informs next mission (Phases 10.3, 6).

---

## Performance

| Metric | Target | v2.0 Status |
|--------|--------|------------|
| Mission ingestion | 10k events/sec | Tested |
| Timeline scrubbing | <100ms latency | Optimized |
| Large mission queries (1M events) | <1s | Passing |
| Forensic analysis (full pipeline) | <5s | Achieved |
| Multispectral fusion | <2s per frame | Efficient |

**Test Coverage:** 826 passing `cargo test --lib` unit tests (0 failing, verified 2026-08-23), plus dedicated Docker-backed integration test suites for the Postgres/S3/BigQuery storage backends and Ollama LLM integration.

---

## Development

### Build
```bash
cargo build --release
maturin develop # Install Python wheel
```

### Test (826 Passing)
```bash
# Unit test suite
cargo test --lib

# By module
cargo test fusion       # Multispectral (Phase 13) fusion
cargo test perception   # Retrospective/scene detection (Phase 12)
cargo test knowledge    # Persistent world model (Phase 10)
cargo test phase14      # Universal temporal fusion
cargo test phase15      # Root cause inference engine

# Examples
cargo run --example root_cause_analysis_demo
cargo run --example compliance_report_demo
```

### Quality Checks
```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo audit
```

---

## Documentation

- **[CLAUDE.md](docs/CLAUDE.md)** — Complete product vision & architecture
- **[Examples](examples/)** — Working demos (replay, causal analysis, compliance reporting, and more)
- **[API Reference](docs/API.md)** — Python & Rust APIs
- **[Architecture Guide](docs/ARCHITECTURE.md)** — Detailed phase descriptions

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, coding conventions, and PR guidelines.

**Easiest ways to help:**
- Report bugs or feature ideas: [GitHub Issues](https://github.com/Mullassery/PyRoboReplay/issues)
- Share how you're using PyRoboReplay: [GitHub Discussions](https://github.com/Mullassery/PyRoboReplay/discussions)
- Star the repo if it helps you

---

## License

Proprietary License — Free to use with explicit attribution to the original author. Not OSI-approved open source; see the full terms in [LICENSE](LICENSE).

---

## Citation

If PyRoboReplay helps your research or product, please star the repo and cite:

```bibtex
@software{pyroboreplay2026,
 title={PyRoboReplay: Forensic Debugging and Multispectral Analysis for Autonomous Robots},
 author={Mullassery, Georgi},
 year={2026},
 version={2.9.2},
 url={https://github.com/Mullassery/PyRoboReplay}
}
```

---

## Get Started Today

New to robot debugging? Start with the quick start above.

Ready for production? Check out the architecture and examples.

Have questions? Open an issue or discussion.

---

Built for robotics teams who demand understanding, not just visibility.

**PyRoboReplay: Because great robots are built on knowledge, not intuition.**

If this helps you, please star the repo!
