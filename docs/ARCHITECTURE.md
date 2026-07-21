# PyRoboReplay Architecture Guide

High-level overview of PyRoboReplay's design and how components interact.

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     User Interfaces                          │
├────────────────────────┬────────────────────────────────────┤
│   CLI (Rust/Ratatui)   │    Python API (PyO3 bindings)      │
│   - Interactive UI      │    - from_ros_bag()                │
│   - Timeline scrubber   │    - get_sensor_frames()           │
│   - Keyboard controls   │    - get_available_sensors()       │
└────────────────────────┴────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│                   Core Replay Engine (Rust)                  │
├────────────────────────┬──────────────────────────────────────┤
│   Timeline Engine      │      Event Model                     │
│  - Event storage       │  - MissionEvent enum                 │
│  - Temporal queries    │  - Sensor data types                 │
│  - Filtering logic     │  - Immutable events                  │
│  - Index structures    │  - Serializable (serde)              │
└────────────────────────┴──────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│                   Input Adapters (Pluggable)                 │
├──────────────────────────────────────────────────────────────┤
│  ROS 2 Adapter  │  Gazebo  │  Isaac Sim  │  Custom Adapters  │
│  ✅ v0.1       │  v0.2    │   v0.3     │  anytime           │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│                    Input Data Formats                        │
├──────────────────────────────────────────────────────────────┤
│  ROS 2 .db3  │  Gazebo Events  │  Isaac Telemetry  │  JSON   │
│  (SQLite)    │  (v0.2)         │  (v0.3)           │  (any)  │
└──────────────────────────────────────────────────────────────┘
```

## Key Design Principles

### 1. **Input-Agnostic**
- Core replay engine doesn't know about ROS 2
- Adapter pattern allows adding new formats without changing core
- Universal event model normalizes all inputs

### 2. **Mapping-Independent**
- No mapping, SLAM, or navigation logic
- PyTerrainMap integration (Phase 1.1) for spatial context
- Stays in "replay and analysis" lane

### 3. **Explainability First**
- Every event has timestamp, type, robot ID, sensor type
- Causal analysis (Phase 0.3) will show event relationships
- Goal: answer "why" not just "what"

### 4. **Simple to Start, Infinite Depth**
- v0.1: Students load first bag in 5 minutes
- v1.0: Production-grade diagnostics
- Scalable from hobbyist to enterprise

## Module Structure

```
src/
├── lib.rs                  # Python module entry point (PyO3)
├── main.rs                 # CLI entry point
├── cli/
│   ├── mod.rs             # CLI orchestrator
│   ├── args.rs            # Argument parser (clap)
│   └── replay_ui.rs       # Interactive terminal UI (ratatui)
├── core/
│   ├── mod.rs
│   ├── event.rs           # Universal event model
│   │   ├── MissionEvent enum (5 sensor types)
│   │   ├── LidarData, CameraFrame, IMUData, Odometry, Costmap
│   │   └── MissionRecord (collection of events)
│   └── timeline.rs        # Replay engine
│       ├── Timeline (in-memory event store)
│       ├── Sensor queries (get_sensor_frames, get_multi_sensor_frames)
│       ├── Temporal queries (get_events_at_timestamp, get_events_in_range)
│       └── Event navigation (jump, step forward/backward)
└── adapters/
    ├── mod.rs             # Adapter trait definition
    └── ros2.rs            # ROS 2 .db3 parser
        ├── SQLite database access
        ├── Topic/message mapping
        └── CDR message deserialization (placeholder)

examples/
├── generate_warehouse_mission.rs  # Synthetic data generator
└── test_python_api.py             # Python usage examples

tests/
└── integration_tests.rs    # End-to-end validation (11 tests)
```

## Core Data Flow

### 1. Loading a Mission

```
User calls: Mission.from_ros_bag("warehouse.db3")
    ↓
Rust/PyO3 catches call
    ↓
ROS2Adapter::read(path)
    ↓
Open SQLite database
    ↓
Query topics table
    ↓
Query messages table (ORDER BY timestamp)
    ↓
For each message: parse → normalize to MissionEvent
    ↓
Create MissionRecord (sorted by timestamp)
    ↓
Return to Python
```

### 2. Querying Events

```
Python: mission.get_sensor_frames("lidar")
    ↓
Rust: Timeline::get_sensor_frames(mission_id, "lidar")
    ↓
Filter: for each event, check event.sensor_type() == "lidar"
    ↓
Return: Vec<&MissionEvent> (zero-copy references)
    ↓
PyO3: Convert references to Event objects
    ↓
Return: List[Event] to Python
    ↓
Python: iterate and analyze
```

### 3. CLI Replay

```
User: pyroboreplay replay mission.bag --sensor lidar
    ↓
Parse arguments (clap)
    ↓
Load mission (ROS2Adapter)
    ↓
Filter events (Timeline::get_sensor_frames)
    ↓
Initialize ReplayState
    ↓
Enter terminal UI (crossterm + ratatui)
    ↓
Render: header, timeline, event details, progress, footer
    ↓
Wait for keyboard input
    ↓
Process input: play/pause, next/prev, speed, jump
    ↓
Update state and re-render
    ↓
Exit on 'q' or ESC
```

## Memory Model

### Event Storage (Phase 0.1)

```
MissionRecord {
    id: Uuid,
    name: String,
    created_at: DateTime<Utc>,
    events: Vec<MissionEvent>,  // All events in memory
}

// ~50MB for 96k events
// Suitable for missions up to 1M events
// For 10M+ events → Stream from disk (Phase 2)
```

### Zero-Copy Queries

```
Timeline doesn't copy events
    ↓
Returns: Vec<&MissionEvent> (references)
    ↓
