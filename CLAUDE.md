# PyRoboReplay: Mission Replay & Spatial Intelligence Observatory

## Product Vision

**PyRoboReplay** is a time-travel debugger for robot fleets that allows engineers to replay, inspect, compare, and understand autonomous missions step-by-step—just as developers use Git history, traces, and debuggers to understand software behavior.

The platform reconstructs and visualizes mission history from existing robotics systems without performing mapping, localization, SLAM, navigation, or terrain generation.

### Elevator Pitch

For robotics teams debugging why missions failed or underperformed, PyRoboReplay is the observability platform that reconstructs replayable timelines of robot behavior from any robotics system (ROS 2, Gazebo, Isaac Sim, digital twins). Unlike simulators or bagfile viewers, PyRoboReplay answers the "why" questions: why was this area never mapped? Why did one robot repeatedly fail while others succeeded? When did an obstacle first appear?

## The Problem We Solve

Current robotics tooling excels at answering:
- Where is the robot now?
- What does the map look like now?
- What is the robot currently seeing?

But struggles with:
- Why was this area never mapped?
- Why did one robot repeatedly fail while others succeeded?
- When did an obstacle first appear?
- Which decision caused a coverage gap?
- Why did the swarm split into inefficient clusters?
- How did exploration strategy evolve over time?

Engineers currently investigate through fragmented tools (ROS bags, logs, sensor recordings, dashboards, manual reconstruction)—slow and hard to scale.

## Core Architecture

### Universal Event Model

All mission history is normalized to core event types:

```rust
enum MissionEvent {
    RobotPose { robot_id, timestamp, x, y, z, orientation },
    SensorObservation { robot_id, timestamp, sensor_type, data },
    NavigationDecision { robot_id, timestamp, decision_type, rationale },
    ObstacleDetected { robot_id, timestamp, location, type },
    CommunicationEvent { timestamp, from, to, event_type },
    CoordinationEvent { timestamp, robots_involved, event_type },
    EnvironmentalChange { timestamp, location, change_type },
    MissionLifecycle { timestamp, event_type }, // start, pause, resume, end
}
```

### Input Adapters (Pluggable)

Not locked into ROS 2. Adapters normalize external data to universal event model:

- **v0.1**: ROS 2 bag reader (most students start here)
- **v0.2**: Gazebo event stream
- **v0.3**: Isaac Sim exporter
- **Future**: Digital twin APIs, custom JSON/CSV, telemetry streams

### Timeline Engine (Rust Core)

- Event storage and indexing
- Temporal queries (jump to events, range queries)
- Spatial correlation (integrates PyTerrainMap context)
- Event causality tracking
- Storage backend abstraction (in-memory, PostgreSQL, BigQuery, S3)

### Python API

Simple for students, powerful for production:

```python
from pyroboreplay import Mission

# Student: 30-second replay
mission = Mission.from_ros_bag("exploration_v1.bag")
mission.play()  # CLI timeline scrubber

# Production: Advanced analysis
mission = Mission.from_events(event_stream, backend="postgresql")
mission.spatial_context(pyterrainmap_graph)
mission.find_coverage_gaps()
mission.root_cause_analysis(failure_timestamp)
mission.compare_with(other_mission)
```

### PyTerrainMap Integration

PyRoboReplay embeds PyTerrainMap as a dependency to provide spatial context during replay:
- PyTerrainMap owns: 3D reconstruction, real-time SLAM, traversability analysis, spatial knowledge graphs
- PyRoboReplay owns: Historical replay, event correlation, mission timeline, coordination analysis, root-cause explanation

This maintains architectural boundaries—PyRoboReplay doesn't duplicate mapping or localization work.

## Core Principles

### Principle 1: Input-Agnostic
Does not lock into ROS 2 alone. Adapter pattern allows seamless integration with Gazebo, Isaac Sim, digital twins, custom formats. Universal event model ensures timeline engine remains independent.

### Principle 2: Mapping-Independent
Does not generate maps. Consumes maps from PyTerrainMap, SLAM systems, GIS platforms. Replay layer remains independent of how spatial data was created.

### Principle 3: Explainability First
Goal is understanding, not just visualization. Every replay event answers: What happened? Why did it happen? What happened next?

### Principle 4: Simple to Start, Infinite Depth
- **Day 1 student**: `pyroboreplay replay mission.bag` → timeline in 30 seconds
- **Production operator**: Distributed event store, spatial correlation, multi-mission federation, real-time streaming, mission-critical SLAs

## Core Features (Roadmap Order)

### v0.1: Timeline Scrubber + ROS 2 Ingestion
- Parse ROS 2 bag files
- Normalize to universal event model
- In-memory timeline storage
- CLI: play/pause/step/jump to events
- Test with single-robot exploration mission

### v0.2: Web UI + Spatial Visualization
- Web-based timeline scrubber
- Robot trajectory visualization
- Spatial context from PyTerrainMap
- Event filtering and search

### v0.3: Swarm Coordination Analysis
- Multi-robot visualization
- Coverage overlap detection
- Coordination event timeline
- Congestion zone identification

### v0.4: Advanced Event Queries
- Event-centric replay (jump to failures, obstacles, communication loss)
- Coverage evolution playback
- Environmental change explorer
- Multi-mission comparison

### v1.0: Production Scale
- Pluggable storage backends (PostgreSQL, BigQuery, S3)
- Streaming event ingestion
- Distributed replay engine
- Mission-critical reliability

### v1.1: AI-Powered Analysis
- Root-cause analysis engine
- Coverage gap explanations
- Exploration efficiency metrics
- Cross-mission learning

## Ideal Users

- **Robotics researchers**: Analyze swarm experiments
- **Robotics students**: Debug first exploration missions
- **Warehouse operators**: Analyze fleet performance
- **Agriculture companies**: Review inspection coverage
- **Security companies**: Investigate patrol behavior
- **Digital twin operators**: Understand environmental evolution

## What This Is NOT

- Not a SLAM system
- Not a mapping engine
- Not a simulator
- Not a path planner
- Not a navigation stack
- Not a terrain database

Those systems remain responsible for generating world knowledge. PyRoboReplay helps humans understand how that knowledge was created over time.

## Long-Term Vision

PyRoboReplay becomes for robotics what:
- **GitHub** is for source history
- **Datadog** is for infrastructure observability
- **Figma version history** is for design evolution

A universal observability layer that lets teams rewind, inspect, compare, explain, and learn from autonomous missions across both real and simulated worlds.

---

## Development Guidelines

### Code Quality
- Type-safe Rust core with comprehensive error handling
- Python API should feel natural (not a thin wrapper)
- All public APIs must have docstrings with examples
- Tests for every adapter and core feature

### Architecture Decisions
- Events are immutable and timestamped
- Storage backend is pluggable from day one
- Spatial queries delegate to PyTerrainMap when applicable
- No hard dependencies on specific robotics frameworks at core level

### Performance Expectations
- Timeline scrubbing: <100ms latency for 1M events
- Adapter ingestion: Real-time or batch, both supported
- Spatial queries: <500ms for complex cross-mission analysis

### Naming Conventions
- Rust modules follow Rust conventions (snake_case)
- Python API follows Python conventions (snake_case)
- Event types use PascalCase
- CLI commands use kebab-case
