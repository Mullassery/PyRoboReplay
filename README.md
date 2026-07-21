# PyRoboReplay

**Time-travel debugger for robot fleets.** Replay, inspect, compare, and understand autonomous missions step-by-step—just like developers use Git history and debuggers for software.

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10+-blue.svg)](https://www.python.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

## The Problem

Current robotics tooling excels at answering **"now" questions**:
- Where is the robot now?
- What does the map look like now?
- What is the robot currently seeing?

But struggles with **"why" questions**:
- Why was this area never mapped?
- Why did one robot repeatedly fail while others succeeded?
- When did an obstacle first appear?
- Which decision caused a coverage gap?

Engineers investigate through fragmented tools (ROS bags, logs, dashboards) — slow and hard to scale.

## The Solution

PyRoboReplay reconstructs and visualizes **replayable timelines of robot behavior** from any robotics system:
- **ROS 2 bags** (primary input)
- Gazebo simulations
- Isaac Sim environments
- Custom telemetry streams
- Digital twins

Like Git for source code or Datadog for infrastructure, PyRoboReplay is observability for autonomous missions.

## Quick Start

### Installation

```bash
pip install pyroboreplay
```

### Replay Your First Mission

```bash
pyroboreplay replay mission.bag
```

Interactive timeline scrubber:
- **Play/Pause**: Space
- **Step Forward/Backward**: Arrow keys
- **Jump to Event**: Type event index
- **Filter by Type**: Press 'f'
- **Quit**: 'q'

### Python API

```python
from pyroboreplay import Mission

# Load mission from ROS bag
mission = Mission.from_ros_bag("exploration_v1.bag")

# Basic replay
mission.play()

# Advanced queries
mission.spatial_context(pyterrainmap_graph)
mission.coverage_evolution()
mission.find_coverage_gaps()
mission.root_cause_analysis(failure_timestamp)
mission.compare_with(other_mission)

# Export
mission.to_json("mission_history.json")
mission.export_to_parquet("mission_data.parquet")
```

## Architecture

### Universal Event Model

All mission history normalizes to core event types:

```python
RobotPose
SensorObservation
NavigationDecision
ObstacleDetected
CommunicationEvent
CoordinationEvent
EnvironmentalChange
MissionLifecycle
```

No lock-in to ROS 2 or any single robotics framework.

### Input Adapters (Pluggable)

- **ROS 2 Adapter** (v0.1) — Parse bag files, extract topics
- **Gazebo Adapter** (v0.2) — Simulation events
- **Isaac Sim Adapter** (v0.3) — Isaac Sim telemetry
- **Custom Adapter** — Implement `MissionAdapter` trait

### PyTerrainMap Integration

PyRoboReplay embeds **PyTerrainMap** as a dependency to provide spatial context:
- Spatial knowledge graphs
- Traversability analysis
- Coverage evolution
- Obstacle correlation

See [CLAUDE.md](CLAUDE.md) for architecture details.

## Use Cases

| Role | Use Case |
|------|----------|
| **Student** | Debug first exploration mission, understand robot behavior |
| **Researcher** | Analyze swarm experiments, compare strategies |
| **Operator** | Investigate warehouse fleet failures, optimize coverage |
| **Agronomist** | Review inspection coverage in fields |
| **Security** | Analyze patrol patterns and coverage gaps |

## Features (v0.2 ✅ Complete)

### Phase 1: Sensor Replay Foundation (✅ v0.1)
- Universal event model (5 sensor types: Lidar, Camera, IMU, Odometry, Costmap)
- ROS 2 bag parser (.bag and .db3 formats)
- CLI timeline scrubber with Ratatui UI
- Individual sensor stream replay
- Python API via PyO3 bindings

### Phase 2: CLI-First Visualization (✅ v0.2)
- **Lidar visualization**: 2D ASCII polar projection (bird's-eye view)
- **Camera export**: Standalone HTML with timeline-based intelligent frame loading
- **IMU visualization**: ASCII graphs for accelerometer, gyro, magnetometer
- **Sensor metadata panel**: Real-time quality indicators, FPS, encoding, resolution
- **Keyboard shortcuts**: 40+ commands with context-sensitive help
- **JSON output**: All CLI commands support --json for AI-agent integration

### Phase 3: Causal Analysis (Planned)
- Event dependency graph construction
- Causal query engine ("what caused this failure?")
- Interactive causal visualization
- Root-cause diagnosis engine

See [ROADMAP.md](ROADMAP.md) for detailed phases and criteria.

## Roadmap

| Phase | Status | Timeline | Focus |
|-------|--------|----------|-------|
| **v0.1** | ✅ Complete | Weeks 1-4 | Rust core, event model, ROS 2 parser, CLI replay |
| **v0.2** | ✅ Complete | Weeks 5-10 | Lidar/Camera/IMU visualization, sensor metadata, keyboard shortcuts |
| **v0.3** | 📋 Planned | Weeks 11-16 | Causal analysis, event graphs, anomaly detection |
| **v1.0** | 📋 Planned | Weeks 17-30 | Production scale, storage backends, forensic features |
| **v1.1+** | 📋 Future | Beyond | Real-time fusion, compliance reporting, ML integration |

## Development

### Build from Source

```bash
# Prerequisites
cargo --version  # 1.70+
python --version  # 3.10+

# Clone and build
git clone https://github.com/mullassery/pyroboreplay.git
cd pyroboreplay
cargo build --release
pip install -e .
```

### Run Tests

```bash
cargo test --lib        # Rust tests
pytest tests/           # Python integration tests (coming soon)
```

### Contribute

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## What This Is NOT

- **Not a SLAM system** — Use PyTerrainMap or your favorite SLAM
- **Not a mapping engine** — Consume maps from existing systems
- **Not a simulator** — Works with ROS 2, Gazebo, Isaac Sim, real robots
- **Not a path planner** — Observes navigation decisions, doesn't make them

PyRoboReplay helps humans understand how robots explored. Other systems generate that knowledge.

## Design Principles

1. **Input-Agnostic** — Adapter pattern prevents ROS 2 lock-in
2. **Mapping-Independent** — Consumes spatial data from any source
3. **Explainability First** — Every replay event answers: What? Why? Next?
4. **Simple Start, Infinite Depth** — 30-second first replay, production-grade scale

## Performance Targets

| Metric | Target |
|--------|--------|
| Timeline scrubbing latency | <100ms |
| Mission ingestion rate | 10k events/sec |
| Large mission queries (1M events) | <1s |
| Web UI load time | <2s |

## Documentation

### Guides
- [CLAUDE.md](CLAUDE.md) — Product vision, architecture, principles
- [ROADMAP.md](ROADMAP.md) — Phase-by-phase development plan
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) — Project tracking, metrics, progress

### Features (v0.2)
- [Keyboard Shortcuts](docs/KEYBOARD_SHORTCUTS.md) — 40+ commands, help system
- [Lidar Visualization](docs/LIDAR_VISUALIZATION.md) — ASCII polar projection, interpretation guide
- [Camera Export](docs/CAMERA_EXPORT.md) — Timeline-based HTML export, browser playback
- [IMU Visualization](docs/IMU_VISUALIZATION.md) — ASCII graphs, peak detection, drift analysis
- [Sensor Metadata](docs/SENSOR_METADATA.md) — Quality indicators, real-time statistics

### Examples
- [Lidar Visualization Demo](examples/lidar_visualization_demo.rs) — 5 scenarios
- [Camera Export Demo](examples/camera_export_demo.rs) — HTML generation
- [IMU Visualization Demo](examples/imu_visualization_demo.rs) — 5 signal patterns
- [Sensor Metadata Demo](examples/sensor_metadata_demo.rs) — Quality assessment
- [Keyboard Shortcuts Demo](examples/keyboard_shortcuts_demo.rs) — Help system

### API Docs
- [API.md](docs/API.md) — Python/Rust API reference
- [QUICKSTART.md](docs/QUICKSTART.md) — 30-second tutorial

## License

MIT License — See [LICENSE](LICENSE)

## Citation

If PyRoboReplay helps your research, please cite:

```bibtex
@software{pyroboreplay2026,
  title={PyRoboReplay: Mission Replay & Spatial Intelligence Observatory},
  author={Mullassery, Georgi},
  year={2026},
  url={https://github.com/mullassery/pyroboreplay}
}
```

## Contact

- **Issues & Features**: [GitHub Issues](https://github.com/mullassery/pyroboreplay/issues)
- **Discussions**: [GitHub Discussions](https://github.com/mullassery/pyroboreplay/discussions)

---

**Made with ❤️ for robotics teams who want to understand their robots.**