Python/PyO3 creates lightweight Event wrappers
    ↓
User reads timestamp, sensor_type, etc. (no data copy)
```

## Performance Characteristics

### Latency

| Operation | Latency | Target | Status |
|-----------|---------|--------|--------|
| Parse 96k events | ~0.4s | <5s | ✅ |
| Query 6k sensor frames | ~0.5ms | <10ms | ✅ |
| Navigation (step forward) | <1ms | <10ms | ✅ |
| CLI render (timeline redraw) | ~1-2ms | <30ms | ✅ |

### Throughput

- **Ingestion**: 240k events/second
- **Query**: <10ms for 96k events
- **Playback**: 30 FPS smooth

### Scalability (v0.1 limits)

- **Maximum mission size**: 1M events (~500MB)
- **Maximum event types**: 10+ (currently 5)
- **Maximum robots**: Unlimited (per event robot_id)
- **Maximum sensors**: Unlimited

## Integration Points

### PyTerrainMap (v0.3, Phase 0.3)

```
Timeline + PyTerrainMap Spatial Knowledge Graph
    ↓
Correlate events with map landmarks
    ↓
Spatial context: "obstacle at location X"
    ↓
Causal analysis: "navigation deadlock due to obstacle"
    ↓
Coverage analysis: "areas never explored and why"
```

### StatGuardian (v0.6, Phase 0.6)

```
Event anomaly detection
    ↓
Drift detection on sensor quality
    ↓
Quality contracts: "lidar should report 360 rays"
    ↓
Root-cause: "why lidar frames are incomplete"
    ↓
Improved diagnosis confidence
```

### Future: Real-Time Streaming

```
Current (v0.1): Load entire mission into memory
    ↓
Future (v1.0): Stream from storage backend
    ↓
Timeline queries: read from PostgreSQL/BigQuery
    ↓
Real-time: ingest while robot is still running
```

## Extension Points

### Adding a New Adapter

1. Create `src/adapters/gazebo.rs`
2. Implement `MissionAdapter` trait:
   ```rust
   pub trait MissionAdapter {
       fn read(&self, path: &str) -> Result<MissionRecord, AdapterError>;
       fn adapter_name(&self) -> &str;
   }
   ```
3. Parse Gazebo events → normalize to `MissionEvent`
4. Update `cli/mod.rs` to support `--format gazebo`

### Adding a New Sensor Type

1. Add variant to `MissionEvent` enum (e.g., `RgbdFrame`)
2. Create data struct (e.g., `RgbdData`)
3. Update adapter to parse the new type
4. Add queries to `Timeline` (already generic, auto-works)
5. Update documentation

### Adding Analytics

1. Create `src/analytics/` module
2. Implement analysis functions (coverage, efficiency, anomalies)
3. Expose via Python API (PyO3)
4. Add CLI subcommand

## Testing Strategy

### Unit Tests (14 tests)

```
src/core/event.rs       - Event model (sensor types, timestamps)
src/core/timeline.rs    - Timeline queries (filtering, navigation)
```

### Integration Tests (11 tests)

```
tests/integration_tests.rs
├── Parsing: 96k warehouse mission
├── Event breakdown: lidar/camera/imu/odometry counts
├── Filtering: individual + multi-sensor queries
├── Discovery: sensor_types detection
├── Performance: query latency <10ms, parse <5s
├── Robustness: empty queries, invalid sensors
└── Data integrity: ordering, robot_id validation
```

### End-to-End Validation

```
1. Generate synthetic warehouse mission (96k events)
2. Parse with ROS2Adapter
3. Filter by sensor (all 4 types)
4. CLI replay (UI navigation)
5. Python API (all query methods)
6. Performance benchmarks
```

## Deployment Models

### v0.1: Standalone CLI

```
User has ROS 2 bag file
    ↓
cargo build --release
    ↓
./pyroboreplay replay mission.bag
    ↓
Interactive terminal UI
```

### v0.2: Python Package

```
pip install pyroboreplay
    ↓
from pyroboreplay import Mission
    ↓
Jupyter notebook analysis
```

### v1.0: Server + Web UI

```
Server stores missions (PostgreSQL/S3)
    ↓
Web dashboard (React/Vue)
    ↓
Multi-user, collaborative replay
```

## Decision Log

### Why Rust?

- Performance: <10ms queries on 96k events
- Memory safety: No crashes from concurrent access
- PyO3 bindings: Easy Python integration
- Deployment: Single binary, no dependencies

### Why SQLite for ROS bags?

- Already ROS 2 standard (.db3 format)
- No external dependencies (bundled)
- Good for 0-100GB file sizes
- Future: migrate to distributed backend

### Why in-memory timeline (not streaming)?

- Query latency <1ms vs <50ms with disk
- v0.1 goal: interactive replay, not archival
- v1.0 will add streaming backend
- Students can load entire missions into brain

---

## Future Architecture (v1.0+)

```
┌─────────────────────────────────────────────────────────────┐
│  Web UI  │  CLI  │  Jupyter  │  Real-Time Streaming API     │
├─────────────────────────────────────────────────────────────┤
│          PyRoboReplay Service Layer (HTTP + gRPC)           │
├─────────────────────────────────────────────────────────────┤
│  Analytics Engine  │  Causal Analysis  │  ML Anomalies     │
├─────────────────────────────────────────────────────────────┤
│         Timeline Replay Engine (distributed)                │
├─────────────────────────────────────────────────────────────┤
│  PostgreSQL  │  BigQuery  │  S3  │  Cache Layer (Redis)     │
└─────────────────────────────────────────────────────────────┘
```

---

For more details, see:
- [Quick Start](QUICKSTART.md)
- [Python API Reference](API.md)
- [Development Setup](../README.md)
