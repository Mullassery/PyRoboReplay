# PyRoboReplay: Mission Debugging & Causal Analysis Platform

## Product Vision

**PyRoboReplay** is the debugging engine for autonomous robot systems. It reconstructs mission history, replays individual sensor streams and holistic timelines, identifies causal relationships between events, and diagnoses root causes of failures—filling the critical gap left by pure visualization tools.

Where Foxglove Studio shows "what happened," PyRoboReplay answers "why did it happen?" and "will it happen again?"

The platform is:
- **Sensor-native**: Replay lidar, camera, IMU, odometry, costmaps individually or holistically
- **Causal**: Track event relationships, not isolated datapoints
- **Forensic**: Immutable audit trails, deterministic replay, compliance-ready
- **Diagnostic**: Probabilistic root-cause analysis, cross-mission pattern learning
- **Agnostic**: Works with ROS 2, Gazebo, Isaac Sim, digital twins, custom telemetry

### Why It Matters

Robotics teams spend **2-16 hours debugging a single mission failure**—context-switching between rosbags, logs, dashboards, and maps. They manually reconstruct causality ("did obstacle at t=1000 cause stop at t=1050?"). They debug identical bugs repeatedly because failures aren't correlated across missions.

PyRoboReplay solves this: unified mission history, causal event graph, automated diagnosis, cross-mission learning.

### Elevator Pitch

For warehouse operators, drone companies, and robotics researchers debugging mission failures, PyRoboReplay is the analysis platform that transforms fragmented sensor data into actionable diagnoses. Unlike Foxglove (viewer-only) or ROS tools (passive), PyRoboReplay answers: "Why did the robot fail?" and "How do we prevent it?"

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

All mission history is normalized to core event types, enabling sensor-level and holistic replay:

```rust
enum MissionEvent {
    // Sensor streams (individually replayable)
    LidarScan { robot_id, timestamp, frame_id, ranges, intensities },
    CameraFrame { robot_id, timestamp, sensor_id, image_data, metadata },
    IMUData { robot_id, timestamp, accel, gyro, magnetometer },
    OdometryUpdate { robot_id, timestamp, pose, velocity, covariance },
    
    // Processed/fused state
    RobotPose { robot_id, timestamp, x, y, z, orientation, confidence },
    Costmap { robot_id, timestamp, resolution, origin, grid_data },
    
    // Navigation & coordination
    NavigationDecision { robot_id, timestamp, decision_type, path, rationale },
    ObstacleDetected { robot_id, timestamp, location, type, confidence },
    
    // Communication & fleet
    CommunicationEvent { timestamp, from, to, event_type, data },
    CoordinationEvent { timestamp, robots_involved, event_type },
    
    // Environmental context
    EnvironmentalChange { timestamp, location, change_type, description },
    MissionLifecycle { timestamp, event_type, mission_id },
    
    // Causal relationships (v0.5+)
    CausalLink { timestamp, source_event, target_event, relationship_type, confidence },
}
```

**Key feature**: Each sensor event is independently replayable (e.g., play just lidar over time) or holistically (play full mission with all sensors synchronized).

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

### PyTerrainMap & StatGuardian Integration

PyRoboReplay embeds two key dependencies for spatial context and quality intelligence:

**PyTerrainMap** (spatial layer):
- Owns: 3D reconstruction, real-time SLAM, traversability analysis, spatial knowledge graphs
- PyRoboReplay uses: Spatial context for causal analysis, coverage evolution, obstacle correlation

**StatGuardian** (quality layer):
- Owns: Data quality contracts, drift detection, anomaly detection
- PyRoboReplay uses: High-accuracy anomaly flagging across all sensor streams, root-cause confidence scoring
- Benefit: <2% false positive anomaly detection vs ~5% for rule-based detection

This maintains architectural boundaries:
- PyTerrainMap handles spatial reconstruction
- StatGuardian handles data quality/contracts
- PyRoboReplay orchestrates replay + causality + diagnosis

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

## Core Features (Roadmap Order) - CLI-First, Aligned to Market Gaps

### v0.1: Sensor Replay Foundation + ROS 2 Ingestion ✅ DONE
**Gap solved**: Data fragmentation (40% of debugging time)
- ✅ Parse ROS 2 bag files → universal event model
- ✅ Individual sensor stream replay (lidar, camera, IMU, odometry)
- ✅ Holistic mission replay (all sensors synchronized)
- ✅ In-memory timeline + temporal queries
- ✅ CLI: play/pause/step forward/backward, jump to event
- ✅ Python API for programmatic access

### v0.2: Enhanced CLI + Camera Browser Export
**Gap solved**: Complete sensor replay in CLI; camera visualization via browser
- Enhanced CLI with multi-panel views (sensor stats, metadata, real-time graphs)
- Terminal-based lidar visualization (ASCII 2D polar projection)
- Terminal-based IMU visualization (graph rendering in terminal)
- Terminal-based odometry display (pose + velocity vectors)
- **Camera frame export to HTML**: `pyroboreplay replay mission.bag --export-camera camera.html`
  - Generates standalone HTML file with embedded camera frames
  - Opens in any browser for playback (no server needed)
  - Frame-by-frame navigation, play/pause/speed controls
  - Zero external dependencies
- Event filtering and search via CLI

### v0.3: Causal Analysis Engine (NEW)
**Gap solved**: Causality invisible (30% of debugging time)
- Build event dependency graph
- Link obstacles → navigation decisions → stops
- Causal queries: "which events led to failure at t=5000?"
- Temporal correlation window (e.g., events within 2s)
- Visualization: causal flowcharts

### v0.4: Cross-Mission Pattern Learning
**Gap solved**: Repeated failures (50% of debug effort)
- Compare missions side-by-side (A vs B)
- Identify similar failure patterns
- Anomaly detection (this mission's failure unusual for this robot?)
- Recommendation engine (if similar failure before, apply previous fix)

### v0.5: Root-Cause Diagnosis (NEW)
**Gap solved**: Manual hypothesis generation
- Probabilistic diagnosis: "Obstacle detected → navigation deadlock → battery drain"
- Confidence scores for each hypothesis
- Counterfactual reasoning: "if obstacle wasn't there, would mission succeed?"
- Actionable recommendations

### v1.0: Production Scale + Forensic Features
**Gap solved**: Compliance & determinism requirements
- Pluggable storage backends (PostgreSQL, BigQuery, S3)
- Streaming real-time ingestion (warehouse ops)
- Immutable audit trails with cryptographic signatures
- Deterministic, bit-perfect replay (defense/aerospace)
- Mission-critical failover + redundancy

### v1.1: Advanced Forensics & Real-Time Fusion
**Gap solved**: Regulatory compliance + operational awareness
- Real-time + historical fusion (live fleet + past missions)
- Compliance reporting (audit-ready logs, chain-of-custody)
- Forensic analysis for accidents/incidents
- Integration with compliance frameworks (ISO 3691-4, etc.)

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
