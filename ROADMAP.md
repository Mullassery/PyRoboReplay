# PyRoboReplay Roadmap

## Phase Overview

**Timeline**: 6-9 months from v0.1 to v1.0  
**Target**: Production-grade mission replay platform with simple student entry point

---

## Phase 0: Foundation (CURRENT - Weeks 1-4)

### Objectives
- [ ] Scaffold Rust core + PyO3 bindings
- [ ] Universal event model (8 event types)
- [ ] Timeline engine with basic queries
- [ ] ROS 2 adapter stub (structure, no parsing yet)
- [ ] Python CLI wrapper
- [ ] Unit tests for core modules (>80% coverage)

### Deliverables
- Rust library compiles and passes tests
- Python package installable via `pip install pyroboreplay`
- `pyroboreplay --help` shows CLI usage
- Example notebook with mock mission

### Key Decisions
- Adapter pattern ensures no ROS 2 lock-in
- Universal event model allows future adapters
- PyO3 for Rust-Python bridge (already in PyTerrainMap ecosystem)

---

## Phase 1: Simple Student Replay (Weeks 5-8)

### Objectives
- [ ] Implement ROS 2 bag file parser
  - Extract `/tf2` (TF transforms)
  - Extract sensor topics (lidar, camera, IMU)
  - Extract navigation topics (cmd_vel, odom)
  - Extract costmap/map topics
- [ ] CLI timeline scrubber
  - `pyroboreplay replay mission.bag` → interactive terminal UI
  - Play/pause/step forward/backward
  - Jump to event by index
  - Filter by event type
- [ ] Spatial visualization (matplotlib/plotly)
  - Robot trajectory over time
  - Coverage area
  - Obstacles
- [ ] First student example
  - Simple warehouse floor exploration
  - Generates synthetic ROS bag for testing

### Acceptance Criteria
- Student can replay any ROS 2 bag in <1 minute setup
- Timeline scrubber responds to user input in <100ms
- At least 3 event types properly parsed from bags

---

## Phase 2: Web UI + Visualization (Weeks 9-12)

### Objectives
- [ ] Build web dashboard (React/Vue)
  - Interactive timeline slider
  - 2D robot trajectory visualization
  - Event log with filtering
  - Play/pause/speed controls
- [ ] Backend API
  - `POST /missions/upload` - accept bag file
  - `GET /missions/{id}/events` - paginated events
  - `GET /missions/{id}/events/{index}` - specific event
  - `GET /missions/{id}/stats` - mission metadata
- [ ] Docker containerization
  - Single-command deployment
  - Examples for local + cloud hosting

### Acceptance Criteria
- Web UI loads in <2 seconds
- Timeline scrubbing is smooth (30 FPS)
- Works with missions up to 100k events

---

## Phase 3: PyTerrainMap Integration (Weeks 13-16)

### Objectives
- [ ] Embed PyTerrainMap as dependency
  - Use spatial knowledge graphs for context
  - Correlate robot poses with known obstacles/terrain
  - Display traversability alongside replay
- [ ] Spatial queries
  - "Which obstacles existed at timestamp T?"
  - "How did coverage evolve from T1 to T2?"
  - "Which areas were never visited?"
- [ ] Replay enhancements
  - Show current coverage as playhead moves
  - Highlight new discoveries
  - Visualize traversability changes

### Acceptance Criteria
- Replay shows spatial context from PyTerrainMap
- Coverage evolution queries return in <500ms
- Example mission with multi-robot exploration

---

## Phase 4: Swarm & Coordination Analysis (Weeks 17-20)

### Objectives
- [ ] Multi-robot visualization
  - Show all robots simultaneously
  - Color-code by status (active, idle, failed)
- [ ] Coordination events
  - Robot separation/clustering
  - Communication loss events
  - Resource contention (overlapping coverage)
- [ ] Swarm metrics
  - Coverage efficiency
  - Exploration rate
  - Idle time distribution
  - Congestion zones

### Acceptance Criteria
- Swarm replay with 10+ robots is smooth
- Coordination analysis identifies coverage gaps
- Example: warehouse fleet optimization scenario

---

## Phase 5: Advanced Event Queries (Weeks 21-24)

### Objectives
- [ ] Event-centric replay
  - Jump directly to failures (localization loss, battery warnings)
  - Jump to obstacles, communication loss
  - Bookmark important moments
- [ ] Multi-mission comparison
  - "Mission A vs Mission B: which explored faster?"
  - Strategy comparison (path planner performance)
  - Environmental change explorer (how did world change between missions?)
- [ ] Query DSL
  - Simple language for complex filters
  - `events where robot_id=robot_1 and event_type=obstacle_detected`
  - Save/replay common queries

### Acceptance Criteria
- Event queries return results in <200ms
- Multi-mission comparison shows clear performance deltas
- Query DSL documented with examples

---

## Phase 6: Production Scale (Weeks 25-30)

### Objectives
- [ ] Storage backends
  - PostgreSQL for mission history
  - BigQuery for analytics queries
  - S3 for long-term archival
  - In-memory for quick prototyping
- [ ] Streaming ingestion
  - Real-time mission recording (while robot is active)
  - Distributed event log
  - Horizontal scaling
- [ ] Mission-critical reliability
  - Failover + redundancy
  - Data validation on ingest
  - Audit trails
- [ ] Performance optimization
  - Lazy loading for large missions (1M+ events)
  - Index structures for time/spatial/robot queries
  - Caching layer

### Acceptance Criteria
- Ingest 10k events/second without drops
- Query 1M-event mission in <1 second
- Seamless failover on storage backend failure

---

## Phase 7: AI-Powered Analysis (Weeks 31-36)

### Objectives
- [ ] Root-cause analysis engine
  - "Coverage gap detected → likely causes: battery depletion, localization drift, exploration priority change"
  - Probabilistic scoring of root causes
- [ ] Anomaly detection
  - Unusual robot behavior
  - Coverage efficiency drops
  - Communication patterns
- [ ] Learning across missions
  - Identify patterns in failures
  - Recommendation engine for exploration parameters
  - Historical trend analysis

### Acceptance Criteria
- Root-cause analysis returns 3+ hypotheses with confidence scores
- Anomaly detection has <5% false positive rate
- Cross-mission insights are actionable

---

## v1.0 Release Criteria

- [ ] All phases 1-7 complete
- [ ] 100+ integration tests
- [ ] Documentation (tutorials, API, deployment guides)
- [ ] Benchmark suite (latency, throughput, scalability)
- [ ] Support for 5+ input adapters (ROS 2, Gazebo, Isaac Sim, custom, digital twins)
- [ ] Production deployment examples (AWS, GCP, Azure, K8s)
- [ ] Community feedback incorporated

---

## Future Roadmap (v1.1+)

- Hardware-specific analysis (e.g., battery drain correlations)
- Virtual environment generation from mission history
- Integration with reinforcement learning systems
- Mobile app for field operators
- Real-time collab editing of mission annotations
- Event replay with AR visualization
