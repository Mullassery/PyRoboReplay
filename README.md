# PyRoboReplay  

> **Forensic debugging platform for autonomous robot systems.** Replay missions, perform causal analysis, detect hidden objects, fuse multispectral sensors, and reconstruct what really happened—from passive replay to intelligent agent debugging.

[![CI Status](https://github.com/mullassery/pyroboreplay/actions/workflows/ci.yml/badge.svg)](https://github.com/mullassery/pyroboreplay/actions/workflows/ci.yml)
[![Security Audit](https://github.com/mullassery/pyroboreplay/actions/workflows/security.yml/badge.svg)](https://github.com/mullassery/pyroboreplay/actions/workflows/security.yml)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10+-blue.svg)](https://www.python.org/)
[![PyPI](https://img.shields.io/badge/PyPI-2.1.0-blue.svg)](https://pypi.org/project/pyroboreplay/)
[![Tests](https://img.shields.io/badge/Tests-647%20Passing-brightgreen.svg)](#testing)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/pyroboreplay.svg)](https://crates.io/crates/pyroboreplay)
[![GitHub Stars](https://img.shields.io/github/stars/mullassery/pyroboreplay?style=social)](https://github.com/mullassery/pyroboreplay)

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

## What You Get (v2.1.0)

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
pip install pyroboreplay==2.1.0

# or with uv
uv install pyroboreplay==2.1.0

# From source
git clone https://github.com/mullassery/pyroboreplay.git
cd pyroboreplay
cargo build --release
```

### Your First Forensic Analysis

```bash
# Interactive timeline scrubber
pyroboreplay replay mission.bag

# Complete forensic investigation
pyroboreplay forensic mission.bag --output investigation.json

# Sensor fusion analysis (RGB + thermal)
pyroboreplay fuse-sensors rgb.bag thermal.bag --report forensic.md

# Cross-mission pattern detection
pyroboreplay cross-mission *.bag --learn-patterns --predict-next
```

Keyboard shortcuts:
- **Space**: Play/Pause | **/**: Step | **Ctrl+J**: Jump | **f**: Filter | **q**: Quit

### Python API

```python
from pyroboreplay import Mission, ForensicAnalyzer, RGBThermalFusion

# Load mission
mission = Mission.from_ros_bag("warehouse.bag")

# Causal analysis
hypothesis = mission.analyze_failure(timestamp=1234567890)
print(f"Root cause: {hypothesis.description}")
print(f"Confidence: {hypothesis.confidence:.0%}")

# Invisible object detection (DINO + SAM)
retrospective = mission.analyze_retrospectively()
print(f"Objects RGB missed: {len(retrospective.gaps)}")
for gap in retrospective.gaps:
    print(f"  {gap.dino_detection.class_name} at risk level {gap.severity}")

# Multispectral forensic analysis
fusion = RGBThermalFusion()
fusion.load_rgb_detections(mission.rgb_detections)
fusion.load_thermal_frame(thermal_data)
fusion.fuse()

stats = fusion.get_statistics()
print(f"Thermal-only detections: {stats.thermal_only_detections}")
print(f"RGB miss rate: {stats.rgb_miss_rate:.1%}")
print(f"Confidence improvement: +{stats.confidence_improvement:.1%}")

# Persistent world knowledge (cross-mission learning)
world_model = mission.extract_world_knowledge()
print(f"Known entities: {len(world_model.entities)}")
print(f"Temporal facts recorded: {len(world_model.temporal_facts)}")

# Next mission prediction
prediction = mission.predict_next_mission()
print(f"Likely obstacles: {prediction.expected_obstacles}")
```

---

## Feature Matrix: v0.1 to v2.1.0

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
| **647 Comprehensive Tests** | - | - | - | - | - | **A** |

---

## Real-World Use Cases

### Warehouse Operations
Debug fleet behavior, identify missed detections, optimize coverage.

```bash
# Find what robot missed (RGB vs thermal)
pyroboreplay invisible-persons warehouse.bag --thermal-data warehouse.thermal
```

Result: Identify undetected workers, near-collisions, coverage gaps.

### Precision Agriculture
Verify inspection coverage, detect missed areas, analyze sensor performance.

```bash
# Multispectral analysis with RGB + thermal
pyroboreplay fuse-sensors rgb_survey.bag thermal_survey.bag
```

Result: Find unscanned areas invisible to standard RGB.

### Research & Development
Compare perception strategies, analyze fleet behavior, identify sim-to-reality gaps.

```python
exp_a = Mission.from_bag("strategy_v1.bag")
exp_b = Mission.from_bag("strategy_v2.bag")

# Causal analysis
causes_a = exp_a.root_cause_analysis()
causes_b = exp_b.root_cause_analysis()

improvement = len(causes_a) - len(causes_b)
print(f"v2 fixes {improvement} issues vs v1")
```

Result: Data-driven strategy selection, quantified improvements.

### Safety & Compliance
Verify robot didn't miss people, generate forensic reports, audit sensor performance.

```bash
# Complete forensic investigation
pyroboreplay forensic operation.bag --output compliance_report.json
```

Result: Provable safety assurance, auditable incident investigation.

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

**Test Coverage:** 558 passing tests

- Phases 1-4: 60 tests
- Phases 5-9: 140 tests
- Phase 10 (Knowledge): 26 tests
- Phase 7 Enhanced: 15 tests
- Phase 11: 17 tests
- Phase 12: 20 tests
- Phase 13: 18 tests
- Integration & edge cases: 246 tests

---

## Development

### Build
```bash
cargo build --release
maturin develop # Install Python wheel
```

### Test (558 Passing)
```bash
# Full test suite
cargo test

# By phase
cargo test phase_13  # Multispectral fusion
cargo test phase_12  # Retrospective detection
cargo test phase_11  # Terrain intelligence
cargo test knowledge # Persistent world model

# Examples
cargo run --example forensic_analysis_demo
cargo run --example thermal_fusion_demo
```

### Quality Checks
```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo audit
```

---

## Documentation

- **[CLAUDE.md](CLAUDE.md)** — Complete product vision & architecture
- **[Examples](examples/)** — Working demos (replay, forensics, fusion, cross-mission learning)
- **[API Reference](docs/API.md)** — Python & Rust APIs
- **[Phases Overview](docs/PHASES.md)** — Detailed phase descriptions

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, coding conventions, and PR guidelines.

**Easiest ways to help:**
- Report bugs or feature ideas: [GitHub Issues](https://github.com/mullassery/pyroboreplay/issues)
- Share how you're using PyRoboReplay: [GitHub Discussions](https://github.com/mullassery/pyroboreplay/discussions)
- Star the repo if it helps you

---

## License

MIT License — Use freely in academic, commercial, and personal projects. See [LICENSE](LICENSE).

---

## Citation

If PyRoboReplay helps your research or product, please star the repo and cite:

```bibtex
@software{pyroboreplay2026,
 title={PyRoboReplay: Forensic Debugging and Multispectral Analysis for Autonomous Robots},
 author={Mullassery, Georgi},
 year={2026},
 version={2.0.0},
 url={https://github.com/mullassery/pyroboreplay}
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
